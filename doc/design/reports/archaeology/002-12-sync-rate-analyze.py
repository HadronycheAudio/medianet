#!/usr/bin/env python3
"""002-12-sync-rate-analyze.py <run-dir> <sync-hz>: sync rate of one run from its four single-writer metrics files.
Run validity of swarm/notes/003-02.md (clean 4-module run with one election, joins, map,
>= 2 intact holder windows after dropping the first); the chimera guard followers x rate x 1.10,
the divisor re-derived from the run and the duration correction (sum served / covered span /
followers) are those of swarm/notes/003-05.md (and the 003-04 entry of swarm/status.md)."""
import csv, glob, os, re, sys
d, hz = sys.argv[1], int(sys.argv[2])
mods = ['sender', 'synth1', 'synth2', 'mixer']
rows = {m: list(csv.DictReader(open(f'{d}/{m}.csv'))) for m in mods}
logs = {m: open(f'{d}/{m}.log', errors='replace').read() for m in mods}
# clean-run precondition
for m in mods:
    L = logs[m]
    assert 'map: 4 modules' in L, (m, 'no 4-module map')
    assert len(re.findall(r'election \d+\)', L)) and not re.search(r'election [2-9]', L), (m, 'elections')
    assert len(re.findall(r'^\[ *\d+s\] net: clock: \S+ holds it', L, re.M)) == 1, (m, 'election count')
    joins = re.findall(r'net: (\w+) joined from', L)
    assert len(joins) == 3 and len(set(joins)) == 3, (m, joins)
holders = {m: [r for r in rows[m] if r['holding'] == '1'] for m in mods}
hm = [m for m in mods if holders[m]]
assert len(hm) == 1, hm
h = hm[0]
followers = [m for m in mods if m != h and any(int(r['asked']) > 0 for r in rows[m])]
nf = len(followers)
assert nf == 3, followers
cap = nf * hz * 1.10
hr = [r for r in holders[h] if all(r.values())]
hr = [r for r in hr if int(r['served']) <= cap]
assert len(hr) - 1 >= 2, 'degenerate'
first, kept = hr[0], hr[1:]
served = [int(r['served']) for r in kept]
span = (int(kept[-1]['unix_ms']) - int(first['unix_ms'])) / 1000
naive = sum(served) / len(served) / nf
corr = sum(served) / span / nf
# row interval of the holder file, ms
st = [int(r['unix_ms']) for r in holders[h]]
iv = [b - a for a, b in zip(st, st[1:])]
fa = {}
for m in followers:
    ar = [r for r in rows[m] if int(r['asked']) > 0]
    f0, k = ar[0], ar[1:]
    sp = (int(k[-1]['unix_ms']) - int(f0['unix_ms'])) / 1000
    fa[m] = sum(int(r['asked']) for r in k) / sp
print(f'{d} hz={hz} holder={h} followers={nf} windows={len(kept)} sum_served={sum(served)} span={span:.3f}s '
      f'mean_row_interval={sum(iv)/len(iv):.1f}ms naive={naive:.2f} corrected={corr:.2f} '
      f'asked_per_follower=' + '/'.join(f'{fa[m]:.2f}' for m in followers))
print(f'RESULT {hz} {naive:.4f} {corr:.4f}')
