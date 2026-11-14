# 025b-08 — Fingerprint mutation coverage
**Epic:** 025b-Replay-cross-target-fingerprint-and-cost-calibration · **Effort:** M · **Depends on:** 025b-06, 025b-07
## Context
Acceptance criterion 6 (fingerprint coverage): **mutation test: for each known divergence mechanism (flush-to-zero, denormals-are-zero, contraction, extended precision), a build or mode exhibiting it is caught by the fingerprint. A target that passes the fingerprint and differs on any vector is a failed suite.**

Open sub-question owned by this epic: **fingerprint vector coverage**. Falsifier: a target passes the fingerprint but differs on a conformance vector.

Fingerprint: a compile-time vector (including subnormal and NaN paths) evaluated at start on the processing thread. It is a compiler promise implemented by epic 024; this ticket tests it and files any extension against 024. Match ⇒ exact-float eligible; mismatch ⇒ bounded (C-123). Strict f32; float operations are never reordered.

How to build the contraction and extended-precision mutants is not given by the epic or the constants table; this ticket owns the choice. A mutant need not be a listed target: an extended-precision build for 32-bit x86 is acceptable as a mutant even though 32-bit x86 is not listed. Claims about IEEE 754 behaviour cite the standard with its clause and the `[standard]` tag (C-132).
## Work
1. Four mutants, each run through fingerprint + exact-float vectors:
   - **flush-to-zero**: FTZ mode set on the processing thread;
   - **denormals-are-zero**: DAZ mode set;
   - **contraction**: a build of the generated code with FMA contraction allowed (e.g. `-ffp-contract=fast` equivalent / target feature enabling fused multiply-add);
   - **extended precision**: x87 80-bit evaluation (e.g. a 32-bit x86 build without SSE2) or equivalent.
2. For each mutant: assert the fingerprint mismatches. If any mutant passes the fingerprint, it must also pass all exact-float vectors, else the suite fails; extend the fingerprint vector (feed back to epic 024) until it catches the mutant.
3. Record the coverage resolution (which vector entries catch which mechanism, and how each mutant is built) in `doc/design/spec/` (C-130) and close the open sub-question, or keep it open with owner and falsifier.
## Acceptance criteria
1. Each of the 4 mutants causes a fingerprint mismatch, in CI.
2. No mutant (and no listed target) passes the fingerprint while differing on any exact-float vector.
3. The mapping mechanism → catching fingerprint entry is documented.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Single altered-mode fallback and regime stamp (025b-07); listed-target publication (025b-13).
