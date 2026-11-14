# 005-13 — Admission handshake states and inputs
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-02, epic 004

## Context
- Machine 3 of `doc/design/spec/control.md`: admission handshake. Pure table, no I/O, in `hadronyche-core`.
- Rules:
  - two-sided: the requester names a source output (or bundle members) and a consumer input; both sides accept or refuse;
  - refusal is explicit and terminal with a named reason (`RefuseReason`); "target not present" is not a refusal: the request stays Pending and visible with **no timeout**, until the target appears or the operator cancels (C-45);
  - "unsatisfied" is a displayed property, not a state: only Live is satisfied; Pending, Refused and every other state are shown by name as unsatisfied (C-154, C-45, system AC-12);
  - the exchange carries: declared rates, the bundle member list, the path-MTU probe pair (padded don't-fragment packet at the intended media packet size plus acknowledgement; the pair is exempt from the 1 252-byte control message bound (C-116, C-17); silence means smaller; fallback 1 280 bytes, giving 1 252 bytes of UDP payload on IPv4 (C-06, C-100); `PathMtuTooSmall` only when the probe at 1 280 bytes also fails (C-173)), the reverse measurement port, the pinned IPv4 addresses, and the manifest including the required E of a presentation group;
  - units checked: bit/s on the wire including headers (each side against its interface, per direction for a declared trunk (C-117), ceiling C-23 = 70 % of the link rate), packets/s, slice rate, input count, buffer memory (alignment rings sized to the group's audition offset A, C-96), compiler cost bound for DSL modules (abstract operations per slice, converted with the target's calibration factor from its regime stamp, C-106);
  - the bundle member list is checked: an unknown or duplicate member, or more than 32 members (C-08), is refused with `BadMemberList` (C-173); the controller member of a bundle source counts towards the 32 (C-08, C-209);
  - a sink whose E exceeds the group's required E is refused (`RequiredEExceeded`); only a commissioned sink has a numeric E, uncertified and self-monitored sinks declare E unknown (C-140); a sink with E unknown is refused with the same reason when the group sets a required E, admitted when it sets none (C-103);
  - destinations are pinned at admission and change only by re-admission (L-5); housekeeping admits unknown sources only and never touches a live data path (L-4);
  - a module whose housekeeping restarts have stopped (C-139) accepts no new connection; its live streams continue. Its housekeeping supervisor keeps a minimal refusal path and answers each new request with `ControlDegraded` (C-185), so the requester receives an explicit refusal;
  - fan-in A inheritance (C-142) and alignment opt-out (C-165) are added to this machine by 005-27; a fan-in with no downstream group and no budget has its aligned inputs refused with `NoSkewBudget` (C-185);
  - `VersionMismatch`, `NameTaken` and `ModuleCap` are raised at discovery, not at admission (C-185).
- `[code]` POC behaviour to retire: unilateral send (the output owner resolves the address and starts sending, `module.rs:775-820`) and waiting forever for a named target.
- Open sub-question owned by this ticket (epic 005): one admission machine per role (requester, source side, consumer side) or one joint machine. Falsifier: a two-sided refusal cannot be expressed as rows of a module-local table.
- Message names and fields come from the epic 004 wire specification, including the operator-cancel message.

## Work
1. Decide the machine split (per role or joint), record it in "Decisions on open sub-questions" with a `[decision]` tag and the falsifier.
2. In `control.md`, Machine 3 "States and data": define the states of each role (at least Idle, Requested, Pending, Probing-MTU, Awaiting-peer-verdict, Admitted, Refused, Cancelled), with data: request id, source output / bundle members, consumer input, declared rates (bit/s, packets/s, slice rate), MTU probe size and result, reverse measurement port, pinned IPv4 addresses and ports, manifest with required E (or "unknown").
3. "Memory bound": bytes per handshake and the number of concurrent handshakes allowed, giving the worst case with 63 peers (C-217).
4. "Input alphabet": admission request, accept, refuse (with reason), MTU probe, MTU acknowledgement, operator cancel (epic 004 names); timers: MTU probe silence; local: target appeared in the discovery map, local resource check results for each of the 6 units, peer regime stamp received (E and calibration factor, from 005-23). There is no Pending timer (C-45).
5. Describe the MTU fallback rule: the probe is sent at the intended media packet size (C-116); silence means smaller; the floor is 1 280 bytes; refuse with `PathMtuTooSmall` only when the probe at 1 280 bytes also fails (C-173).
6. Describe the member-list check (`BadMemberList`, C-173): unknown member, duplicate member, more than 32 members, the controller member included (C-08, C-209).
7. Mirror into `control.toml` under machine `admission`.

## Acceptance criteria
1. Each role's states are listed with data and byte sizes; a worst-case memory number with 63 peers (C-217) is stated.
2. The 6 checked units each map to exactly one `RefuseReason`, and the member-list check maps to `BadMemberList`: bit/s → `PortCeiling`, packets/s → `PacketRate`, slice rate → `SliceRate`, input count → `InputCount`, buffer memory → `BufferMemory`, cost bound → `CostBound`.
3. The 70 % ceiling is applied on each side against its own interface rate, per direction, headers included.
4. The MTU fallback is 1 280 bytes, the probe is not bounded by C-17, and `PathMtuTooSmall` requires the 1 280-byte probe to fail (C-173).
5. The alphabet has no Pending timeout; it has an operator-cancel input.
6. The machine-split decision is recorded with its falsifier.
7. `control.toml` declares the same states and inputs as the prose.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The transition table (005-14).
- Connection life after admission (005-15, 005-16).
- Implementation (epic 012).
