# 007b-13 — Transition coverage check
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** S · **Depends on:** 007b-01, 007b-02, 007b-03, 007b-04, 007b-05, 007b-06, 007b-07, 007b-08, 007b-09, 007b-10, 007b-11, 007b-12, 007b-24

## Context
- Epic criterion 007-AC3: every transition row of every control state machine is exercised by at least one scenario vector; coverage is checked mechanically.
- Epic 005 supplies the state tables for discovery (007b-01), election and clock holder (007b-02, 007b-03, 007b-04), admission (007b-05, 007b-06) and connection (007b-07) (C-193); the scenario tickets also cover plus behaviours whose owning machine epic 005 has still to name: alignment health (007b-08), connection alarms (007b-09), reassembly (007b-10), regime stamp distribution (007b-11; the message is defined by epic 013 and sent by epic 015, C-144) the commissioning window (007b-12) and a change of A while inputs are live (007b-24). The check must therefore read the list of machines from the tables, not from this ticket.
- Scenario vectors in `vectors/control/*.json` name their `machine` and list steps `(time_ns, input) → expected (outputs, events, state)`.

## Work
1. Have each scenario step identify the transition row it exercises (by row id from the tables), or derive it from (state, input) → next state.
2. Add a test that loads every transition table and every scenario, and fails listing each uncovered row per machine.
3. Print a coverage summary (rows covered / total, per machine) in CI.

## Acceptance criteria
1. 100 % of transition rows of all state machines in the tables are covered.
2. Deleting one scenario that is the only one covering a row makes the test fail, naming the machine and row.
3. A machine present in the tables but with no scenario file fails the test.
4. A scenario naming a machine absent from the tables fails the test.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Writing scenarios (007b-01 to 007b-12, 007b-24).
