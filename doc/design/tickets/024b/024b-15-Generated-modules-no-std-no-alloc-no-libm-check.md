# 024b-15 — Generated modules: no_std, no-alloc, no-libm check
**Epic:** 024b-DSL-code-generator · **Effort:** S · **Depends on:** 024a-05, 024a-06, 024b-12

## Context
Epic acceptance criterion 024b/AC-1: "No allocation, no libm. Generated modules build `no_std` without `alloc`; symbol inspection of the built artefact shows no libm symbol and no allocator symbol. Running under an allocator that aborts on use passes." The kit half is 024a/AC-1, done in 024a-05 (which provides the `check-symbols` binary and an aborting `#[global_allocator]` test helper in the `hadronyche-tools` crate, C-133). Rules `[decision]`: never blocks, no allocation at process time, no libm, no transcendental functions.

## Work
- CI job: for each program of the epic's corpus (024b-04), run `hadronyche-dsl build`, compile the generated module in a `#![no_std]` crate without `alloc` for the hosted target and the 32-bit device target, and run `check-symbols` on the artefact.
- Hosted test: call `init` and `process` for 1000 slices of 48 frames under the aborting allocator.

## Acceptance criteria
1. Every corpus program's generated module builds `no_std` without `alloc` on the hosted and 32-bit device targets.
2. `check-symbols` reports zero libm and zero allocator symbols for every artefact.
3. 1000 slices of `process` under the aborting allocator complete without abort for every corpus program.
4. A fixture that injects an allocation into generated code is detected by the job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Stack bounds (024b-16); cost bound (024b-19).
