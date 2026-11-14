# 005-14 — Admission handshake transition table
**Epic:** 005-Control-state-machines-specification · **Effort:** M · **Depends on:** 005-03, 005-04, 005-13, 005-23

## Context
- Machine 3 (admission handshake) of `doc/design/spec/control.md`; states, inputs and the per-role split from 005-13 in `doc/design/spec/control.toml` (machine `admission`); regime stamp inputs from 005-23; checker from 005-03.
- Rules to encode:
  - two-sided accept or refuse; refusal explicit and terminal with a named `RefuseReason` (`ConnectionRefused(reason)` event);
  - "target not present" → Pending and visible, not a refusal; no timeout; Pending → Requested when the target appears; Pending ends only by operator cancel (C-45); only Live is shown as satisfied, so Pending and Refused are shown by name as unsatisfied (C-154);
  - checks: bit/s on the wire incl. headers against each side's interface at C-23 = 70 %, per direction for a declared trunk (C-117) (`PortCeiling`), packets/s (`PacketRate`), slice rate (`SliceRate`), input count (`InputCount`), buffer memory with rings sized to A (C-96) (`BufferMemory`), DSL compiler cost bound in abstract operations per slice converted with the target's calibration factor (C-106) (`CostBound`); also `ContentMismatch`, `NotAnInput`, `PathMtuTooSmall` (the probe at the 1 280-byte fallback also fails, C-06, C-173), `RequiredEExceeded` (sink E above the group's required E, or E unknown when the group sets one, C-103; only commissioned sinks have a numeric E, C-140), `BadMemberList` (unknown or duplicate bundle member, or more than 32 members with the controller member counted, C-173, C-08, C-209), `NoSkewBudget` (aligned input of a fan-in with no downstream group and no budget, C-142, C-185; rows from 005-27), and `ControlDegraded`, received by the requester from a target whose housekeeping restarts have stopped (C-185); `VersionMismatch`, `NameTaken` and `ModuleCap` are discovery refusals, never produced here (C-185);
  - addresses are pinned at admission (L-5); housekeeping admits unknown sources only (L-4);
  - no unilateral send (`[code]` POC `module.rs:775-820` retired);
  - a module whose housekeeping restarts have stopped accepts no new connection (C-139); its housekeeping supervisor answers the request with `ControlDegraded` through a minimal refusal path (C-185), so the requester enters Refused with that reason.
- Open sub-question owned by this ticket (epic 005): the requester's rows when the peer never answers. The C-139 case is no longer one: it is an explicit `ControlDegraded` refusal (C-185). Falsifier: such a request is shown neither unsatisfied nor in a named state, or loops on retries.
- 005-AC-6 (system AC-12): a request exceeding 70 % of a port is refused with a named reason, the connection shows unsatisfied, and there is no retry loop.

## Work
1. Write the complete admission transition table in `control.md`: every (state, input) pair has a row, including `ignore`, with guard, new state, outputs, events, L-ids preserved.
2. Encode the rows in `control.toml`; checker passes.
3. Make the order of checks explicit (which check runs first when several fail) so that the named reason is deterministic.
4. State that a Refused request is terminal: no row leaves Refused, and no timer re-sends the request (no retry loop).
5. Write the requester's row for a `ControlDegraded` refusal (C-185): Refused, terminal, `ConnectionRefused(ControlDegraded)`. Decide the requester's rows when the peer never answers at all: which state holds the request, that it is shown unsatisfied (C-154), and that no retry loop arises; record it in "Decisions on open sub-questions" with a `[decision]` tag and the falsifier.
6. "Worst-case work per input": a number.

## Acceptance criteria
1. The checker reports 0 missing pairs for machine `admission`.
2. Each of the 13 admission-related reasons (`PortCeiling`, `PacketRate`, `SliceRate`, `InputCount`, `BufferMemory`, `CostBound`, `ContentMismatch`, `NotAnInput`, `PathMtuTooSmall`, `RequiredEExceeded`, `BadMemberList`, `NoSkewBudget`, and `ControlDegraded` received from the target's supervisor) appears in at least one row (C-185); no row produces `VersionMismatch`, `NameTaken` or `ModuleCap`.
3. A request at 71 % of the interface rate produces `ConnectionRefused(PortCeiling)`; at 70 % it is not refused for bit/s.
4. A sink with E unknown is refused with `RequiredEExceeded` when the group sets a required E and is not refused for E when the group sets none (C-103).
5. Refused has no outgoing transition and no row re-sends a request from it.
6. "Target not present" leads to Pending, never Refused; no timer row leaves Pending; operator cancel leaves it.
7. No row changes a pinned address after Admitted (L-5).
8. A request whose 1 280-byte probe is acknowledged is not refused with `PathMtuTooSmall` (C-173); a 33-member list (counting the controller member, C-209) and a duplicate member each yield `BadMemberList`.
9. A request to a module whose housekeeping restarts have stopped ends in Refused with `ControlDegraded` and no re-send (C-185); the never-answering-peer decision is recorded with its falsifier.

## Definition of done
- [ ] Transition tables are total (checked mechanically from the machine-readable form)
- [ ] Every new factual claim carries an evidence tag
- [ ] Tests green in CI, unprivileged

## Out of scope
- Post-admission connection states (005-16).
- Refusal scenarios (005-21); Pending scenario (005-25).
- Implementation (epic 012).
