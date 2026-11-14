# 024a — Runtime ABI and arithmetic kit

**Phase:** P4 (C-197) · **Scope:** L (15 tickets, about 17 days) · **Depends on:** 009, 010 · **Blocks:** 018, 021, 022a, 024b, 025, 026, 023

Epic 024 is split into two parts by rule C-143: this part (024a) delivers the runtime ABI and the arithmetic kit; part 024b delivers the DSL code generator, which compiles against both. A reference to "epic 024" means both parts.

## Goal

Build the contract that every DSL-generated or hand-written module is compiled against: (1) the **runtime ABI** in `hadronyche-core` through which the runtime calls any module's process function, and (2) the **arithmetic kit** `hadronyche-kit`, the only arithmetic library generated code may use, including the edge resampler and the edge software rate actuator. Part 024b's code generator emits code against these two crates.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules exchange *slices* — all content of one stream for one stretch of media time — addressed by *position* on a domain-wide timeline (`position = (network_time − epoch_origin) × rate`, where rate is the domain rate, 96 kHz, C-01). Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers. Every edge, Linux, macOS, Windows or embedded, uses the kit's resamplers (C-197). The same generated code must run on a hosted machine and on a `no_std` device.

**Boundary.** The runtime owns *when and whether*: it stamps positions, aligns inputs, fires deadlines, classifies gaps; on hosted platforms the hosted backend (epic 015) emits the gap slice when a module misses its own deadline (C-144). Module code owns *what*: the content of the outputs for position p given the inputs at position p.

**Rules for module code that the ABI and kit must encode `[decision]`.**

- **One process function:** `process(state, inputs at position p with per-input gap flags, position) -> outputs`.
- **Never blocks. No allocation at process time.** State size and worst-case stack are compile-time constants carried in `ModuleInfo`. Devices need static memory. For generated modules `stack_bytes` is produced by part 024b's code generator from the compiled artefact.
- **No libm, no transcendental functions.** Only the kit: strict f32 operations; Q1.31 (signed) and Q0.32 (unsigned, range [0, 1)) fixed point; shipped deterministic tables; the resampler node. The rounding rule of the Q1.31 multiply is any fixed rule chosen and documented in the kit; table size, checksum algorithm and interpolation rule are chosen by the kit and recorded in each table's generation note.
- **Float operations are never reordered**; no fused multiply-add contraction; summation order is defined by DSL semantics. No auto-vectorisation that changes results.
- **Position invariant.** Output for position p is derived from inputs at p. The module cannot set positions. If the module has structural latency L, the *runtime* stamps the output p − L; module code simply produces the delayed content and exposes L as a constant. L is always a whole number of domain-rate samples, in the manifest and in the ABI (C-105).
- **Resampler node.** It is offered only in designs whose delay is a whole number of domain-rate samples: 2:1 and 1:2; other ratios are out of v1 (C-105). It may appear only inside an edge module; the compiler rejects it anywhere else (C-134). The wire carries the domain rate only (C-01); an edge whose hardware runs at 48 kHz converts 2:1 through it (C-80). An edge's conversion delay is counted only in its plumbing delay D, never in any output's L (C-134, C-207); L is the module's structural delay in the domain-rate timeline, and the manifest reports the conversion delay separately, on the device-facing port (C-207); D is a sink's total plumbing delay from hand-off to emission, driver buffer, resampler and converter included, and for capture the mirror (C-170).
- **Software rate actuator.** It is not the DSL resampler node: the DSL node is fixed-ratio (2:1 or 1:2, whole-sample delay, C-105), and an edge's rate actuator is a separate, steerable resampler whose ratio varies by parts per million around its nominal value to track the hardware crystal (C-189). Its trait, the rate-actuator interface every edge consumes, is defined in this part together with its implementation, with a ratio range of ±1 000 ppm around the nominal ratio (C-210); that range covers the L2 drift ramp (±100 ppm at no more than 0.5 ppm/s, C-119), and the edge rate-loop bandwidth is well below 1 Hz (C-68); L2 requires zero accumulated slip (C-76); laptop sleep resets it (C-124). Its varying delay is part of D (C-134, C-170, C-189, C-207); no output L reports it.
- **Single builder of both resamplers.** This part builds both the fixed 2:1 / 1:2 node and the steerable rate actuator, once; every edge (Linux, macOS, Windows, embedded) uses the kit's and builds none of its own. That is why this part sits in phase P4, ahead of the Linux edge, and blocks epics 018, 021, 022a and 026 (C-197).
- **Frequency-dependent delay.** `OutputInfo` keeps a "frequency-dependent delay" property, for recursive filters (C-189). It does not report resampler delay, which is part of D.
- **Even-tap linear-phase FIR.** Rejected by the compiler in v1, with a diagnostic suggesting an odd-tap design, so that L stays exact (C-183). The kit's resampler filters are odd-tap linear-phase designs, consistent with that rule.
- **Degraded flag** follows the content (C-122): an output at position p is degraded if and only if an input it depends on was flagged gap at p − L. The ABI carries the input gap flag and the output degraded flag; generated code sets the latter (part 024b).
- **Determinism class** is carried as a constant: exact-fixed (bit-exact everywhere), exact-float (bit-exact only where the fingerprint matches), bounded. The bounded class arises only from an exact-float fingerprint mismatch and for hand-written (non-DSL) modules that do not claim exactness; the DSL has no non-deterministic node in v1, so a generated module never declares bounded (C-123).
- **Fingerprint.** An exact-float module carries a compile-time input vector and expected output bits. The runtime evaluates it at start **on the processing thread itself**, because floating-point modes are per thread and audio stacks change them. Match ⇒ exact-float eligible; mismatch ⇒ the module runs as bounded-class and the regime stamp says so. Vector layout: words are laid out by input port in `INFO.inputs` order, 48 frames per slice, channel-interleaved 4-byte samples, as many slices as the vector holds; the state is a fresh one from `init`; an exact-float module without a fingerprint is not exact-float eligible.
- **Cost bound.** `cost_ops_per_slice` counts abstract operations per slice and is identical on every machine. Each target has a calibration factor (time per operation), measured once and recorded in the regime stamp; admission converts with it (C-106). The conformance suite (epic 025) measures the factor with a reference workload once per listed target (C-176).

**Constants.** Slice 48 frames = 0.5 ms at 96 kHz (C-01, C-02); 96 kHz is the only rate on the wire, so a 48 kHz edge port carries 24 frames per slice. Samples are 4 bytes, f32 or Q1.31 (C-03). A stream carries 6 channels at defaults: 192 bytes per channel per slice, 1 152 bytes of audio per slice, no per-channel id byte; the channel map is in the manifest (C-04). Wide ports are bundles of up to 32 members, the controller member included (C-08, C-209). Edge hardware at 48 kHz converts 2:1 through the kit's resampler node (C-80). The machine pair for cross-machine bit-exactness is x86-64 and 64-bit ARM (C-151).

**Where outputs live.** Specifications go in `doc/design/spec/`, reports and measurement results in `doc/design/reports/`, the constants table is `doc/design/02-constants.md` (C-130); the architecture specification is `doc/design/01-architecture.md` (C-171). Throwaway probes go in `probes/`; harness and tool binaries (symbol checker, table generators) go in the `hadronyche-tools` crate; protocol types live in `hadronyche-core` (C-133). Nothing is written into the frozen POC tree.

**Evidence tags.** `[code]`, `[measured]`, `[kernel-doc]`, `[memory]`, `[decision]`, and `[standard]` for a figure cited from a named, published standard with its clause (C-132), for example IEEE 754 behaviour.

**POC lessons that the kit generalises `[code]`.**

- `src/music.rs:263` `sine()`: sin(2πx) in single precision from additions and multiplications only, folded to a quarter cycle and evaluated as a short series, "so that it gives the same bits on every machine". Port it.
- `src/music.rs:281` `mul_high`: 32 × 32-bit unsigned product, high word; the basis of the fixed-point path. Port it.
- `src/music.rs:311`: the fixed-point sine table is built at start with libm `f64::sin` and rounded to i32. A libm that differs by one unit in the last place could flip an entry, so the fixed path is not strictly cross-machine deterministic. **Ship tables as constants** (or generate them with add/multiply only) and pin them with a checksum test.
- `src/music.rs:340` / `:382`: float and fixed renderers of the same instrument, with every field moving linearly from its current value to its target over the block. This is the semantics of a sparse keyed controller.

**Cross-platform float exactness (why exact-float is scoped).** Results can differ through: fused multiply-add contraction; non-IEEE handling of subnormals (flush-to-zero or denormals-are-zero modes, set per thread); NaN payloads; 80-bit intermediate precision on legacy x86 targets. The kit avoids the first by construction; the fingerprint detects the others at run time. Exactness between a host and a small device is promised only for exact-fixed.

## Deliverables

**Crate `hadronyche-core`, module `abi/`** (no_std; shapes, names and integer widths may be refined):

```rust
pub const ABI_VERSION: u16 = 1;

pub enum DeterminismClass { ExactFixed, ExactFloat, Bounded }

/// Structural latency: always a whole number of domain-rate samples (C-105).
pub struct Latency { pub samples: u64 }

/// Compile-time facts about a module. All `const`.
pub struct ModuleInfo {
    pub abi_version: u16,
    pub state_bytes: u32,
    pub stack_bytes: u32,
    pub cost_ops_per_slice: u64,             // counted abstract operations (C-106)
    pub determinism: DeterminismClass,       // never Bounded for a generated module (C-123)
    pub inputs: &'static [PortInfo],
    pub outputs: &'static [OutputInfo],      // includes latency: Latency and the frequency-dependent delay property (C-189)
    pub fingerprint: Option<&'static Fingerprint>,
}

/// One input stream's slice for position p, already aligned by the runtime.
pub struct InputSlice<'a> {
    pub gap: bool,                 // true: content is the generated default
    pub frames: u16,
    pub audio: &'a [u8],           // 4-byte samples, channel-interleaved
    pub dense: &'a [u8],
    pub sparse: &'a [SparsePart],
    pub transport: Option<&'a TransportState>,
}

pub struct OutputSlice<'a> {
    pub degraded: bool,            // set by generated code from dataflow (C-122)
    pub audio: &'a mut [u8],
    pub dense: &'a mut [u8],
    pub sparse: &'a mut SparseOut,
}

pub trait Module {
    const INFO: ModuleInfo;
    type State;
    /// Deterministic initial state; no allocation.
    fn init(state: &mut core::mem::MaybeUninit<Self::State>);
    /// Never blocks, never allocates.
    fn process(state: &mut Self::State, inputs: &[InputSlice<'_>], position: u64,
               outputs: &mut [OutputSlice<'_>]);
}

/// Refuses a module built against another ABI version, with a named reason.
pub enum AbiError { VersionMismatch { module: u16, runtime: u16 } }
pub fn check_module_info(info: &ModuleInfo, runtime_abi: u16) -> Result<(), AbiError>;

pub struct Fingerprint { pub inputs: &'static [u32], pub expected: &'static [u32] }
/// Call on the processing thread, after its floating-point mode is final.
/// Runs `process` on a fresh `init` state; compares output words bitwise.
pub fn fingerprint_matches<M: Module>() -> bool;
/// `Bounded` for an exact-float module whose fingerprint is absent or does not match;
/// otherwise the declared class.
pub fn effective_class<M: Module>() -> DeterminismClass;
```

`PortInfo`, the remaining fields of `OutputInfo`, `SparsePart`, `SparseOut` and `TransportState` come from the content layouts of epic 010. `OutputInfo` carries, besides `latency: Latency`, the "frequency-dependent delay" property for recursive filters (C-189; restored after the earlier drop of `non_constant_delay`; name may be refined). L is always a whole number of samples (C-105), and a resampler delay, fixed or variable, belongs to D (C-134, C-170, C-207), not to this property or to L.

The ABI is documented in the crate with one hand-written example module that uses no generated code.

**Crate `hadronyche-kit`** (no_std, no allocation, no libm):

- `fixed.rs` — Q1.31 (signed) and Q0.32 (unsigned) types; saturating and wrapping operations; `mul_high`.
- `float.rs` — strict f32 helpers with defined evaluation order; `sine` (add/multiply series).
- `tables.rs` — shipped tables (sine for the fixed path, resampler coefficients) with checksums.
- `ramp.rs` — linear ramp of a field across a slice, float and fixed.
- `resample.rs` — the fixed-ratio 2:1 and 1:2 DSL resampler node (odd-tap linear-phase, C-183) with a constant group delay that is a whole number of domain-rate samples, exposed as a `Latency` constant (C-105) for use as the edge conversion delay (counted only in D and never in L; reported separately in the manifest on the device-facing port, C-134, C-207); and, separately, the rate-actuator trait and the steerable rate-actuator resampler that implements it, used at edges, whose ratio varies by parts per million around nominal within ±1 000 ppm (C-210) and whose delay is part of D (C-189).

**Crate `hadronyche-tools`** (C-133): the symbol checker, the aborting test allocator and the table generators used by the kit.

**CI:** three targets (x86-64, AArch64, one 32-bit device target), recorded as the listed-target set for exact-float, with a kit cross-target determinism suite.

## Dependencies

- **Needs:** the runtime's slice delivery shape and gap flags (009); content message layouts and the ABI auxiliary types (010).
- **Provides:** the ABI and kit to part 024b (code generator), to the conformance suite (025) and to the embedded device (026); both resamplers, the fixed 2:1 / 1:2 node and the separate rate actuator (C-189) with its trait (C-210), built once here and used by every edge, Linux, macOS, Windows and embedded, which builds none of its own (C-197); hence it blocks epics 018, 021, 022a and 026.
- **Between the parts:** 024b depends on this part as a whole; this part depends on nothing in 024b.

## Acceptance criteria

1. **024a/AC-1 No allocation, no libm (kit and ABI).** `hadronyche-kit` and `hadronyche-core::abi` build `no_std` without `alloc`; symbol inspection of the built artefact shows no libm symbol and no allocator symbol. Running under an allocator that aborts on use passes. (Kit half of the former 024/AC-1; the generated-module half is 024b/AC-1.)
2. **024a/AC-2 Kit determinism.** Table checksums match on every CI target. `sine`, the fixed-point operations, the ramps and the resampler produce identical bits on every CI target (x86-64, AArch64, one 32-bit device target).
3. **024a/AC-3 Resampler node.** For 2:1 and 1:2 (the edge conversion of C-80), the probe-measured group delay is a whole number of domain-rate samples and equals the kit's `group_delay` constant (C-105). No other ratio is offered. The constant is the edge's conversion delay, a component of D and never of L (C-134, C-170, C-207).
4. **024a/AC-4 ABI stability.** A module built against `ABI_VERSION` 1 is rejected by a runtime expecting another version, with a named reason (`AbiError::VersionMismatch`).
5. **024a/AC-5 Fingerprint check.** `fingerprint_matches` compares output words bitwise on a fresh `init` state, and `effective_class` returns `Bounded` for an exact-float module whose fingerprint is absent or does not match. (ABI side of the former 024/AC-4 and 024/AC-5; the processing-thread test is 024b/AC-4.)

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The ABI is documented in `hadronyche-core` with one hand-written example module that uses no generated code
- [ ] Every shipped table has a checksum test and a note on how it was generated
- [ ] The CI target list is recorded as the listed-target set for exact-float

**Not applicable in this part (C-125):** "Every parser touched is fuzzed" — this part has no parser; the manifest and vectors decoders are in 024b. "Regime stamp fields touched are complete and documented" — this part computes the effective class (`effective_class`) but touches no stamp field; the field is set in 024b. Every other line has a subject here; the POC is read for the ported `sine`, `mul_high` and table lessons and must stay unmodified.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Resampler filter length versus cost on a device, for 2:1 and 1:2 | This part (ticket 024a-11), then task 026 | Does not fit the reference device |
| Which 32-bit device target is the third CI target and member of the listed-target set | This part (ticket 024a-06), checked with task 026 | The target cannot be emulated unprivileged in CI, or task 026's reference device differs from it in integer or float behaviour |
| Format and resolution of the variable ratio of the rate-actuator resampler (C-189; the range is settled at ±1 000 ppm around nominal, C-210, and covers the ±100 ppm, 0.5 ppm/s drift ramp of C-119; the edge rate loop is well below 1 Hz, C-68) | This part (ticket 024a-13) | Ratio quantisation alone produces slip or exceeds E in the L2 drift-ramp scenario (C-76, C-119) |
| Definition of `PortInfo`, the rest of `OutputInfo`, `SparsePart`, `SparseOut`, `TransportState`, and the fingerprint word layout for non-audio content | Epic 010 | The types cannot be expressed in a `no_std` const context |
