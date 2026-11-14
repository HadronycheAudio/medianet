# 024b-13 — Codegen: operation-counting build for the cost bound
**Epic:** 024b-DSL-code-generator · **Effort:** S · **Depends on:** 024b-04, 024b-12, epic 023

## Context
Rule (C-106): `cost_ops_per_slice` counts abstract operations per slice and is identical on every machine; each target has a calibration factor (time per operation), measured once and recorded in the regime stamp, and admission converts with it. The conformance suite (epic 025) measures the factor with a reference workload once per listed target (C-176). The count declared in `INFO` comes from the cost analysis of epic 023, which defines what an abstract operation is and its weight. To check the declared bound, the generated code must be able to count the operations it actually performs.

Rules for generated code `[decision]`: no allocation; the normal build must not pay for counting. Slice 48 frames (C-02).

## Work
- A codegen option (for example a cargo feature on the generated module) that wraps each lowered kit call with an increment of a per-call counter, using the operation weights of epic 023's cost analysis; off by default.
- A harness that runs a module's `process` over seeded corpus input, including gap inputs, and reports the maximum count per slice.
- Run it on the three CI targets (024a-06).

## Acceptance criteria
1. For every corpus program, the maximum counted operations per slice is identical on x86-64, AArch64 and the 32-bit device target.
2. For every corpus program, the counted maximum does not exceed `cost_ops_per_slice`.
3. The default build of a generated module contains no counter code (snapshot or symbol test).
4. A fixture whose declared `cost_ops_per_slice` is deliberately too small is detected.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Defining operation weights (epic 023).
- Timing (024b-19); the calibration-factor procedure (epic 025, C-176).
