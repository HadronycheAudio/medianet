# 014-15 — Loss-floor comparator
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-04
## Context
The epic delivers a loss-floor comparator that raises the **cable-health flag** when measured loss exceeds the threshold. Constants:

| Id | Constant | Value |
|---|---|---|
| C-90 | Expected loss floor | at a bit error rate of 1e-12, about 1.1e-8 per default packet (about 1 320 bytes on the wire): one loss per stream per link about every 13 hours at 2 000 packets/s `[memory]`; pending measurement |
| C-91 | Loss alarm threshold | set from clean-network loss measured at L1 |
| C-167 | L1 test parameters | the loss alarm C-91 is 10 × the measured clean-wire loss (proposal, pending first runs) |
| C-179 | Redundancy decision | clean-medianet loss above 10 × the C-90 floor (about 580 losses in the C-118 soak) reopens the redundancy decision; between 1 × and 10 × it is recorded and investigated as a cabling or host problem and does not |
| C-178 | Open commissioning window | ordinary production loss figures are unqualified while a window is open; the commissioning tool's own measurements (including the soak) are valid commissioning results recorded with their window |
| C-118 | Loss soak | 24 h at the two-bundle load (64 000 packets/s, about 5.5e9 packets); the statistic is **aggregate loss over all streams**; the floor predicts about 58 losses |

Settled: the comparator's statistic is aggregate loss over all streams (C-118), compared per default packet (C-90). C-91 is defined as 10 × the clean-wire loss measured at L1 (C-167), and that measurement does not exist yet, so the threshold is a parameter, not a hard-coded number. C-90 keeps its `[memory]` tag until the pre-implementation check cites the standard, when it becomes `[standard]` (C-132).

Loss inputs come from receiver reports: per-member `missing` (absent at the deadline; remedy is the network) as opposed to `late` (remedy is budget) and `marked_gap` (loss further upstream, not this link). A slice counted missing and later reclassified late (missing −1, late +1, C-108) is not a loss; the comparator therefore uses the latest missing counts, which already include reclassifications. Counters count slices; at defaults one slice is one packet (C-05).

Still open (epic open sub-question 7, owner L1 / task 019): the minimum sample size before the flag may fire. Falsifier: on a clean network the flag fires, or over a 24 h soak (C-118) it stays silent with loss well above C-91. Still open (README remaining gaps, owner epic 015): which event the caller raises from the flag.

Settled (C-179): reading a soak result against the floor is the L1 soak's job (task 019): above 10 × the C-90 floor it reopens the redundancy decision, between 1 × and 10 × it is recorded and investigated as a cabling or host problem. The comparator supports that reading because its threshold is a parameter. Settled (C-178): loss counted while a commissioning window is open is unqualified as a production figure, so a production comparator does not take it in; a soak run by the commissioning tool inside a window counts it and is recorded with the window.

## Work
1. Implement `LossFloor` with an integer-ratio threshold (no floating point needed in `no_std`), `fn observe(&mut self, received: u64, missing: u64)` accumulating totals over **all streams** of all reports it is given, and `fn alarm(&self) -> bool` returning true when `missing / (received + missing)` exceeds the threshold.
2. Accept a decrease of a member's missing count between reports (C-108) by accumulating deltas as signed values (014-02 helper) and never letting the total go below zero.
3. Take the minimum number of packets before the flag can fire as a parameter; its default comes from open sub-question 7.
4. Exclude `late` and `marked_gap` from the loss figure.
5. Expose the flag; the caller turns it into the cable-health event.
6. `observe` takes whether the observation was made while a commissioning window was open (from the stamp, C-153); a comparator configured for production ignores such observations, one configured for a commissioning-tool soak counts them (C-178).

## Acceptance criteria
1. With threshold 1e-6 and minimum 10 000 000 packets: 9 missing of 10 000 000 does not alarm; 11 missing of 10 000 000 alarms.
2. Below the minimum packet count, no alarm fires regardless of loss.
3. `late` and `marked_gap` values never change the result; a reclassification (missing −1, late +1) lowers the loss total by one.
4. Totals summed over 32 streams equal the sum of the per-stream totals (aggregate statistic, C-118).
5. With the C-118 figures (about 5.5e9 packets, 58 losses) and a threshold set at the C-90 floor, and again at 10 × that floor (the C-179 redundancy line), the result is computed without overflow; no allocation, no panics, no overflow over 2^63 packets.
6. Observations marked as made inside a commissioning window leave a production comparator unchanged and are counted by a soak comparator (C-178).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Measuring the clean-network loss and setting C-91 (task 019, L1); checking C-90 (task 001); reading the soak result against the C-179 redundancy line (task 019).
- Raising and presenting the cable-health event (epics 015, 017).
