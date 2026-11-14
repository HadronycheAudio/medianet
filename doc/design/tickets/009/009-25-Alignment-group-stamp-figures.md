# 009-25 — Alignment-group stamp figures: lateness percentiles and reclaimable latency
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-13, 009-02

## Context
During the live state the consumer keeps measuring origin-to-arrival lateness per aligned input (arrival time minus `T(p)`, 009-13). The offset uses only the maximum (C-51: percentiles are reported, not used). Two rulings make the consumer publish more of this measurement in its regime stamp `[decision]`:

- C-160: lateness percentiles in the stamp: 50th, 99th, 99.9th and the maximum.
- C-157: reclaimable latency = offset − (current maximum lateness + headroom), when positive; published in the stamp's alignment group. Headroom is the C-52 headroom of that maximum (largest of 25 % of it, one slice period, twice the published clock bound). `FanIn::reclaimable_ns` (009-13) computes it.
- The stamp section layout comes from epic 004; sending the stamp is the hosted backend's (epic 015, C-144). This ticket produces the values.

- C-182: the percentiles (C-160) and reclaimable latency (C-157) use a rolling 60 s window; the maximum since the connection went live is reported beside them. Publishing the stamp does not reset the window, so two consumers with the same lateness history publish the same figures whenever their stamps are sent.
- C-148: the A of the ladder runs and of the commissioning proposal is computed from the maximum lateness since the connection went live, never from the rolling 60 s figure; `max_since_live_ns` is therefore the figure commissioning reads, and the rolling maximum must not be substituted for it.

## Work
In `rx::fanin` (or `rx::lateness`):
```rust
pub struct LatencyHistogram<const BUCKETS: usize> { /* fixed buckets, no allocation */ }
impl<const BUCKETS: usize> LatencyHistogram<BUCKETS> {
    pub const fn new() -> Self;
    pub fn record(&mut self, lateness_ns: i64);
    pub fn percentile_ns(&self, per_mille: u16) -> i64; // 500, 990, 999
    pub fn max_ns(&self) -> i64;                         // exact, not bucketed
    pub fn reset(&mut self);
    pub fn add(&mut self, other: &Self); // merge, for the rolling window
}
pub struct AlignmentSummary { pub p50_ns: i64, pub p99_ns: i64, pub p999_ns: i64, pub max_ns: i64,
                              pub reclaimable_ns: i64, pub max_since_live_ns: i64 }
impl<..> FanIn<..> {
    pub fn alignment_summary(&self, clock_bound_ns: i64, slice_ns: i64) -> AlignmentSummary; // read-only
}
```
- Per consumer, a rolling 60 s window (C-182) kept as 60 one-second histograms in a fixed ring, fed by `observe_lateness` from every aligned input while live (unaligned inputs excluded, 009-18) and slid by `advance_window` (009-13); the oldest second is reset as it leaves the window. Percentiles are read from the merged 60 histograms.
- Bucket layout is a code choice (for example log-linear); its resolution is documented and is a test parameter, not a design constant. A percentile is reported as the upper edge of its bucket, so it never under-reports.
- `alignment_summary` fills the four C-160 figures over the rolling window (`max_ns` = `rolling_max_ns`, 009-13), `reclaimable_ns` (C-157, from 009-13 over the same window) and `max_since_live_ns` (009-13); it resets nothing.

## Acceptance criteria
1. 10 000 samples uniform on 0..10 ms: p50, p99 and p99.9 are each within one bucket width above the exact value; `max_ns` is exact.
2. A single sample of 3 ms: all four figures report 3 ms within one bucket (max exact).
3. Rolling window (C-182): samples older than 60 s no longer count; calling `alignment_summary` twice with no new sample and no window advance gives identical results (publication resets nothing); `max_since_live_ns` keeps a 5 ms sample after it has left the 60 s window.
4. `reclaimable_ns` in the summary equals `reclaimable_ns()` of 009-13 computed over the same window, and is never negative (C-157).
5. Samples from an unaligned input do not change the summary.
6. Runs under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] Regime stamp fields touched are complete and documented (alignment group: percentiles C-160, reclaimable latency C-157)
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The stamp section layout (epic 004) and sending the stamp (epic 015, C-144).
- Using percentiles for the offset (C-51 forbids it).
- Qualifying the figures while a commissioning window is open: alignment figures in stamps are unqualified then (C-178), which the window stamped beside them shows (C-153); `alignment_summary` produces them unchanged and resets nothing when a window opens or closes.
