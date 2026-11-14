# 022b-03 — L2 Windows: cross-sink skew against the Linux reference sink
**Epic:** 022b-Windows-edge-L2-certification · **Effort:** S · **Depends on:** 022b-02

## Context
- Part of epic 022b (Windows edge L2 certification), L2 certification.
- Target (022-AC-1 / AC-28): cross-sink skew of the Windows sink against the **Linux reference sink** at most **1.05 ms**, derived as the sum of the two E targets: 50 µs for the Linux reference sink (C-70) plus 1 ms for Windows (C-72) (C-82) — pending measurement.
- Statistic, as defined for skew in C-79: the 99.9th percentile of absolute skew over one hour and across restarts of either sink, with the maximum reported beside it.
- Both sinks play the same stream in the same presentation group: `emit(p) = T(p) + A`. D of the Windows sink is the commissioned measured value from 022b-02.
- Pass is against the sum of the target E values (C-149), never the E values measured in the same run. The run is in strict mode (C-147) with A computed and recorded by the harness (C-148).
- Tolerance for a pair of different sinks (C-203): any no-step or recovery tolerance applied to the pair (for example after a restart) is the larger of the two sinks' own, here the Windows restart spread rather than the Linux ±1 sample; both sinks' figures are reported beside each verdict.

## Work
1. On the rig (022b-01), play one stream to the Windows sink and the Linux reference sink in one group for one hour, restarting each sink at least once during the run (the number of restarts is recorded).
2. Capture both outputs with the L2 harness; compute skew per marker; report the 99.9th percentile of absolute skew and the maximum.

## Acceptance criteria
1. **(022-AC-1 / AC-28, part.)** The 99.9th percentile of absolute cross-sink skew is ≤ 1.05 ms over the one-hour run including restarts of either sink, maximum reported beside it.
2. Result recorded `[measured]` in `doc/design/reports/` (C-130) for 022b-07, with both sinks' own figures reported beside the verdict (C-203).

## Definition of done
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- E and D measurement (022b-02).
