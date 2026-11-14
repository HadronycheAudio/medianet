# 008b-16 — Case 15: ratchet with a flapping input
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** M · **Depends on:** 008b-15
## Context
The fan-in offset is a ratchet `[decision]`:
- it grows by one slice period, 0.5 ms (C-53), when an input is persistently late, after a dwell of 10 s (C-54);
- "persistently late" means at least one late slice in each of 10 consecutive 1 s windows; this is the dwell C-54 expressed as a test (C-101);
- it never shrinks automatically;
- its ceiling is the presentation group's audition offset A (C-53), which has no default and is set per presentation group (C-55); here A is a scenario parameter;
- an input that is persistently late with the offset already at A enters the health state Demoted: treated as gapped, counted separately, cleared only by re-admission (C-109).

Late slices are counted per C-108 (absent at the deadline = missing; arriving later within the ring span = reclassified late).

Initial offset comes from the estimator (008b-15): max observed lateness over 4 s (C-50) plus headroom = max(25 % of max, 0.5 ms, 2 × clock bound) (C-51, C-52).

Simulation case 15: one flapping (bimodal) input; termination at the ceiling and demotion. Epic criterion A8 (system AC-17, simulation part).

Last step, settled by C-163 `[decision]`: when the gap between offset and A is less than one slice period, the offset steps to A exactly. Persistent lateness with the offset at A then demotes (C-109).

## Work
1. In the `sim` module of `hadronyche-tools`: a reference ratchet model implementing C-53/C-54 with the C-101 predicate, ceiling A as a scenario parameter, the Demoted state of C-109 (gapped, separately counted, not cleared automatically), and the C-163 last step to A.
2. Bimodal lateness model for one input (alternating between a normal mode and a late mode whose lateness exceeds A).
3. Named test `case_15_ratchet`: A ∈ {2, 5, 10} ms plus one value that is not a multiple of 0.5 ms above the initial offset (exercises C-163), at least 20 seeds, run until steady.
4. Record C-53 and C-54 as "falsifier not triggered" or the counter-example (oscillation, or A exhausted in normal use) with a proposed replacement, in `doc/design/reports/008-simulation-results.md`.

## Acceptance criteria
1. Each offset increase is exactly 0.5 ms, except a final increase of A − offset when that gap is below 0.5 ms (C-163), and occurs only when the C-101 predicate holds: one late slice in each of 10 consecutive 1 s windows (checked from the trace).
2. The offset never decreases.
3. The offset never exceeds A and reaches A exactly when the ratchet is exhausted (C-163); the flapping input enters Demoted when persistently late with the offset at A, and the offset then stays constant for the rest of the run.
4. No oscillation: the number of offset changes is ≤ ceil((A − initial offset) / 0.5 ms), and the Demoted input is not re-admitted automatically during the run; Demoted slices are counted separately from gapped ones.
5. Clean inputs in the same group are never demoted.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Production ratchet in the fan-in (epic 009); the value of A per presentation group (commissioning, C-55); re-admission.
