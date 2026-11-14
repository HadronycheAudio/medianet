# 024b-03 — Codegen: emit module scaffold and `INFO` from manifest
**Epic:** 024b-DSL-code-generator · **Effort:** M · **Depends on:** 024a-01, 024b-01, epic 023

## Context
`hadronyche-dsl/src/codegen/emit.rs` turns checked IR (from epic 023, with its analyses and manifest) into Rust source implementing `hadronyche_core::abi::Module`, with `INFO` filled from the manifest. Output is one `no_std` pure-Rust process function with its constants (per the output-form decision of 024b-01).

ABI facts (024a-01): `ABI_VERSION: u16 = 1`; `ModuleInfo { abi_version, state_bytes: u32, stack_bytes: u32, cost_ops_per_slice: u64, determinism: DeterminismClass, inputs: &'static [PortInfo], outputs: &'static [OutputInfo] (includes latency: Latency, a whole number of domain-rate samples (C-105)), fingerprint: Option<&'static Fingerprint> }`; trait `Module { const INFO; type State; fn init(&mut MaybeUninit<State>); fn process(&mut State, &[InputSlice], position: u64, &mut [OutputSlice]) }`.

Rules `[decision]`: never blocks; no allocation; state size and worst-case stack are compile-time constants emitted with the code; the module cannot set positions. `cost_ops_per_slice` is the cost analysis's count of abstract operations per slice (C-106). The DSL has no non-deterministic node in v1, so a generated module is declared ExactFixed or ExactFloat, never Bounded (C-123).

## Work
- `emit.rs`: generate a Rust module containing `#![no_std]`-compatible code: a `State` struct (fixed-size arrays only), `init` writing a deterministic initial state, a `process` function body placeholder that calls the lowered graph (024b-04), and `const INFO` filled from the manifest: `abi_version = ABI_VERSION`, `state_bytes = size_of::<State>()` checked against the manifest value by a `const` assertion, `stack_bytes`, `cost_ops_per_slice`, `determinism`, `inputs`, `outputs`, `fingerprint` (None until 024b-11).
- Deterministic output: same IR ⇒ byte-identical source.
- Snapshot tests on small IR fixtures; generated code compiled in a test crate.

## Acceptance criteria
1. For a fixture IR with 1 input and 1 output, the emitted source compiles against `hadronyche-core` in a `no_std` test crate.
2. `INFO` fields equal the manifest values for 3 fixtures (asserted in a test); a `const` assertion fails compilation if `size_of::<State>()` differs from `state_bytes`.
3. Emitting twice from the same IR yields byte-identical source.
4. The emitted code contains no `alloc`, `Vec`, `Box`, `String` or `std::` reference (grep test).
5. The emitter never writes `DeterminismClass::Bounded`; an IR that would require it is rejected with a named error (C-123).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Lowering of expressions (024b-04), gap defaults (024b-07, 024b-08, 024b-09), degraded flags (024b-10), fingerprint (024b-11), CLI (024b-12).
- The IR, analyses and manifest themselves (epic 023).
