# 012-14 — Tick probe timeout and pending recheck
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** 012-12, 012-13

## Context
Epic 012's engine has two time- or map-driven duties `[decision]`:
- `on_tick(&mut self, now_ns: i64, out: &mut Outbox) -> Option<ConnEvent>` drives the MTU probe timeout. Silence after a padded don't-fragment probe means failure only because an ack is expected; on failure probe again at 1 280 bytes (C-06) and continue admission at that size; if the 1 280-byte probe also fails, refuse with `PathMtuTooSmall` (C-173). The path MTU is exposed for the regime stamp (C-18, record in 012-23).
- `on_peer_map(&mut self, now_ns: i64, map: &PeerMap, out: &mut Outbox) -> Option<ConnEvent>` is how housekeeping (epic 015) passes peer-map changes to the engine. It may only move Pending requests to Requested; it never touches an admitted connection (L-4, L-1, L-5).
- "Target not present" yields Pending, visible and unsatisfied; Pending → Requested when the target appears (012-A10). Pending has no timeout: it persists until the target appears or the operator cancels (C-45).

## Work
- On each tick, drive every slot's `MtuProbe::on_tick`; when it asks for the 1 280-byte probe, encode and send it; on `Fallback(1280)` record `path_mtu = 1280` and proceed with admission; on `TooSmall` refuse with `PathMtuTooSmall` (C-173) and emit `ConnEvent::State(id, Refused(PathMtuTooSmall))`.
- In `on_peer_map`, for each Pending slot whose consumer is now in the map: Pending → Requested, send the request (as 012-12) and emit `ConnEvent::State(id, Requested)`.
- Return at most one event per call; further events are returned by later calls (document the draining rule).

## Acceptance criteria
1. 012-A5 (AC-19, core): with size-dependent drop injected above 1 280 bytes, the first probe times out, the 1 280-byte probe is acknowledged, the connection is admitted with `path_mtu == 1280`, and the value is readable for the stamp record. With drop at every size, the request is refused with `PathMtuTooSmall` one timeout after the 1 280-byte probe (C-173).
2. 012-A10: a Pending connection becomes Requested on the first `on_peer_map` after its target appears, and a request is sent exactly once.
3. A Pending connection with no target stays Pending over 600 simulated seconds of ticks (C-45: no timeout).
4. `on_peer_map` with any map content leaves every Admitted, Locking, Aligning, Live, Interrupted and Demoted slot unchanged.
5. Several simultaneous events are all delivered over successive calls, none lost.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: behaviour follows the epic
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection)
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Operator cancel (012-24). Re-probe heuristic (012-10). Data-plane transitions (012-15). How housekeeping calls `on_peer_map` (epic 015).
