# 013-03 — Stamp parser fuzz target
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-02

## Context
- `Stamp::parse(bytes: &'a [u8]) -> Result<Stamp<'a>, Reject>` in `hadronyche-core` decodes the regime stamp: a versioned envelope of length-prefixed, versioned sections; unknown sections are skipped and counted. `[decision]`
- Epic acceptance criterion 013-AC-1: every parser of the epic **survives fuzzing with zero panics**. Definition of done: every parser touched is fuzzed.
- Fuzz budget (C-95): **each parser 60 s on every change and 1 hour nightly; the corpus is kept.**
- The fuzzing tool is the implementing team's choice (C-192); the target path below is an example.
- Stamps arrive over the network from any node, including embedded devices (multicast to the control group and unicast replies, C-18), so the parser handles untrusted input.

## Work
1. Add a fuzz target (e.g. `hadronyche-core/fuzz/fuzz_targets/stamp_parse.rs`) that calls `Stamp::parse` on arbitrary bytes and, on success, calls every accessor (`id()`, `qualified()`, `is_complete()`, section readers as they exist, skipped-section count).
2. Add a round-trip target: parse, and if `Ok`, re-encode through `StampBuilder` where possible and parse again.
3. Provide the shared harness that other parsers of the epic plug into (RecordHeader decoder 013-14, split-section reassembly 013-13), one target per parser.
4. Seed the corpus with valid stamps produced by the builder and, when 013-12 lands, the vectors from epic 007. Keep the corpus in the repository (or in a CI cache that persists across runs) so it grows over time.
5. Add CI jobs: 60 s per target on every change to `hadronyche-core/src/stamp*`, and 1 hour per target nightly, both unprivileged.
6. Document in the module docs that each later section ticket must extend the corpus with a stamp containing its section.

## Acceptance criteria
1. The fuzz targets build and run in CI, unprivileged.
2. The per-change job runs each target for 60 s and the nightly job for 1 hour (C-95), as stated in the CI config; both report zero panics, zero out-of-bounds accesses and zero timeouts.
3. The corpus is kept between runs: a second nightly run starts from the first run's corpus.
4. The seed corpus contains at least one valid stamp and one stamp with an unknown section.
5. Any crash found is fixed and its input added to a regression test.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Regime stamp fields touched are complete and documented", "Every stamp field has a stated source on each of the four profiles" and "The stamp format is versioned independently of the media format" (no field or format change); "Design documents updated where behaviour differs, constants table first" (implements C-95 as written); "Every new factual claim carries an evidence tag" (no factual claim is made).

## Out of scope
- Section contents (013-06 to 013-10, 013-23, 013-25) — those tickets extend the corpus.
- Byte-for-byte vectors (013-12).
