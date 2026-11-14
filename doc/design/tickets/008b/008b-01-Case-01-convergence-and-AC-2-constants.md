# 008b-01 — Case 01: convergence and AC-2 constants
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** M · **Depends on:** 008a-07, 008a-15
## Context
Simulation case 1: convergence time and steady-state error under constant drift up to ±100 ppm, across jitter distributions.

Epic criterion A1 (system AC-2): bound at most 25 µs within 10 s under the wired jitter profile C-94 and drift up to ±100 ppm — or the task replaces the numbers and says why.

The four proposed constants, status pending-simulation `[decision]`:
- C-60: correction term at most 10 ppm from the holder's rate while locked.
- C-61: correction term changes by at most 1 ppm/s while locked.
- C-62: Locked requires bound ≤ 25 µs (E/2 for the Linux reference sink E = 50 µs, C-70).
- C-63: Locked within 10 s of the first exchange.
C-60 and C-61 bound the correction term, not the drift estimate it tracks (C-119). Acquisition is exempt from C-60/C-61. Inherited, also confirmed here: C-66 (200 ms window, min-RTT exchange).

Open sub-question owned here: "Are 10 ppm and 1 ppm/s on the correction term compatible with a 25 µs bound and 10 s acquisition?" Falsifier: no parameter set meets all four; the constants are replaced.

## Work
1. Named test `case_01_convergence` in the `sim` module of `hadronyche-tools`: drift grid {−100, −50, 0, +50, +100} ppm × `WIRED_PROFILE` (C-94) and the sensitivity presets from 008a-07 × at least 20 seeds each.
2. Report per cell: time to Locked from first exchange, time to `bound_ns ≤ 25 µs`, steady-state |true error| (p50, p99, max) and bound.
3. Tune `ServoConfig` gains to meet C-60..C-63 simultaneously; if impossible, choose replacement values and record the trade-off (which constant moved, by how much, why).
4. Write the result (confirmed or replacement + justification for C-60..C-63, C-66) to the results file `doc/design/reports/008-simulation-results.md` (create it; results live under `doc/design/reports/`, C-130), consumed by 008b-18.

## Acceptance criteria
1. Under `WIRED_PROFILE`, for every drift in ±100 ppm and every seed: `bound_ns ≤ 25 000` within 10 s of the first exchange — or replacement values are recorded and the test asserts those instead.
2. Same test asserts the C-60/C-61 limits on the correction term held while Locked (0 violations).
3. `doc/design/reports/008-simulation-results.md` lists each of C-60, C-61, C-62, C-63, C-66 as "confirmed" or a replacement value with justification.
4. The open question above is answered in the results file with the evidence tag `[measured]` (simulation) and the falsifier outcome.
5. Sensitivity presets are reported but never used to pass or fail A1.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Writing `servo.md` and the constants table (008b-18).
- Every-scenario continuity/slew sweep (008b-12); drift ramp (008b-19).
