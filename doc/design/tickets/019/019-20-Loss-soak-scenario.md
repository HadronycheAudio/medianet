# 019-20 — Loss-soak scenario
**Epic:** 019-L1-two-machine-harness · **Effort:** M · **Depends on:** 019-07, 019-08

## Context
v1 has no redundancy (no retransmission, no forward error correction); it detects, classifies, conceals and accounts. That stance assumes clean-medianet loss sits near the floor.

Constant **C-90 loss floor** is from memory `[memory]`: at a bit error rate of 1e-12, about 1.1e-8 per default packet (about 1 320 bytes on the wire), one loss per stream per link about every 13 hours at 2 000 packets/s.

**Loss soak (C-118).** 24 hours at the two-bundle load (64 000 packets/s, about 5.5e9 packets). The statistic is aggregate loss over all streams, where the floor predicts about 58 losses. (One stream alone would give only about 2 expected losses in 24 hours, too few to separate "at" from "somewhat above" the floor; aggregating over 32 streams fixes that.) The two-bundle load is the `two-bundles` scenario (019-08): 2 × 96 channels = 32 streams of 2 000 packets/s from host `a` to host `b`.

Scenario `loss-soak`: 24 hours, counts every lost packet per stream per direction, "on a clean wire".

Open sub-question owned here: **"Is clean-wire loss near the floor?"** Falsifier, settled by C-179: aggregate clean-medianet loss above 10 × the floor (C-90), about 580 losses in the 24-hour soak, reopens the redundancy decision; between 1 × and 10 × (about 58 to 580 losses) it is recorded and investigated as a cabling or host problem and does not. The verdict itself is 019-21's.

AC-24 (part): **aggregate loss at or below the floor.**

Accounting: a source assigns one session number per stream, bumped on restart and on timeline epoch change (C-19); loss is counted per stream and session, so a restart is not counted as loss. A is computed by the harness with the commissioning rule and recorded (C-148, 019-26). The soak is a strict-mode run (C-147), since its result enters the constants table as C-90 and C-91.

## Work
- Scenario `loss-soak`: the `two-bundles` load (32 streams, 64 000 packets/s) for 86 400 s (dry-run: 5 s, reduced streams).
- Count every lost packet per stream, session and direction from the receiver's sequence accounting; classify each loss using the overflow counter, NIC counters (`ethtool -S` rx errors/CRC/drops sampled every 60 s) and switch counters where available, so wire loss is separated from host-local loss.
- Record each loss event with its timestamp, stream and classification.
- Report section: packets sent/received per stream and in aggregate, aggregate wire-loss count and rate per packet, host-local loss count, comparison with the about 58 losses C-118 predicts, with 1.1e-8 per packet and with the C-179 threshold of 10 × the floor (about 580 losses, scaled to the packets actually sent), and a Poisson 95 % interval on the aggregate rate.
- Harness robustness for 24 h: periodic checkpoint of record files so an abort after hour N keeps hours 0 to N.
- Record the result in `doc/design/reports/l1-results.md` under C-90 with hardware, stamp, tag `[measured]`.

## Acceptance criteria
1. Dry-run passes in CI.
2. On the hardware pair, a completed 24-hour run at 64 000 ± 1 % packets/s reports a loss count per stream and in aggregate, each loss classified (wire / socket overflow / NIC / unknown).
3. The measured aggregate wire-loss rate per packet with its 95 % interval is stated against C-90's 1.1e-8 and C-118's about 58 expected losses; AC-24's "loss at or below the floor" is reported as met or not met, and the aggregate count is placed in one of three bands: at or below 1 × the floor, between 1 × and 10 ×, above 10 × (C-179).
4. A run aborted after at least 1 hour still yields a valid partial report, with the expected loss scaled to the packets actually sent.
5. A stream restart during the run produces a new session row and no loss attributed to the restart.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Each pending-measurement constant in the table above has a measured value tagged `[measured]`, or a stated reason why not
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Setting C-91 and the redundancy recommendation (019-21).
- Writing C-90 to the constants table (019-22).
