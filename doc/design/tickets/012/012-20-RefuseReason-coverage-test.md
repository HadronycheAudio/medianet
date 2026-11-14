# 012-20 — RefuseReason coverage test
**Epic:** 012-Admission-handshake · **Effort:** XS · **Depends on:** 012-19

## Context
Epic 012 criterion 012-A3: every `RefuseReason` raised by this epic is produced by at least one test. The 18 reasons `[decision]` (C-185): `PortCeiling`, `PacketRate`, `SliceRate`, `InputCount`, `BufferMemory`, `CostBound`, `ContentMismatch`, `NotAnInput`, `VersionMismatch`, `ForeignDomain`, `PathMtuTooSmall`, `RequiredEExceeded`, `FollowerCap`, `ModuleCap`, `NameTaken`, `BadMemberList` (C-173), `ControlDegraded`, `NoSkewBudget` (C-185). Refusal is explicit, terminal, named; no retry.

Settled: `FollowerCap` is the refusal of a 65th follower by a clock holder (C-35). It does not pass through the stream handshake, so it is covered through the 012-06 check with synthetic followers (C-114). Settled by C-173: `PathMtuTooSmall` when the 1 280-byte probe also fails; `ForeignDomain` for a different non-zero domain; `BadMemberList` for an unknown or duplicate member or more than 32 members. Settled by C-185: `ControlDegraded` for a request reaching a control-degraded module (012-27); `NoSkewBudget` for an aligned input of a fan-in with no downstream group and no configured budget (012-26); `VersionMismatch`, `NameTaken` and `ModuleCap` are raised at discovery, where they stop a module joining, not at admission, so their tests belong to epic 011 and this ticket only checks that admission never produces them.

## Work
- One end-to-end test per reason other than `FollowerCap`, `VersionMismatch`, `NameTaken` and `ModuleCap` through the full `Admission` engine (not only the unit check function), asserting the refuse message on the wire, `Refused(reason)` in the graph view and the refusal alarm (C-173).
- A `FollowerCap` test through `check_follower_cap` with 64 synthetic followers.
- A guard test that iterates `0u8..=17` via `RefuseReason::try_from` and fails if a variant has no registered test (a const list of covered reasons compared with all variants); the three discovery reasons are registered as "raised at discovery, epic 011" (C-185).
- A check that no refusal emitted across the 012-19 and 012-25 vectors and the end-to-end tests above carries `VersionMismatch`, `NameTaken` or `ModuleCap`.

## Acceptance criteria
1. 14 end-to-end tests (including `ControlDegraded` and `NoSkewBudget`) and the `FollowerCap` check test pass.
2. Adding a 19th variant without a test makes the guard test fail.
3. No admission test or vector produces `VersionMismatch`, `NameTaken` or `ModuleCap` (C-185).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: tests only
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag — not applicable: tests only, no factual claim
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable: refusals happen before Live
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- The check implementations (012-03 to 012-09, 012-12, 012-13, 012-26, 012-27). The clock holder's follower handling (clock epic). Tests of the discovery reasons (epic 011).
