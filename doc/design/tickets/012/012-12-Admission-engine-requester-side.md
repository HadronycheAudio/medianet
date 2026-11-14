# 012-12 — Admission engine requester side
**Epic:** 012-Admission-handshake · **Effort:** M · **Depends on:** 012-02, 012-06, 012-08, 012-09, 012-11, epic 011

## Context
Epic 012's `Admission<const CONNS: usize>` holds both roles (requester/source side and consumer side) in a fixed-size table, `no_std`, no allocation. This ticket implements the requester path.

Rules `[decision]`:
- Connection is two-sided: the requester names a source output (or bundle members, at most 32, C-08) and a consumer input; both sides accept or refuse.
- The session number belongs to the source, one per stream, bumped on restart and on timeline epoch change (C-19). The request carries the source's current session of each requested stream; the consumer does not allocate one, and `Accept` carries none.
- The sender checks wire bit/s (including headers) against its own interface with the 70 % ceiling (C-23); failure → `Refused(PortCeiling)`, terminal, no retry.
- "Target not present" is not a refusal: the request stays **Pending**, visible and unsatisfied, with no timeout (C-45).
- **L-5:** destinations are pinned at admission and change only by re-admission. Addresses are IPv4 (C-100).
- When every `CONNS` slot is used, `request` returns `Err(RequestError::TableFull)` and sends nothing; no refusal reason is used, since no peer refused (epic 012 handshake rules).
- Every message carries the authentication reservation with scheme 0, zero tag and the live counter (C-115).
- The POC behaviour being retired `[code]`: the output owner resolved the target's address from the gossip map and started sending without the receiver's consent (`module.rs:775-820`); a missing target was waited for indefinitely (`"{to}?"` labels).

## Work
- `pub struct Admission<const CONNS: usize>` and `pub fn request(&mut self, now_ns: i64, req: Request<'_>, map: &PeerMap, out: &mut Outbox) -> Result<ConnId, RequestError>`.
  - Allocate a slot (Requested). If the target consumer is absent from `map` → Pending, emit `ConnEvent::State(id, Pending)` on the next poll.
  - Otherwise run local checks (`PortBudget::can_admit`, peer checks of 012-06), then encode a request with the member list and their sessions (012-08) into `out` and start an `MtuProbe` (012-09) at `intended_packet_bytes`.
- Handle in `on_message` the accept and refuse replies for requester-side slots: on accept record `Accept` (pinned addresses, reverse port, path MTU) and commit the budget → Admitted; on refuse → `Refused(reason)`, no re-send.
- Keep the sessions of the request in the slot so epic 014 can read them with the reverse port.

## Acceptance criteria
1. Request to a present consumer emits exactly one request message and one MTU probe into `out`; state Requested; the request carries one session per requested stream.
2. Request to an absent consumer gives Pending, sends nothing.
3. Accept reply → Admitted with every `Accept` field stored (including `group_a_ns`, used by 012-26, C-142); budget committed.
4. Refuse reply with reason R → `Refused(R)`; no further message is ever sent for that `ConnId` over 60 simulated seconds of ticks.
5. Third 338 Mbit/s request on a 1 Gbit/s sender interface with two admitted (676 Mbit/s, C-92) → `Refused(PortCeiling)` locally, no message sent.
6. With all `CONNS` slots used, `request` returns `Err(RequestError::TableFull)` and `out` is unchanged.
7. No allocation; builds in the `no_std` job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: decoding is 012-08, fuzzed by 012-21
- [ ] Regime stamp fields touched are complete and documented — not applicable: the stamp record is 012-23
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: behaviour follows the epic
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable here: requester transitions reach Admitted at most; the property is tested in 012-16
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Consumer side (012-13). Probe timeout and Pending re-check (012-14). Inherited A (012-26). Control-degraded gate (012-27). Cancel (012-24). Data-plane evidence (012-15). Graph view (012-18). Session bumps during a connection (epic 005).
