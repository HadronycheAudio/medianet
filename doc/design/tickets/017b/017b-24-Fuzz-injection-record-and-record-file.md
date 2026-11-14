# 017b-24 — Fuzz the injection record and the record-file reader
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017b-05, 017b-09

## Context
- Definition-of-done item of epic 017b: **every parser touched is fuzzed**.
- Parsers touched by this part: the injection record in the stamp sections (017b-05) and the L0 record-file reader (017b-09). The reference modules' payload decoders are fuzzed in epic 017a (017a-11).
- **Fuzz budget (settled, C-95):** each parser 60 s on every change and 1 hour nightly; the corpus is kept.
- CI runs unprivileged. The fuzz job is separate from the L0 job, so it does not count against L0's 10-minute budget (AC-017.13).

## Work
1. Add fuzz targets (e.g. `cargo fuzz` under `hadronyche-std/fuzz/` and `hadronyche-tools/fuzz/`) for the injection-record parser and the record-file reader.
2. Seed corpora from real L0 record files and stamp sections; keep the corpus between runs.
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
- The reference modules' payload decoders (017a-11).
- Fuzzing runtime packet parsers owned by epics 010/012 (covered there); the `malformed` scenario (017b-19).
