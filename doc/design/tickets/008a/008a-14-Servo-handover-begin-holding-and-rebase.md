# 008a-14 — Servo handover: begin_holding and rebase
**Epic:** 008a-Servo-core-and-simulation-harness · **Effort:** S · **Depends on:** 008a-11, 008a-12, epic 006
## Context
POC handover: the new holder carried the phase on but switched to its own crystal, a rate step of up to the crystal difference `[code]`.

Requirement `[decision]`: handover is frequency-continuous — the new holder starts at the old holder's rate and relaxes to its own crystal within C-61 (1 ppm/s). Epic criterion A4 (verified in simulation by 008b, system AC-5; the unit part here is criterion A14 of 008a): handover at 50 ppm difference (C-78) stays within 1 ppm/s with no step. Holder eligibility, settled by C-213 `[decision]`: a holder-eligible module whose time is usable is a candidate, that is `Locked`, or in `Holdover` with its alignment claim not lapsed (C-65, C-169); such a candidate takes over with frequency continuity and the timeline epoch is kept (consumed by epic 011). Only when no candidate has usable time does the holder-eligible candidate with the lowest module id win and bump the timeline epoch (C-113), which the servo sees as an ordinary epoch change (008a-13).

Epic meaning of the two calls: `begin_holding` is used by the follower that becomes holder; `rebase` is used by a follower whose holder changed without an epoch change, and keeps published time and rate while dropping the measurement history tied to the old holder. The handover/bump distinction and the merge rule come from epic 006.

Handover through holdover (C-195, C-212) `[decision]`: a holder loss is silence, so followers enter holdover (entry delay at or below C-37), the election is called after 1.5 s (C-37) and runs 400 ms (C-33); the successor and the other followers are therefore in `Holdover`, not `Locked`, when they call `begin_holding` or `rebase` about 2 s after the loss, and the takeover must be frequency-continuous. A successor in `Holdover` whose claim has not lapsed is eligible (C-213), so this takeover keeps the epoch.

API (from the epic):
```rust
pub fn begin_holding(&mut self, now_local_ns: i64);   // frequency-continuous takeover
pub fn rebase(&mut self);                              // new holder, same time
```

## Work
1. `begin_holding`: the follower becoming holder freezes its current published time and rate as the network time definition, then ramps its rate towards its own crystal (rate deviation 0) at no more than C-61 (1 ppm/s); no phase step.
2. `rebase`: keeps published time and rate, resets the measurement history tied to the old holder, and continues tracking the new holder without a phase step and with the correction term within C-60/C-61.
3. Expose an `is_eligible_holder()` for epic 011: true when time is usable, that is `Locked`, or `Holdover` with the bound at or below the lapse reference (the `is_usable()` rule of 008a-12, C-213). Whether the module itself is holder-eligible (a Linux host in v1) is decided outside the servo.
3a. Both calls accept a servo in `Locked` or in `Holdover` (C-212): from `Holdover`, `begin_holding` starts from the held rate and `rebase` returns to tracking with the bound continuing from its holdover value; neither steps phase.
4. Document both behaviours on the methods.

## Acceptance criteria
1. Unit test: Locked servo with rate deviation +50 ppm calls `begin_holding`; published time is continuous (0 ns step) and rate changes by ≤ 1 ppm/s at every tick until it reaches the holder's own crystal rate (≈ 50 s for 50 ppm).
2. `rebase` produces a 0 ns phase step and no correction change above 1 ppm/s.
2a. The same two tests started from a servo that has been in `Holdover` for 2 s (C-212) give a 0 ns phase step and rate changes of at most 1 ppm/s.
3. A servo in `Locked`, or in `Holdover` with the bound at or below the lapse reference, reports eligible; a servo in `Acquiring` or `Unlocked` (including after the lapse) reports not eligible (C-213).
4. Builds for the 008a-01 no_std target.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Multi-holder simulation (008a-16), case 6 (008b-06), the poor-estimate question (008b-07).
- Election and candidate rules (epic 011).
