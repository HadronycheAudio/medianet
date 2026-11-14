# 008b-15 — Case 14: fan-in offset estimator
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** M · **Depends on:** 008a-15
## Context
Epic 008 uses the same simulation harness to validate the fan-in offset estimator; epic 009 then implements the production fan-in and consumes the validated parameters. This ticket builds a reference model in the `sim` module only.

Rules `[decision]`:
- Slices are addressed by position on a domain-wide timeline (position = network time counted in samples at 96 kHz, the only wire rate, C-01). Slice period 0.5 ms (48 frames, C-02).
- A fan-in consumer measures origin-to-arrival lateness per input during a 4 s aligning state (C-50).
- Offset = maximum observed lateness plus headroom (C-51); percentiles are reported, not used.
- Headroom = the largest of 25 % of the observed maximum, one slice period (0.5 ms), and twice the published clock bound (C-52).
- A slice absent at its deadline is counted missing; if it arrives later while its position is within the ring span it is reclassified as late (missing −1, late +1) and discarded (C-108). Rings are sized to A (C-96), and A has no default (C-55), so A is a scenario parameter.

C-50 to C-52 are decided; their owner for falsification is simulation, then L1. Falsifier: a late slice after alignment on a clean network. Epic criterion A11: zero late slices after a 4 s alignment on the clean-network model, or the counter-example and a proposed replacement for the design owner. Open sub-question (this task, then L1): "Is 4 s of aligning enough for maximum-plus-headroom?"

## Work
1. In the `sim` module of `hadronyche-tools`: a clean-network lateness model per input (source clock error from a simulated servo's published view, link delay from `WIRED_PROFILE` (C-94), sender scheduling jitter), producing origin-to-arrival lateness per slice.
2. A reference offset estimator implementing C-50/C-51/C-52 exactly, reading `bound_ns` from a `PublishedView`, and the C-108 late/missing classification.
3. Named test `case_14_offset_estimator`: 1 to 8 inputs, at least 20 seeds, alignment 4 s then 10 min of running; count late slices per C-108 (and missing slices separately); report lateness at the 50th, 99th and 99.9th percentiles and the maximum, the set the stamp carries (C-160), as information only (C-51); like the stamp, compute them over a rolling 60 s window with the maximum since the end of aligning beside them (C-182).
4. Answer the open question with `[measured]` (simulation) in `doc/design/reports/008-simulation-results.md`: for each of C-50, C-51, C-52, "falsifier not triggered" or the counter-example with a proposed replacement, naming L1 as the confirming owner.

## Acceptance criteria
1. Estimator unit tests: with observed max lateness 1 ms and bound 10 µs, offset = 1 ms + max(250 µs, 500 µs, 20 µs) = 1.5 ms; with max 4 ms, offset = 5 ms; with bound 400 µs and max 1 ms, offset = 1.8 ms.
2. Classification unit test: a slice absent at its deadline counts missing; when it then arrives within the ring span, missing −1 and late +1.
3. Case 14 reports the late-slice count per configuration and asserts 0 on the clean-network model with C-50 = 4 s — or records the counter-example and the aligning time that achieves 0, and asserts that.
4. Results file has C-50, C-51, C-52 lines and the open-question answer with owner and falsifier.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The ratchet (008b-16); production fan-in code (epic 009).
