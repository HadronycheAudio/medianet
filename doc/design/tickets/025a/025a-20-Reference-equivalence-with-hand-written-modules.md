# 025a-20 — Reference equivalence with hand-written modules
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** S · **Depends on:** 025a-18, 025a-19, epic 017, epic 017b
## Context
Acceptance criterion 8 (reference equivalence): **the DSL echo and mixer produce the same bits as the hand-written reference modules of task 017, float and fixed, over the L0 baseline scenario.**

The DSL versions are `hadronyche-reference/dsl/echo.hdsl` (025a-18) and `dsl/mixer.hdsl` (025a-19). The hand-written modules and the L0 harness come from epic 017. The L0 baseline scenario is defined by epic 017b (C-192) and is not restated in epic 025a; this ticket uses it as epic 017b defines it rather than defining one here.

Note: "Do POC voice windows compare bit for bit?" is owned by **task 002 (archaeology)**, falsifier: tolerance-only comparison in the POC. That question governs what equivalence with the POC can mean; this ticket compares only DSL vs 017 hand-written modules, which must be bit-identical.
## Work
1. A test (run in `dsl-conformance`) that runs the L0 baseline scenario through the 017 hand-written echo and mixer and through the DSL echo and mixer, in float and in fixed, and compares output slices bit for bit.
2. On mismatch, report the first differing slice, output and sample.
## Acceptance criteria
1. Four comparisons (echo float, echo fixed, mixer float, mixer fixed) are bit-identical over the full L0 baseline scenario.
2. A deliberate one-operation reorder in the DSL echo float variant makes the test fail, reporting the first differing slice.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- POC equivalence (task 002); cross-target equivalence (025b-06).
