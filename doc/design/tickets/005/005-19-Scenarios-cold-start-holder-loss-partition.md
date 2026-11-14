# 005-19 — Scenarios: cold start, holder loss, partition and heal
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-07, 005-10

## Context
- Deliverable 4 of epic 005: scenario scripts in prose, to become vectors in task 007. This ticket covers three of them: cold start of three hosts; holder loss; partition and heal.
- Facts: hello period C-30 = 1 s; hello expiry C-31 = 3 s (non-live peers only); listen-before-claim C-32 = 1.5 s; election window C-33 = 400 ms; election on reply silence C-37 = 1.5 s; sync requests 50/s per follower (C-36), accepted at 10 to 200/s (C-112). Linux hosts are the only clock holders in v1. Election triggers: 1.5 s without sync replies (never hello expiry, L-3, L-6), holder pressure, operator force. No rotation. Merge: higher election epoch wins; equal-epoch rule from 005-10. A candidate needs usable time: Locked, or in holdover with the alignment claim not lapsed (C-213, C-65, C-169); handover to such a candidate is frequency-continuous and keeps the timeline epoch (C-213); among several such candidates a Locked one is preferred over one in holdover, and among equals the lowest module id wins (C-221); with no candidate with usable time the lowest eligible module id wins and bumps the timeline epoch (C-113), which bumps every stream's session number (C-19).
- In holder loss, the holdover entry delay is at most C-37 = 1.5 s, a hard limit (its value is pending simulation, epic 008b), so each follower is always in holdover with a growing bound when it opens the election (C-195); a holder-eligible follower in holdover whose claim has not lapsed has usable time and takes over with the timeline epoch kept (C-213). As every follower of the lost holder is in holdover, none is Locked, so the lowest module id among the holder-eligible followers in holdover wins (C-221).
- At cold start no module has usable time (there is no holder yet), so S1 is the C-113 case.
- 005-AC-4 (system AC-36): partition heal converges to one holder within 400 ms + 1 s = 1.4 s, counted from restored connectivity.
- Machine tables: `doc/design/spec/control.toml` (machines `discovery`, `election`).

## Work
1. In the "Scenarios" section of `control.md`, write each script as a timed sequence of inputs per module with the expected states, outputs and events after each step, using only inputs present in the tables:
   - S1 cold start of three Linux hosts (all eligible, none with usable time): listen-before-claim, discovery, the lowest module id elected (C-113), `HolderChanged`, `TimelineEpochChanged`;
   - S2 holder loss: followers see 1.5 s reply silence → `ElectionOpened` → winner within 400 ms → frequency-continuous handover to the holder-eligible follower in holdover with its claim not lapsed that has the lowest module id among two such followers (C-221), timeline epoch kept (C-213, C-195); a variant in which no candidate has usable time elects the lowest eligible id with a timeline epoch bump (C-113);
   - S3 partition and heal: two sides each elect a holder; after heal the higher election epoch wins; convergence to one holder within 1.4 s of restored connectivity.
2. Give each script an id and a format directly translatable to a vector (time, module, input, expected next state, expected events).
3. Walk each script through the tables (by hand or with the explorer of 005-11) and record that the expected results match.

## Acceptance criteria
1. Three scripts S1, S2, S3 exist with timestamps in ms.
2. S1 elects the lowest module id and bumps the timeline epoch once.
3. S2 opens the election at 1.5 s after the last sync reply, not at hello expiry (3 s); every follower is in holdover by then (C-195), and the main case elects the lower-id of the two eligible followers in holdover (C-221) and emits `HolderChanged` with no `TimelineEpochChanged` (C-213).
4. S3 reaches exactly one holder at ≤ 1 400 ms after connectivity is restored.
5. Every input in the scripts exists in the machine-readable tables.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Tests green in CI, unprivileged

## Out of scope
- Id collision and gossip-robustness scenarios (005-20); refusal scenarios (005-21); Pending and demotion scenarios (005-25).
- Turning scripts into vectors (epic 007).
