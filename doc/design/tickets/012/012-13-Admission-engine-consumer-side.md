# 012-13 — Admission engine consumer side
**Epic:** 012-Admission-handshake · **Effort:** M · **Depends on:** 012-03, 012-04, 012-05, 012-06, 012-07, 012-08, 012-09, 012-11, 012-12

## Context
The consumer side of epic 012's two-sided handshake decides accept or refuse on a received request. Both sides must accept. Refusal is explicit and terminal, with a named `RefuseReason`; no retry loop `[decision]`.

Checks the receiver performs `[decision]`: bit/s on the wire including headers against its interface (70 % ceiling, C-23) → `PortCeiling`; packets/s → `PacketRate`; slice rate → `SliceRate`; input count, including a full connection table → `InputCount`; buffer memory (A × rate of the subscribed members, C-96, 012-04) → `BufferMemory`; cost bound in abstract operations converted with the receiver's calibration factor (C-106) → `CostBound`; manifest → `ContentMismatch`, `NotAnInput`, `BadMemberList` (unknown or duplicate member, or more than 32, C-173); peer → `ForeignDomain` (a different non-zero domain, C-173, 012-06); presentation group → `RequiredEExceeded` (C-103); path MTU → `PathMtuTooSmall` when the 1 280-byte probe also fails (C-173, 012-09). `FollowerCap` is raised by a clock holder through the 012-06 check (C-35), not by this path. `VersionMismatch`, `NameTaken` and `ModuleCap` are raised at discovery, not at admission (C-185), so this path never produces them. `NoSkewBudget` (C-142, C-185) is added to this path by 012-26 and `ControlDegraded` (C-139, C-185) by 012-27.

On accept, the consumer returns `Accept { pinned_source: Addr, pinned_dest: Addr, reverse_port: u16, path_mtu: u16, group_a_ns: Option<u32> }` with IPv4 addresses (C-100); `group_a_ns` is filled by 012-26 (C-142) and is `None` until then. It records the source's session of each requested stream from the request (C-19); it never allocates a session. **L-5:** destinations are pinned at admission and change only by re-admission. The reverse measurement port and the sessions are consumed by epic 014.

An unknown or duplicate member, or more than 32 members, is refused with `BadMemberList` (C-173, 012-05); it is no longer dropped as malformed.

## Work
- `pub fn on_message(&mut self, now_ns: i64, from: Addr, bytes: &[u8], out: &mut Outbox) -> Option<ConnEvent>`: decode (012-08); malformed → ignored, `None`.
- For a request: if no slot is free → refuse with `InputCount`. Otherwise run 012-06, 012-05, 012-07, 012-03 (with 012-04) checks; first failure → encode refuse(reason), slot state `Refused(reason)`. Success → allocate the reverse port, record the sessions, pin `from` as source and the local address as destination, answer the MTU probe acknowledgement (012-09), commit the receiver budget, encode accept → Admitted.
- Order of checks is fixed and documented.

## Acceptance criteria
1. A valid request yields one accept message with the pinned addresses, reverse port and path MTU set, the sessions recorded as sent, and state Admitted.
2. For each of the 12 receiver-checked reasons listed in Context (all but `FollowerCap`, the three discovery reasons and the two added by 012-26 and 012-27), a request violating only that check yields a refuse message with that reason and state `Refused(reason)`. `PathMtuTooSmall` is driven by dropping both the intended-size and the 1 280-byte probe (C-173).
3. A refused requester re-sending the same request gets the same refusal; the consumer never sends anything unsolicited after a refusal.
4. A received MTU probe of size s is acknowledged with size s.
5. Malformed bytes (any length 0..=1 500) return `None` and change no state; a request with a duplicate, unknown or 33rd member is refused with `BadMemberList` (C-173).
6. 012-A4 (AC-30, core part): a sink with E above the group's required E, or with unknown E in a group that sets a required E, is refused with `RequiredEExceeded` and its slot never leaves `Refused`; a sink with unknown E in a group that sets none is admitted (C-103).
7. With every slot used, a new request is refused with `InputCount`.
8. Two consumers of the same stream record the same session (C-19).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented — not applicable: the stamp record is 012-23
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: behaviour follows the epic
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable here: tested in 012-16 and 012-21
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Requester side (012-12). Source-filter and socket-buffer outputs (012-17). Fuzz target (012-21). `VersionMismatch`, `NameTaken`, `ModuleCap` (raised at discovery, epic 011, C-185). Inherited A and `NoSkewBudget` (012-26). Control-degraded gate and `ControlDegraded` (012-27).
