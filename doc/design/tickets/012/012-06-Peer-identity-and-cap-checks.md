# 012-06 — Peer identity and cap checks
**Epic:** 012-Admission-handshake · **Effort:** XS · **Depends on:** 012-01, epic 011

## Context
Epic 012 defines five refusal reasons tied to the peer rather than the stream: `VersionMismatch`, `ForeignDomain`, `FollowerCap`, `ModuleCap`, `NameTaken`. Of these, only `ForeignDomain` is checked at admission and `FollowerCap` by a clock holder; `VersionMismatch`, `NameTaken` and `ModuleCap` are raised at discovery, where they stop a module joining, not at admission (C-185). The peer map and capability bits come from epic 011. Refusal is explicit and terminal, with a named reason; no retry loop `[decision]`.

Settled:
- `FollowerCap` (C-35, C-114): a clock holder has at most 64 followers; a 65th is refused with this named reason. With at most 64 modules (C-34) a holder has at most 63 real followers, so the 64-follower load and the refusal of the 65th are tested with synthetic followers (C-114). This epic owns the reason code and the check; the holder calls it. The value's owner is L1 (task 019), which checks that holder serving lateness does not grow below 64 followers (C-35, C-167); the constant is the only thing to change if it moves.
- `ModuleCap` value (C-34): at most 64 active modules per medianet; commissioning refuses more.
- `VersionMismatch`, `NameTaken`, `ModuleCap` (C-185): raised at discovery (epic 011, which builds the peer map), where they stop a module joining; admission never produces them. A module stopped from joining is not in the peer map, so a request to it stays Pending (C-45). This replaces the provisional admission-time conditions this ticket used to implement.
- Domain id (C-11): 16 bits; domain 0 means unconfigured.
- `ForeignDomain` (C-173): the peer's domain is non-zero and differs from ours. A hello with domain 0 is ignored and counted, not refused (epic 011); such a peer never enters the peer map, so a request to it stays Pending (C-45).
- Module id and instance nonce (C-11): 64 / 64 bits. Third parties quarantine a colliding id until only one instance nonce has been seen for 3 s (C-46).

## Work
- `pub fn check_peer(local: &LocalIdentity, peer: &PeerEntry, map: &PeerMap) -> Result<(), RefuseReason>` in `hadronyche-core`, checking `ForeignDomain` only; it never returns `VersionMismatch`, `ModuleCap` or `NameTaken` (C-185).
- `pub fn check_follower_cap(current_followers: u8) -> Result<(), RefuseReason>` returning `FollowerCap` when admitting one more would exceed 64 (C-35).
- `pub const MAX_ACTIVE_MODULES: usize = 64; // C-34` and `pub const MAX_FOLLOWERS_PER_HOLDER: usize = 64; // C-35`.
- Module docs list each peer reason with its C-id and where it is raised (admission, clock holder or discovery, C-185).

## Acceptance criteria
1. `ForeignDomain` and `FollowerCap` are each produced by a dedicated unit test with only that condition violated.
2. A peer whose domain is non-zero and different is refused with `ForeignDomain`; a peer entry with domain 0 is never refused with `ForeignDomain` (C-173).
3. `check_follower_cap(63)` passes; `check_follower_cap(64)` returns `FollowerCap` (C-35), driven with synthetic followers (C-114).
4. A peer in our domain passes `check_peer`; no input to `check_peer` yields `VersionMismatch`, `ModuleCap` or `NameTaken` (C-185).
5. Each peer reason is documented in the module docs with its C-id and where it is raised.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: behaviour follows C-173 and C-185
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable: no transitions
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Building the peer map, hello parsing and quarantine, and raising `VersionMismatch`, `NameTaken` and `ModuleCap` at discovery (epic 011, C-185). The clock holder's follower handling (clock epic). Capacity checks (012-03). Wiring into the handshake (012-12, 012-13).
