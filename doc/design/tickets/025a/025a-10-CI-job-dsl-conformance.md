# 025a-10 — CI job `dsl-conformance`
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** XS · **Depends on:** 025a-07, 025a-08, 025a-09, 025a-21, 025a-22
## Context
Epic 025 defines three CI jobs; `dsl-conformance` belongs to this part (025a). It compiles the corpus, compares L (whole samples, C-105), class, dependencies and inserted join delays (C-104) with `expect/`, runs the exact vectors on the host target, and checks executed abstract operations against the bound, which is target-independent (C-106). The other two (`dsl-cross-target`, `dsl-cost`) are 025b-06 and 025b-11, in part 025b.

Inputs: `hadronyche-dsl/conformance/corpus/` (positive programs, at least 40, and negative programs, at least 25 across 025a-07, 025a-08, 025a-21 and 025a-22), `expect/`, `vectors/`; the harness from 025a-01/025a-07 and the host runner from 025a-09.
## Work
1. Add CI job `dsl-conformance` that runs the conformance harness (positive and negative corpus) and the host vector runner.
2. The job runs unprivileged and fails on any mismatch.
3. Later tickets (025a-15, 025a-16, 025a-17, 025a-18, 025a-19, 025a-20, 025a-06, 025a-11) add their tests to this job.
## Acceptance criteria
1. `dsl-conformance` runs on every change to `hadronyche-dsl` and is green on the main branch.
2. A deliberately broken expectation on a branch turns the job red.
3. The job runs without root or special capabilities.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Cross-target and cost jobs (025b-06, 025b-11); probe comparison (025a-15).
