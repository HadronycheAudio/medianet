# 008b-03 — Case 03: asymmetry bias
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** S · **Depends on:** 008a-15
## Context
Simulation case 3: asymmetric delay produces a bias of exactly half the asymmetry. Epic criterion A7: case 3 shows bias equal to half the injected asymmetry within 1 %.

Requirement `[decision]`: one-way figures carry the worst-case asymmetry bound; the estimate is biased by exactly half the path asymmetry. Sign convention (epic Background): with offset = ((t2 − t1) + (t3 − t4)) / 2, a follower-to-holder delay larger than the holder-to-follower delay by a makes the offset estimate exceed the true offset by a/2.

## Work
1. Named test `case_03_asymmetry_bias`: forward base delay − reverse base delay ∈ {10, 50, 200, 1 000} µs, `WIRED_PROFILE` (C-94) jitter otherwise, +20 ppm drift, at least 10 seeds, 120 s runs.
2. Measure steady-state mean true error after lock (last 60 s).
3. Assert bias = +asymmetry/2 within 1 %, with the sign convention above.
4. Check that `bound_ns` includes the configured worst-case asymmetry bound term.

## Acceptance criteria
1. For every asymmetry value and seed, |mean error − asymmetry/2| ≤ 1 % of asymmetry/2, with the sign given by the convention.
2. When the configured worst-case asymmetry bound ≥ injected asymmetry/2, no honest-lock violation occurs.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Measuring real-network asymmetry and the production asymmetry bound (L1, task 019).
