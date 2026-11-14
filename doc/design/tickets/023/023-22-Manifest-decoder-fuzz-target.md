# 023-22 — Manifest decoder fuzz target
**Epic:** 023-DSL-compiler-front-end · **Effort:** XS · **Depends on:** 023-05, 023-21

## Context
- Epic 023, 023-AC-9: **the parser and the manifest decoder are fuzzed within the C-95 budget: zero panics, zero hangs.** The parser target is 023-05; this ticket covers `Manifest::decode(&[u8]) -> Result<Manifest, DecodeError>` from 023-21.
- Fuzz budget (C-95, decided): **each parser 60 s on every change and 1 hour nightly; the corpus is kept.** The CI jobs are set up in 023-05.
- DoD line: "Every parser touched is fuzzed".

## Work
1. Fuzz target `manifest_decode` feeding arbitrary bytes to `Manifest::decode`; on `Ok(m)`, also assert `decode(encode(m)) == Ok(m)`.
2. Seed corpus with encoded manifests of all analysis-corpus programs.
3. Add the target to the 023-05 CI jobs (60 s per change, 1 hour nightly, per-input timeout 1 s, corpus kept).

## Acceptance criteria
1. Target builds and runs in CI unprivileged.
2. The per-change run lasts 60 s and the nightly run 1 hour (C-95); each finds 0 panics, 0 timeouts, 0 round-trip mismatches.
3. The corpus grown by one run is the starting corpus of the next.
4. Every crash found has a regression test.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed

## Out of scope
- Parser fuzzing (023-05).
