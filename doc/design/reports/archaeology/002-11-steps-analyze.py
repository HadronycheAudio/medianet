#!/usr/bin/env python3
"""002-11: servo phase step per update.

  002-11-steps-analyze.py csv <run.err> <out.csv>   SERVOLOG lines -> per-update CSV
  002-11-steps-analyze.py stats <steps.csv>         statistics, histograms, residuals

A locked update is one whose own result satisfies P-04: updates >= 8 and the
smoothed error below max(20 us, delay_ns / 4) (delay_ns is the servo's dmin).
Percentiles are nearest-rank; the median of an even count is the mean of the
two middle values.

s = theta0_after - pred is the jump of published time at the reference instant
tm only. The same update changes the drift (clock.rs:363) and the new view is
published later, so a reader sees s + (drift_after - drift_before) * (t_pub - tm)
(drift in ns/s, t_pub - tm in s). Two figures for the extra term:
  bound:    t_pub - tm < the interval to the servo's previous update (the window
            opens at the first exchange after the previous one was taken,
            clock.rs:293-295, and tm is inside the window), so
            |extra| <= |drift change| * interval;
  estimate: t_pub - tm from t_mono_ns (the update's entry, a few us before the
            publication at module.rs:1588) mapped to the servo's simulated
            local clock (LocalClock::from_mono, clock.rs:65-67): offset 250 ms
            and drift 35 ppm times the module index (main.rs:493), the index
            read from the first theta0 (about 250 ms x index), base = first
            logged t_mono_ns - 2.4 s (an approximation; the error in base
            is under 0.3 ms of t_pub - tm).
"""
import collections, csv, math, statistics, sys


def pct(v, p):
    v = sorted(v)
    return v[max(1, math.ceil(p / 100 * len(v))) - 1]


def make_csv(src, dst):
    rows = [dict(kv.split('=', 1) for kv in l.split()[1:]) for l in open(src) if l.startswith('SERVOLOG ')]
    ids = {}
    out = ['servo,t_mono_ns,updates,locked,theta0_before,theta0_after,pred,err,alpha,w,delta,s,s_pred,delta_pred,tm,drift_before,drift_after']
    for r in rows:
        sid = ids.setdefault(r['thread'], 's%d' % (len(ids) + 1))
        ua = int(r['updates_after'])
        locked = int(ua >= 8 and float(r['ema_err']) < max(int(r['delay_ns']) / 4.0, 20000.0))
        tb, ta, pred = int(r['theta0_before']), int(r['theta0_after']), int(r['pred'])
        a, w, err = float(r['alpha']), float(r['w']), float(r['err'])
        s_pred = int(a * w * err)
        out.append(','.join(map(str, [sid, r['t_mono_ns'], ua, locked, tb, ta, pred, r['err'], r['alpha'], r['w'],
                                      ta - tb, ta - pred, s_pred, (pred - tb) + s_pred,
                                      r['tm'], r['drift_before'], r['drift_after']])))
    open(dst, 'w').write('\n'.join(out) + '\n')


def extra_stats(rows, L):
    by = collections.defaultdict(list)
    for r in rows:
        by[r['servo']].append(r)
    base = min(int(r['t_mono_ns']) for r in rows) - 2_400_000_000
    ext_b, ext_e, jump_e, jump_b, dd, tpm = [], [], [], [], [], []
    for sid, xs in by.items():
        xs.sort(key=lambda r: int(r['t_mono_ns']))
        idx = round(int(xs[0]['theta0_after']) / 250e6)
        off, d = idx * 250_000_000, idx * 35e-6
        for prev, r in zip(xs, xs[1:]):
            if r['locked'] != '1':
                continue
            ddrift = float(r['drift_after']) - float(r['drift_before'])
            m = int(r['t_mono_ns'])
            iv = (m - int(prev['t_mono_ns'])) / 1e9
            tp = (m + off + (m - base) * d - int(r['tm'])) / 1e9
            sv = int(r['s'])
            ext_b.append(abs(ddrift * iv))
            ext_e.append(abs(ddrift * tp))
            jump_e.append(abs(sv + ddrift * tp))
            jump_b.append(abs(sv) + abs(ddrift * iv))
            dd.append(abs(ddrift))
            tpm.append(tp * 1e3)
    # the first locked update of a servo has its predecessor in the CSV too
    for k, v in [('extra term bound |dDrift| * interval (ns)', ext_b),
                 ('extra term estimate |dDrift| * (t_pub - tm) (ns)', ext_e),
                 ('jump at publication estimate |s + extra| (ns)', jump_e),
                 ('jump at publication bound |s| + bound (ns)', jump_b),
                 ('|drift change| (ppb)', dd),
                 ('t_pub - tm estimate (ms)', tpm)]:
        print(k, 'count', len(v), 'min %.3f median %.3f p99 %.3f max %.3f' % (min(v), statistics.median(v), pct(v, 99), max(v)))


def stats(path):
    rows = list(csv.DictReader(open(path)))
    L = [r for r in rows if r['locked'] == '1']
    print('rows', len(rows), 'locked', len(L))
    cols = {
        'abs delta': [abs(int(r['delta'])) for r in L],
        'abs s': [abs(int(r['s'])) for r in L],
        'abs delta - delta_pred': [abs(int(r['delta']) - int(r['delta_pred'])) for r in L],
        'abs s - s_pred': [abs(int(r['s']) - int(r['s_pred'])) for r in L],
    }
    for k, v in cols.items():
        print(k, 'count', len(v), 'min', min(v), 'median', statistics.median(v), 'p99', pct(v, 99), 'max', max(v))
    print('s == 0:', sum(1 for x in cols['abs s'] if x == 0))
    extra_stats(rows, L)
    by = collections.defaultdict(list)
    for r in L:
        by[r['servo']].append(int(r['t_mono_ns']))
    iv = [(b - a) / 1e6 for ts in by.values() for a, b in zip(sorted(ts), sorted(ts)[1:])]
    print('interval ms (consecutive locked updates of one servo) n', len(iv), 'min', min(iv),
          'median', statistics.median(iv), 'p99', pct(iv, 99), 'max', max(iv))
    h = collections.Counter(min(d // 5000, 9) for d in cols['abs delta'])
    print('histogram abs delta, 5 us bins:', [h[i] for i in range(10)])
    edges = [0, 25, 50, 100, 200, 400, 800]
    hs = [sum(1 for s in cols['abs s'] if edges[i] <= s < edges[i + 1]) for i in range(6)]
    print('histogram abs s, bins', edges, hs)


if __name__ == '__main__':
    if sys.argv[1] == 'csv':
        make_csv(sys.argv[2], sys.argv[3])
    else:
        stats(sys.argv[2])
