# 027-06 — Drift, step and recovery checks
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-05

## Context
Several L3 acceptance criteria are about the *shape* of the skew series (channel 2 minus channel 1, device minus Linux), not only its percentiles. This ticket adds pure functions in `report.rs` of `hadronyche-l3` that later scenario tickets call:
- **027-AC-2, No drift:** over the hour the skew shows no trend: a linear fit has a slope below 1 µs per hour (two crystals, one timeline).
- **027-AC-3, Handover:** across a holder change the skew stays within the limit and shows no step, judged with the same pair tolerance (C-203).
- **027-AC-4, Loss and interruption:** after injected loss on one path, and after an interruption shorter than 2 s, the skew returns to its prior value within the pair tolerance: the larger of the two sinks' own tolerances, ±1 sample for the Linux sink (C-166) and the device's declared restart spread (C-191), both reported beside the verdict (C-203).
- "One sample" is one domain sample: 96 kHz is the only rate on the wire (C-01), so the tolerance is 1e9 / 96 000 ≈ 10 417 ns, independent of the capture rate.
- Context numbers: slew at most 10 ppm (C-60), rate change at most 1 ppm/s (C-61), lock bound at most 25 µs (C-62, pending simulation). Resume from interrupted is immediate at the frozen offset, so a short interruption must not shift a sink's output in time.
- Parameters (C-166, proposals pending first runs): no-step tolerance at handover ±1 sample on Linux (the platform restart spread elsewhere, C-74); comparison windows of 10 s; sample tolerances are measured against the median of the first 60 s of the run; a one-sample slip passes on Linux and is reported as a slip event. Each 10 s comparison window starts when the injected event ends (C-191, proposal pending first runs). The embedded device declares its own restart spread (C-191), which is its own no-step and recovery tolerance (C-166: the platform restart spread elsewhere than Linux). For the Linux–device pair the tolerance is the larger of the two sinks' own tolerances, and both sinks' figures are reported beside each verdict (C-203); this replaces the earlier provisional reading "±1 sample applies to the pair". They are defaults of parameters, so a revision of C-166 or C-191 changes only defaults.

## Work
1. `pub fn drift_slope_ns_per_hour(series: &[SkewSample], rate: u32) -> f64` — ordinary least-squares fit of `skew_ns` against time (position converted to seconds at the domain rate), returned in ns per hour. Pass if `|slope| < 1000` ns/h.
2. `pub fn pair_tolerance_ns(linux_tol_ns: i64, device_tol_ns: i64) -> i64` — the larger of the two sinks' own tolerances (C-203): the Linux sink's ±1 domain sample (10 417 ns, C-166) and the device's declared restart spread (C-191), read from its stamp or declaration (epic 026) by the caller.
3. `pub fn baseline_median(series, seconds) -> i64` — median skew over the first `seconds` of the run, default 60 s (C-166). This is the reference for every sample tolerance.
4. `pub fn step_at(series, event_end, window, baseline, tolerance) -> StepResult` — median skew in the 10 s window that starts when the event ends (C-191), compared with the baseline; pass if within ±`tolerance`, the pair tolerance from `pair_tolerance_ns` (C-203). The difference between the 10 s windows before and after the event is also returned, as a diagnostic.
5. `pub fn recovered(series, event_end, window, baseline, tolerance) -> Recovery` — median skew in the 10 s window that starts at `event_end` (samples during the event excluded), compared with the baseline; pass if within ±`tolerance`, the pair tolerance from `pair_tolerance_ns` (C-203). A shift that rounds to one whole domain sample passes and is flagged as a slip event (C-166).
6. Every report that uses these functions records the pair tolerance, windows (with their start at the event end) and baseline used, citing C-166, C-191 and C-203, and beside each verdict both sinks' own tolerances: the Linux sink's ±1 sample and the device's declared restart spread (C-203).
7. Unit tests use the synthetic generator options of 027-04 (drift, step) or in-test series.

## Acceptance criteria
1. A series with an injected drift of 0.5 µs/h reports `|slope|` in [0.4, 0.6] µs/h and passes; 2 µs/h fails.
2. Over 1 hour with ±5 µs random noise and no drift, the fitted slope is below 1 µs/h.
3. A 20 µs step at a known position is detected (step ≈ 20 µs ± 1 µs) and fails when both sinks' own tolerances are one sample (pair tolerance 10.4 µs, C-166); a 5 µs step passes.
4. With both sinks' own tolerances at one sample, `recovered` passes with no slip event for a series that returns to within 3 µs of its baseline, passes with a slip event at 8 µs, and fails at 15 µs.
5. Samples inside the event window are excluded from the recovery comparison (test with a large excursion during the event).
6. The baseline is the median of the first 60 s: a series whose first 60 s sit 5 µs above the rest of the pre-event data is judged against the first-60 s median.
7. `step_at` and `recovered` take their 10 s window from the event end: a series with a large excursion between event start and event end, and none after, passes (C-191).
8. `pair_tolerance_ns` returns the larger of its two arguments whichever order they come in; with a device tolerance of 30 µs (test value) the 20 µs step of criterion 3 passes and the 15 µs shift of criterion 4 passes, and the recorded result lists both sinks' tolerances beside the verdict (C-203).
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier (the tolerances and windows are the C-166 and C-191 proposals; whether they hold is reported by 027-20; the pair's tolerance is settled by C-203)
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — none; "Design documents updated" — no design behaviour changed; "The skew analysis runs in CI on recorded two-channel files" — 027-08; "The rig is photographed or diagrammed" — no rig work; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Running the hardware scenarios (027-11 to 027-19).
- Percentiles and the C-79 limit (027-05).
- Revising the C-166 and C-191 parameters (design owner, from the 027-20 results).
