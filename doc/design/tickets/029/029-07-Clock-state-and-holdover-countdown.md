# 029-07 — Clock state and holdover countdown
**Epic:** 029-Monitor · **Effort:** XS · **Depends on:** 029-06, epic 008a

## Context
- `hadronyche-monitor` (epic 029) shows each module's clock state from the Clock group of its regime stamp: discipline mode, lock state and bound, timeline epoch.
- The lock state and bound are published by the servo core (epic 008a) through the published time view, and reach the monitor in the stamp (C-187).
- Clock states: **unlocked**, **locking**, **locked** (with bound; target at most 25 µs, C-62), **holdover** (bound grows 1 µs/s, C-64; the alignment claim lapses after about 25 s, C-65).
- After the claim lapses the clock state is **Unlocked**, with time unusable: one-way figures and alignment claims stop (C-162). The runtime publishes that as a state change, so a new stamp follows (C-18).
- The lapse is when the bound reaches the reference E of 50 µs (C-70): C-65 is (C-70 − C-62) / C-64. So the countdown follows from the published bound, `(C-70 − bound) / C-64`, counted down from the time of the stamp that published the bound; no holdover start time is needed. C-62 is pending simulation and C-70 pending measurement, so both are read from one constants module.
- For a module whose E is unknown (C-103, C-140) the lapse rule takes the reference as twice the lock bound, 2 × C-62 = 50 µs (C-169). The value equals C-70 today, but the two follow different pending constants, so the monitor keeps them apart.
- Stamps arrive on any regime change and every 10 s (C-18); each refresh gives a new published bound and restarts the countdown from it.
- Timeline epoch changes only on a timebase discontinuity (including an election won with no locked candidate, C-113, and a holder stall, C-136); it is distinct from the election epoch.

## Work
1. In `model.rs` (or `model/clock.rs`) add `fn clock_view(&self, module, now) -> ClockView` with: state enum (Unlocked, Locking, Locked, Holdover), bound as published, `over_target: bool` (bound > 25 µs, C-62), discipline mode, timeline epoch. `now` is the time of the latest observation (029-10).
2. For Holdover: `lapse_in: Duration = max(0, (reference − published_bound) / HOLDOVER_GROWTH − (now − stamp_time))`; once 0, the view reports state `Unlocked` with `lapsed = true` (C-162) until the next stamp confirms it, and one-way figures for the module are blanked (029-08). The bound shown is the published bound, labelled with its age.
   `reference` is `REFERENCE_E` for a module with a numeric E and `REFERENCE_E_UNKNOWN` for a module whose E is unknown (C-169).
3. Constants `TARGET_BOUND = 25 µs` (C-62), `REFERENCE_E = 50 µs` (C-70), `REFERENCE_E_UNKNOWN = 2 × TARGET_BOUND` (C-169), `HOLDOVER_GROWTH = 1 µs/s` (C-64) in one constants module with their C-ids and statuses.

## Acceptance criteria
1. A locked stamp with bound 12 µs gives `Locked`, bound 12 µs, `over_target = false`; bound 30 µs gives `over_target = true`.
2. A holdover stamp at t0 with bound 25 µs gives `lapse_in` = 25 s at t0, 10 s at t0 + 15 s, and 0 with state `Unlocked`, `lapsed = true`, at t0 + 25 s (C-162).
3. A holdover stamp with bound 40 µs gives `lapse_in` = 10 s at its stamp time; a refresh at t0 + 10 s with bound 50 µs gives 0 and state `Unlocked`, `lapsed = true`.
4. A holdover stamp at t0 with bound 25 µs from a module whose E is unknown gives `lapse_in` = 25 s at t0; with `TARGET_BOUND` changed in a test build to 30 µs it gives 35 s, showing the reference is derived from C-62 (C-169), not from C-70.
5. A timeline epoch change is reported as such and is not confused with the election epoch.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Rendering (029-12); blanking one-way figures before lock (029-08).
