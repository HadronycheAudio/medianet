# 009-06 — Reassembler: slice assembly and dedup
**Epic:** 009-Gap-and-reassembly-core · **Effort:** M · **Depends on:** 009-05, epic 004

## Context
After the source filter (009-04) admits a packet, the per-source reassembler turns packets into slices.

- C-05 `[decision]`: the default slice is one packet; multi-packet slices are an explicit, stamped regime (the packet carries the packet count and its packet index, per epic 004).
- C-04 `[decision]`: 6 channels per stream at defaults; 192 bytes per channel per slice (48 frames × 4 bytes, C-03), so 1 152 bytes of audio per slice. There is no per-channel id byte; the channel map is in the manifest. The audio payload of a default slice is therefore exactly 1 152 bytes.
- C-02 `[decision]`: slice = 48 frames = 0.5 ms at 96 kHz (C-01, the only rate on the wire).
- C-07 `[decision]`: copy index 2 bits; dedup by (stream, position, packet index); first valid copy wins. Use `Dedup` from 009-05.
- C-19 `[decision]`: every slice carries the session number assigned by its source (32 bits, C-10); it is bumped on restart and on timeline epoch change. The reassembler carries it through so the fan-in can check it (009-24).
- `[code]` POC behaviour being retired: a kernel-reassembled 9.6 KB datagram per slice. Here, multi-packet slices are reassembled in user space, in fixed memory.

Open sub-question of the epic, owned by this ticket: what to do when all `SLOTS` are busy. This ticket implements "evict the oldest incomplete slot, counted" and records it as a `[decision]` of this epic. Falsifier: `slot_overflow` rises on a clean network (L1) with `SLOTS` sized for the admitted multi-packet streams.

## Work
In `hadronyche-core`, module `rx::reassembly`:
```rust
pub struct Reassembler<const SLOTS: usize, const MAX_SLICE: usize> { /* per source */ }
impl<const SLOTS: usize, const MAX_SLICE: usize> Reassembler<SLOTS, MAX_SLICE> {
    pub const fn new() -> Self;
    /// dedup by (stream, position, index); returns the slice when complete
    pub fn push(&mut self, pkt: &MediaPacket, arrival_ns: i64) -> Option<SliceRef<'_>>;
    pub fn counters(&self) -> ReassemblyCounters; // duplicate, oversize, slot_overflow
}
pub struct SliceRef<'a> { pub stream: StreamId, pub position: u64, pub epoch: Epoch, pub session: u32,
                          pub flags: SliceFlags /* gap slice, degraded */, pub payload: &'a [u8],
                          pub first_arrival_ns: i64, pub last_arrival_ns: i64 }
```
- Single-packet slice (C-05 default): returned immediately, zero-copy (payload borrows the packet), after dedup.
- Multi-packet slice: payload of packet index i copied to its offset in a slot of `MAX_SLICE` bytes; returned when all k packets are present. Slot keyed by (stream, position).
- Duplicate (dedup says not first): discarded, `duplicate += 1`.
- Slice larger than `MAX_SLICE` or packet count/index inconsistent with epic 004 limits: discarded and counted (`oversize`).
- All `SLOTS` busy when a new position starts: the oldest incomplete slot is evicted and counted as a dropped incomplete slice (its missing packets are counted as in 009-07) and `slot_overflow += 1`.
- Flags (gap slice, degraded), the timeline epoch and the session number are carried through unchanged.

## Acceptance criteria
1. A single-packet slice is returned by the first `push`, with an audio payload of 1 152 bytes for a default 6-channel stream (C-04: 6 × 192 B).
2. A 4-packet slice pushed in order 2, 0, 3, 1 is returned on the 4th push with bytes identical to the original slice.
3. Copies 0..3 of every packet of a 4-packet slice (16 pushes) yield exactly one completed slice and `duplicate == 12` (EC-8).
4. Two interleaved multi-packet slices (positions p and p + 48) both complete correctly.
5. A packet claiming a slice larger than `MAX_SLICE` is discarded, `oversize == 1`, and no panic occurs.
6. With `SLOTS = 2`, starting a 3rd incomplete position evicts the oldest and increments `slot_overflow` to 1.
7. Gap-slice and degraded flags, the epoch and the session number set on input packets appear unchanged on the returned `SliceRef`.
8. All tests run under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed (fuzz target delivered in 009-09)
- [ ] Regime stamp fields touched are complete and documented (the multi-packet regime fields read here)
- [ ] No new open item without an owner and a falsifier (slot eviction: owner this epic, then L1)
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Deadline-driven expiry of incomplete slices and the missing-packet count (009-07).
- Conformance vectors (009-08) and fuzz targets (009-09).
- Alignment by position (009-10) and the session check (009-24).
