# 007b-12 — Commissioning window scenario vectors
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** S · **Depends on:** 007a-04, 007a-08, 007a-09, epic 005

## Context
- Scenario vectors are ordered lists of `(time, input) → expected (outputs, events, resulting state)`, stored in `vectors/control/*.json`, driven through `UnderTest::control_step` (007a-04). Every transition row must be exercised (epic criterion 007-AC3).
- The commissioning tool may drive test traffic only inside a declared commissioning window that every module stamps; it is read-only outside it (C-107) `[decision]`.
- Life cycle (C-153) `[decision]`: the tool announces the window on the control group with an expiry time; modules stamp it while it is open; it closes at expiry or on explicit close. While a window is open, ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified; the commissioning tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results, each recorded with the window it was taken in (C-178).
- Encodings of the announcement and close are in 007a-08, the stamp field in 007a-09. Which control state machine holds the window is owned by epic 005; the `machine` field takes the name epic 005 gives.

## Work
1. Write scenarios in `vectors/control/commissioning-*.json`:
   - announcement with an expiry time: the module's next stamp carries the open window;
   - expiry: the window is still stamped 1 ns before the expiry time and no longer at it;
   - explicit close before expiry: the window is no longer stamped from the close on;
   - no announcement: no window is stamped;
   - disqualification (C-178): the latency, loss and alignment figures of each stamp issued while the window is open are unqualified (in the form the stamp field of 007a-09 gives), and those of stamps issued after close or expiry are qualified again; a commissioning measurement (D, restart spread, group certification or soak) taken while the window is open is a valid result recorded with that window.
2. Add one scenario per remaining commissioning-window transition row in the tables.

## Acceptance criteria
1. The open window is stamped after the announcement and before expiry or close.
2. Expiry is tested at the expiry time and 1 ns before.
3. An explicit close ends stamping before the announced expiry.
4. No scenario stamps a window without an announcement.
5. All files carry `spec_version`.
6. Every stamp issued while a window is open is expected with its production figures unqualified, and no stamp outside a window is; each commissioning measurement in a window is expected with that window recorded beside it (C-178).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Design documents updated where behaviour differs, constants table first

Not applicable (C-125): "Regime stamp fields touched are complete and documented": the scenarios read the commissioning window field defined by epic 004 and add or change none.

## Out of scope
- The commissioning tool itself and the qualification of measurements (other epics).
- Encoding of the window messages and the stamp (007a-08, 007a-09).
