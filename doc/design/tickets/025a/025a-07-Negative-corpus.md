# 025a-07 — Negative corpus
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** S · **Depends on:** 025a-01
## Context
Acceptance criterion 11 of epic 025a: **at least 15 programs that must be rejected** (unbounded loop, transcendental call, allocation, cross-rate without the resampler, transport authoring) are rejected with the expected diagnostic.

Compiler rules being enforced:
- `process(state, inputs at position p with per-input gap flags, position) -> outputs` never blocks and performs **no allocation**.
- **No libm, no transcendental functions**; only the kit (strict f32, Q1.31 / Q0.32 fixed point, shipped tables, the resampler node).
- A finite **cost bound** must exist, so unbounded loops are rejected.
- **Multi-rate only through the kit's resampler node.**
- Content domains v1 include **read-only** transport state; **transport authoring is absent**.

The diagnostic ids are defined by the epic 023 front end; that list is still owned by epic 023. If an id is missing when this ticket starts, file it against 023 rather than matching on message text.

Two rejection categories settled after the split, a resampler node outside an edge module (C-134) and a resampler ratio other than 2:1 or 1:2 (C-105), are added by ticket 025a-08 on top of this ticket's harness support. Two later ones, an even-tap linear-phase FIR (C-183) and an odd-sample join on the device side of an edge module (C-190), are added by ticket 025a-21, and a source with controllers and more than 31 audio members (C-209) by ticket 025a-22.
## Work
1. Add `conformance/corpus/negative/` (or a `reject` marker in the expect format from 025a-01) with at least 15 programs, at least 3 per category: unbounded loop, transcendental call (e.g. `sin`, `exp`, `log`), allocation, cross-rate connection without the resampler, transport authoring.
2. Each expect file names the expected diagnostic id emitted by the epic 023 front end.
3. Extend the 025a-01 harness: a negative program passes only if compilation fails with exactly the expected diagnostic.
## Acceptance criteria
1. At least 15 negative programs, at least 3 in each of the 5 categories.
2. Every one is rejected with the expected diagnostic; the harness passes.
3. A negative program that compiles, or fails with a different diagnostic, fails the harness (verified by a deliberately wrong expectation in a unit test).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Diagnostic wording and implementation (epic 023); positive corpus (025a-02 to 025a-06); resampler placement and ratio rejections (025a-08); even-tap FIR and device-side join rejections (025a-21); the bundle-cap rejection (025a-22).
