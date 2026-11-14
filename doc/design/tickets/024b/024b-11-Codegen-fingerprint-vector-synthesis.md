# 024b-11 — Codegen: fingerprint vector synthesis
**Epic:** 024b-DSL-code-generator · **Effort:** M · **Depends on:** 024a-03, 024b-02, 024b-04

## Context
Rule `[decision]`: for exact-float modules the generator emits a compile-time input vector and expected output bits, covering subnormal and NaN-producing paths. The runtime evaluates it at start on the processing thread (`fingerprint_matches::<M>()`, 024a-03). Match ⇒ exact-float eligible; mismatch ⇒ bounded-class.

Cross-platform float differences come from FMA contraction (avoided by construction), non-IEEE subnormal handling (FTZ/DAZ, per thread), NaN payloads, and 80-bit intermediates on legacy x86. ABI shape: `pub struct Fingerprint { pub inputs: &'static [u32], pub expected: &'static [u32] }`; `ModuleInfo.fingerprint: Option<&'static Fingerprint>`. The input layout rule is the one documented on `fingerprint_matches` in 024a-03. Subnormal handling follows the decision of 024b-02.

Open sub-question (owner: task 025, not this task): "Does the fingerprint vector cover every path that can differ (subnormals, NaN, rounding)?" Falsifier: a target passes the fingerprint and differs on a conformance vector.

## Work
- `hadronyche-dsl/src/codegen/fingerprint.rs`: for exact-float modules, synthesise input words that drive (a) subnormal inputs and subnormal-producing operations, (b) NaN-producing operations (0/0, ∞ − ∞) and NaN propagation, (c) rounding-sensitive cases (ties, large-magnitude cancellation); compute expected output bits with a reference evaluation in strict IEEE mode on the build host.
- Emit `static FINGERPRINT: Fingerprint` and set `INFO.fingerprint = Some(&FINGERPRINT)`; `None` for exact-fixed modules. A generated module is never bounded (C-123).
- Report which IR operations each vector covers (a coverage list in the readable manifest or a comment) to support task 025.

## Acceptance criteria
1. For each exact-float corpus program, `fingerprint_matches::<M>()` returns `true` on the hosted target in default FP mode.
2. With flush-to-zero and denormals-are-zero enabled on the calling thread, `fingerprint_matches::<M>()` returns `false` for every exact-float corpus program that contains a float operation.
3. The vector contains at least one subnormal input word and at least one input that produces a NaN in an intermediate, for every program where the IR allows it (checked by the coverage list).
4. Exact-fixed modules have `INFO.fingerprint == None`; every exact-float corpus program has `Some`.
5. Synthesis is deterministic: same IR ⇒ same vector.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Proving full coverage (task 025).
- Thread placement test and regime stamp (024b-17).
- Writing the vectors file (024b-12).
