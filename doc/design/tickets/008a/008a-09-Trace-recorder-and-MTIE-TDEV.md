# 008a-09 — Trace recorder and MTIE/TDEV
**Epic:** 008a-Servo-core-and-simulation-harness · **Effort:** M · **Depends on:** 008a-04
## Context
The `sim` module of the `hadronyche-tools` crate of epic 008 must record, for every run: true error, published bound, state, phase steps, rate, the correction term and its rate of change, and MTIE/TDEV of published time. These feed the epic criteria:
- A2 (verified in 008b, system AC-3): zero phase step across updates; correction term at most 10 ppm from the holder's rate (C-60); its rate of change at most 1 ppm/s (C-61), while locked. The limits bound the servo's correction term, not the drift estimate it tracks (C-119) `[decision]`.
- A3 (verified in 008b, system AC-4) honest lock: never Locked while true error exceeds the published bound.
- Case 13: MTIE at most 10 µs for intervals up to 1 s and at most E for any interval (C-77), with E = 50 µs for the Linux reference sink (C-70); TDEV reported.

`TimeView` fields: `offset_ns`, `at_local_ns`, `rate_ppb`, `state: LockState {Unlocked, Acquiring, Locked, Holdover}`, `bound_ns`, `epoch`. The correction term is not in `TimeView`; the runner (008a-15) passes it to the recorder from the servo's diagnostic accessor (008a-11).

## Work
1. `Recorder` sampling each published `TimeView`, the servo's correction term and the ideal (true) network time at every update and at a fixed sampling interval; stores: true error (published time − true network time), `bound_ns`, `state`, `epoch`, `rate_ppb` relative to the true holder rate (informational), correction term (ppb) and its rate of change (ppb/s).
2. Phase-step detector: at each update, published time evaluated at the same local instant using the view before and after; the difference is the phase step, in integer ns.
3. MTIE(τ) and TDEV(τ) over the time-error series for τ on a log grid from the sampling interval to the run length (ITU-T G.810 definitions; document the estimator and cite the definitions with their clause under the `[standard]` tag, C-132).
4. Summary struct: max |phase step| while Locked, max |correction term| while Locked, max |rate of change of correction term| while Locked, max true rate deviation from holder (informational), honest-lock violations count, MTIE/TDEV tables; serialisable for reports.

## Acceptance criteria
1. MTIE/TDEV of a synthetic linear phase ramp and of white phase noise match analytic values within 2 %.
2. A synthetic view sequence with one injected 1 ns phase step reports exactly one step of 1 ns; a continuous sequence reports 0.
3. A synthetic sequence with a Locked sample whose true error exceeds its bound by 1 ns reports 1 honest-lock violation.
4. A synthetic correction series stepping by 2 ppm within 1 s reports a rate of change of 2 ppm/s; a drifting `rate_ppb` with a constant correction term reports 0 ppm/s for the correction.
5. Recording is deterministic (summary digest identical across two runs with the same seed).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Driving the servo (008a-15); asserting the limits across scenarios (008b-12, 008b-13).
