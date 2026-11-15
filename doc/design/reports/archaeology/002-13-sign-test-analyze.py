#!/usr/bin/env python3
"""002-13: the falsifier of the 002-10 open item (run-1 degradation).

Reads the run directory written by the 002-13 pair runs (one `<name>.err`
per run: the scratch copy's stderr, carrying the 002-10 ticklog lines) and
applies the falsifier exactly as 002-10 defined it (REPORT-0b, section (e),
002-10 subsection):

* per run, count the one-second windows whose sender ticker p99
  (`tick_p99_us` of the `ticklog module=sender` lines) is at least 50 us;
* a pair is flooded-worse if the flooded run has at least 5 more such windows
  than its baseline, baseline-worse if it has at least 5 fewer, a tie
  otherwise;
* F flooded-worse, B baseline-worse: flood effect if F - B >= 5, host noise
  (no flood effect shown at this rate) if F - B <= 4.

It also prints the one-sided sign-test probability P(X >= F) for X binomial
(n = F + B non-tied pairs, p = 1/2), and, per flooded run, the mixer's
acceptance counters on the flooded feed input (in0): datagrams received,
carrying an audio message, accepted by `ok`, mixed, echoed with / without an
audio message — so acceptance, not just reception, is shown.

Usage: 002-13-sign-test-analyze.py <run-dir> [pairs]
"""
import math
import os
import re
import sys

THRESH_US = 50.0
MARGIN = 5


def windows(path, module):
    rows = []
    with open(path, encoding='utf-8', errors='replace') as fh:
        for line in fh:
            if 'ticklog module=%s ' % module not in line or 'wake_n=' not in line:
                continue
            kv = dict(re.findall(r'(\w+)=([0-9.]+(?:\.\.[0-9]+)?)', line))
            rows.append(kv)
    return rows


def count_high(rows):
    return sum(1 for r in rows if float(r.get('tick_p99_us', 0)) >= THRESH_US)


def acceptance(rows):
    keys = ('in0', 'in0_audio', 'in0_ok', 'in0_mixed', 'in0_echo_full', 'in0_echo_hdr')
    return {k: sum(int(float(r.get(k, 0))) for r in rows) for k in keys}


def tail_prob(f, n):
    return sum(math.comb(n, k) for k in range(f, n + 1)) / 2 ** n if n else 1.0


def main():
    d = sys.argv[1]
    pairs = int(sys.argv[2]) if len(sys.argv) > 2 else 10
    F = B = 0
    print('pair order        baseline_high flooded_high diff class           '
          'b_win f_win b_tick_n f_tick_n b_maxlate_us f_maxlate_us exit_b exit_f')
    acc_lines = []
    for p in range(1, pairs + 1):
        name = 'pair%02d' % p
        res = {}
        for kind in ('baseline', 'flooded'):
            err = os.path.join(d, '%s-%s.err' % (name, kind))
            snd = windows(err, 'sender')
            mix = windows(err, 'mixer')
            ex = open(os.path.join(d, '%s-%s.exit' % (name, kind))).read().strip()
            res[kind] = dict(high=count_high(snd), nwin=len(snd),
                             tick_n=sum(int(float(r.get('tick_n', 0))) for r in snd),
                             maxlate=float(snd[-1].get('tick_cum_max_us', 'nan')) if snd else float('nan'),
                             acc=acceptance(mix), exit=ex)
        diff = res['flooded']['high'] - res['baseline']['high']
        cls = 'flooded-worse' if diff >= MARGIN else ('baseline-worse' if diff <= -MARGIN else 'tie')
        F += cls == 'flooded-worse'
        B += cls == 'baseline-worse'
        order = 'baseline-first' if p % 2 else 'flooded-first'
        b, f = res['baseline'], res['flooded']
        print('%-4s %-14s %13d %12d %4d %-15s %5d %5d %8d %8d %12.1f %12.1f %s | %s'
              % (p, order, b['high'], f['high'], diff, cls, b['nwin'], f['nwin'], b['tick_n'], f['tick_n'],
                 b['maxlate'], f['maxlate'], b['exit'], f['exit']))
        for kind in ('baseline', 'flooded'):
            a = res[kind]['acc']
            acc_lines.append('%s %-8s in0 received=%d audio=%d ok=%d rejected=%d mixed=%d echo_full=%d echo_hdr=%d'
                             % (name, kind, a['in0'], a['in0_audio'], a['in0_ok'], a['in0_audio'] - a['in0_ok'],
                                a['in0_mixed'], a['in0_echo_full'], a['in0_echo_hdr']))
    n = F + B
    print()
    print('\n'.join(acc_lines))
    print()
    print('F = %d, B = %d, ties = %d, F - B = %d' % (F, B, pairs - n, F - B))
    print('one-sided sign-test probability P(X >= F | n = %d, p = 1/2) = %.4f' % (n, tail_prob(F, n)))
    print('decision: %s' % ('flood effect (F - B >= 5)' if F - B >= 5
                            else 'host noise, no flood effect shown at this rate (F - B <= 4)'))


if __name__ == '__main__':
    main()
