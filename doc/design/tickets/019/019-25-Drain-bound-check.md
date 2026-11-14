# 019-25 — Drain-bound check
**Epic:** 019-L1-two-machine-harness · **Effort:** XS · **Depends on:** 019-04, 019-08, 019-11, 019-17

## Context
A consumer drains at most 2 × the admitted packets per slice from one input per cycle (C-16, decided); the cycle is one slice period (C-156). Its falsifier is assigned to L1: "L1 shows legitimate bursts exceeding it". Bursts arise from coherent bundles (32 packets per slice period at the `two-bundles` load), from NIC interrupt coalescing, and from host load. The `flood` scenario (019-19) exceeds the bound on purpose and is excluded here.

Epic 019 criterion **019-D**: for every legitimate-traffic scenario the report gives the largest number of packets drained from one input in one cycle and the number of cycles that reached the bound.

The per-input drain counter must come from the hosted runtime; it is not yet stated as an epic 015 deliverable (open, owner epic 015). If it is absent, this ticket records the reason and stops.

## Work
- Add to the report (019-04) a `drain` section: per input, admitted packets per slice, bound (2 ×), largest packets drained in one cycle (slice period, C-156), cycles at the bound, and packets left queued after a cycle at the bound.
- Evaluate it on the hardware-pair records of `baseline`, `two-bundles`, `coalescing` with coalescing on, and `stress`.
- Record under C-16 in `doc/design/reports/l1-results.md`: falsifier triggered (any cycle at the bound with packets left that were legitimate traffic) or not, tagged `[measured]`.

## Acceptance criteria
1. A synthetic record with one cycle draining exactly the bound and leaving packets queued shows `cycles_at_bound = 1` for that input.
2. `l1-results.md` has a C-16 table with one row per evaluated scenario and a falsifier verdict, or a stated reason why not.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Each pending-measurement constant in the table above has a measured value tagged `[measured]`, or a stated reason why not
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The flood case (019-19).
- Changing C-16 or the drain in the runtime (epic 015).
