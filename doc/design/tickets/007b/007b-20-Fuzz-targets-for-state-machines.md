# 007b-20 — Fuzz targets for state machines
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** S · **Depends on:** 007a-15, 007b-13

## Context
- The fuzz harness skeleton has one target per parser (007a-15) and one per state machine.
- Take the set of state machines from the tables; epic 005 supplies those for discovery (007b-01), election and clock holder (007b-02, 007b-03, 007b-04), admission (007b-05, 007b-06) and connection (007b-07) (C-193), with alignment health (007b-08), connection alarms (007b-09), reassembly (007b-10), regime stamp distribution (007b-11) the commissioning window (007b-12) and the live change of A (007b-24) inside whichever machines epic 005 names. The fuzzing tool is the implementing team's choice (C-192). Targets drive `UnderTest::control_step(machine, now_ns, input)`.
- Invariants to assert `[decision]`: no panic; bounded time per step, using the per-input bound of C-164 (linear in packet length, at most 10 µs per packet; a proposal pending L1, read from configuration); connection states only in {Requested, Pending, Refused, Admitted, Locking, Aligning, Live, Interrupted, Dead, Ended}; discovery map never above 64 entries (C-34); no election while sync replies flow; never more than 64 followers served by one holder (C-35); Pending never left by a timeout (C-45); a Demoted input never leaves Demoted except by re-admission (C-109); the alignment offset never exceeds A (C-53, C-163); an other-epoch slice never counts as a miss (C-135).

## Work
1. Add one target per state machine in the tables that decodes the fuzz input as a sequence of `(time delta, ControlInput)` with non-decreasing time.
2. Assert the invariants of Context after every step.
3. Seed each corpus from the scenario vectors of that machine.

## Acceptance criteria
1. One target exists per state machine in the tables.
2. Each has a non-empty seed corpus from `vectors/control/`.
3. Each runs 60 s locally against a stub without harness errors.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed

## Out of scope
- CI scheduling and corpus keeping (007b-21).
