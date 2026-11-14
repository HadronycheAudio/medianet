# 017a-11 — Fuzz the reference modules' payload decoders
**Epic:** 017a-Reference-modules · **Effort:** S · **Depends on:** 017a-07, 017a-08

## Context
- Definition-of-done item of epic 017a: **every parser touched is fuzzed**.
- Parsers touched by this part: the parts payload decoder in the reference echo (017a-07) and the audio and controller payload decoder in the reference mixer (017a-08). The other two parsers of the original epic 017 (the injection record in the stamp sections and the L0 record-file reader) belong to epic 017b and are fuzzed in 017b-24.
- **Fuzz budget (settled, C-95):** each parser 60 s on every change and 1 hour nightly; the corpus is kept.
- CI runs unprivileged. The fuzz job is separate from the L0 job, so it does not count against L0's 10-minute budget (AC-017.13, epic 017b).
- **Tooling (settled, C-192):** the fuzzing tool is the implementing team's choice; the vectors crate sits at the workspace root.
- Bound on processing one bad input: a proposal of linear in packet length and at most 10 µs per packet, pending L1 (C-164). It is not asserted here: fuzzing checks for crashes and panics, not timing.

## Work
1. Add fuzz targets with the fuzzing tool the implementing team chooses (C-192; e.g. `cargo fuzz` under `hadronyche-reference/fuzz/`) for the echo's parts payload decoder and the mixer's audio and controller payload decoder.
2. Seed corpora from payloads produced by the reference sender (017a-09) and the conformance vectors of epic 007 (vectors crate at the workspace root, C-192) where they apply; keep the corpus between runs.
3. Run each target 60 s on every change and 1 hour nightly (C-95).

## Acceptance criteria
1. One fuzz target per parser listed above; each builds and runs.
2. Each target runs 60 s on every change and 1 hour nightly without a crash or panic.
3. The corpus persists between runs.
4. Any crash found is fixed and its input added as a regression test.

## Definition of done
- [ ] Every parser touched is fuzzed
- [ ] Tests green in CI, unprivileged

## Out of scope
- The injection record and the L0 record-file reader (017b-24).
- Fuzzing runtime packet parsers owned by epics 010/012 (covered there).
