# 027-03 — Marker location and skew series
**Epic:** 027-L3-cross-sink-rig · **Effort:** M · **Depends on:** 027-01, epic 017, epic 020

## Context
- L3 proves cross-sink simultaneity: every sink in a presentation group emits sample p at `emit(p) = T(p) + A` (T(p) = network time of position p, A = group audition offset). The Linux sink output is on capture channel 1, the embedded device output on channel 2, both recorded by one capture device on one clock.
- The reference source module (epic 017) emits a marker stream; epic 020 owns the L2 marker analysis code and the `MarkerPlan` type describing the markers. L2 already showed each sink's marker offset stable within ±2 samples (C-75).
- `skew.rs` must locate **the same marker** in both channels with **sub-sample interpolation** and produce the skew series:
  ```rust
  pub struct SkewSample { pub position: u64, pub skew_ns: i64 }   // channel 2 minus channel 1
  pub fn measure_skew(capture: &[[f32; 2]], rate: u32, markers: &MarkerPlan) -> Vec<SkewSample>;
  ```
- One sample at 96 kHz is 10.4 µs, so sub-sample precision is mandatory: the harness self-check (027-AC-8, ticket 027-04) requires a known 37 µs skew to be reported within 2 µs.
- The wire carries only the domain rate, 96 kHz (C-01); the device's 48 kHz hardware converts 2:1 at the edge (C-80). The capture is at one rate; `rate` is the capture rate (96 kHz on the rig).
- How a marker identifies its timeline epoch is still open (owner: epics 017 and 020). Pairing must therefore key on whatever identity `MarkerPlan` provides, so that an epoch field can be added without changing this ticket's pairing logic.

## Work
1. Reuse the marker detection of epic 020 per channel where possible; do not duplicate it.
2. For each marker in `MarkerPlan`, find its arrival in channel 1 and in channel 2 at integer-sample resolution, then refine to sub-sample (for example parabolic interpolation of a cross-correlation peak, or zero-crossing interpolation of the marker edge). Document the method chosen and its measured precision.
3. Pair arrivals by marker identity (the identity key `MarkerPlan` provides: timeline `position`, plus epoch when epics 017/020 add it), not by order, so a marker missing in one channel (during loss, interruption or epoch tests) does not shift the pairing. Unpaired markers are skipped and counted.
4. `skew_ns = round((t_ch2 − t_ch1) × 1e9)` with `t` in seconds of capture time.
5. Expose the count of unpaired markers per channel (for example a second function `measure_skew_detailed` returning the series plus diagnostics); keep `measure_skew` with the exact signature above.
6. Unit tests on synthetic signals (generated in-test) with integer and fractional sample offsets.

## Acceptance criteria
1. For synthetic markers offset by k samples (k = −5…5) at 96 kHz, every reported skew equals k × 10 416.7 ns ± 500 ns.
2. For a fractional offset of 3.55 samples at 96 kHz (≈ 36 979 ns), reported skew is within 1 µs.
3. With a marker deleted from channel 2 only, the series has exactly one fewer sample and all other `position` values match channel 1's.
4. Sign check: delaying channel 2 yields positive `skew_ns`.
5. With white noise at −40 dBFS added, criterion 2 still holds.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag (the interpolation precision is stated as `[measured]` from the tests)
- Not applicable (C-125): "Every parser touched is fuzzed" — reads in-memory frames, no parser; "Regime stamp fields touched" — none; "Design documents updated" — no design behaviour changed; "No new open item" — opens none (the epoch identity question is already owned by epics 017/020); "The skew analysis runs in CI on recorded two-channel files" — 027-08; "The rig is photographed or diagrammed" — no rig work; "`l3-results.md` is committed" — 027-20.

## Out of scope
- The 37 µs harness self-check on a file (027-04).
- Statistics and pass/fail (027-05, 027-06).
- Subtracting the capture device's inter-channel skew (027-10).
- Defining the marker's epoch identity (epics 017, 020).
