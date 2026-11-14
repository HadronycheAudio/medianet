# 024b-16 — Stack bound emission and measurement
**Epic:** 024b-DSL-code-generator · **Effort:** M · **Depends on:** 024a-06, 024b-12

## Context
Rule `[decision]`: state size and worst-case stack are compile-time constants emitted with the code, because devices need static memory. `ModuleInfo.stack_bytes: u32` carries the worst-case stack of `process`. Epic acceptance criterion 024b/AC-2: "Bounded stack. Measured stack use at process time never exceeds `stack_bytes` on the hosted target and on one device target."

Settled by the epic: `stack_bytes` is produced by this epic's code generator from the compiled artefact. The method is an open sub-question owned by this ticket (falsifier: measured stack exceeds the declared value on any listed target): a post-compile analysis (for example `-Z emit-stack-sizes` and the call graph), an epic 023 estimate plus a measured margin, or both with a check.

## Work
- Decide and document the method and implement it so the value written in `INFO` and the manifest is an upper bound.
- Measurement harness: paint the stack with a pattern before `process`, run `process` on worst-case corpus inputs (including gap inputs), measure the high-water mark. Run on the hosted target and on the 32-bit device target (emulated, 024a-06).
- Record measured versus declared values per corpus program `[measured]`.

## Acceptance criteria
1. For every corpus program, measured stack high-water mark ≤ `stack_bytes` on the hosted target.
2. Same on the 32-bit device target.
3. A fixture whose declared `stack_bytes` is deliberately too small is detected (test fails).
4. A table of declared versus measured stack per program is committed in `doc/design/reports/` (C-130), tagged `[measured]`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- `state_bytes` (checked by const assertion in 024b-03).
- Device fitting for epic 026.
