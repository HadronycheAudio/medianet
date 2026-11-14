# 023-05 — Parser fuzz target
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-04

## Context
- Epic 023, 023-AC-9: **the parser and the manifest decoder are fuzzed within the C-95 budget: zero panics, zero hangs.** This ticket covers the parser (lexer + parser of `hadronyche-dsl/syntax/`); the manifest decoder fuzz target is 023-22.
- Fuzz budget (C-95, decided): **each parser 60 s on every change and 1 hour nightly; the corpus is kept.**
- DoD line: "Every parser touched is fuzzed".
- The fuzz target ships with the `hadronyche-dsl` crate (C-133 puts tool binaries in `hadronyche-tools` and throwaway probes in `probes/`; a fuzz target is neither).

## Work
1. Add a fuzz target (the fuzzing tool is the implementing team's choice, C-192; e.g. `cargo fuzz`, target `parse`) that feeds arbitrary bytes, converted lossily to UTF-8, through lexer and parser.
2. Seed the corpus with all parser test inputs and the 023-01 sketches.
3. Add a CI job that runs the target for 60 s on every change and a scheduled job that runs it for 1 hour nightly, both with a per-input timeout of 1 s to detect hangs.
4. Keep the corpus between runs (stored and restored by CI), as C-95 requires.
5. Fix every panic or hang found.

## Acceptance criteria
1. The fuzz target builds and runs in CI unprivileged.
2. The per-change run lasts 60 s and the nightly run 1 hour (C-95); each finds 0 panics and 0 timeouts with a per-input timeout of 1 s.
3. The corpus grown by one run is the starting corpus of the next.
4. Every crash found during development has a regression test in the parser test suite.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed

## Out of scope
- Fuzzing the manifest decoder (023-22); fuzzing the full `check()` pipeline (optional, not required by the epic).
