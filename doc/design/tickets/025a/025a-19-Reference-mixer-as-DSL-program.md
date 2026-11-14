# 025a-19 — Reference mixer as a DSL program
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** S · **Depends on:** 025a-10, epic 017, epic 024
## Context
Deliverable: crate `hadronyche-reference`, file `dsl/mixer.hdsl` — the reference mixer re-expressed as a DSL program, built in CI.

The reference mixer's rule on a gap `[decision]`: **a missing input contributes zero and the output slice is marked degraded.** Acceptance criterion 9 (second half): the reference mixer program contributes zero for a missing input and marks its output degraded.

Process function: `process(state, inputs at position p with per-input gap flags, position) -> outputs`; audio gap default is zero-fill; degraded marking per output comes from dataflow and follows the content: an output at p is degraded if its input at p − L was gapped (C-122). If the mixer's L is 0, that is the same slice.

The mixer's volume is the reference dense controller (C-192): the DSL mixer takes it as a dense controller input, with the hold-last-value gap default and a declared initial value in the manifest (C-158).

Every accepted DSL program is exact-fixed or exact-float (C-123). If the mixer's inputs reach it through paths of different latency, the compiler pads them (C-104); the reference mixer's inputs are expected to share one latency, and the expect file states any inserted delay.

Kit rules: strict f32, Q1.31 / Q0.32 fixed point; no libm; float operations never reordered. Constants: domain rate 96 kHz (C-01); slice = 48 frames (C-02); 4-byte samples (C-03); 6 channels per stream (C-04).
## Work
1. Write `hadronyche-reference/dsl/mixer.hdsl`, float and fixed variants, matching the epic 017 hand-written mixer's operation order.
2. Add `expect/` entries and include both in the conformance corpus run; build in `dsl-conformance`.
3. Gap test: with one input gapped at input position q, the output at q + L equals the mix computed with that input all zeros, and is marked degraded; with no input gapped, output is not degraded.
## Acceptance criteria
1. Both variants compile in CI; classes are `exact-float` and `exact-fixed`.
2. With input k gapped, output is bit-identical to the mix with input k replaced by zeros, for each k.
3. The output slice at q + L is marked degraded whenever any input is gapped at q, and no slice is marked otherwise (C-122).
4. Expect entries pass the 025a-01 harness and the 025a-15 probe check.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Bit equivalence with the 017 mixer (025a-20); generic gap defaults (025a-16).
