# 017a-06 — Analyzer tolerance tier and per-voice projections
**Epic:** 017a-Reference-modules · **Effort:** S · **Depends on:** 017a-04, 017a-01, epic 002

## Context
- Tolerance tier `[decision]`: non-exact modules are within stated epsilons; mechanism is analyzer windows and **per-voice projections**.
- POC source `[code]`: `src/audio.rs` (`analyse` was never read by the design; epic 002 reads it).
- Open sub-question (owner: epic 002): are voice windows compared bit for bit in the POC, or only within tolerance? If tolerance-only, voices are compared here rather than in the exact tier (017a-05).
- Settled: the tolerance tier is where the bounded class arises for hand-written (non-DSL) modules that do not claim exactness (C-123).
- Open sub-question (owner: epic 002): the epsilon values. They come from epic 002's reading of `analyse` and are stated as named constants, added to the constants table through 017a-12. Falsifier: the reading yields no epsilon, leaving the tier without an oracle.

## Work
1. Port the per-voice projection and windowed comparison from POC `audio.rs` into `analyzer/tolerance.rs`.
2. Express each epsilon as a named `pub const` with its unit and an evidence tag in its doc comment.
3. Count tolerance results per family (`within`, `outside`) in `analyzer::Report`, reusing the window states of 017a-04.
4. Header comment citing POC file/lines and the epic 002 finding.

## Acceptance criteria
1. A stream compared to itself: 100 % `within`.
2. A stream with added noise of 0.5 × epsilon: 100 % `within`; with 2 × epsilon: 100 % `outside`.
3. Projections of a known two-voice signal recover each voice's parameters within the stated epsilon (unit test with synthetic voices from `instrument::voices`).
4. Every epsilon constant has a doc comment with unit and evidence tag.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Each ported POC piece cites its source file and the archaeology finding that cleared it
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Exact-tier comparison (017a-05); window assembly (017a-04).
