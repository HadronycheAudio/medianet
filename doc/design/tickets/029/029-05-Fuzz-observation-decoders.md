# 029-05 — Fuzz observation decoders
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-02, 029-03, 029-04

## Context
- `hadronyche-monitor` (epic 029) decodes untrusted bytes from the discovery and control groups and from recorded stamp files. Epic criterion 029-AC-8: the observation decoders are fuzzed with zero panics. Definition of done: every parser touched is fuzzed.
- Fuzz budget (C-95): each parser 60 s on every change and 1 hour nightly; the corpus is kept.
- Decoder entry points in this crate: hello decode (029-03), `StampAssembler::push_section` (029-02, sections at most 1 252 bytes, C-17, C-18), recorded-file record parser (029-04). The wire decoders themselves live in `hadronyche-core` (C-133) and are fuzzed there.

## Work
1. Add fuzz targets under `hadronyche-tools/fuzz/`, using the fuzzing tool the implementing team chooses (C-192): `fuzz_hello`, `fuzz_stamp_sections` (sequence of sections for several module ids and versions), `fuzz_stamp_file`.
2. Seed corpora from real captures and recordings of the test medianet; keep the corpus in the repository and grow it from each run (C-95).
3. A CI job runs each target for 60 s on every change to the crate, and a nightly job runs each target for 1 hour (C-95); both fail on any crash.

## Acceptance criteria
1. Three fuzz targets exist and build.
2. The per-change job runs each target for 60 s and the nightly job for 1 hour, with zero panics, zero OOMs and zero timeouts; the first nightly run's execution counts are recorded in `doc/design/reports/` (C-130).
3. Both CI jobs are green; the corpus is kept between runs.
4. Any crash found is fixed and its input added as a regression test.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Fuzzing the epic 013/014/015 wire decoders in `hadronyche-core` (owned there); this ticket fuzzes the monitor's entry points that wrap them.
