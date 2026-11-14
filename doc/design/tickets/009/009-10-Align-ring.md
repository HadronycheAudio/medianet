# 009-10 — Per-input alignment ring
**Epic:** 009-Gap-and-reassembly-core · **Effort:** M · **Depends on:** 009-06, epic 006

## Context
Fan-in alignment `[decision]`: single-input modules are arrival-driven and forward immediately; deadlines exist only at fan-in points and at sinks. A fan-in holds each aligned input in a ring indexed by position and processes position p at the fixed time `T(p) + offset` with whatever has arrived. Alignment is sample-exact by position whatever the clock error. An input whose slices carry another timeline epoch is treated as a gap until it switches.

- Position = network time counted in domain-rate samples (96 kHz, the only rate on the wire, C-01) from a domain epoch; one slice = 48 frames = 0.5 ms (C-02), so consecutive slice positions differ by 48.
- Stamps denote content position and are never restamped per hop (position invariant).
- C-96 `[decision]`: the ring spans the presentation group's audition offset A (C-55); for a fan-in outside any presentation group, the A inherited from the downstream group or a configured budget (C-142). `DEPTH` is computed from it by 009-11. A slice further ahead than the ring can hold is refused as `TooEarly` and counted (`too_early`, 009-14).
- Marked gap `[decision]`: slice p present as a gap slice, or with the degraded flag.
- Late `[decision]`: slice p arrives after the deadline: discard, count separately. C-108: while p is still within the ring span, the arrival reclassifies the position from missing to late (009-14 does the counting; the ring reports `Late` and whether p is still within its span).
- Missing `[decision]`: slice p absent at the deadline.
- Other epoch `[decision]`: a slice on another timeline epoch is not stored, but the ring remembers that one arrived for p, because it proves the upstream alive: it is not a miss (C-102), it restarts the dead clock and it has its own counter (C-135), while it is still a gap for the round.
- `[code]` POC behaviour being retired: the mixer keeps only the latest block per input and mixes on arrival with no alignment (`mixer.rs:101-124`).
- The `Line` type, `T(p)` and the other-epoch rule come from epic 006.

## Work
In `hadronyche-core`, module `rx::align`:
```rust
pub enum Inserted { Stored, Late { within_span: bool }, OtherEpoch, Duplicate, TooEarly }
pub enum InputAt<'a> { Content(&'a [u8], SliceFlags), MarkedGap, OtherEpoch, Missing }
pub struct AlignRing<const DEPTH: usize, const MAX_SLICE: usize> { /* per aligned input */ }
impl<const DEPTH: usize, const MAX_SLICE: usize> AlignRing<DEPTH, MAX_SLICE> {
    pub const fn new() -> Self;
    pub fn set_epoch(&mut self, epoch: Epoch);           // the consumer's current timeline epoch
    pub fn insert(&mut self, slice: SliceRef<'_>, arrival_ns: i64) -> Inserted;
    pub fn take(&mut self, position: u64) -> InputAt<'_>;
    pub fn clear(&mut self);                             // used on Dead and re-admission
}
```
- Storage: `DEPTH` slots of `MAX_SLICE` bytes; slot = `(position / 48) % DEPTH`, tagged with its position, plus one "other epoch seen" bit per slot.
- `insert`: other epoch (per epic 006 rule) → `OtherEpoch`, not stored, bit set for p; position ≤ last taken position → `Late { within_span }`, not stored, where `within_span` is true when p is within `DEPTH` slices of the last taken position (C-108); same position already stored → `Duplicate`; position more than `DEPTH − 1` slices ahead of the next position to take → `TooEarly`, not stored; otherwise copy and `Stored`.
- `take(p)`: stored content → `Content`; stored with gap-slice or degraded flag → `MarkedGap`; nothing stored but an other-epoch slice seen → `OtherEpoch`; nothing → `Missing`. After `take(p)`, the slot is free and p counts as taken.
- No restamping: positions are compared, never modified.

## Acceptance criteria
1. Insert positions p, p + 48, p + 96; `take` of each returns `Content` with identical bytes.
2. A slice with the degraded flag and a gap slice both `take` as `MarkedGap`.
3. `take(p)` with nothing inserted returns `Missing`; a later `insert` of p returns `Late { within_span: true }` and does not alter the next `take`; an insert `DEPTH` + 1 slices after the last taken position returns `Late { within_span: false }`.
4. A second insert of the same position returns `Duplicate`; the first stored content is kept.
5. A slice whose epoch differs from `set_epoch` returns `OtherEpoch` and the position then takes as `OtherEpoch`, never `Content` (EC-5 at ring level); after `set_epoch` to the new epoch, slices of that epoch are `Stored`.
6. With `DEPTH = 20`, a slice 20 slices ahead returns `TooEarly`; 19 ahead is `Stored`.
7. Out-of-order arrival (p + 48 before p) yields correct `Content` for both.
8. `clear` empties every slot and the "other epoch" bits.
9. Tests run under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Deadlines, offset and rounds (009-12, 009-13).
- Counters (009-14) and health (009-15).
- Choosing `DEPTH` from A and the memory report (009-11).
- Unaligned (mix-on-arrival) inputs (009-18).
