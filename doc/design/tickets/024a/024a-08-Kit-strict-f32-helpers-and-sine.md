# 024a-08 — Kit strict f32 helpers and `sine`
**Epic:** 024a-Runtime-ABI-and-arithmetic-kit · **Effort:** S · **Depends on:** 024a-05

## Context
Rules for generated code `[decision]`: no libm, no transcendental functions; strict f32 operations only; float operations are never reordered; no fused multiply-add contraction; summation order is defined by DSL semantics; no auto-vectorisation that changes results.

POC lesson `[code]`: `src/music.rs:263` `sine()` computes sin(2πx) in single precision from additions and multiplications only, folded to a quarter cycle and evaluated as a short series, "so that it gives the same bits on every machine". Port it (do not modify the POC).

Claims about IEEE 754 behaviour in the documentation are tagged `[standard]` with the clause (C-132).

## Work
In `hadronyche-kit/src/float.rs` (no_std, no alloc, no libm):
- Strict helpers with defined evaluation order, e.g. `add`, `sub`, `mul`, `mul_add_unfused(a, b, c) = (a * b) + c` (two roundings), `sum_left(&[f32]) -> f32` (strict left-to-right), `dot_left`.
- `pub fn sine(x: f32) -> f32` returning sin(2πx), ported from `src/music.rs:263`: quarter-cycle fold, short polynomial in Horner form with fixed order, only `+`, `−`, `×`.
- Mark helpers `#[inline]` and guard against vectorisation changing order (sequential loops with no reassociation; Rust does not reassociate floats, document this).
- Tests: golden bit vectors for `sine` on 4096 points in [0, 1) and specific points (0, 0.25, 0.5, 0.75, subnormal input, NaN input); accuracy bound against f64 reference recorded `[measured]`.

## Acceptance criteria
1. `sine` source uses no `f32` method other than arithmetic operators and bit conversions (no `sin`, `mul_add`, `powf`).
2. `sine(0.0) == 0.0`, `sine(0.25) == 1.0`, and output bits on the 4096-point grid equal the committed golden file on the hosted target.
3. Max absolute error versus `f64::sin(2πx)` (computed in a std-only test) is measured and recorded in the doc comment with tag `[measured]`.
4. `sum_left` of `[1e8, 1.0, -1e8]` returns `0.0` (left-to-right) and a test documents the order.
5. `check-symbols` (024a-05) on the kit reports no `fmaf`/`sinf`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Cross-target bit identity (024a-14).
- Subnormal policy for exact-float (024b-02).
