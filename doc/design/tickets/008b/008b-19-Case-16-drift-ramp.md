# 008b-19 — Case 16: drift ramp
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** S · **Depends on:** 008a-15, 008b-01
## Context
Rule C-119 `[decision]`: the L2 drift-ramp scenario sweeps the injected crystal drift over ±100 ppm at no more than 0.5 ppm/s; L2 passes if E is held throughout. The slew limits C-60 (10 ppm) and C-61 (1 ppm/s) bound the servo's correction term, not the drift estimate it tracks. The servo runs this scenario in simulation first so that L2 does not discover a tracking failure.

Epic criterion A12: case 16 stays Locked throughout with true error inside the bound and the bound within C-62 (25 µs, E/2 against the Linux reference sink E = 50 µs, C-70).

## Work
1. Named test `case_16_drift_ramp`: follower crystal ramping between −100 and +100 ppm at 0.5 ppm/s and at 0.25 ppm/s (two full sweeps each, two-clock model 008a-05), holder at 0 ppm, `WIRED_PROFILE` (C-94), at least 20 seeds; start the ramp after lock.
2. Record state, bound, true error, drift estimate, correction term and its rate of change.
3. Register the scenario in the shared scenario list used by 008b-12 and 008b-17.
4. Write the outcome to `doc/design/reports/008-simulation-results.md` with tag `[measured]` (simulation), as the simulation forecast for the L2 drift-ramp test.

## Acceptance criteria
1. The servo stays `Locked` for the whole ramp in every seed.
2. |True error| ≤ `bound_ns` ≤ 25 µs (C-62, or its replacement from 008b-01) at every sample.
3. |Correction term| ≤ 10 ppm and its rate of change ≤ 1 ppm/s at every sample (C-60, C-61 as bounded by C-119), while the drift estimate follows the ramp.
4. Results file has a "Drift ramp (C-119)" line.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The L2 hardware run (L2 ladder rung); edge rate-loop behaviour (C-68).
