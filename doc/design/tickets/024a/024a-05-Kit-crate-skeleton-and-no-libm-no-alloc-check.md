# 024a-05 — Kit crate skeleton and no-libm / no-alloc check
**Epic:** 024a-Runtime-ABI-and-arithmetic-kit · **Effort:** S · **Depends on:** none

## Context
`hadronyche-kit` is the only arithmetic library generated code may use. Rules `[decision]`: `no_std`, no allocation, no libm, no transcendental functions; only strict f32 operations, Q1.31 and Q0.32 fixed point, shipped deterministic tables and the 2:1 / 1:2 resampler node (C-105). Float operations are never reordered; no fused multiply-add contraction.

Epic acceptance criterion 024a/AC-1: "`hadronyche-kit` and `hadronyche-core::abi` build `no_std` without `alloc`; symbol inspection of the built artefact shows no libm symbol and no allocator symbol. Running under an allocator that aborts on use passes." This ticket delivers that check and the reusable checking tools; generated modules are checked in 024b-15 (024b/AC-1).

## Work
- Create crate `hadronyche-kit` with `#![no_std]`, no `alloc`, and empty modules `fixed.rs`, `float.rs`, `tables.rs`, `ramp.rs`, `resample.rs`.
- A binary `check-symbols` in the `hadronyche-tools` crate, where tool binaries live (C-133), that takes a built artefact (static library or ELF) and fails if it references any libm symbol (`sin`, `sinf`, `cos`, `cosf`, `exp`, `expf`, `log`, `logf`, `pow`, `powf`, `fma`, `fmaf` and the rest of the C math library list) or any allocator symbol (`__rust_alloc`, `__rust_dealloc`, `__rust_realloc`, `__rust_alloc_zeroed`, `malloc`, `free`, `calloc`, `realloc`). Prints the offending symbols.
- A test helper library in the `hadronyche-tools` crate (dev-dependency, std; C-133) providing a `#[global_allocator]` that aborts on any allocation, for running kit and module tests.
- Lint: forbid `f32::mul_add`, `f64::mul_add` and `core::intrinsics` float intrinsics via a `clippy.toml` `disallowed-methods` list, so fused multiply-add cannot enter the kit.

## Acceptance criteria
1. `hadronyche-kit` builds for the hosted target with `#![no_std]` and no `extern crate alloc`.
2. `check-symbols` run on the built kit reports zero libm and zero allocator symbols; a deliberately broken fixture that calls `f32::sin` via std fails the check with the symbol name printed.
3. A test using the aborting allocator passes when nothing allocates and aborts (detected by the test harness as failure) when a `Vec` is allocated.
4. Calling `f32::mul_add` in the kit fails `cargo clippy`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The CI target matrix (024a-06).
- The contents of each kit module (024a-07 to 024a-13).
- Checking generated modules (024b-15).
