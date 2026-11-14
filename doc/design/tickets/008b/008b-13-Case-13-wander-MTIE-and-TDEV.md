# 008b-13 — Case 13: wander, MTIE and TDEV
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** S · **Depends on:** 008a-09, 008a-15
## Context
Simulation case 13: wander — MTIE at most 10 µs for intervals up to 1 s and at most E for any interval (the L2 wander figure, C-77); TDEV reported. E is the edge error target; the servo is held against the Linux reference sink, E = 50 µs (C-70; C-62 = E/2 = 25 µs) `[decision]`. The threshold is the platform's target E, never an E measured in a run (C-149). C-70 is pending measurement at L2; if L2 replaces it, this case's threshold follows the constants table. Sources pace position from network time, so servo behaviour reaches hardware edges as resampler ratio `[decision]`; wander of published time is what edges see.

## Work
1. Named test `case_13_wander`: Locked runs of at least 1 000 s under `WIRED_PROFILE` (C-94) with drift ±100 ppm, thermal wander, at least 10 seeds.
2. Compute MTIE(τ) and TDEV(τ) of published time error (008a-09) for τ from the sampling interval to the run length.
3. Write the MTIE and TDEV tables (per seed, worst case) into `doc/design/reports/008-simulation/` for 008b-18.

## Acceptance criteria
1. MTIE(τ) ≤ 10 µs for every τ ≤ 1 s, every seed.
2. MTIE(τ) ≤ E (C-70, 50 µs) for every τ, every seed; the threshold is read from `ServoConfig`, not a literal.
3. TDEV table present in the results directory.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Resampler behaviour at edges (epics 018–020).
