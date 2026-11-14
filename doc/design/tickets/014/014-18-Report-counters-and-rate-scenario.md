# 014-18 — Report counters and rate scenario (AC-15, AC-2)
**Epic:** 014-Measurement-path · **Effort:** M · **Depends on:** 014-03, 014-04, 014-06, epic 009
## Context
Receiver report counters count slices and distinguish `[decision]`:
- **late**: arrived after the consumer's deadline (remedy: budget);
- **missing**: absent at the deadline (remedy: network);
- **marked gap**: upstream alive, loss further up.

Late versus missing (C-108): at the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span (rings sized to A, C-96) it is reclassified as late (missing −1, late +1) and discarded. System criterion AC-15, as refined by the epic: injected delay beyond the offset that arrives within the ring span ends in the late count only; injected drop ends in the missing count only. Epic AC-2: exactly one report per bundle per 100 ms (C-40) (±1 slice period, 0.5 ms, C-02), sent from the data thread; a 16-member bundle produces 10 reports per second, not 160. Per-input counters and the reclassification come from epic 009. A slice on another timeline epoch is not a miss, restarts the dead clock and is counted in its own per-connection counter, other-epoch (C-135, C-159).

This ticket checks final totals; the intermediate reports around a reclassification and the beyond-ring-span case are in 014-23.

## Work
1. Build a deterministic simulated-time harness: a producer of a 16-member bundle emitting one slice per 0.5 ms per member, each member with its own session (C-19), a fault injector (delay one member's slices beyond the consumer offset but within the ring span; drop slices; mark a gap upstream), the epic 009 per-input counters on the consumer, and the data-thread loop calling `ReportTimer::due` and `encode_report`.
2. Capture every encoded report and parse it with `ReceiverReport::parse`.
3. Scenarios: (a) clean 10 s; (b) delay 50 slices of member 3 beyond the offset, within the ring span; (c) drop 50 slices of member 5; (d) 50 upstream-marked gaps on member 7; (e) 50 slices of member 9 sent on another timeline epoch (C-120 hook).

## Acceptance criteria
1. (a) 100 reports in 10 s, each within ±500 000 ns of its 100 ms deadline, each carrying 16 members with their own sessions; all late/missing/marked-gap counts are 0.
2. (b) In the last report, member 3's `late` totals 50 and its `missing` totals 0; other members unchanged.
3. (c) Member 5's `missing` totals 50 and its `late` totals 0.
4. (d) Member 7's `marked_gap` totals 50; its `missing` and `late` are 0.
5. (e) Member 9's `other_epoch` totals 50; its `missing`, `late` and `interrupted` are 0 (C-135, C-102).
6. Reports are emitted only from the data-thread loop of the harness (no housekeeping call site exists).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Intermediate reports during reclassification and delay beyond the ring span (014-23).
- Real network fault injection (epics 015, 019).
- Drop counters and DSCP (014-19).
