# 009-05 — Copy-index dedup
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-01, 009-02, epic 004

## Context
A sender may transmit several copies of each packet (redundancy). C-07 `[decision]`: the copy index is 2 bits (copies 0..3); dedup is by stream, position and packet index; the first valid copy wins. Duplicates are counted and discarded (epic EC-8).

- C-05 `[decision]`: the default slice is one packet (packet index 0); multi-packet slices are an explicit, stamped regime with packet indices 0..k−1.
- C-01, C-02 `[decision]`: one slice = 48 frames = 0.5 ms at the single domain rate of 96 kHz; positions are counted in domain-rate samples from the domain epoch, so consecutive slices differ by 48 in position.
- C-10 `[decision]`: the stream number is 16 bits.
- Copy index, packet index, stream and position field locations come from epic 004.

The dedup state is used by the reassembler (009-06) on every packet, before assembling.

## Work
In `hadronyche-core`, module `rx::dedup`:
```rust
pub struct Dedup<const WINDOW: usize> { /* (stream, position, packet index) seen-set, bounded window of positions */ }
impl<const WINDOW: usize> Dedup<WINDOW> {
    pub const fn new() -> Self;
    /// Returns true the first time (stream, position, packet_index) is seen in the window.
    pub fn first_seen(&mut self, stream: StreamId, position: u64, packet_index: u16) -> bool;
}
```
- The copy index is ignored by the key.
- "First valid copy wins": callers call `first_seen` only after the packet has passed validation (009-04), so an invalid first copy does not shadow a valid later copy.
- Positions older than the window are treated as already seen (the reassembler/align ring classify them separately).
- A per-packet bitmap per tracked position is acceptable (packet index bounded by the multi-packet maximum from epic 004).

## Acceptance criteria
1. For copies 0, 1, 2, 3 of the same (stream, position, packet index), `first_seen` returns `true` for the first only and `false` for the 3 others (EC-8).
2. Copy order 3, 0, 1 yields `true` only for copy 3 (first arrival wins, not lowest index).
3. Same position, different packet index → both `true`; same packet index, position + 48 → both `true`.
4. Window wrap: after advancing more than `WINDOW` positions, an old key returns `false` without false positives for new keys over 10 000 consecutive positions.
5. Tests run under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Incrementing `InputCounters::duplicate` (done by the reassembler, 009-06, and the fan-in counters, 009-14).
- Choosing how many copies a sender sends (sender side, other epics).
