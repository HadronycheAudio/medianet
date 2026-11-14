# 009-11 — Alignment memory sizing and report
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-10

## Context
`AlignRing<DEPTH, MAX_SLICE>` (009-10) holds `DEPTH` slots of `MAX_SLICE` bytes per aligned input. Epic EC-10: compiles for a `no_std` target without an allocator; memory use for a 16-member bundle at a 10 ms skew budget is about 370 KB and is reported by a const function; the ring depth for a given A is computed by a const function and `check_depth` refuses an A deeper than the compiled capacity.

Constants `[decision]`:
- C-02: slice = 48 frames = 0.5 ms (at 96 kHz, the only rate on the wire, C-01).
- C-04: 6 channels per stream at defaults; 192 bytes per channel per slice (48 × 4 B, C-03); 1 152 bytes of audio per slice; no per-channel id byte.
- C-08: at most 32 members per bundle, the controller member included (C-209).
- C-27 (derived): alignment buffer per 16-member bundle about 370 KB at a 10 ms skew budget.
- C-55: the audition offset A has no default; one per presentation group; proposed by commissioning.
- C-96: the skew budget cap is the presentation group's A; alignment rings are sized to A. This settles the former open sub-question "maximum skew budget a ring must support".
- C-142: a fan-in outside any presentation group inherits the A of the downstream group it feeds (the largest if several), learned at admission; with no downstream group it needs an explicit budget in configuration, or its aligned inputs are refused, with reason NoSkewBudget (C-185).
- C-211: the commissioning tool checks declared buffer memory before applying a change of A and refuses one that does not fit; `check_depth` is that check. A change that reaches a fan-in anyway is handled in 009-29.

Arithmetic: 10 ms / 0.5 ms = 20 slices; 20 × 1 152 B = 23 040 B per member; × 16 = 368 640 B of audio, which C-27 rounds to about 370 KB. The 1 152 B is audio only (C-04), with no per-channel id byte and no per-slice header in that figure.

Because A has no default (C-55), the compiled `DEPTH` is a build parameter of the consumer; commissioning (task 028) checks each commissioned A against it.

## Work
In `rx::align`:
```rust
pub const fn ring_depth_for_skew(skew_ns: i64, slice_ns: i64) -> usize;       // ceil(skew / slice); skew = A (C-96)
pub const fn align_ring_bytes(depth: usize, max_slice: usize) -> usize;        // payload + per-slot metadata
pub const fn bundle_alignment_bytes(members: usize, skew_ns: i64, slice_ns: i64, slice_bytes: usize) -> usize;
impl<const DEPTH: usize, const MAX_SLICE: usize> AlignRing<DEPTH, MAX_SLICE> {
    pub const BYTES: usize; // = size_of::<Self>()
}
pub enum SkewBudget { GroupA(i64), InheritedA(i64), Configured(i64), None } // C-96, C-142
pub fn check_depth(compiled_depth: usize, budget: SkewBudget, slice_ns: i64) -> Result<(), DepthError>;
```
`check_depth` is what commissioning (task 028) and admission call with the commissioned A (or the inherited A or configured budget, C-142); it returns `DepthError::Exceeded { required, compiled }` when the C-96 falsifier holds (the budget needs a deeper ring than the compiled capacity), and `DepthError::NoBudget` for `SkewBudget::None`, in which case the fan-in's aligned inputs are refused (C-142) with the refusal reason NoSkewBudget (C-185; the reason itself is epic 004's and the refusal is sent by admission). Learning the inherited A at admission is the admission epic's work.

## Acceptance criteria
1. `ring_depth_for_skew(10_000_000, 500_000) == 20`; `ring_depth_for_skew(10_000_001, 500_000) == 21`.
2. `bundle_alignment_bytes(16, 10_000_000, 500_000, 1152)` is between 368 640 and 380 000 bytes (payload 16 × 20 × 1 152 = 368 640 B plus metadata; about 370 KB, C-27), checked with a `const` assertion.
3. `AlignRing::<20, 1152>::BYTES` is within 2 % of `align_ring_bytes(20, 1152)`.
4. `check_depth(20, SkewBudget::GroupA(10_500_000), 500_000)` returns `Exceeded { required: 21, compiled: 20 }`; `check_depth(20, SkewBudget::GroupA(10_000_000), 500_000)` returns `Ok`; `InheritedA` and `Configured` give the same results for the same values.
5. `check_depth(20, SkewBudget::None, 500_000)` returns `NoBudget` (C-142), documented as mapping to the refusal reason NoSkewBudget (C-185).
6. The values are available in a `const` context on the `no_std` target (built by the 009-01 CI job).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Choosing A for a presentation group (commissioning, task 028; C-55), refusing a change of A that does not fit (the commissioning tool, C-211), learning the inherited A at admission (C-142), and sending the NoSkewBudget refusal (admission, C-185).
- Ring behaviour (009-10).
