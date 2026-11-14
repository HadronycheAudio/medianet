# 029-12 — Clock panel
**Epic:** 029-Monitor · **Effort:** XS · **Depends on:** 029-07, 029-10

## Context
- `hadronyche-monitor` (epic 029) panel **Clock**: per module lock state and bound; holdover countdown to lapse.
- Clock states: unlocked, locking, locked (with bound, target at most 25 µs, C-62), holdover (bound grows 1 µs/s, C-64; alignment claim lapses after about 25 s, C-65, that is when the bound reaches the reference E of 50 µs, C-70). After the lapse the state is Unlocked with time unusable (C-162).
- Data comes from `clock_view` (029-07), whose countdown is `(reference − published bound) / C-64` from the stamp's time, the reference being C-70, or 2 × C-62 for a module whose E is unknown (C-169). The POC's lock colour idea `[code]` (`src/tui.rs`) may be reused.

## Work
1. `views/clock.rs` implementing `Panel`: one row per module with state, bound (µs) and its age, discipline mode, timeline epoch, and for holdover a countdown "alignment lapses in N s".
2. Colour: locked within 25 µs normal; locked over 25 µs warning; locking / holdover warning; unlocked and lapsed error.
3. Text capture for the documentation.

## Acceptance criteria
1. Fixture: module A locked 9 µs; B in holdover with published bound 25 µs, 15 s after that stamp; C unlocked → rows show "locked 9 µs", "holdover, lapses in 10 s", "unlocked".
2. Bound 26 µs is rendered in the warning style.
3. A holdover module whose countdown has reached 0 shows "unlocked (holdover lapsed)" in the error style (C-162).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] A screenshot or text capture of each panel is in the tool's documentation
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Clock state derivation (029-07); one-way latency blanking (029-08, shown on 029-13).
