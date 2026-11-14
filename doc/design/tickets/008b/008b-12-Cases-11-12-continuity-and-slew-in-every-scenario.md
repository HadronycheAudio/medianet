# 008b-12 — Cases 11-12: continuity and slew in every scenario
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** S · **Depends on:** 008b-01, 008b-02, 008b-03, 008b-04, 008b-05, 008b-06, 008b-08, 008b-09, 008b-10, 008b-11, 008b-19, 008b-20
## Context
Simulation case 11: zero phase step across updates while locked. Case 12: maximum correction term and its rate of change in every scenario. Epic criterion A2 (system AC-3): zero phase step across updates; correction term at most 10 ppm (C-60); its rate of change at most 1 ppm/s (C-61), in every scenario while locked.

Rules `[decision]`: published time is continuous while locked, checked as 0 ns on the integer-ns published time. Corrections act on frequency only. C-60 and C-61 bound the servo's correction term, not the drift estimate it tracks (C-119). Acquisition is exempt.

## Work
1. Named tests `case_11_zero_phase_step` and `case_12_rate_limits` that iterate over every scenario defined by cases 1–10, 16 and 17 (008b-01 to 008b-11, 008b-19, 008b-20, including the multi-holder scenarios) and the honest-lock regression corpus.
2. Use the recorder summary (008a-09): max |phase step| while Locked, max |correction term| while Locked, max |rate of change of the correction term| while Locked; also report the true rate deviation from the holder for information.
3. Emit a table (scenario × the three maxima) into `doc/design/reports/008-simulation/` (results live under `doc/design/reports/`, C-130) for 008b-18.

## Acceptance criteria
1. Max |phase step| while Locked = 0 ns in every scenario.
2. Max |correction term| ≤ 10 ppm (or the C-60 replacement from 008b-01) in every scenario while Locked.
3. Max |rate of change of the correction term| ≤ 1 ppm/s (or the C-61 replacement) in every scenario while Locked.
4. The results table lists every scenario with its three maxima.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Scenarios added after this ticket must register themselves in the shared scenario list; cases 13–15 register in their own tickets.
