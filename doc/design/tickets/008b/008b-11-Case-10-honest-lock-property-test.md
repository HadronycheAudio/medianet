# 008b-11 — Case 10: honest lock property test
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** M · **Depends on:** 008a-12, 008a-13, 008a-15
## Context
Simulation case 10: honest lock, as a property test over randomised scenarios. Epic criterion A3 (system AC-4): no counter-example to honest lock in the property test.

Rule `[decision]`: the servo never reports `Locked` while true error exceeds its published bound (`TimeView.bound_ns`). Locked requires bound ≤ 25 µs (C-62). In holdover the bound grows by 1 µs/s (C-64). The estimate is biased by half the path asymmetry, and one-way figures carry the worst-case asymmetry bound. Wall-clock steps are removed by the backend before the servo (C-121).

The run also checks the holdover entry delay proposed by 008b-05 for C-195 (at most 1.5 s, C-37, a hard limit; the value within it pending simulation): a counter-example at the proposed value is that constant's falsifier.

## Work
1. Named property test `case_10_honest_lock` (e.g. `proptest`, with a fixed seed corpus checked in so CI is deterministic) generating scenarios from the full model space: constant drift ±100 ppm, drift ramps up to 0.5 ppm/s (C-119), thermal wander, every link feature (jitter distributions, bursts, asymmetry up to the configured worst-case bound, loss, duplication, reordering, coalescing), wall-clock steps, holder steps with epoch change, outages.
2. Property: at every recorded sample with `state == Locked`, |true error| ≤ `bound_ns`.
3. Shrinking enabled; any counter-example found is added to the checked-in regression corpus.
4. CI budget: a fixed number of cases per run (document it, at least 1 000), plus a longer nightly or manual run.

## Acceptance criteria
1. CI run of at least 1 000 randomised scenarios finds 0 counter-examples.
2. Regression corpus replays deterministically (same seeds, same verdict).
3. A deliberately broken servo (bound halved) is caught by the property within the CI budget (demonstrated locally; not merged).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Asymmetry beyond the configured worst-case bound (unobservable by design; documented in 008b-03).
