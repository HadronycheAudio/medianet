# 008a-13 — Servo step threshold and epoch change
**Epic:** 008a-Servo-core-and-simulation-harness · **Effort:** S · **Depends on:** 008a-11, epic 006
## Context
POC: an error above 2 ms on a wired network reset the servo `[code]`.

Requirement `[decision]`: a step larger than 2 ms (C-67, inherited, to be confirmed by case 2) is never slewed; it is a timeline epoch change. Every reply carries the timeline epoch (16 bits, C-10); the timeline epoch type and the handover/bump distinction come from epic 006. After an epoch change the estimator restarts, so re-lock is an acquisition bounded by C-63 (10 s) from the first exchange of the new epoch.

The same threshold governs holder stall detection: a holder whose monotonic clock diverges from a clock that keeps running through suspend by more than C-67 bumps the timeline epoch (C-136); the servo sees that as an ordinary epoch change. Simulation case 2 (008b-02) measures re-lock after a step, case 17 (008b-20) after a holder stall.

## Work
1. In `Servo::feed`/`tick`: if a measured offset differs from the prediction by more than `cfg.step_threshold_ns` (C-67, 2 000 000 ns) the servo does not slew towards it.
2. On an exchange carrying a new epoch: reset the estimator, adopt the new epoch, publish state `Acquiring` with the new epoch in `TimeView.epoch`; exchanges of the old epoch are then discarded.
3. On a >2 ms step with an unchanged epoch: discard the measurement, count it, and if it persists for a configurable number of windows, drop to `Unlocked` and await an epoch change (document the rule).
4. Make sure no published view ever mixes epochs (old offset with new epoch or vice versa).

## Acceptance criteria
1. Synthetic exchanges with a +3 ms step and a new epoch: the next published view has the new epoch and state `Acquiring`; it never shows a slew toward 3 ms under the old epoch.
2. A +3 ms step with an unchanged epoch never changes the correction term by more than C-61 (1 ppm/s) and never moves published time by more than the pre-step bound.
3. A +1.9 ms step with an unchanged epoch is handled as a normal measurement (no epoch change).
4. Builds for the 008a-01 no_std target.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Who decides to bump the epoch (epic 006; C-113 for an election with no candidate whose time is usable, C-213).
- Re-lock time measurement in simulation (008b-02).
