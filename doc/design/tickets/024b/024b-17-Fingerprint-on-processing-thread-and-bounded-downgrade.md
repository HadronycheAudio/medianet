# 024b-17 — Fingerprint on processing thread and bounded downgrade
**Epic:** 024b-DSL-code-generator · **Effort:** M · **Depends on:** 024a-03, 024b-11, 024b-12, epic 009, epic 013, epic 015

## Context
Rule `[decision]`: the runtime evaluates the fingerprint at start **on the processing thread itself**, because floating-point modes are per thread and audio stacks change them. Match ⇒ exact-float eligible; mismatch ⇒ the module runs as bounded-class and the regime stamp says so. ABI: `fingerprint_matches::<M>() -> bool` and `effective_class::<M>() -> DeterminismClass` (024a-03), "Call on the processing thread, after its floating-point mode is final."

Epic acceptance criteria:
- 024b/AC-4: "The fingerprint is evaluated on the processing thread; a test that changes the floating-point mode on that thread only (not the main thread) is detected."
- 024b/AC-3 (part): "a forced mismatch (flush-to-zero enabled on the processing thread) makes the module run bounded-class and sets the stamp field."

The regime stamp is a dedicated control message, split into sections that each fit one packet at the fallback MTU, multicast on any regime change and refreshed every 10 s (C-18). Epic 013 defines the message and its field layout, and the hosted backend (epic 015) sends it (C-144); the name and encoding of the effective determinism class field is an open item owned by epic 013. The bounded class arises only from this mismatch for generated modules (C-123).

## Work
- In the runtime's module start path (epic 009 code, touched here minimally) or a `hadronyche-core` helper it calls: evaluate `effective_class::<M>()` on the processing thread after its FP mode is set, and set the result in the regime stamp's determinism-class field, which the hosted backend sends (epic 015, C-144).
- Test: spawn a processing thread; set FTZ/DAZ on that thread only (x86-64 MXCSR, AArch64 FPCR.FZ); keep the main thread in default mode; start an exact-float generated module; assert the module runs bounded-class and the stamp field says bounded, while evaluating on the main thread would have matched.
- Document the stamp field (name, values, meaning) in the regime-stamp specification of epic 013 under `doc/design/spec/` (C-130).

## Acceptance criteria
1. With FTZ enabled on the processing thread only, the module's effective class is `Bounded` and the regime stamp field records `Bounded`, on x86-64 and AArch64.
2. In the same test, `fingerprint_matches` called on the main thread returns `true` (proving the detection depends on the processing thread).
3. With default FP mode on the processing thread, effective class is `ExactFloat` and the stamp records it.
4. The stamp field is documented with every value it can take.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Cross-target bit identity of programs (024b-18).
- Fingerprint coverage (task 025).
