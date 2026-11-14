# 008a-12 — Servo holdover and lapse
**Epic:** 008a-Servo-core-and-simulation-harness · **Effort:** S · **Depends on:** 008a-11
## Context
The POC servo entered holdover after 2 s without updates; holdover never expired and stayed "usable" with no growing bound `[code]`.

Requirement `[decision]`: in holdover the bound grows by 1 µs per second (C-64, decided); with a 25 µs lock bound (C-62) against E = 50 µs, the target E of the Linux reference sink (C-70; the target, never a measured E, C-149), alignment claims lapse after about 25 s (C-65, derived: (C-70 − C-62) / C-64). Epic criterion A5 (verified in simulation by 008b, system AC-6; the unit part here is criterion A14 of 008a): holdover bound grows by 1 µs/s; the state is never reported usable beyond the lapse.

Published time stays continuous; holdover free-runs at the last rate (frequency only).

State after the lapse, settled by C-162 `[decision]`: the clock is `Unlocked`, with time unusable: one-way figures and alignment claims stop.

E unknown (C-169) `[decision]`: a sink with no commissioned E (C-103, C-140) has no fixed reference for the lapse, so an unknown E is taken as twice the lock bound (2 × C-62, 50 µs); the lapse then still comes about 25 s into holdover at the C-64 growth.

Holdover entry delay (C-195) `[decision]`: pending simulation; part 008b proposes it from cases 4, 5 and 10 (008b-04, 008b-05, 008b-11) and records it in the constants table with its results (008b-18). It is at most the election silence threshold, 1.5 s (C-37), a hard limit, so a follower is always in holdover with a growing bound when it calls an election; the POC's 2 s is above that. If no value at or below 1.5 s works, 008b reports that as a finding (C-195). Here it is `cfg.holdover_after_ns`, default 1.5 s (the C-37 limit) until 008b's value is recorded.

## Work
1. In `Servo::tick`, enter `Holdover` from `Locked` when no measurement has arrived for `cfg.holdover_after_ns`.
2. While in holdover: `bound_ns = bound_at_entry + cfg.holdover_growth_ns_per_s × elapsed_s` (C-64, 1 000 ns/s); rate held constant; no phase step at entry.
3. Lapse: once `bound_ns > cfg.edge_target_ns` (C-70, 50 000 ns, the target E per C-149), switch to `Unlocked` with time unusable (C-162). `edge_target_ns` is optional; when E is unknown the lapse reference is 2 × `cfg.lock_bound_ns` (C-169).
3a. Default `cfg.holdover_after_ns` to 1.5 s, citing C-195 and C-37, and document that it is a placeholder for 008b's value. Reject a configuration whose `holdover_after_ns` exceeds 1.5 s (C-37): the limit is hard (C-195).
4. Exit: a new valid measurement returns to `Acquiring` or `Locked` without a phase step while the bound allows.
5. Provide a `TimeView`-level helper `is_usable()`: true only in `Locked`, or in `Holdover` with `bound_ns` at or below the lapse reference (`cfg.edge_target_ns`, or 2 × `cfg.lock_bound_ns` when E is unknown, C-169); false in `Unlocked`, so one-way figures and alignment claims stop after the lapse (C-162); document it.

## Acceptance criteria
1. Starting Locked with bound 25 µs and then no exchanges: `bound_ns` after t s of holdover equals 25 000 + 1 000·t ns (±1 ns) at t = 1, 10, 24 s.
2. At 25 s + one tick after entry (bound > 50 µs), state is `Unlocked` and `is_usable()` is false, and stays false until re-acquisition.
3. Phase step at holdover entry and exit is 0 ns.
3a. With E unknown and lock bound 25 µs, the lapse to `Unlocked` happens when the bound first exceeds 50 µs (2 × C-62, C-169), at 25 s + one tick as in criterion 2.
3b. The default `holdover_after_ns` is at most 1.5 s (C-37, C-195): a Locked servo with no exchanges is in `Holdover` by 1.5 s. A configuration with `holdover_after_ns` above 1.5 s is rejected (C-195 hard limit).
4. Builds for the 008a-01 no_std target.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Simulation case 5 with drifting crystals and true-error check, and the entry-delay sweep (008b-05).
