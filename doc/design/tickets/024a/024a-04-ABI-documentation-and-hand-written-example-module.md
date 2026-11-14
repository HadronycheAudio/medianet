# 024a-04 — ABI documentation and hand-written example module
**Epic:** 024a-Runtime-ABI-and-arithmetic-kit · **Effort:** S · **Depends on:** 024a-01, 024a-02, 024a-03

## Context
The ABI in `hadronyche-core::abi` is the contract between the runtime and any module. The epic requires it to be documented in the crate with one hand-written example module that uses no generated code. Rules to document `[decision]`:
- One process function `process(state, inputs at position p with per-input gap flags, position) -> outputs`; never blocks; no allocation at process time; state size and worst-case stack are compile-time constants (`state_bytes`, `stack_bytes`).
- Position invariant: output for p is derived from inputs at p; the module cannot set positions; structural latency L is exposed per output (`latency: Latency`, always a whole number of domain-rate samples, C-105) and the runtime stamps the output p − L; L is the module's structural delay in the domain-rate timeline (C-207); an edge's resampler conversion delay, and the varying delay of the rate actuator, are counted only in D, never in L, and the manifest reports the conversion delay separately on the device-facing port (C-134, C-170, C-189, C-207); each output carries a "frequency-dependent delay" property, for recursive filters (C-189).
- `InputSlice.gap == true` means the content is the generated default; `OutputSlice.degraded` follows the content: an output at position p is degraded if an input it depends on was gapped at p − L (C-122); for L = 0 this is an input flagged gap in this slice.
- Determinism classes ExactFixed / ExactFloat / Bounded; a hand-written module that does not claim exactness declares Bounded, and a generated module never does (C-123); `fingerprint_matches` is called on the processing thread after its floating-point mode is final.
- `ABI_VERSION = 1`; mismatches are rejected with a named reason (024a-02).
- Constants: slice 48 frames = 0.5 ms at 96 kHz (C-01, C-02); 4-byte samples, f32 or Q1.31 (C-03); 6 channels, 192 bytes per channel per slice, 1 152 bytes of audio, no per-channel id byte (C-04); bundles up to 32 members (C-08).

## Work
- Module-level rustdoc in `hadronyche-core/src/abi/mod.rs` covering every rule above.
- `hadronyche-core/examples/hand_gain.rs` (or a doctest): a hand-written exact-fixed gain module on one 6-channel audio input, Q1.31 samples, implementing `Module`, with `INFO` filled by hand (latency 0, frequency-dependent delay not set), gap handling (zero-fill) and degraded flag.
- The example is built and run in CI (`cargo test --examples` or doctest).

## Acceptance criteria
1. `cargo doc -p hadronyche-core` produces the `abi` module page with sections for each rule listed in Context.
2. The example implements `Module` without depending on `hadronyche-dsl` (checked: `hadronyche-dsl` is not in its dependency graph).
3. The example's test processes one 48-frame, 6-channel slice and checks output bytes; with `gap: true` on the input, the output is zeros and `degraded` is `true`.
4. The example passes `check_module_info(&INFO, ABI_VERSION)`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The ABI is documented in `hadronyche-core` with one hand-written example module that uses no generated code
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Design documents outside the crate (024a-15).
- Generated modules (part 024b).
