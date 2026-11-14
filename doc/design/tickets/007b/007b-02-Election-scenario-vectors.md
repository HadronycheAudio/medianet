# 007b-02 — Election scenario vectors
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** S · **Depends on:** 007a-04, epic 004, epic 005, epic 006

## Context
- Scenario vectors are ordered lists of `(time, input) → expected (outputs, events, resulting state)`, stored in `vectors/control/*.json`, driven through `UnderTest::control_step` (007a-04). Every transition row must be exercised (epic criterion 007-AC3). The election state table and scenario scripts come from epic 005 (C-193).
- Modules agree on one clock holder. Election rules `[decision]`:
  - an election starts on 1.5 s of sync-reply silence (C-37); the election window is 400 ms (C-33);
  - never elect on hello expiry while sync replies still flow (hello expiry is 3 s for non-live peers, C-31);
  - no rotation: a working clock holder is never replaced by a "better" candidate;
  - merge of two partitions: the side with the higher election epoch wins;
  - who may take over as holder (C-213): a holder-eligible module whose time is usable, that is Locked, or in holdover with its alignment claim not lapsed (C-65, C-169). Such a candidate takes over with frequency continuity and the timeline epoch is kept;
  - only when no candidate has usable time does the holder-eligible candidate with the lowest module id win and bump the timeline epoch (C-113, C-213).
- A timeline epoch change also bumps the session number of every stream of the source (C-19).
- The holdover entry delay is at most the 1.5 s silence threshold, a hard limit, so a follower is always already in holdover with a growing bound when it calls an election; the value within that limit is pending simulation (epic 008b) (C-195); election scenarios that script the follower's clock state show it in holdover at the call. The entry itself is tested in 007b-04.

## Work
1. Write scenarios in `vectors/control/election-*.json`:
   - silence of 1.5 s triggers an election; 1.5 s − 1 ns does not;
   - the window closes at 400 ms with the expected winner;
   - hello expiry while replies flow: no election;
   - a better candidate arrives while the holder is healthy: no rotation;
   - two partitions merge: higher election epoch wins, the other side follows;
   - takeover with usable time (C-213): the holder falls silent; the one holder-eligible candidate whose time is usable takes over, once Locked and once in holdover with its alignment claim not lapsed (the scenario states the figures from which the lapse is computed, as in 007b-04); a holder-eligible candidate with a lower module id whose claim has lapsed does not win; the timeline epoch is kept and no session number is bumped;
   - no candidate with usable time: among at least 3 holder-eligible candidates, all Unlocked (never locked, or holdover claim lapsed, C-162), the lowest module id wins and the expected output carries a bumped timeline epoch (C-113, C-213).
2. Add one scenario per remaining election transition row.

## Acceptance criteria
1. At least 8 scenarios, one per case in step 1 (the usable-time takeover once Locked and once in holdover).
2. The 1.5 s and 400 ms boundaries are each tested at the value and 1 ns before.
3. In the no-usable-time scenario, the winner is the lowest module id among holder-eligible candidates, a non-eligible candidate with a lower id does not win, and the timeline epoch is bumped by the winner (C-113).
4. Every election transition row is covered (007b-13 report).
5. In the usable-time scenarios, a candidate in holdover with its claim not lapsed counts as usable exactly as a Locked one; the expected output keeps the timeline epoch and every session number, and no scenario expects an epoch bump while a holder-eligible candidate has usable time (C-213).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Discovery (007b-01), holder follower cap and sync request rate (007b-03), holder stall and holdover lapse (007b-04), admission (007b-05), connection lifecycle (007b-07).
- Timeline position math and session bump on the wire (007b-14).
