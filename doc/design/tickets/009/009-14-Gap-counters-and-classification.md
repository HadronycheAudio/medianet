# 009-14 — Gap counters and late/missing attribution
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-12

## Context
Gap contract `[decision]`: timing is always the consumer's deadline; markers exist for classification. At the deadline for position p on an input:

| Observation | Class | Consequence |
|---|---|---|
| Slice p present with content | — (received) | Use it |
| Slice p present as a gap slice, or with the degraded flag | Marked gap | Upstream alive, loss further up |
| Slice p on another timeline epoch | Gap, not a miss (C-102) | Upstream alive |
| Slice p absent | Missing | Unknown cause |
| Slice p arrives after the deadline, p within the ring span | Late | Reclassified from missing (C-108); discarded |
| Input Demoted | Gap, counted as demoted (C-109) | Until re-admission |

- C-108 `[decision]`: at the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span it is reclassified as late (missing −1, late +1) and discarded.
- C-109 `[decision]`: a Demoted input is treated as gapped and counted separately.
- C-47 `[decision]`, completed by C-159: the consumer publishes per-connection counters in its regime stamp: received, late, missing, marked gap, other-epoch (C-135), too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead. The observed DSCP is also in that summary but is read by the backend, not here.
- Epic EC-3 (system AC-15): injected delay beyond the offset is counted as late only; injected drop as missing only. Counters also feed the receiver report (epic 014).

`InputCounters` (009-01): `received, late, missing, marked, other_epoch, demoted, too_early, duplicate, incomplete_reassemblies, reassembly_missing_packets: u64; interrupted, dead: u32; rejected: [u32; REJECT_REASONS]`.

The stamp summary carries the eleven C-159 counters (stamp section layout by epic 004). `rejected` and `reassembly_missing_packets` are not in C-159; they are exposed through `counters()` for the receiver report and diagnostics.

## Work
In `rx::fanin` / `rx::counters`:
- `on_deadline`: `Content` → `received += 1`; `MarkedGap` → `marked += 1`; `OtherEpoch` → `other_epoch += 1`; `Demoted` → `demoted += 1`; `Missing` → `missing += 1` and remember p as provisionally missing in a fixed bitmap of `DEPTH` positions.
- `FanIn::insert` returning `Inserted::Late { within_span: true }` with p provisionally missing → `missing −= 1, late += 1`, clear the bit (C-108). `Late { within_span: false }` → `late += 1` without reclassification (the position is beyond the ring span, so C-108 does not apply and its missing count stands).
- `Inserted::Duplicate` → `duplicate += 1`. `Inserted::TooEarly` → `too_early += 1`. `Inserted::OtherEpoch` → no counter here (the position is counted at its deadline).
- `pub fn record_reject(&mut self, input: InputIndex, reason: Reject)` → `rejected[reason] += 1` (called by the data thread with the `SourceFilter::admit` result, 009-04).
- `pub fn record_reassembly(&mut self, input: InputIndex, e: Expired)` → `incomplete_reassemblies += e.slices_dropped`, `reassembly_missing_packets += e.packets_missing`.
- `pub fn counters(&self, input: InputIndex) -> &InputCounters`.
- `pub const fn stamp_summary(c: &InputCounters) -> StampSummary` with exactly the C-159 fields: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (the last two filled by 009-15).
- All counters saturate instead of wrapping.

## Acceptance criteria
1. EC-3 delay: 100 slices on one input delayed 1 ms beyond the offset (within the ring span) → `late == 100`, `missing == 0`, `received == 0`.
2. EC-3 drop: 100 slices dropped → `missing == 100`, `late == 0`.
3. Mixed: 10 delayed + 10 dropped + 80 on time → `late == 10`, `missing == 10`, `received == 80`.
4. 5 degraded-flag slices → `marked == 5`, not counted as missing or received.
5. 5 rounds with an other-epoch slice → `other_epoch == 5`, `missing == 0`.
6. Each epic 004 rejection reason recorded once → each `rejected[i] == 1`.
7. Copies 0..3 of a single-packet slice → `received == 1` at the deadline, `duplicate == 3` (EC-8 at fan-in level).
8. A slice arriving `DEPTH` + 1 slices after its deadline → `late += 1` and `missing` unchanged (beyond the ring span, no reclassification).
9. A slice too far ahead → `too_early == 1`.
10. `stamp_summary` contains exactly the eleven C-159 counters, with the values of the counters, and neither `rejected` nor `reassembly_missing_packets`.
11. Two incomplete multi-packet slices dropped with 3 missing packets in total → `incomplete_reassemblies == 2`, `reassembly_missing_packets == 3`.
12. Runs under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] Late, missing, marked and rejected counters are all exposed for the receiver report and the stamp
- [ ] Regime stamp fields touched are complete and documented (the C-159 per-connection counters)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Interrupted / Dead health and their counters (009-15); Demoted state (009-16).
- The receiver report wire format (epic 014), the stamp section layout (epic 004) and the stamp transport (C-18, control plane).
- The observed DSCP (backend).
- Qualifying the counters while a commissioning window is open: they are unqualified then (C-178), which the window stamped beside them shows (C-153); `stamp_summary` produces them unchanged.
