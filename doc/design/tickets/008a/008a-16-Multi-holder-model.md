# 008a-16 — Multi-holder model
**Epic:** 008a-Servo-core-and-simulation-harness · **Effort:** S · **Depends on:** 008a-14, 008a-15, epic 006
## Context
The `sim` module of the `hadronyche-tools` crate must contain a multi-holder model for handover and merge. Linux hosts are the only holders in v1; modules agree on one clock holder `[decision]`. Handover must be frequency-continuous: the new holder starts at the old holder's rate and relaxes to its own crystal within C-61 (1 ppm/s) `[decision]`. A candidate is a holder-eligible node whose time is usable: Locked, or in Holdover with its alignment claim not lapsed; it takes over with frequency continuity and the epoch kept (C-213) `[decision]`. Among several candidates with usable time a Locked candidate is preferred over one in holdover, and among equals the lowest module id wins (C-221) `[decision]`. Only when no candidate has usable time does the holder-eligible candidate with the lowest module id win and bump the timeline epoch (C-113) `[decision]`. A handover run in which holdover was not observed is failed, not passed (C-212) `[decision]`. The handover/bump distinction and the merge rule come from epic 006; the merge rule is still a remaining gap owned by epic 006, so the merge event follows whatever epic 006 states and is kept as one replaceable function.

Servo API involved: `begin_holding(now_local_ns)` (follower becoming holder) and `rebase()` (follower whose holder changed, same epoch, same time).

## Work
1. Extend the runner (008a-15) to N nodes, each with its own clock model, module id and servo; one node is holder at a time; links per node pair.
2. Scripted handover event as a holder loss (C-212): old holder stops answering; after a takeover delay, default 1.9 s (C-37 election silence plus C-33 election window), during which followers enter holdover (C-195), the designated successor calls `begin_holding`; remaining followers redirect requests to it and call `rebase`. The candidates are the nodes the servo's replaceable eligibility predicate accepts (008a-14); the successor among them is a Locked node if one exists, otherwise a node in holdover, and in either class the one with the lowest module id (C-221). The runner records whether each follower was observed in `Holdover` before takeover and reports the handover scenario failed if it was not (C-212). The successor keeps the epoch (C-213).
3. Scripted election with no candidate whose time is usable (C-113, C-213), for example a takeover delay beyond the lapse (C-65) so every follower is `Unlocked`: the lowest-id holder-eligible node becomes holder and bumps the epoch; followers see an epoch change.
4. Scripted merge event: two domains with different holders and epochs join; per epic 006's rule, one side adopts the other's epoch (an epoch change on that side).
5. Recorder per node; true error of each node measured against the original holder's time base until handover, then the new holder's.

## Acceptance criteria
1. Three-node scenario (holder + 2 followers, crystals at 0, +20, −30 ppm), holder loss at 60 s: all followers' traces recorded and no panic; runner reports the loss and takeover instants; with the default takeover delay every follower is reported in `Holdover` before takeover (C-195, C-212), the successor in `Holdover` takes over and every node keeps the same epoch (C-213), and at takeover each follower's rate changes within C-61 with no phase step (C-212). A run scripted with a takeover delay shorter than the holdover entry delay, so holdover is not observed, is reported failed (C-212).
2. C-113 scenario: with no follower whose time is usable (C-213), the lowest-id node becomes holder and every node publishes the bumped epoch.
3. C-221 scenario: in a scripted election where one candidate is `Locked` and a lower-id candidate is in `Holdover` (its link to the holder cut for longer than the holdover entry delay before the election), the `Locked` candidate takes over; in the holder-loss scenario of AC 1, where every candidate is in `Holdover`, the one with the lowest module id takes over.
4. Merge scenario: after merge, every node publishes the same epoch within 10 s (document actual).
5. Deterministic: same seed, same digest.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Assertions of case 6 (008b-06); the election protocol (epic 011).
