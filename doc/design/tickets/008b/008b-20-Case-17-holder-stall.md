# 008b-20 — Case 17: holder stall
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** S · **Depends on:** 008a-05, 008a-13, 008a-15
## Context
Rule C-136 `[decision]`: the holder compares its monotonic clock with a clock that keeps running through suspend; a divergence above the step threshold (2 ms, C-67) bumps the timeline epoch. The holder's monotonic clock defines network time `[decision]`, so a holder that stalls (for example through a suspend) makes network time stand still against true time; on resume, followers see a step of the stall length.

A step larger than 2 ms (C-67) is never slewed; it is a timeline epoch change, and re-lock is an acquisition bounded by C-63 (10 s) from the first exchange of the new epoch `[decision]`. While the holder is stalled it sends no replies, so a long stall also exercises holdover (C-64) and, beyond about 25 s, the lapse to `Unlocked` with time unusable (C-65, C-162).

The servo side of an epoch change exists (008a-13); this ticket adds the stall to the models and checks the servo's behaviour across it. The production holder-side check is outside this epic.

## Work
1. Extend the two-clock model (008a-05) so a holder's monotonic clock can pause for a scheduled interval while a second holder clock that keeps running through suspend does not, and add a scheduled "holder stall" event to the runner (008a-15).
2. In the simulated holder, apply the C-136 rule on resume: compare the two clocks and bump the timeline epoch when the divergence exceeds `step_threshold_ns` (C-67, read from configuration, not a literal).
3. Named test `case_17_holder_stall`: after lock under `WIRED_PROFILE` (C-94), holder stalls of 1 ms, 2.5 ms, 100 ms, 5 s and 40 s; at least 20 seeds.
4. Register the scenario in the shared scenario list used by 008b-12 and 008b-17, and write the outcome to `doc/design/reports/008-simulation-results.md` with tag `[measured]` (simulation), for 008b-18.

## Acceptance criteria
1. A stall whose divergence exceeds C-67 bumps the epoch exactly once; a 1 ms stall bumps nothing.
2. For every bumping stall: no published view mixes epochs, no view of the old epoch shows a phase movement toward the step, and followers are `Locked` again within 10 s (C-63, or its replacement from 008b-01) of the first exchange of the new epoch, for all seeds.
3. For the 1 ms stall: no epoch change; the follower absorbs it by frequency only with the correction term within C-60/C-61, or drops out of `Locked` honestly.
4. For the 40 s stall: followers are in `Holdover` with the bound growing at 1 µs/s, then `Unlocked` with `is_usable()` false after the lapse (C-65, C-162), before the bump.
5. 0 honest-lock violations in every run.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The production holder-side stall check and the choice of the suspend-aware clock on each platform.
- Who bumps the epoch in other situations (epic 006; C-113 for an election).
