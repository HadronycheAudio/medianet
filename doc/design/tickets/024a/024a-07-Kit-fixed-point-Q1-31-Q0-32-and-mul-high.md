# 024a-07 — Kit fixed point: Q1.31, Q0.32 and `mul_high`
**Epic:** 024a-Runtime-ABI-and-arithmetic-kit · **Effort:** S · **Depends on:** 024a-05

## Context
Fixed point is the basis of the exact-fixed determinism class (bit-exact on every target). Samples are 4 bytes, f32 or Q1.31 (C-03). POC lesson `[code]`: `src/music.rs:281` `mul_high` computes a 32 × 32-bit unsigned product and keeps the high word; it is the basis of the POC fixed-point path. Port it (do not modify the POC).

Settled by the epic: Q0.32 is unsigned with range [0, 1) (no sign bit); the rounding rule of the Q1.31 multiply is any fixed rule, chosen and documented here.

## Work
In `hadronyche-kit/src/fixed.rs` (no_std, no alloc, no libm):
- `Q1_31` (signed, `i32` storage, range [−1, 1 − 2⁻³¹]) and `Q0_32` (unsigned, `u32` storage, range [0, 1 − 2⁻³²]) newtypes with `from_bits` / `to_bits`.
- Saturating and wrapping add, sub, mul for both types; Q1.31 × Q1.31 multiply with one fixed rounding rule (for example truncate or round-half-up), chosen and documented here.
- `pub fn mul_high(a: u32, b: u32) -> u32` = `((a as u64 * b as u64) >> 32) as u32`, ported from the POC with a reference comment `src/music.rs:281`.
- Conversions Q1.31 ↔ Q0.32 and Q1.31 ↔ sample bytes (little-endian, 4 bytes).
- Exhaustive edge tests and property tests against a `u64`/`i64`/`i128` reference.

## Acceptance criteria
1. `mul_high` equals `(a as u64 * b as u64) >> 32` for 10⁶ random pairs and for all pairs from {0, 1, 2³¹, 2³²−1}.
2. Saturating ops clamp: `Q1_31::MAX.saturating_add(Q1_31::from_bits(1)) == Q1_31::MAX`, `Q1_31::MIN.saturating_sub(..1) == MIN`, and `MIN × MIN` saturates to `MAX`.
3. Wrapping ops equal two's-complement `wrapping_*` on the storage type for 10⁶ random inputs.
4. The rounding rule of multiply is documented and tested on at least 4 hand-computed cases.
5. No floating-point instruction is used in `fixed.rs` (checked by a code review note and by the absence of `f32`/`f64` in the file).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Cross-target bit-identity run (024a-14).
- Fixed-point sine table (024a-09) and fixed ramp (024a-10).
