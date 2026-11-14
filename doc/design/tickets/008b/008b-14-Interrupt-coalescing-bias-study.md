# 008b-14 — Interrupt-coalescing bias study
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** S · **Depends on:** 008a-06, 008a-15
## Context
Open sub-question: "Does interrupt coalescing bias survive the minimum-round-trip filter?" Owner: this task (model), L1 (measure, task 019). Falsifier: offset shifts when coalescing is toggled.

The servo keeps only the minimum-round-trip exchange per 200 ms window (C-66) `[decision]`. The link model (008a-06) can apply interrupt-coalescing delay on the receive side (holder receive, follower receive, or both). C-94 contains no coalescing, so this study adds it on top of `WIRED_PROFILE`.

Any surviving bias acts like path asymmetry and must sit inside the worst-case asymmetry bound. The production value of that bound is still open (owner: L1, task 019); this study gives L1 the coalescing share of it.

## Work
1. Named test `coalescing_bias`: identical seeds with coalescing off and on, coalescing periods {10, 50, 100, 250} µs, applied at the follower only, the holder only, and both; `WIRED_PROFILE` otherwise; at least 10 seeds.
2. Measure steady-state mean true error difference (on − off).
3. Record the answer with evidence tag `[measured]` (simulation) in `doc/design/reports/008-simulation-results.md` under "Open questions", with the hand-off to L1 (task 019) and its falsifier.

## Acceptance criteria
1. The test reports the offset shift for every configuration.
2. The test asserts the documented conclusion: either shift ≤ a stated threshold (bias does not survive), or the shift is characterised and included in the worst-case asymmetry bound used by `bound_ns`.
3. With coalescing on, 0 honest-lock violations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Hardware measurement (L1, task 019).
