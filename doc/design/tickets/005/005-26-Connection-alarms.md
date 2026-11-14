# 005-26 — Connection alarms
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-04, 005-16, 005-28

## Context
- Machine 4 (connection states) of `doc/design/spec/control.md`; tables in `doc/design/spec/control.toml` (`connection_consumer`, `connection_producer`, written by 005-16); admission rows from 005-14; enumerations from 005-04.
- C-173: a connection alarm is raised on refusal, interrupted, dead, demoted, cannot-meet-A and control-degraded.
- C-174: "cannot meet A" is maximum lateness + D + headroom exceeding A in each of 10 consecutive 1 s windows. The maximum lateness is taken over a rolling 60 s window (C-182). The figures come from the alignment stage of the data plane; for the connection machine it is a local input that changes no state.
- Control-degraded is raised by the housekeeping supervisor, not by a table row (epic decision); under C-139 a module whose restarts have stopped stays control-degraded with an alarm. Its supervisor answers new requests with the `ControlDegraded` refusal reason (C-185); at the requester that is an entry into Refused like any other, and raises the refusal alarm.
- A fan-in whose rings cannot hold a changed A keeps the deepest ring it can hold as its ceiling and raises the cannot-meet-A alarm (C-211); that row is written by 005-28 and carries `alarm = true`.
- Four of the six causes are already rows: entry into Refused (005-14), Interrupted, Dead and Demoted (005-16).
- Open sub-question owned by this ticket (epic 005): how an alarm is carried, as a new `ControlEvent` variant or derived from the existing `ConnectionRefused` and `ConnectionState` events. Falsifier: an alarm cause of C-173 has no row or emitter that raises it.

## Work
1. Decide the alarm carrier; record it in "Decisions on open sub-questions" with a `[decision]` tag and the falsifier. If a `ControlEvent` variant is added, make the documented spec change in 005-04's Enumerations section.
2. Set `alarm = true` (schema of 005-02) on every row entering Refused, Interrupted, Dead or Demoted.
3. Add the local input "cannot meet A" (C-174) to `connection_consumer`, with a row in every state: in Live (and any other state the input can reach) it raises the alarm and keeps the state; elsewhere `ignore`.
4. Write the control-degraded alarm in the Enumerations or Invariant preservation section as raised by the housekeeping supervisor (L-7, C-139), with no table row.
5. Add a checker rule: for each of the six C-173 causes, at least one row or the named supervisor raises it.
6. Give a boundary case for C-174: 9 consecutive windows over A raise no alarm; 10 raise one.

## Acceptance criteria
1. Each of the 6 C-173 causes maps to rows or to the housekeeping supervisor, listed in one table in `control.md`; cannot-meet-A lists both the C-174 input rows and the 005-28 ring-fit row (C-211).
2. The cannot-meet-A input changes no connection state in any row.
3. The checker rule fails on a mutation fixture that clears `alarm` on the row entering Dead.
4. The carrier decision is recorded with its falsifier; the checker of 005-03 passes.

## Definition of done
- [ ] Transition tables are total (checked mechanically from the machine-readable form)
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Computing lateness, D and headroom (alignment, other epics).
- Alarm presentation to the operator and the housekeeping supervisor implementation (epic 015, C-144).
