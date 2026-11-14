# 023-19 — Manifest type and human-readable dump
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-07, 023-12, 023-14, 023-15, 023-16, 023-17, 023-18, 023-28, 023-29

## Context
- `hadronyche-dsl/manifest.rs` — manifest type and its serialisation (binary form in 023-21; a readable dump for humans here).
- Manifest contents `[decision]`: module name; ABI version; ports (direction, content type, rate, channels, bundle layout and channel map; for an edge module, its device-facing side and that side's rate, 023-29); slice sizes; admission figures (bit/s on the wire, packets/s, input count, buffer memory) for network ports only; state size; stack bound; cost bound, total and per output, in abstract operations (C-106); structural latency L per output as a whole number of domain-rate samples (C-105); non-constant-delay property; delays inserted at joins, joins opted out and joins flagged for a frequency-dependent path (C-104); for an edge module, the conversion delay of each device-facing port, reported separately, counted only in D and never in L (C-134, C-207, from 023-29); determinism class; gap overrides; initial value of each dense controller input (C-158, from 023-12); (later, from epic 025) generated conformance vectors for exact-class programs.
- The channel map is the only record of channel identity: the wire carries no per-channel id byte (C-04).
- Shape from the epic (names and integer widths may be refined):
  ```rust
  pub struct Manifest {
      pub name: String,
      pub abi_version: u16,
      pub ports: Vec<Port>,                     // incl. bundle layout and channel map (C-04)
      pub slice_frames: u32,                    // 48 at defaults (C-02)
      pub state_bytes: u32,
      pub stack_bytes: u32,
      pub cost_ops_per_slice: u64,              // total, abstract operations (C-106)
      pub cost_per_output: Vec<OutputCost>,
      pub latency: Vec<OutputLatency>,          // per output, domain-rate samples (C-105)
      pub joins: Vec<JoinPadding>,              // inserted delays, opt-outs, frequency-dependent flag (C-104)
      pub determinism: DeterminismClass,
      pub gap_overrides: Vec<PortId>,
      pub dense_initial: Vec<DenseInitial>,     // default 0 (C-158)
      pub admission: AdmissionFigures,
  }
  pub enum DeterminismClass { ExactFixed, ExactFloat, Bounded }
  pub struct Samples(pub u64);  // whole domain-rate samples (C-105)
  pub struct OutputLatency { pub output: PortId, pub samples: Samples, pub non_constant_delay: bool }
  ```
- The readable dump's line format is a presentation detail decided by this epic and recorded in the crate documentation (C-206); only the binary form (023-21) goes into a specification.
- The value of `abi_version` and its versioning policy are a remaining gap owned by epic 012; use a placeholder constant in one place.

## Work
1. Define `Manifest`, `Port` (with a network / device-facing distinction from 023-29), `PortId`, `Samples`, `OutputLatency`, `OutputCost`, `JoinPadding`, `DenseInitial`, `DeterminismClass`, `AdmissionFigures` (filled by 023-20); reserve a slot for conformance vectors (epic 025).
2. `Manifest::from_ir(&Graph, analyses) -> Manifest` assembling all analysis outputs.
3. `impl Display` (or `dump()`) producing a stable, line-oriented readable text; L and conversion delays are printed as whole domain-rate samples.
4. Snapshot tests of the dump for 3 corpus programs, one of them an edge module with a padded join.

## Acceptance criteria
1. Every item of the manifest contents list (except conformance vectors) is present in the struct and in the dump.
2. A 96-channel port appears in the dump as a 16-member bundle with its 96-entry channel map.
3. A padded join appears with its inserted delay; an opted-out join appears as opted out; a join with a frequency-dependent path appears flagged.
4. An edge-module program shows its device-facing port with rate and conversion delay, and that delay is not part of any output's L; a dense controller input shows its initial value.
5. The dump is byte-identical for reformatted variants of a program.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Admission figure computation (023-20); binary form (023-21); conformance vectors (epic 025).
