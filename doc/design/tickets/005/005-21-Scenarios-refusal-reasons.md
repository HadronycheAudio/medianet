# 005-21 — Scenarios: refusal for each reason
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-07, 005-10, 005-14, 005-27

## Context
- Deliverable 4 of epic 005: a scenario script for refusal for each reason. 005-AC-6 (system AC-12): the refusal-reason enumeration is closed at 18 reasons (C-173, C-185) and each reason has a scenario; a request exceeding 70 % of a port (C-23) is refused with a named reason, the connection shows unsatisfied, no retry loop.
- The 18 `RefuseReason` variants and where they arise:
  - admission (machine 3): `PortCeiling` (bit/s on the wire incl. headers > 70 % of either side's interface, per direction, C-117), `PacketRate`, `SliceRate`, `InputCount`, `BufferMemory` (rings sized to A, C-96), `CostBound` (DSL compiler cost bound in abstract operations per slice, converted with the target's calibration factor, C-106), `ContentMismatch`, `NotAnInput`, `PathMtuTooSmall` (the probe at the 1 280-byte fallback also fails, C-06, C-100, C-173), `RequiredEExceeded` (sink E above the group's required E, or E unknown when the group sets one, C-103; only commissioned sinks have a numeric E, C-140), `BadMemberList` (unknown or duplicate bundle member, or more than 32 members with the controller member counted, C-173, C-08, C-209), `NoSkewBudget` (aligned input of a fan-in with no downstream group and no configured budget, C-142, C-185), `ControlDegraded` (the target's housekeeping restarts have stopped, C-139; answered by its housekeeping supervisor, C-185);
  - discovery (machine 1), where they stop a module joining, not at admission (C-185): `VersionMismatch`, `ForeignDomain` (a different non-zero domain; domain 16 bits, 0 = unconfigured, C-11; a domain 0 hello is ignored and counted, not refused, C-173), `ModuleCap` (65th module, C-34 = 64; the map is full at 63 peers, C-217), `NameTaken` (listen-before-claim 1.5 s, C-32);
  - election (machine 2): `FollowerCap` (65th follower, C-35 = 64). With at most 64 modules a holder has at most 63 real followers, so this script uses synthetic followers (C-114).
- The event each non-admission refusal emits is decided in 005-04.
- "Target not present" is not a refusal (Pending, covered by 005-25).
- Capacity figures for `PortCeiling` (derived constants): one 96-channel source is 16 streams, 338 Mbit/s (C-22); two such bundles on one GbE port are 676 Mbit/s and a third is refused (C-92); the GbE ceiling is 700 Mbit/s (C-23).

## Work
1. In the "Scenarios" section of `control.md`, write one short timed script per reason (R1–R18): setup, the triggering input, expected state (Refused or the machine's equivalent), and the event decided in 005-04.
2. For `PortCeiling`, use the C-92 case: on a GbE port, two 96-channel bundles admitted (676 Mbit/s ≤ 700 Mbit/s), a third bundle refused (1 014 Mbit/s > 700 Mbit/s). Show the third connection stays Refused (unsatisfied) and no request is re-sent over at least 10 s.
3. For `RequiredEExceeded`, include two cases: commissioned E above the required E, and E unknown (an uncertified or self-monitored sink, C-140) in a group that sets a required E; and show that an E-unknown sink in a group with no required E is admitted (C-103).
4. For `PathMtuTooSmall`, show that a path whose intended-size probe fails but whose 1 280-byte probe succeeds is admitted at the fallback, and that refusal follows only when the 1 280-byte probe also fails (C-173). For `ForeignDomain`, add a domain 0 hello that is ignored and counted without a refusal (C-173). For `BadMemberList`, use a duplicate member and a 33-member list (32 audio members plus the controller member, C-209). For `ControlDegraded`, drive the target's housekeeping to 5 panics within 60 s (C-139) and show the requester refused by the supervisor with no re-send (C-185). For `NoSkewBudget`, reuse the S13 setup of 005-27.
5. Add a closedness check: a CI test that every `RefuseReason` variant appears in at least one script.

## Acceptance criteria
1. Exactly 18 scripts, one per `RefuseReason` variant (C-185).
2. The `PortCeiling` script admits two 96-channel bundles on one GbE port, refuses the third, and shows 0 re-sent requests over 10 000 ms.
3. The `RequiredEExceeded` script covers the E-unknown case of C-103 and C-140.
4. The `PathMtuTooSmall` script refuses only after the 1 280-byte probe fails; the domain 0 hello produces no refusal (C-173).
5. The `FollowerCap` script uses 64 followers of which at least one is synthetic, and refuses the 65th.
6. The CI test fails if a variant is added without a script.
7. Every input used exists in the machine-readable tables.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Tests green in CI, unprivileged

## Out of scope
- Other scenarios (005-19, 005-20, 005-25).
- Vector generation (epic 007).
