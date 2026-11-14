# 023-02 — Crate skeleton, source spans and diagnostics
**Epic:** 023-DSL-compiler-front-end · **Effort:** XS · **Depends on:** 023-01

## Context
- Epic 023 delivers crate `hadronyche-dsl` with modules `syntax/`, `types/`, `ir/`, `analysis/` (`latency.rs`, `cost.rs`, `memory.rs`, `determinism.rs`, `dataflow.rs`), `manifest.rs`, and `LANGUAGE.md`.
- Public entry point given by the epic (names may be refined):
  `pub fn check(source: &str) -> Result<(Ir, Manifest), Vec<Diagnostic>>;`
- The epic defines "clear diagnostic" (023-AC-1): every rejection is a `Diagnostic` with a source span, a **stable code** and a message that names the violated rule. This task defines the codes; `LANGUAGE.md` lists them (023-26).

## Work
1. Create crate `hadronyche-dsl` with empty modules `syntax/`, `types/`, `ir/`, `analysis/{latency,cost,memory,determinism,dataflow}.rs`, `manifest.rs`.
2. In `syntax/`: `Span { start: u32, end: u32 }` (byte offsets) and `Diagnostic { span: Span, severity, code: DiagCode, message, notes }`, plus a renderer that prints the source line with a caret under the span.
3. `DiagCode`: a closed enum with a stable textual form (for example `E0001`); a code once published is never reused for another rule. Seed it with one code per 023-AC-1 rejection category; later tickets add codes.
4. Stub `check` with placeholder `Ir` and `Manifest` types that returns a "not implemented" diagnostic.
5. Add the crate to CI (build + test, unprivileged).

## Acceptance criteria
1. `cargo test -p hadronyche-dsl` passes in CI without elevated privileges.
2. A unit test renders a diagnostic for a 3-line source with a span on line 2 and asserts the caret column equals the span start column.
3. `check("")` returns `Err` with exactly 1 diagnostic.
4. A unit test asserts that every `DiagCode` has a distinct textual form.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Lexing/parsing (023-03, 023-04), real IR (023-09), real manifest (023-19).
