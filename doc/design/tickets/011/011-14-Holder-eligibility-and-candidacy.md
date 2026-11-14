# 011-14 — Holder eligibility and candidacy
**Epic:** 011-Gossip-and-discovery-core · **Effort:** S · **Depends on:** 011-06, epic 008a, epic 002

## Context
Only some modules may hold the clock. Rules `[decision]`:
- **Candidates** must set the holder-eligible capability bit, have **usable time** (C-213): Locked, or in holdover with its alignment claim not lapsed (C-65, C-169) (clock state published by the servo core, epic 008a, through the published time view, C-187; a module whose holdover claim has lapsed is Unlocked with time unusable, C-162, and so is not a candidate), be on a host not under I/O pressure, and be able to serve up to 64 followers (C-35).
- If no candidate has usable time, the holder-eligible candidate with the lowest module id wins and bumps the timeline epoch (C-113, C-213). The election flow for that case is 011-17; this ticket provides the predicate it needs.
- Embedded, macOS and Windows nodes do not set the bit in v1. Linux hosts are the only clock holders in v1. The reasons devices are excluded are serving load, availability (devices get power-cycled) and oscillator quality, not timestamp quality.
- A medianet needs at least one holder-capable node, or there is no audio.

`[code]` POC behaviour to keep: eligibility by I/O pressure read from the kernel's pressure-stall information (`module.rs:193-213`). The I/O-pressure rule rests on experiment data whose integrity task 002 checks. Open sub-question owned by task 002: is the I/O-pressure eligibility rule inherited? Falsifier: the POC experiment data behind it is corrupt or multi-writer. Keep the pressure input behind one flag so that 002's answer is a one-line change. Reading pressure-stall information is platform I/O and is not in core; core receives `under_pressure: bool`.

Shape from the epic:
```rust
pub struct Candidacy { pub eligible_bit: bool, pub time_usable: bool, pub under_pressure: bool, pub followers: u8 }   // time_usable: Locked, or holdover with the alignment claim standing (C-213)
```

## Work
1. Define `Candidacy` as above. `time_usable` is true when the published clock state is Locked, or holdover with the alignment claim not lapsed (C-213); false when Unlocked (C-162).
2. Implement `fn is_candidate(c: &Candidacy) -> bool` = `eligible_bit && time_usable && !under_pressure` (with the pressure term switchable per task 002) and the equivalent predicate over a `Peer` entry of the map, using the peer's advertised capability bits and clock state (usable time per C-213).
3. Implement `fn is_fallback_candidate(c: &Candidacy) -> bool` for the no-usable-time case of C-113: the same predicate without the `time_usable` term (the eligible bit is always required).
4. Implement `fn candidates(map: &PeerMap, me: &Candidacy) -> impl Iterator<Item = ModuleId>` and `fn fallback_candidates(...)` with no allocation.

## Acceptance criteria
1. Truth-table test over all 8 combinations of (eligible_bit, time_usable, under_pressure): only (true, true, false) is a candidate.
2. A holder-eligible module not under pressure in holdover with its alignment claim standing is a candidate; the same module after the claim lapses (Unlocked, C-162) is not, and is only a fallback candidate (C-213).
3. A map of 63 peers (C-217) of which 3 are eligible with usable time (one of them in holdover with its claim standing) yields exactly those 3 (plus self if eligible with usable time).
4. A peer without the holder-eligible bit is never yielded by either iterator, whatever its other state.
5. With eligible peers none of which has usable time, `fallback_candidates` yields every eligible peer, and `candidates` yields none.
6. Builds in the `no_std`, no-allocator CI job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

Not applicable (C-125): "Every parser touched is fuzzed" (no parser); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (reads the bit from the hello; the stamp field is 011-22); "Design documents updated" (behaviour follows the epic, C-113 and C-213); "The frozen POC is not modified" (POC cited, not touched); "No election path reachable from hello expiry" (predicates only, no transitions).

## Out of scope
- Reading the kernel's pressure-stall information: hosted platform (epic 015).
- Election flow: 011-15. Lowest-id choice and timeline epoch bump: 011-17. Follower cap enforcement: 011-18.
