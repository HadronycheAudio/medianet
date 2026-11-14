# 007b-24 — Live A change scenario vectors
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** S · **Depends on:** 007b-06, 007b-08

## Context
- Scenario vectors are ordered lists of `(time, input) → expected (outputs, events, resulting state)`, stored in `vectors/control/*.json`, driven through `UnderTest::control_step` (007a-04). Every transition row must be exercised (epic criterion 007-AC3). The transition tables come from epic 005 (C-193).
- A fan-in outside any presentation group inherits the A of the downstream presentation group it feeds, the largest if several, learned at admission; with no downstream group it needs an explicit budget in configuration (C-142) `[decision]`. Inheritance at admission is tested in 007b-06.
- A changed while inputs are live (C-186) `[decision]`: every fan-in that inherited the old A re-enters aligning at an announced position, with rings and ratchet ceiling resized, as the sinks do.
- Carrier of the change (C-198) `[decision]`: the receiver report (one per bundle every 100 ms, C-40) carries the consumer's current A, its own or the one it inherited. A change propagates upstream one hop per report, on the data plane, and each fan-in that inherited the old value re-aligns (C-186).
- Re-alignment position (C-208) `[decision]`: each fan-in that learns of the change picks its own position, one lead (64 slices, C-43) ahead, and announces it in its own output (re-issued every 16 slices, C-44), exactly as for a ratchet-triggered re-alignment. Fan-ins at different depths switch at different, announced positions.
- A larger than a fan-in's buffer memory (C-211) `[decision]`: the commissioning tool refuses a change of A that does not fit declared buffer memory; if one reaches a fan-in anyway, the fan-in keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm (C-174) and keeps its streams running.
- A clamped A in detail (C-216) `[decision]`: the cannot-meet-A alarm stays raised until a later change of A that fits; the clamped fan-in's receiver report (C-198) carries the A it was given, not the clamped value, so upstream modules size themselves to what the sinks need.
- Rings are sized to A (C-96); A is the ceiling of the ratchet (C-53), and the last step lands on A exactly (C-163). Alignment health scenarios at a fixed A are in 007b-08.
- A has no default (C-55); each scenario states the old and new A it uses.

## Work
1. Write scenarios in `vectors/control/a-change-*.json`:
   - a fan-in that inherited the A of the one group it feeds, inputs Live; the group's A is raised: the fan-in picks its position 64 slices after the report that carries the new A, announces it in its output, and re-enters aligning there (C-208); the group's sinks re-enter at their own announced position; and afterwards its ring span and ratchet ceiling equal the new A;
   - the same with A lowered: re-entry at the announced position, ring span and ratchet ceiling equal the new A, and no later expected offset exceeds the new A;
   - a fan-in running on an explicit configured budget (no inherited A): unaffected by the group's A change;
   - a chain of two inheriting fan-ins upstream of the group: the nearer fan-in learns the new A from the first receiver report from downstream that carries it, the farther one from the next receiver report of the nearer fan-in, which carries the inherited new A; each re-enters aligning only after the report that reaches it (C-198, C-40), at its own position one lead after it learned the change, so the two positions differ (C-208);
   - A raised beyond a fan-in's declared buffer memory (the scenario states the memory): the fan-in's ring span and ratchet ceiling are the deepest ring it can hold, the cannot-meet-A alarm is raised and its streams keep running (C-211); the alarm stays raised through later windows, including windows where the C-174 condition alone would not hold, and is cleared only by a later change of A that fits the memory (C-216); an inheriting fan-in upstream of the clamped one learns from the clamped fan-in's receiver report the A it was given, not the clamped value, and sizes its ring span and ratchet ceiling to that A (C-216, C-198).
2. Add one scenario per remaining A-change transition row in the tables.

## Acceptance criteria
1. Each A-change scenario expects the fan-in to re-enter aligning at the position it announced itself, 64 slices after it learned the change, not before it (C-208).
2. After re-entry, the expected ring span and ratchet ceiling equal the new A, for both a raised and a lowered A, except in the buffer-memory scenario, where they equal the deepest ring the fan-in can hold (C-211).
3. No expected offset after the change exceeds the new A.
4. Every scenario states the old and new A it uses.
5. Every A-change transition row is covered (007b-13 report).
6. All files carry `spec_version`.
7. In the chain scenario, each fan-in learns the new A only from a receiver report carrying it, one hop per report, and no fan-in re-enters aligning before that report reaches it (C-198); the two fan-ins announce and switch at different positions (C-208).
8. The buffer-memory scenario expects the cannot-meet-A alarm exactly once, no ring deeper than the stated memory allows, and no stream stopped (C-211).
9. In the buffer-memory scenario the alarm is still raised at every step until the later fitting change of A and cleared after it, and every receiver report the clamped fan-in sends carries the A it was given, never the clamped value (C-216).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Inheritance of A at admission and the NoSkewBudget refusal (007b-06).
- Ratchet and Demoted at a fixed A (007b-08).
- The commissioning tool's buffer-memory check before applying a change of A (C-211; the tool's epic).
- The encoding of A in the receiver report (epic 004, C-193; encodings in 007a).
