# 024a-10 — Kit linear ramp, float and fixed
**Epic:** 024a-Runtime-ABI-and-arithmetic-kit · **Effort:** S · **Depends on:** 024a-07, 024a-08

## Context
Sparse keyed controllers in content domain v1 are "one field of one voice moving linearly to a value over the slice" `[decision]`. POC lesson `[code]`: `src/music.rs:340` / `:382` are float and fixed renderers of the same instrument, with every field moving linearly from its current value to its target over the block; this is the semantics of a sparse keyed controller. A ramp runs over one slice and ends at its target, so no ramp is in progress across a slice boundary (epic). A slice is 48 frames (C-02).

## Work
In `hadronyche-kit/src/ramp.rs` (no_std, no alloc, no libm):
- Float ramp: `pub struct RampF32 { current: f32, step: f32 }` with `start(current, target, frames: u16)` and `next() -> f32`; step and accumulation order defined (document: `value_i = current + step * i` or incremental addition; pick one and keep it fixed).
- Fixed ramp: `RampQ1_31` with the same API, using `fixed.rs`; define how the remainder of `(target − current) / frames` is distributed so the last frame equals `target` exactly.
- Tests for 48-frame slices, including target == current, full-scale swings, and frames = 1.

## Acceptance criteria
1. For the fixed ramp, the value after 48 frames equals `target` exactly for 10⁵ random (current, target) pairs.
2. For the float ramp, the sequence for a fixed input equals a committed golden bit vector on the hosted target.
3. When `target == current`, every frame equals `current` in both ramps.
4. Ramps are monotone (non-decreasing or non-increasing) for all tested pairs.
5. The accumulation rule is documented in the doc comment.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Using ramps for sparse gap defaults (024b-07).
- Cross-target bit identity (024a-14).
