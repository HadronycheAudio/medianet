# 007b-08 — Alignment health scenario vectors
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** M · **Depends on:** 007b-07

## Context
- Scenario vectors are ordered lists of `(time, input) → expected (outputs, events, resulting state)`, stored in `vectors/control/*.json`, driven through `UnderTest::control_step` (007a-04). Every transition row must be exercised (epic criterion 007-AC3).
- Settled rules `[decision]`:
  - late versus missing: at the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span it is reclassified as late (missing −1, late +1) and discarded (C-108);
  - the ring span is the presentation group's audition offset A: alignment rings are sized to A (C-96, C-55);
  - "persistently late" is at least one late slice in each of 10 consecutive 1 s windows (C-101), the 10 s dwell of C-54 expressed as a test; it triggers a ratchet step of one slice period, with A as the ceiling (C-53); when the gap between the offset and A is less than one slice period, the last step goes to A exactly (C-163);
  - a ratchet step is a re-alignment like any other: it takes effect at a position the fan-in announces one lead (64 slices, C-43) ahead in its own output, re-issued every 16 slices (C-44), never silently on the next round (C-215);
  - an input persistently late with the offset already at A enters the health state Demoted: treated as gapped, counted separately, cleared only by re-admission (C-109); the separate count is the per-connection demoted-rounds counter of the regime stamp, beside the late and missing counters (C-159).
- A slice period is 0.5 ms at defaults (C-02). A has no default (C-55); each scenario states the A it uses.
- Which control state machine holds the health state is an open sub-question of the epic owned by epic 005.

## Work
1. Write scenarios in `vectors/control/health-*.json`:
   - a slice absent at the deadline is counted missing; it arrives within the ring span: missing −1, late +1, content discarded; it arrives beyond the ring span: counts unchanged;
   - one late slice in each of 10 consecutive 1 s windows: one ratchet step of one slice period, announced in the fan-in's own output 64 slices ahead and taking effect at the announced position, with the offset unchanged until then (C-215); the same with one window clean (9 of 10): no step and no announcement;
   - repeated persistence: successive ratchet steps stop at A, the offset never exceeds A; with A not a whole number of slice periods above the starting offset, the last step is shorter than one slice period and lands on A exactly (C-163);
   - persistence with the offset at A: the input enters Demoted, its slices are treated as gapped and counted in the Demoted count;
   - Demoted persists through a clean period and clears only through re-admission.
2. Add one scenario per remaining transition row of alignment health in the tables.

## Acceptance criteria
1. The reclassification scenario expects missing −1 and late +1 within the ring span, and no change beyond it.
2. The ratchet triggers at 10 consecutive windows and not at 9.
3. No scenario expects an offset above A, and the last-step scenario expects the offset equal to A after a step smaller than one slice period.
4. The Demoted scenario expects gapped treatment, the demoted-rounds counter (C-159) rising, and no exit except re-admission.
5. All files carry `spec_version`.
6. Every ratchet step, the last one included, is preceded by an announcement in the fan-in's output 64 slices ahead and takes effect exactly at the announced position; no scenario expects a step on the round after the trigger without that announcement (C-215).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Initial alignment estimation and headroom (C-50 to C-52; simulation epic).
- Interrupted and Dead (007b-07).
- Reassembly deadlines (007b-10).
- The "cannot meet A" alarm and other connection alarms (007b-09).
