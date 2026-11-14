# 025a-03 — Corpus: linear-phase FIR, recursive filters, table oscillators
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** M · **Depends on:** 025a-01
## Context
Part of the conformance corpus in `hadronyche-dsl/conformance/corpus/` (at least 40 programs in total across 025a-02 to 025a-06, float and fixed variants, hand-derived expectations).

Rules this ticket relies on:
- Structural latency L is always a whole number of **domain-rate samples** (C-105) and includes **linear-phase filter centres**: a linear-phase FIR of N taps has L = (N − 1) / 2, so the corpus uses odd N only. An even-tap linear-phase FIR, whose centre falls between two samples, is rejected by the compiler in v1 with a diagnostic suggesting an odd-tap design (C-183); its negative programs are 025a-21.
- **Recursive filters** give an output **non-constant (frequency-dependent) delay**, reported as a property beside the output's structural L, which stays a whole number (C-104, C-105). Expect files give the hand-derived structural L and the `non-constant` flag.
- Kit only: strict f32, Q1.31 / Q0.32 fixed point, shipped tables, the resampler node (2:1 and 1:2, edge modules only, C-105, C-134). No libm, no transcendental functions; float operations never reordered.
- Determinism class: every accepted DSL program is `exact-fixed` or `exact-float`; recursive filters are no exception, because the DSL has no non-deterministic node in v1 (C-123).
- Cost bound: finite count of abstract operations per slice (C-106).
- Table oscillators use shipped tables (constant data) only, not computed sines.
## Work
Add at least 10 programs with `expect/` files and written hand derivations:
1. Linear-phase FIR, odd tap counts (e.g. 7, 31, 127 taps), float and fixed.
2. Recursive filters: one-pole low-pass, biquad; float and fixed; expected: the hand-derived structural L and the `non-constant` flag.
3. A FIR followed by a delay (L = centre + delay).
4. Table oscillators: a sine table oscillator and a wavetable oscillator reading a shipped table, float and fixed.
## Acceptance criteria
1. At least 10 new programs in `corpus/` with matching `expect/` files and hand derivations.
2. The 025a-01 harness passes on all of them.
3. Every recursive-filter output is flagged by the compiler as non-constant delay with the hand-derived structural L, and every FIR output has L = (N − 1) / 2 plus any explicit delay, a whole number (C-105).
4. No program calls a transcendental function or libm.
5. Every program, recursive filters included, is assigned `exact-fixed` or `exact-float` (C-123).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Basic families (025a-02), resamplers (025a-04), controllers/transport (025a-05), joins (025a-06), probe detection of non-constant delay (025a-13, 025a-15).
- Degraded-flag tests (025a-16); a non-constant-delay output follows its structural L like any other (C-105, C-122).
- Even-tap linear-phase FIR: rejected by the compiler (C-183); negative corpus in 025a-21.
