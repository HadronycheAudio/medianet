# 018-09 — No-libm symbol inspection in CI
**Epic:** 018-Linux-ALSA-edge · **Effort:** XS · **Depends on:** 018-02, epic 024a

## Context
- Epic criterion 018-A7: **no libm call in the resampler or the regression**, checked by symbol inspection of the `no_std` build.
- The regression is `DeviceLine` in `hadronyche-core` `edge/` (018-02); the resamplers are the steerable rate actuator and the fixed-ratio 2:1 / 1:2 node (C-189), both built by the arithmetic kit (epic 024a) in `hadronyche-kit`, not by this epic (C-197); this check covers them as linked into the edge's `no_std` build.

## Work
1. Add a CI step that builds `hadronyche-core` (`edge/`) and `hadronyche-kit` (resampler) for a `no_std` target, links a small test binary or static library that references `DeviceLine` and the kit's rate actuator and fixed-ratio node (epic 024a, C-197), and lists undefined/imported symbols (`nm -u` / `llvm-nm`).
2. Fail on any libm symbol (`sin`, `cos`, `sqrt`, `exp`, `log`, `pow`, `floor`, `ceil`, `round`, `fma` and their `f` variants, `__*` math intrinsics lowering to libm).
3. Keep the denylist in one file with a comment.

## Acceptance criteria
1. The CI step passes on the current code.
2. A deliberate `f64::sqrt` (via `libm` crate or intrinsic) inserted into `DeviceLine` on a test branch makes the step fail with the symbol name printed.
3. The step runs unprivileged in under 2 minutes.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The `edge/` module builds `no_std` with no allocation

## Out of scope
- Fixing libm usage found (owned by 018-02 for the regression; by epic 024a for the kit's resamplers, C-197).
