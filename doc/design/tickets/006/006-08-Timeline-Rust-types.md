# 006-08 — Timeline Rust types and inverse tests
**Epic:** 006-Timeline-epoch-specification · **Effort:** S · **Depends on:** 006-02, 006-03, 006-06
## Context
Epic 006 delivers, alongside `doc/design/spec/timeline.md`, the Rust types that implement the position map. Shape given by the epic (names may be refined):
```rust
pub const DOMAIN_RATE_HZ: u32 = 96_000; // C-01, the only rate on the wire
pub struct TimelineEpoch(pub u16);
pub struct Line { pub epoch: TimelineEpoch, pub origin_ns: i64 }
impl Line {
    pub fn position_at(&self, network_ns: i64) -> u64;
    pub fn time_of(&self, position: u64) -> i64;
}
pub struct Discontinuity { pub from: TimelineEpoch, pub to: TimelineEpoch, pub step_ns: i64 }
```
`Line` has no rate field: the wire carries one sample rate only, 96 000 Hz (C-01), and a resampler node may appear only inside an edge module (C-80, C-134).

Rules the code implements (`[decision]`, from the spec sections written in 006-02, 006-03 and 006-06):
- `position = (network_time − epoch_origin) × 96 000 Hz`, positions 64-bit, map affine per epoch and never extrapolated across epochs.
- Rounding and behaviour before the origin: as specified in 006-02 (Q-1, Q-2; proposed: `position_at` floors, `time_of` ceils, 128-bit intermediate).
- Slice 48 frames = 0.5 ms = 500 000 ns (C-02); one sample = 10 416⅔ ns.
- Epoch is 16 bits (C-10), compared with the modular rule from 006-03.
- `Discontinuity` payload and `step_ns` sign as specified in 006-06 (Q-4).

Crate placement is settled (former Q-9): all protocol types, the timeline types included, live in the `hadronyche-core` crate (C-133).

## Work
1. Add the types above in a `timeline` module of the `hadronyche-core` crate (C-133).
2. Implement `Line::position_at` and `Line::time_of` with the rounding rule of 006-02, using `i128` intermediates.
3. Implement the epoch comparison of 006-03 on `TimelineEpoch` (e.g. `is_newer_than`), plus `Eq`.
4. Unit tests:
   - exhaustive-in-range test: for every slice boundary `p = 48·k`, `k` in `0..=1_000_000` (≈ 500 s at 96 kHz), `position_at(time_of(p)) == p` and `time_of(position_at(t)) == t` where `t` is the boundary time;
   - property test (e.g. proptest) for random `origin_ns` and random `p` up to 2^40, including positions off a slice boundary: `position_at(time_of(p)) == p`;
   - every worked example in the 006-02 table (e.g. `time_of(1) == origin + 10 417`, `time_of(3) == origin + 31 250`);
   - epoch comparison across the wrap (65 535 → 0 is newer);
   - before-origin behaviour.
5. No parsing code is added; if any is, fuzz it to the budget of C-95 (60 s per change, 1 hour nightly, corpus kept).

## Acceptance criteria
1. `position_at` and `time_of` are exact inverses for every tested slice-boundary position at 96 kHz (006-AC-1); test count ≥ 1 000 001 boundary checks.
2. The rounding rule for positions off a slice boundary is tested by at least 3 examples from the 006-02 table (006-AC-1).
3. No intermediate overflow or panic for `origin_ns` in the full i64 range combined with `p < 2^40` (tested); a result that does not fit `i64`/`u64` is handled as 006-02 specifies.
4. `cargo test` passes without privileges (no root, no capabilities, no real clock).
5. No rate parameter appears in the public API other than the constant `DOMAIN_RATE_HZ`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable (C-125): this ticket adds no parser; if one is added it is fuzzed to C-95
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Public items have doc comments citing the spec section

## Out of scope
- Clock thread, sync reply handling, and source switch logic (later epics, e.g. 010).
- Slice header encoding of the epoch (epic 004).
- Sample-rate conversion at edges (C-80).
- Changing the rounding rule (decided in 006-02; any change goes back to that section).
