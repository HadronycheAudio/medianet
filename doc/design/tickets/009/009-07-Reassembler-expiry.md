# 009-07 — Reassembler expiry of incomplete slices
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-06

## Context
Multi-packet slices (C-05 `[decision]`: an explicit, stamped regime; default is one packet per slice) can be incomplete when a packet is lost. The reassembler (`Reassembler<SLOTS, MAX_SLICE>` in `rx::reassembly`, 009-06) must drop an incomplete slice at its deadline and count how many packets were missing (epic EC-9).

- C-15 `[decision]`: reassembly deadline for an arrival-driven multi-packet input = 2 slice periods (2 × 0.5 ms = 1 ms, C-02).
- C-110 `[decision]`: reassembly deadline for an aligned multi-packet input (one that feeds a fan-in) = the consumer's own deadline for that position, `T(p) + offset`. The caller passes it in, since the reassembler does not know the offset.
- A dropped incomplete slice is not delivered; at the fan-in it becomes a `Missing` position (009-12).

## Work
Extend `Reassembler`:
```rust
pub struct Expired { pub slices_dropped: u32, pub packets_missing: u32 }
impl<..> Reassembler<..> {
    pub fn set_deadline_mode(&mut self, mode: ReassemblyDeadline); // ArrivalDriven { slice_ns } (C-15) | Aligned (C-110)
    pub fn expire(&mut self, now_ns: i64) -> Expired;              // ArrivalDriven: counts packets missing per dropped slice
    pub fn expire_before(&mut self, position: u64) -> Expired;     // Aligned: drop slots for positions < position
}
```
- ArrivalDriven: a slot whose first packet arrived at `t0` is dropped when `now_ns >= t0 + 2 × slice_ns` (C-15).
- Aligned: the fan-in calls `expire_before(p + 48)` when it runs the round for p at its deadline (C-110), so any incomplete slot for p or earlier is dropped then.
- `packets_missing` adds, for each dropped slot, `k − received` where k is the slice's packet count.
- Totals accumulate into `InputCounters::incomplete_reassemblies` (`slices_dropped`, a C-159 stamp counter) and `InputCounters::reassembly_missing_packets` (`packets_missing`, receiver report) (fields from 009-01) via the caller or a `counters()` accessor.
- Work per `expire` call is bounded by `SLOTS`.

## Acceptance criteria
1. ArrivalDriven, slice_ns = 500 000: a 4-packet slice with 3 packets, first at t = 0, is still held at `expire(999_999)` and dropped at `expire(1_000_000)` with `Expired { slices_dropped: 1, packets_missing: 1 }` (EC-9).
2. A 4-packet slice with only 1 packet received yields `packets_missing == 3`.
3. A slice completed before its deadline is never reported by `expire`.
4. Aligned mode: `expire_before(p + 48)` drops an incomplete slot at position p and reports its missing packets; slots at p + 48 and later are kept.
5. Aligned mode: a slice of position p completed before the round for p is delivered, even when its first packet arrived more than 2 slice periods earlier (C-15 does not apply, C-110).
6. A late packet for a dropped position starts no new slot that could later complete a dropped slice from fewer than k fresh packets (it opens a fresh slot or is discarded; behaviour documented and tested).
7. Tests run under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] Late, missing, marked and rejected counters are all exposed for the receiver report and the stamp
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Fan-in classification of the dropped position as `Missing` (009-12, 009-14).
- Fuzzing the reassembler (009-09).
