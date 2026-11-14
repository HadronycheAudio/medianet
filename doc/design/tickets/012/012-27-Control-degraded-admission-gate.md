# 012-27 — Control-degraded admission gate
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** 012-12, 012-13, 012-14, 012-15

## Context
C-139 `[decision]`: housekeeping restarts with exponential backoff from 100 ms to 10 s; after 5 panics within 60 s restarts stop. The module then stays control-degraded with an alarm, its live streams continue, and it accepts no new connection until an operator restarts it. The restart criterion and the panic counting belong to the hosted backend (epic 015, C-144); epic 015 tells the engine through `Admission::set_control_degraded(on: bool)` (epic deliverable). C-173 lists control-degraded among the events that raise a connection alarm.

L-1 and L-4 still hold: being control-degraded never tears down or changes an admitted connection; data-plane evidence keeps driving them (012-15).

Settled since this ticket was written (C-185) `[decision]`: a request reaching a control-degraded module is refused with `ControlDegraded` (one of the 18 reasons; 012-01). It is answered by the housekeeping supervisor, which keeps a minimal refusal path while housekeeping itself has stopped restarting. This replaces the provisional "leave the request unanswered". The refusal is terminal like any other; the requester sees `Refused(ControlDegraded)` with the refusal alarm (C-173). Who re-requests after the operator restart is the open question of re-requesting a wanted connection (epic 005).

## Work
- `set_control_degraded(on)`: store the flag. On entering the degraded state, emit `ConnEvent::Alarm(conn)` for every admitted, non-terminal connection (C-173), one per call as the draining rule of 012-14 requires.
- While degraded:
  - `request` returns `Err(RequestError::ControlDegraded)` and sends nothing;
  - an incoming request in `on_message` allocates no slot and is answered with refuse(`ControlDegraded`) (C-185);
  - `on_peer_map` moves no Pending request to Requested; Pending requests stay Pending and visible (C-45);
  - probe acknowledgements, accepts and refusals for connections already past Requested are still handled, and `on_data_plane`, `on_counters` and `stamp_records` work unchanged.
- `set_control_degraded(false)` (after the operator restart) restores normal admission; Pending requests are re-checked on the next `on_peer_map`.
- The supervisor's minimal refusal path (C-185): a stateless function in `hadronyche-core` (suggested `pub fn refuse_control_degraded(from: Addr, bytes: &[u8], out: &mut Outbox) -> bool`) that decodes a message with the 012-08 codec and, if it is a request, encodes refuse(`ControlDegraded`) into `out`; any other message, malformed or not, is ignored. It needs no `Admission` table, no allocation and no housekeeping state, so the supervisor can call it while housekeeping is stopped. Running it in the supervisor is epic 015's (C-144).
- Requester side: a received refuse(`ControlDegraded`) is handled like any refusal (012-12): `Refused(ControlDegraded)`, refusal alarm, no re-send.

## Acceptance criteria
1. 012-A18: with two Live connections, `set_control_degraded(true)` emits one alarm per connection, and both stay Live and keep accepting data-plane evidence.
2. While degraded, `request` returns `Err(RequestError::ControlDegraded)` with `out` unchanged, and an incoming valid request creates no slot and is answered with exactly one refuse(`ControlDegraded`) message (C-185).
3. While degraded, a Pending request whose target appears stays Pending; after `set_control_degraded(false)` it becomes Requested on the next `on_peer_map`.
4. No alarm is emitted on leaving the degraded state.
5. `refuse_control_degraded` answers a valid request with exactly one refuse(`ControlDegraded`) message and returns true; for any other message, including malformed bytes, it writes nothing and returns false. A requester engine receiving that refusal goes to `Refused(ControlDegraded)` with one alarm and sends nothing further.
6. No allocation; builds in the `no_std` job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — no new parser: the gate is exercised by the 012-21 target, and `refuse_control_degraded` (which calls the 012-08 decoder) gets its own 012-21 target
- [ ] Regime stamp fields touched are complete and documented — not applicable: the control-degraded state is stamped by epic 015
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection)
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Counting panics, the backoff and the restart criterion AC-20 (c) (epic 015, C-144). The panic-injection hook (C-145, used by epic 015's tests). The operator restart command (epics 015, 028). Running the minimal refusal path in the housekeeping supervisor (epic 015). Re-requesting after the operator restart (epic 005).
