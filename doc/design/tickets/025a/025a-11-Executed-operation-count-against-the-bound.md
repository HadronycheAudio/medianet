# 025a-11 — Executed operation count against the cost bound
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** S · **Depends on:** 025a-10, epic 024
## Context
Settled unit (C-106): the compiler's cost bound is a count of **abstract operations per slice, identical on every machine**. **AC-33, cost (operation-count half):** for every corpus program, the executed abstract-operation count per slice never exceeds the bound. Because the count does not depend on the machine, checking it on the host covers every listed target; the time half, which does depend on the target, is 025b-10 and 025b-11.

Still open, owned by epic 024: the definition of an abstract operation (what the cost model counts) and an instrumented build of the generated code that counts executed operations. This ticket consumes both; if they are missing, file the need against 024 before starting.
## Work
1. Build every positive corpus program with the epic 024 counting instrumentation.
2. Run each over worst-case inputs (the inputs that maximise data-dependent paths, documented per program family) and record the maximum executed count per slice.
3. A test in the `dsl-conformance` job asserting executed count ≤ bound for every program.
4. Report the ratio executed / bound per program as an artefact, so loose bounds are visible.
## Acceptance criteria
1. Every positive corpus program: maximum executed count per slice ≤ cost bound.
2. A program whose bound is artificially lowered below its executed count fails the test.
3. Running the test twice gives identical counts (the count is deterministic).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Time measurement and calibration (025b-10, 025b-11); the cost model and instrumentation themselves (epic 024).
