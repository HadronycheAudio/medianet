# 005-10 — Election transition table
**Epic:** 005-Control-state-machines-specification · **Effort:** M · **Depends on:** 005-03, 005-04, 005-09

## Context
- Machine 2 (election) of `doc/design/spec/control.md`; states and inputs from 005-09 in `doc/design/spec/control.toml` (machine `election`); winner function from 005-08; checker from 005-03.
- Rules to encode:
  - trigger: 1.5 s without sync replies at a follower (C-37) opens an election regardless of hellos (L-3); hello expiry never opens an election while replies flow (L-6); also holder pressure or operator force (`ElectionOpened`);
  - no rotation: holder changes only on loss or pressure;
  - window 400 ms (C-33), deterministic winner (005-08): among candidates with usable time a Locked candidate is preferred over one in holdover, and among equals the lowest module id wins (C-221);
  - candidates need usable time: Locked, or in holdover with the alignment claim not lapsed (C-213, C-65, C-169); with no candidate with usable time the lowest eligible module id wins and bumps the timeline epoch (C-113);
  - merge: higher election epoch wins; rank not compared across epochs;
  - handover to a candidate with usable time is frequency-continuous (new holder starts at the old holder's rate) and keeps the timeline epoch (C-213); a follower is always in holdover when it calls an election (C-195), so after a holder loss such a follower is a candidate with usable time; with no candidate with usable time the new holder bumps the timeline epoch (`TimelineEpochChanged`); a holder detecting its own stall (C-136: monotonic clock against a suspend-proof clock, divergence above 2 ms, C-67) bumps the timeline epoch; holder change raises `HolderChanged`;
  - a lock-state input reporting a holdover lapse (C-162: Unlocked, time unusable) removes the module from the set of candidates with usable time;
  - follower cap 64 (C-35): the 65th is refused (`FollowerRefused`); the holder accepts 10 to 200 sync requests per second per follower (C-112); a request beyond 200 per second from one follower is dropped before any work is done and counted per follower in the holder's stamp (C-204);
  - timeline epoch switch travels in-band in slices (L-2): the table emits the decision; the in-band carriage is the data plane's.
- 005-AC-4 (system AC-36): partition heal converges to one holder within election window plus hello period: 400 ms + 1 s = 1.4 s, counted from the moment connectivity is restored; the hello-period term covers the wait for the first hello across the heal.
- Open sub-question owned by this ticket (epic 005): merge of equal election epochs naming different holders. Falsifier: the model check finds two holders after convergence with equal epochs.

## Work
1. Write the complete election transition table in `control.md`: every (state, input) pair has a row, including `ignore`, with guard, new state, outputs, events, L-ids preserved.
2. Encode the rows in `control.toml`; checker passes.
3. Write the merge rule as rows: on a hello or claim with a higher election epoch, adopt it; with lower epoch, ignore or answer; with equal epoch and a different holder, decide the behaviour, record it in "Decisions on open sub-questions" with a `[decision]` tag and the falsifier.
4. Show the partition-heal argument from the moment connectivity is restored: the higher-epoch side's hello reaches every module within one hello period (1 s), then at most one election window (400 ms) concludes: ≤ 1.4 s.
5. Write the over-rate row of the holder: a sync request failing the C-112 guard of 005-09 produces no reply, no state change and no event, and increments that follower's dropped-request counter (C-204) in the `counters` column.
6. "Worst-case work per input": a number with 64 candidates / 64 followers.

## Acceptance criteria
1. The checker reports 0 missing pairs for machine `election`.
2. No row opens an election from hello expiry (L-6); the only silence trigger is 1.5 s of sync-reply silence (C-37).
3. No row changes the holder except on reply-silence loss, holder pressure, operator force, or a merge (no rotation).
4. A 65th follower request produces `FollowerRefused` and the follower list stays at 64 (exercised with synthetic followers, C-114).
5. A window concluding with no candidate with usable time elects the lowest eligible id and emits `TimelineEpochChanged` (C-113); a window whose winner has usable time, including one in holdover with its claim not lapsed, emits `HolderChanged` and no `TimelineEpochChanged` (C-213). A window with a Locked candidate and a lower-id candidate in holdover elects the Locked one; among candidates of the same clock class it elects the lowest module id (C-221).
6. The written argument bounds partition-heal convergence at 1.4 s from restored connectivity.
7. The equal-epoch merge has a recorded decision with a falsifier.
8. Worst-case work per input is stated as a number.
9. The own-stall input (C-136) in the Holder state emits `TimelineEpochChanged`.
10. A sync request beyond 200 per second from one follower is answered by no reply and increments only that follower's dropped-request counter (C-204); requests within 10 to 200 per second are served (C-112).

## Definition of done
- [ ] Transition tables are total (checked mechanically from the machine-readable form)
- [ ] Every new factual claim carries an evidence tag
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The model check of single-holder convergence (005-11).
- Election scenarios (005-19).
- Implementation (epic 011).
