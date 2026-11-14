# 014-23 — Late reclassification scenario (C-108)
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-18, epic 009
## Context
Late versus missing (C-108) `[decision]`: at the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span it is reclassified as late (missing −1, late +1) and discarded. Rings are sized to the presentation group's A (C-96). A slice that arrives after its position has left the ring span, or never arrives, stays missing. Epic AC-5 (AC-15 refined by C-108): a delay within the ring span ends in the late count only, and a report made between the deadline and the arrival shows it as missing, the next report showing it reclassified; a drop, or a delay beyond the ring span, ends in the missing count only.

The reclassification itself is done by epic 009's per-input counters; this ticket checks that receiver reports (one per bundle every 100 ms, C-40) carry it correctly. It reuses the simulated-time harness of 014-18.

## Work
1. In the 014-18 harness, set a ring span and inject on one member a delay longer than the consumer offset but shorter than the ring span, timed so that a report falls between the deadline and the late arrival.
2. Inject on another member a delay longer than the ring span.
3. Capture and parse every report.

## Acceptance criteria
1. Within-span delay: the report made between the deadline and the arrival shows `missing` +1 for that slice; the next report shows `missing` back down by one and `late` up by one; final totals are late 1, missing 0.
2. Beyond-span delay: the slice stays in `missing` in every later report and `late` is unchanged.
3. Consecutive reports parse and the missing decrease is computed as −1 by the 014-02 delta helper.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The reclassification logic and ring sizing (epic 009).
- Final-total scenarios for late, missing and marked gap (014-18).
