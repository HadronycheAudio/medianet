# 024a-01 — ABI core types and Module trait
**Epic:** 024a-Runtime-ABI-and-arithmetic-kit · **Effort:** S · **Depends on:** epic 009, epic 010

## Context
The runtime calls every module (generated or hand-written) through one ABI defined in crate `hadronyche-core`, module `abi/`. This ticket creates the types and the `Module` trait; later tickets add the version check (024a-02), `fingerprint_matches` (024a-03) and documentation (024a-04).

Facts the ABI must encode `[decision]`:
- The runtime owns *when and whether* (positions, alignment, deadlines, gap classification); the module owns *what* (content of outputs for position p from inputs at p).
- One process function: `process(state, inputs at position p with per-input gap flags, position) -> outputs`. It never blocks and never allocates. State size and worst-case stack are compile-time constants.
- The module cannot set positions. If it has structural latency L, the runtime stamps the output p − L; the module exposes L as a constant per output. L is the module's structural delay in the domain-rate timeline and is always a whole number of domain-rate samples, in the manifest and the ABI (C-105, C-207). A resampler's conversion delay at an edge, and the varying delay of the rate actuator, are counted only in the plumbing delay D, never in L (C-134, C-189, C-207); the manifest reports the conversion delay separately, on the device-facing port (C-207, part 024b). Each output keeps a "frequency-dependent delay" property, for recursive filters (C-189).
- Determinism classes: exact-fixed (bit-exact everywhere), exact-float (bit-exact only where the fingerprint matches), bounded. Bounded arises only from an exact-float fingerprint mismatch and for hand-written modules that do not claim exactness; a generated module never declares it (C-123).
- `cost_ops_per_slice` counts abstract operations per slice, identical on every machine (C-106).
- Constants: slice 48 frames = 0.5 ms at 96 kHz, the only rate on the wire (C-01, C-02); samples are 4 bytes, f32 or Q1.31 (C-03); 6 channels per stream at defaults, 192 bytes per channel per slice, 1 152 bytes of audio, no per-channel id byte, the channel map is in the manifest (C-04); wide ports are bundles of up to 32 members, the controller member included (C-08, C-209).
- The crate is `no_std` and the module must not need `alloc`.

## Work
In `hadronyche-core/src/abi/` (no_std; shapes as below, names may be refined):

```rust
pub const ABI_VERSION: u16 = 1;
pub enum DeterminismClass { ExactFixed, ExactFloat, Bounded }
pub struct Latency { pub samples: u64 }   // whole domain-rate samples (C-105)
pub struct ModuleInfo {
    pub abi_version: u16,
    pub state_bytes: u32,
    pub stack_bytes: u32,
    pub cost_ops_per_slice: u64,
    pub determinism: DeterminismClass,
    pub inputs: &'static [PortInfo],
    pub outputs: &'static [OutputInfo],      // includes latency: Latency and the frequency-dependent delay property (C-189)
    pub fingerprint: Option<&'static Fingerprint>,
}
pub struct InputSlice<'a> {
    pub gap: bool,                 // true: content is the generated default
    pub frames: u16,
    pub audio: &'a [u8],           // 4-byte samples, channel-interleaved
    pub dense: &'a [u8],
    pub sparse: &'a [SparsePart],
    pub transport: Option<&'a TransportState>,
}
pub struct OutputSlice<'a> {
    pub degraded: bool,            // set by generated code from dataflow
    pub audio: &'a mut [u8],
    pub dense: &'a mut [u8],
    pub sparse: &'a mut SparseOut,
}
pub trait Module {
    const INFO: ModuleInfo;
    type State;
    fn init(state: &mut core::mem::MaybeUninit<Self::State>);
    fn process(state: &mut Self::State, inputs: &[InputSlice<'_>], position: u64,
               outputs: &mut [OutputSlice<'_>]);
}
pub struct Fingerprint { pub inputs: &'static [u32], pub expected: &'static [u32] }
```

- Define `PortInfo` and `OutputInfo` (`OutputInfo` carries at least `latency: Latency` and the "frequency-dependent delay" property for recursive filters, for example `frequency_dependent_delay: bool`, name may be refined; C-189 restores it after the earlier drop of `non_constant_delay`), and reuse or re-export `SparsePart`, `SparseOut`, `TransportState` from the content layouts of epic 010. Their definition is an open item owned by epic 010; if epic 010 has not provided them, define minimal versions here and record the divergence.
- All `ModuleInfo` fields must be constructible in a `const` context.
- Unit tests: a trivial test module implementing `Module` with a `const INFO`, called with 48-frame, 6-channel audio slices (48 × 6 × 4 = 1152 bytes per slice).

## Acceptance criteria
1. `hadronyche-core` builds with `#![no_std]` and without the `alloc` crate.
2. A test module declares `const INFO: ModuleInfo` with `abi_version: ABI_VERSION` (= 1) and compiles; `ModuleInfo` is fully const-constructible.
3. A test calls `init` then `process` on that module with one 1152-byte audio input (48 frames, 6 channels, 4-byte samples) and one output, and the output bytes match the expected content.
4. `OutputInfo` exposes `latency: Latency` as a whole number of domain-rate samples (C-105); a test declares and reads back from `INFO` a latency of 72 samples (not a whole number of 48-frame slices).
5. `InputSlice.gap` and `OutputSlice.degraded` exist as plain `bool` fields.
6. `OutputInfo` exposes the frequency-dependent delay property (C-189); a test declares it set on one output and clear on another in a `const INFO` and reads both back.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Types derive or document the traits needed by later tickets (`Debug`, `Clone`, `Copy`, `PartialEq` on `DeterminismClass`)

## Out of scope
- Version rejection logic (024a-02), `fingerprint_matches` (024a-03), crate documentation and hand-written example module (024a-04).
- The runtime's slice delivery and gap classification (epic 009); content message layouts and bundle demultiplexing (epic 010).
