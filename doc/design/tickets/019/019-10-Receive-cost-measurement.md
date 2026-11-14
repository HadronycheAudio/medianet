# 019-10 — Receive cost measurement
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-07, 019-08

## Context
Constant **C-93 receive cost per packet** is currently a guess: 2 to 4 µs per packet, 13 to 26 % of the slice period for two bundles `[memory]`. Epic 019 must measure it as "data-thread time in batch receive divided by packets, at 2 000, 32 000 and 64 000 packets/s".

The three rates map onto existing scenarios:
- 2 000 packets/s: one 6-channel stream, `baseline` (019-07).
- 32 000 packets/s: one 96-channel bundle = 16 streams (C-22), `two-bundles` (019-08) with one bundle.
- 64 000 packets/s: two bundles = 32 streams, 676 Mbit/s, `two-bundles`.

Slice period is 0.5 ms (C-01, C-02, C-05). At 64 000 packets/s, the consumer receives 32 packets per slice period, so per-packet cost × 32 / 500 µs is the fraction of the period used.

This measurement also feeds a downstream decision: whether the 10 GbE rule (any port above about 198 channel-copies, C-25) must be tightened. The criterion is settled by the C-93 falsifier, "measured cost makes C-92 fail": the rule is tightened if the `two-bundles` test (C-92, 019-08) has late slices and the measured receive cost accounts for them. This ticket reports the data and the fraction of slice period; 019-22 records the recommendation.

The instrumentation (data-thread time in batch receive and packet count) must come from the hosted runtime; it is not yet stated as an epic 015 deliverable (open, owner epic 015).

## Work
- Read the data thread's CPU time spent in batch receive and the packet count from the runtime (epic 015 instrumentation; use `CLOCK_THREAD_CPUTIME_ID` around the batch receive if the runtime exposes the hook). If the runtime does not expose it, file the gap against epic 015 and stop.
- Use the `--bundles <1|2>` parameter of `two-bundles` (019-08) for the 32 000 and 64 000 packets/s rates.
- Run every measurement in strict mode (C-147) and record the capability states (C-127, C-129) of both hosts; a run with strict mode not in force is not used for C-93.
- Report section `rx_cost`: packets received, data-thread time in batch receive, µs per packet, and fraction of slice period at that rate.
- Run on the hardware pair at the three rates, 10 minutes each (C-167; the 1-hour runs of 019-07/019-08 may be reused).
- Record in `l1-results.md` under C-93: µs/packet at each rate, fraction of slice period at 64 000 packets/s, hardware, stamp, tag `[measured]`.

## Acceptance criteria
1. Dry-run of `baseline` and `two-bundles` includes an `rx_cost` section with non-zero packet count.
2. `l1-results.md` has C-93 values at 2 000, 32 000 and 64 000 packets/s (3 values), each tagged `[measured]` with stamp.
3. The fraction of slice period used at 64 000 packets/s is stated and compared with the guessed 13 to 26 %, together with a statement whether the C-93 falsifier (cost makes C-92 fail) is triggered.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Each pending-measurement constant in the table above has a measured value tagged `[measured]`, or a stated reason why not
- [ ] Every new factual claim carries an evidence tag
- [ ] Regime stamp fields touched are complete and documented
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Changing the C-25 10 GbE rule or the constants table (019-22).
- Optimising receive cost (epic 015).
