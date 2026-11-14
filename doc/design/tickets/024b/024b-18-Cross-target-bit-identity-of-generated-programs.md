# 024b-18 — Cross-target bit identity of generated programs
**Epic:** 024b-DSL-code-generator · **Effort:** M · **Depends on:** 024a-06, 024b-12, 024b-17

## Context
Epic acceptance criterion 024b/AC-3 (contributes to project AC-32): "exact-fixed programs are bit-identical on every listed target; exact-float programs are bit-identical where the fingerprint matches; a forced mismatch (flush-to-zero enabled on the processing thread) makes the module run bounded-class and sets the stamp field." The forced-mismatch part is done in 024b-17.

Facts `[decision]`: exact-fixed is bit-exact everywhere; exact-float is bit-exact only where the fingerprint matches; exactness between a host and a small device is promised only for exact-fixed. Listed targets = the CI target set recorded in 024a-06 (x86-64, AArch64, one 32-bit device target); x86-64 and 64-bit ARM are the machine pair for cross-machine bit-exactness (C-151).

## Work
- The exact-fixed and exact-float programs of the epic's corpus (024b-04); build each with `hadronyche-dsl build`; on every listed target run 1000 slices of seeded input (48 frames, 6 channels, 4-byte samples: C-02, C-04, C-03) and write output bits plus the fingerprint result.
- Comparison job: exact-fixed outputs identical across all listed targets; exact-float outputs identical across all targets where `fingerprint_matches` returned `true`.

## Acceptance criteria
1. Every exact-fixed program's outputs are bit-identical on x86-64, AArch64 and the 32-bit device target over 1000 slices.
2. Every exact-float program's outputs are bit-identical across all listed targets on which its fingerprint matched; the job prints which targets matched.
3. An injected one-bit difference on one target fails the job and names program, slice and target.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The conformance suite's project-level AC-32 test (epic 025).
- Kit-level identity (024a-14).
