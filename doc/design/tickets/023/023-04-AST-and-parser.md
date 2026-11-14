# 023-04 — AST and parser
**Epic:** 023-DSL-compiler-front-end · **Effort:** M · **Depends on:** 023-03

## Context
- Part of `hadronyche-dsl/syntax/`: lexer, parser, AST, source spans, diagnostics.
- Grammar comes from the 023-01 decision record. Constructs the AST must represent: module header (name, edge-module marker); network port declarations (direction, content type, channels, optional bundle layout and channel map); device-facing port declarations with their rate (edge modules only, C-105); state cells; expressions over the closed node set (arithmetic, delay, table lookup, linear ramp, mix, the resampler node at 2:1 or 1:2, C-105, state cells); gap-override constructs; dense-controller initial values (C-158); the per-join opt-out construct (C-104); reads of read-only transport state (playing, bar, beat, tempo, time code at position p).
- Some 023-AC-1 rejections are naturally syntactic: the grammar has no loop or transport-authoring construct, so such input must fail with a diagnostic that names the construct (e.g. "loops are not part of the language", "transport authoring is not available in v1"), not a generic "unexpected token".

## Work
1. Define the AST in `syntax/ast.rs`; every node carries a `Span`.
2. Implement a recursive-descent (or equivalent) parser `syntax::parse(tokens) -> Result<Module, Vec<Diagnostic>>` with an explicit nesting-depth limit so deep nesting gives a diagnostic, not a stack overflow.
3. Error recovery: continue after an error to report several diagnostics in one run.
4. Dedicated diagnostics (with codes) for loop-like keywords and transport-authoring attempts.

## Acceptance criteria
1. Every construct of the 023-01 grammar outline, including the edge-module marker, device-facing ports and the join opt-out, has a parse test asserting AST shape.
2. A loop construct (e.g. `while`/`loop`/`for` if those keywords are reserved by 023-01) yields a diagnostic whose message names loops as unsupported; span covers the keyword.
3. An attempt to write transport state yields a diagnostic naming transport authoring as unavailable in v1.
4. Input nested 10 000 levels deep returns a diagnostic, no panic or stack overflow.
5. A source with 3 independent syntax errors reports at least 3 diagnostics.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed (fuzz target in 023-05)

## Out of scope
- Type and rate checking (023-08); edge-module rules (023-29); recursion and out-of-kit call rejection on resolved names (023-10); join padding (023-28); fuzz harness (023-05).
