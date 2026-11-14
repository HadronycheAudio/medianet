# 025b-01 — Recording format and parser
**Epic:** 025b-Replay-cross-target-fingerprint-and-cost-calibration · **Effort:** S · **Depends on:** epic 007
## Context
Record and replay is the exact verification tier for exact-class modules `[decision]`: record input slices and output hashes; replay the same generated code offline; compare.

A **recording** is, for one module: the slices delivered to it (with per-input gap flags and their positions) and the hashes of what it emitted, per output per slice, plus each output's degraded mark. It is written by the runtime recorder (`hadronyche-std/src/record.rs`, 025b-02) and read by `hadronyche-replay` (025b-03). The replay shape is `pub fn replay<M: Module>(recording: &Recording) -> ReplayOutcome;` so a shared `Recording` type is needed.

The format and the output hash are not given by the epic or the constants table; this ticket owns the choice (reuse the epic 007 vector format and hash where possible). Falsifier: a recording cannot represent a case the replay must check (gap, degraded mark, effective class).

The header records the module's effective determinism class: `exact-fixed`, `exact-float`, or `bounded` when the fingerprint mismatched (C-123).

Constants: domain rate 96 kHz, the only rate on the wire (C-01); slice = 48 frames = 0.5 ms (C-02); 4-byte samples (C-03); 6 channels per stream, 192 bytes per channel per slice, 1 152 bytes of audio, no per-channel id byte (C-04). Fuzz budget: 60 s on every change and 1 hour nightly, corpus kept (C-95); the fuzzing tool is the implementing team's choice (C-192).
## Work
1. Define the `Recording` type and its on-disk format: header (module identity and generated-code version, effective determinism class, fingerprint result, sample format), then per slice: position, per-input gap flag and slice data, per-output hash and degraded mark.
2. Write the writer and the streaming reader in a crate both `hadronyche-std` and `hadronyche-tools` can depend on.
3. Fuzz the reader.
4. Document the format in `doc/design/spec/` (C-130).
## Acceptance criteria
1. Write-then-read round-trips a synthetic recording of 120 000 slices (one minute at 0.5 ms per slice) byte for byte.
2. A truncated or corrupted header is rejected with an error, not a panic.
3. The reader fuzz target runs 60 s on every change and 1 hour nightly without a crash, and its corpus is kept (C-95).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Runtime recording (025b-02); replay logic (025b-03).
