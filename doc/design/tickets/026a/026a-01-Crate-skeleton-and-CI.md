# 026a-01 — Crate skeleton and CI
**Epic:** 026a-Embedded-HAL-and-host-integration · **Effort:** S · **Depends on:** none

## Context

Epic 026 introduces the crate `hadronyche-embedded`: `no_std`, **no alloc**. It drives the same `hadronyche-core` state machines as hosted machines and implements only transport, time and audio hardware `[decision]`. It must build both for at least one device target and for the host (host builds run the tests).

Epic criterion 026-AC-11: **no allocator and no libm symbol in the firmware image**. This ticket provides the check as a reusable CI step; 026b-02 applies it to the firmware image.

Fuzzing in this epic follows the fuzz budget C-95: each parser 60 s on every change and 1 hour nightly, corpus kept. This ticket provides the CI jobs that later tickets register their fuzz targets with.

## Work

1. Create crate `hadronyche-embedded` with `#![no_std]`, no `alloc` dependency, depending on `hadronyche-core`.
2. Add modules as empty stubs: `lib.rs`, `profile.rs`, `stamp_publish.rs` (filled by later tickets).
3. CI jobs:
   - host build and `cargo test` (unprivileged);
   - cross build for one device target (the target triple of the MCU chosen in 026b-01; if 026b-01 is not done, use a generic Cortex-M `thumbv7em-none-eabihf` build and switch later);
   - a fuzz job per C-95: 60 s per registered target on every change, 1 hour nightly, corpus kept between runs.
4. A symbol-check script (e.g. `ci/check-no-alloc-no-libm.sh`) that inspects a linked ELF with `nm`/`llvm-nm` and fails if it finds allocator symbols (`__rust_alloc`, `__rust_dealloc`, `__rust_realloc`, `__rust_alloc_zeroed`, `__rdl_*`, `__rg_*`) or libm symbols (e.g. `sin`, `cos`, `exp`, `log`, `pow`, `sqrt`, `fmod`, `floor`, `ceil` and their `f` variants).
5. A minimal linked test binary for the device target to run the check against in CI.

## Acceptance criteria

1. `cargo build -p hadronyche-embedded --target <device target>` succeeds in CI.
2. `cargo test -p hadronyche-embedded` on the host succeeds in CI without privileges.
3. The crate has `#![no_std]` and no `extern crate alloc`; adding `extern crate alloc` with a use of `Vec` makes the device build fail (verified once manually, noted in the PR).
4. The symbol-check script fails on a deliberately linked binary that calls `libm::sinf` or uses an allocator, and passes on the skeleton binary.
5. The fuzz job runs a placeholder target for 60 s per change and 1 hour nightly, and keeps its corpus (C-95).

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] The crate builds for at least one device target and for the host (for tests) in CI
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Symbol-check script committed and wired into CI

## Out of scope

- Trait definitions (026a-02).
- Applying the symbol check to the real firmware image (026b-02).
