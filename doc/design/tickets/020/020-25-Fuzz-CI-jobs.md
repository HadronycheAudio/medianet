# 020-25 — Fuzz CI jobs
**Epic:** 020-L2-certification-harness · **Effort:** XS · **Depends on:** 020-02, 020-09

## Context
The L2 harness has two parsers: the recording format (020-02, cargo-fuzz target `recording_parse`) and the rig file (020-09, cargo-fuzz target `rig_parse`). The epic's definition of done requires every parser touched to be fuzzed.

The fuzz budget is settled by C-95 `[decision]`: **each parser 60 s on every change and 1 hour nightly; the corpus is kept.**

## Work
- Add a per-change CI job that runs `recording_parse` and `rig_parse` for 60 s each, unprivileged, and fails on any crash or panic.
- Add a nightly CI job that runs each target for 1 hour.
- Keep the corpus: the jobs start from the committed seed corpus plus the corpus kept from earlier runs, and store the grown corpus where the next run picks it up.
- On a crash, keep the reproducing input as a CI artefact and as a new seed.

## Acceptance criteria
1. A change touching `hadronyche-tools` triggers the 60 s run of both targets (C-95).
2. The nightly job runs each target for 1 hour (C-95).
3. The corpus of a run is available to the next run of the same target; its size is printed in the job log.
4. A deliberately planted panic in either parser makes the per-change job fail and keeps the reproducing input.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The parsers and their fuzz targets (020-02, 020-09).
- Fuzz jobs for parsers outside this epic.
