# 025a-12 — Latency probe core
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** M · **Depends on:** epic 017, epic 024
## Context
Positional delay cannot be seen live: timestamps show how long a module took (residence), not whether the content in the slice stamped p is really input p − L. A wrong L silently skews every downstream alignment. The only check is a **probe**: send a marker through the module in test mode and locate it in the output.

Structural latency L is a compile-time constant, always a **whole number of domain-rate samples** in the manifest and ABI (C-105; the earlier rational-L rule is withdrawn): explicit delays, lookahead, block buffering, linear-phase filter centres. An edge's resampler delay is never part of L: it is counted only in D, and the manifest reports it separately as the conversion delay on the device-facing port (C-207). The runtime stamps output at p − L. The probe reports in the same unit so it can be compared exactly.

The probe lives in crate `hadronyche-tools` (tool `hadronyche-probe`, C-133). It must work on **generated (epic 024) and hand-written (epic 017) modules**, through the module interface both implement (`Module`). Shape given by the epic (names may be refined; `Samples` is the ABI's whole-number type for L, C-105):
```rust
pub enum Marker { Impulse, PseudoRandom { seed: u64, len: u32 } }
pub enum Delay { Constant(Samples), NonConstant { min: Samples, max: Samples } }
pub struct ProbeResult { pub output: usize, pub delay: Delay, pub confidence: f32 }
pub fn probe<M: Module>(marker: Marker, slices: usize) -> Vec<ProbeResult>;
```
Constants: domain rate 96 kHz, the only rate on the wire (C-01); slice = 48 frames = 0.5 ms (C-02); 6 channels per stream (C-04).
## Work
1. In `hadronyche-tools`, a library module `probe` with `Marker`, `Delay`, `ProbeResult`, and `probe<M: Module>`.
2. Instantiate the module in test mode; feed the marker at a known position on each input, silence/neutral elsewhere; run `slices` slices.
3. Locate the marker in each output by cross-correlation; report `Delay::Constant` with a whole number of domain-rate samples relative to the injection position, and a `confidence` (e.g. normalised peak height over next-highest peak). `NonConstant` detection is 025a-13.
4. Unit tests with hand-written modules: pass-through (L = 0), fixed delay of 1, 48, 1 000 samples, two-output module with different delays.
## Acceptance criteria
1. For a hand-written delay of d ∈ {0, 1, 47, 48, 49, 1 000} samples, `probe` reports exactly `Constant(d)` for both impulse and pseudo-random markers.
2. For a two-output module with delays 3 and 100, results are per output and exact.
3. Confidence for those clean delays is above a documented threshold; for a module that outputs silence, confidence is below it.
4. Works on at least one generated DSL module (e.g. a corpus gain program) as well as hand-written ones.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Non-constant-delay detection (025a-13); CLI and user docs (025a-14); running over the corpus (025a-15). No fractional delay is measured: L is always a whole number of samples (C-105).
