# 008a-11 — Servo frequency loop, bound and lock
**Epic:** 008a-Servo-core-and-simulation-harness · **Effort:** M · **Depends on:** 008a-10, 008a-02
## Context
The core of the new servo in `hadronyche-core` (fixed size, no allocation, `no_std`). It replaces the POC servo, whose actuation stepped the phase: `theta0 = pred + alpha·w·err` at every update, i.e. a step in published time of a fraction of the error every 200 ms `[code]`. POC gains were a least-squares line fit settling at `alpha = 0.1`, `beta = 0.005` `[code]`; frequency-only correction changes the loop dynamics those gains assume, so this is effectively a new servo `[decision]`.

Requirements `[decision]`:
- Continuity: published time is continuous while locked — zero phase step across updates, 0 ns on the integer-ns published time. Corrections act on frequency only.
- Slew: while locked, the correction term is at most 10 ppm from the holder's rate (C-60) and changes by at most 1 ppm per second (C-61). These limits bound the servo's correction term, not the drift estimate it tracks (C-119): published rate = drift estimate of the holder rate + correction term, and only the correction term is clamped. Acquisition is exempt.
- Lock is a number: the published view carries an error bound. Locked requires bound ≤ 25 µs (C-62 = E/2 for the Linux reference sink E = 50 µs, C-70).
- Acquisition: Locked within 10 s of the first exchange (C-63).
- Honest lock: never reports Locked while true error exceeds its published bound.
- Asymmetry: one-way figures carry the worst-case asymmetry bound. Its production value is not decided (L1, task 019, measures it); here it is a `ServoConfig` field.
- C-60..C-63 are proposals never simulated; 008b-01 tunes and confirms or replaces them, so all four must be `ServoConfig` fields, not literals.

## Work
1. `Servo::tick(&mut self, now_local_ns: i64)` closes windows and runs the control loop; `Servo::view(&self) -> TimeView`.
2. Frequency-only loop: each update changes `rate_ppb` and re-anchors `offset_ns`/`at_local_ns` such that published time at the update instant is unchanged (phase-continuous). Gains are `ServoConfig` fields.
3. Split the rate into a drift estimate and a correction term. While Locked, clamp |correction| ≤ `cfg.max_slew_ppb` (C-60, 10 000 ppb) and |Δcorrection| ≤ `cfg.max_rate_change_ppb_per_s` (C-61, 1 000 ppb/s). The drift estimate is not clamped by them (C-119). In Acquiring these clamps do not apply.
4. A diagnostic accessor for the correction term (for example `Servo::correction_ppb()`), used by the recorder (008a-09).
5. Error bound `bound_ns`: computed from measurement noise, round-trip/2, `cfg.asymmetry_bound_ns`, and rate uncertainty × time since last measurement. Document the formula.
6. States: `Unlocked` before the first exchange; `Acquiring` after it; `Locked` when `bound_ns ≤ cfg.lock_bound_ns` (C-62, 25 000 ns) and the estimator is settled; fall back to `Acquiring` if the bound exceeds C-62 without entering holdover.
7. A helper that publishes `view()` into a `PublishedView` (008a-02) after each update.

## Acceptance criteria
1. Unit test with exact synthetic exchanges (zero jitter, +50 ppm drift): `Locked` reached and `bound_ns ≤ 25 000` within 10 s (C-63) of the first exchange.
2. Across every update while Locked in that test, phase step (published time before vs after at the same local instant) is 0 ns.
3. |Correction| ≤ 10 ppm and |Δcorrection| ≤ 1 ppm/s at every update while Locked.
4. Unit test with a drift ramp of 0.5 ppm/s: the drift estimate follows the ramp while the correction term stays within the C-60/C-61 clamps (C-119).
5. True error ≤ `bound_ns` at every update while Locked.
6. `size_of::<Servo>()` asserted constant; builds for the 008a-01 no_std target.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Tuning to the wired profile and deciding C-60..C-63 (008b-01).
- Holdover (008a-12), steps/epoch (008a-13), handover (008a-14).
