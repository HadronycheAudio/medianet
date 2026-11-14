# 024b-12 — `hadronyche-dsl build` command producing module, manifest and vectors
**Epic:** 024b-DSL-code-generator · **Effort:** S · **Depends on:** 024b-03 … 024b-11, 024b-21, epic 004, epic 010, epic 023

## Context
The epic deliverable: a command `hadronyche-dsl build <source> --out <dir>` producing the Rust module, the manifest (binary and readable) and a vectors file. The DSL has no non-deterministic node in v1, so every generated module is exact-fixed or exact-float (C-123) and every build writes a vectors file.

The manifest schema and its binary and readable formats come from epic 023 (an open item owned by epic 023); this ticket writes it. Settled content: each output's L as a whole number of domain-rate samples (C-105); the delay inserted at each padded join and the flag on each join padded on structural latency only because a path has frequency-dependent delay (C-104); an edge module's conversion delay, a component of D and not of L, reported separately on the device-facing port (C-134, C-207, 024b-05); each dense controller's declared initial value, default 0 (C-158); `cost_ops_per_slice` in counted abstract operations (C-106); the channel map, since packets carry no per-channel id byte (C-04); controller placement (C-181, C-205, C-209). Determinism class, `state_bytes`, `stack_bytes`, inputs, outputs (with `latency` and the "frequency-dependent delay" property, C-189, 024b-21) and fingerprint are the facts of `ModuleInfo`.

Controller placement: a bundle source's controllers always ride in their own member stream, which counts towards the 32 members, so a source with controllers has at most 31 audio members (186 channels) (C-08, C-205, C-209). A single-stream source may always put its controllers in a second stream, and must do so when headers plus controllers exceed the 100-byte cap (C-137); it may share the audio packet while they fit (C-181). The front end (epic 023) implements the rejections; this command records the placement the front end decided and only asserts that it never receives a program that breaks these rules (C-218). This replaces the former ticket 024b-22, deleted. Header sizes come from epic 004 and controller encodings from epic 010. Epic acceptance criterion 024b/AC-10.

For exact-float modules the vectors file holds the fingerprint. Its content for exact-fixed modules is an open item owned by task 025 (falsifier: the conformance suite cannot check bit identity from the file); until it rules, this ticket writes a seeded input vector and the expected output bits computed the same way, and marks the format version.

## Work
- Subcommand `build` in the `hadronyche-dsl` binary: run front end (epic 023) then codegen; write into `<dir>`:
  - the Rust module source (e.g. `<name>.rs`),
  - the manifest in binary form (e.g. `<name>.manifest.bin`) and readable form (e.g. `<name>.manifest.toml`), with the same content,
  - a vectors file (e.g. `<name>.vectors.bin`) with a format version, holding input words and expected output bits.
- A decoder for the binary manifest and the vectors file (used by tests and by 024b-14 fuzzing).
- Exit codes and named error messages for front-end and codegen errors.
- Record each source's controller placement in the manifest as the front end decided it, and assert that a bundle source with controllers has at most 31 audio members and its controllers in their own member stream, and that a single-stream source shares the audio packet only when headers plus controllers fit 100 bytes; a violation is an internal compiler error naming the rule, written before any output (C-137, C-181, C-205, C-209, C-218).

## Acceptance criteria
1. `hadronyche-dsl build gain.dsl --out out/` creates the module, both manifests and the vectors file; nothing else.
2. Binary and readable manifests decode to equal structures (round-trip test), including an L of 72 samples (not a whole number of slices), an inserted join delay, a flagged frequency-dependent join, an edge conversion delay carried on the device-facing port and in no output's L (C-207), and a dense initial value.
3. Every corpus program produces a vectors file, and no manifest declares the bounded class (C-123).
4. An invalid source exits non-zero with a named error and writes no partial output.
5. Running `build` twice yields byte-identical outputs.
6. A bundle source with controllers and 31 audio members (186 channels) builds with the controllers in their own member stream in the manifest; a single-stream source's controllers are recorded in the audio packet when headers plus controllers fit 100 bytes, and in a second stream when the source places them there, at any size (C-181, C-205, C-209).
7. A hand-built input that bypasses the front end with a bundle source with controllers and 32 audio members, or a single-stream source sharing the audio packet one byte beyond the cap, trips the assertion as an internal compiler error and writes no output; the user-facing rejections are tested by epic 023 and epic 025a (C-218).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Fuzzing the decoders (024b-14).
- The manifest schema itself (epic 023).
- The controller-placement and bundle-cap rejections (front end, epic 023, C-218) and their tests (conformance corpus, epic 025a).
