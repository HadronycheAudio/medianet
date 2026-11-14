# 007b-10 — Multi-packet reassembly scenario vectors
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** S · **Depends on:** 007a-04, 007a-07

## Context
- The default slice fits one packet; a multi-packet slice is an explicit, stamped regime (C-05). Valid multi-packet encodings are in 007a-07.
- Reassembly deadlines `[decision]`:
  - an arrival-driven multi-packet input has a reassembly deadline of 2 slice periods (C-15), 1 ms at defaults (C-02);
  - an aligned multi-packet input uses the consumer's own deadline for that position (C-110);
  - a slice not completed by its deadline is counted in the per-connection incomplete-reassemblies counter of the regime stamp (C-159).
- Scenario vectors are ordered lists of `(time, input) → expected (outputs, events, resulting state)`, driven through `UnderTest::control_step` (007a-04). Which control state machine owns reassembly is an open sub-question of the epic owned by epic 005.

## Work
1. Write scenarios in `vectors/control/reassembly-*.json`:
   - arrival-driven input: all packets of a slice within 2 slice periods of the first: slice complete; last packet at 2 slice periods + 1 ns: slice not complete, incomplete-reassemblies counter +1 (C-159);
   - aligned input: the last packet arrives after 2 slice periods but before the consumer's deadline for that position: slice complete; after that deadline: slice absent at the deadline (counted missing, C-108);
   - packets of one slice arriving out of order: slice complete.
2. Add one scenario per remaining reassembly transition row in the tables.

## Acceptance criteria
1. The arrival-driven deadline is tested at 2 slice periods and just after.
2. The aligned case shows completion after 2 slice periods when still before the consumer's deadline.
3. Every slice not completed by its deadline raises the incomplete-reassemblies counter by exactly 1.
4. Out-of-order arrival completes the slice.
5. All files carry `spec_version`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Valid multi-packet encodings (007a-07). Late/missing reclassification beyond the deadline (007b-08).
- Bounded drain per input per cycle (C-16, where the cycle is one slice period, C-156; load test, not a conformance vector).
