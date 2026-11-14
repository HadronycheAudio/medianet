# 009-23 — Degraded flag through structural latency L
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-12

## Context
A consumer that processed a gap marks its own output slice degraded (gap contract `[decision]`). A module with declared structural latency L has its output stamped p − L while it processes input p; stamps denote content position (position invariant `[decision]`).

- C-122 `[decision]`: the degraded flag on an output with latency L follows the content. While a module processes input position p it emits the output stamped p − L, whose content came from input position p − L, so it is degraded exactly when the input round at p − L was gapped, not when the round being processed (p) was.
- C-105 `[decision]`: L is always a whole number of domain-rate samples; the earlier rational-L rule is withdrawn. A resampler node exists only inside an edge module and its delay is part of the edge's D, not of L (C-134), so no fractional delay arises here. When L is not a multiple of 48, the 48 samples of an output slice come from two input rounds; the output is degraded if either was gapped.
- C-104 `[decision]`: paths of different latency are padded by the compiler with whole-sample delay lines; the module's L is the longest path.
- C-02: positions step by 48 per slice.
- A round is gapped when `Round::degraded` (009-12) is true.

This epic provides the delay line; stamping p − L and emitting the output belong to the hosted backend (epic 015).

## Work
In `hadronyche-core`, module `rx::degraded`:
```rust
pub struct DegradedDelay<const ROUNDS: usize> { /* gapped bit per past round, indexed by position */ }
impl<const ROUNDS: usize> DegradedDelay<ROUNDS> {
    pub const fn new(l_samples: u64) -> Result<Self, LatencyTooLong>; // L in whole samples (C-105); ROUNDS must cover ceil(L / 48) + 1
    pub fn record(&mut self, position: u64, gapped: bool);    // called once per round
    pub fn degraded_for_output(&self, output_position: u64) -> bool; // true if any input round overlapping [q, q + 48) was gapped
}
pub const fn rounds_for_latency(l_samples: u64) -> usize;
```
- Fixed bitmap of `ROUNDS` entries; no allocation.
- Integer multiple of 48: exactly one input round maps to each output slice.
- Whole-sample L that is not a multiple of 48: the two input rounds overlapping the output's content samples are consulted. There is no rational or fractional L (C-105).
- A position older than the bitmap span is a caller error (debug assert, documented).

## Acceptance criteria
1. L = 96 samples (2 slices): a gap at input round p marks exactly the output stamped p (emitted while processing p + 96) and no other (EC-12).
2. L = 0: the output for round p is degraded iff round p was gapped.
3. L = 72 samples (1.5 slices): a gap at round p marks the two outputs whose content overlaps round p, and only those.
4. L = 1 001 samples (whole, not a multiple of 48): same overlap rule; result matches a brute-force sample-by-sample reference over 10 000 positions with random gaps (seeded).
5. `new` with an L longer than the bitmap span returns `Err(LatencyTooLong)`; `rounds_for_latency(96) == 3`.
6. Runs under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] Regime stamp fields touched are complete and documented (degraded flag on output slices of modules with L > 0)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Stamping p − L, emitting the output slice and emitting a gap slice when a module misses its deadline (epic 015).
- Computing L for a module and inserting whole-sample padding delay lines (DSL compiler, epic 024, C-104).
- Resampler delay at an edge (part of D, C-134).
