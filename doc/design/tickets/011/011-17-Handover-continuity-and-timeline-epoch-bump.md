# 011-17 — Handover continuity and timeline epoch bump
**Epic:** 011-Gossip-and-discovery-core · **Effort:** S · **Depends on:** 011-15, epic 008

## Context
Slices are addressed by *position* on a domain-wide timeline; the timeline epoch (16 bits, C-10) is carried in the hello. Rules `[decision]`:
- **Handover** is frequency-continuous: the new holder enters through the servo's takeover entry point from epic 008.
- C-213 (decided): candidates must set the holder-eligible bit and have usable time: Locked, or in holdover with the alignment claim not lapsed (C-65, C-169). Such a candidate takes over with frequency continuity and the timeline epoch is kept.
- C-113 (decided): when no candidate has usable time, the holder-eligible candidate with the lowest module id wins and bumps the timeline epoch — an announced discontinuity. A module without the holder-eligible bit is never elected.

Epic AC 7: only candidates with the eligible bit set and usable time are elected while one exists; with no candidate with usable time, the lowest-id holder-eligible candidate wins and the result carries a timeline epoch bump.

Events: `ElectionEvent::HolderChanged { holder, epoch, continuous: bool }`, `ElectionEvent::TimelineEpochBump`. The two are delivered in that order: `step` returns `HolderChanged`, and `next_event()` returns `TimelineEpochBump` (epic event-queue rule).

## Work
1. When a winner is decided and had usable time (Locked, or in holdover with its claim standing, C-213): emit `HolderChanged { continuous: true }`, keep the timeline epoch; the winner (if self) calls epic 008's frequency-continuous takeover entry point via an outbox message.
2. When no candidate with usable time exists: choose the lowest module id among `fallback_candidates` (011-14), emit `HolderChanged { continuous: false }` then `TimelineEpochBump`; the new holder increments the timeline epoch it advertises by 1 (16-bit, wrapping per C-10).
3. When no holder-eligible module exists at all, elect nobody and make the condition observable to the caller.

## Acceptance criteria
1. With at least one eligible candidate with usable time, the result is `continuous: true`, no `TimelineEpochBump`, and the timeline epoch is unchanged; this holds also when the only such candidate is in holdover with its alignment claim standing (C-213).
2. With eligible candidates none of which has usable time (all Unlocked, C-162), the winner is the one with the lowest module id, `step` returns `HolderChanged { continuous: false }`, `next_event()` returns `TimelineEpochBump`, and the advertised timeline epoch increases by exactly 1.
3. Over 1 000 random candidate sets, no module without the holder-eligible bit is ever elected; whenever an eligible candidate with usable time exists, the winner has usable time and is chosen by C-221 (Locked before holdover, then lowest module id); whenever none has, every simulated module picks the same lowest-id winner.
4. With no eligible module at all, no holder is elected and the condition is observable to the caller.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] Design documents updated where behaviour differs, constants table first

Not applicable (C-125): "Every parser touched is fuzzed" (no new parser); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "The frozen POC is not modified" (no code near the POC); "No election path reachable from hello expiry" (acts only after an election is open; covered by 011-15 and 011-19).

## Out of scope
- The servo's takeover implementation: epic 008.
- Applying the timeline epoch to slice positions: data-plane epics.
- The timeline epoch bump on a holder stall (monotonic clock diverging from the suspend-aware clock by more than 2 ms, C-136): detected outside the election, by the holder's clock side (epics 008, 015).
