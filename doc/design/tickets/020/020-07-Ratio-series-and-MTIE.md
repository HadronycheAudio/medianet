# 020-07 — Ratio series and MTIE
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-05, epic 018

## Context
Each edge has two loops: loop 1 maps the host clock to the domain line (the servo); loop 2 maps the device sample counter to the host clock. The steered ratio, the quotient of the two slopes, is that of the edge's rate actuator, the steerable resampler built by the kit (epic 024a), separate from the fixed 2:1 / 1:2 node, with a range of ±1 000 ppm around nominal (C-189, C-197, C-210). The rate loop's bandwidth is well below 1 Hz (C-68) `[decision]`. The L2 procedure records the **steered resampler ratio** throughout each run and computes wander `[decision]`.

Wander pass number (C-77) `[decision]`: **MTIE at most 10 µs for intervals up to 1 s, and at most E for any interval** (E is the platform's target E, never the E measured in the same run, C-149: Linux 50 µs, C-70).

Definition `[decision]`: MTIE for an interval τ is the largest peak-to-peak variation of the time error within any window of length τ.

Servo limits `[decision]`: slew at most 10 ppm from the holder's rate (C-60); rate change at most 1 ppm/s (C-61). These bound the servo's **correction term**, not the drift estimate it tracks (C-119), so they are checked on the correction-term series, not on the steered ratio. Separately, the epic's AC-5 (criterion 020-A2) bounds the rate of change of the steered ratio at 1 ppm/s during the 50 ppm handover.

## Work
In `analysis.rs`:
- Ratio series: ingest `(host_ns or network_ns, ratio)` samples of the sink's steered resampler ratio (as recorded by the runner, read through the interface of the epic 018 sink) and store them in the recording side channel (020-02). Ingest the servo's drift estimate and correction term the same way, as two separate series (C-119); the epic lists this sink interface as still to be described by epic 018.
- Derived series: ratio deviation in ppm from nominal, and its rate of change in ppm/s (over a documented smoothing window).
- `max_rate_of_change_ppm_per_s(series)` for any series: the steered ratio (AC-5) and the correction term (C-61).
- `max_abs_ppm(correction_term)` for the C-60 check (C-119).
- `mtie(time_error_series, taus) -> Vec<(tau, mtie_ns)>` over the emission-error series of 020-05, using an O(n log n) or O(n) sliding-window min/max algorithm. τ grid, as the epic states: logarithmic from one marker period to the run length, always including 1 s.
- `wander_verdict(mtie, e_target_ns)`: pass if MTIE(τ) ≤ 10 µs for all τ ≤ 1 s and MTIE(τ) ≤ the platform target E for all τ (C-149).

## Acceptance criteria
1. For a synthetic time-error series that is a sine of amplitude 5 µs and period 10 s, MTIE(τ ≥ 5 s) = 10 µs ± 0.1 µs and MTIE at τ = 1 s matches a brute-force reference within 0.01 µs.
2. `mtie` on a 1-hour series with one marker per 10 ms (360,000 points) completes in under 10 s on a CI machine and matches the brute-force result on a 5,000-point subsample exactly.
3. A synthetic correction-term ramp of 2 ppm/s is reported as 2 ppm/s ± 0.1 and fails the 1 ppm/s (C-61) check; a ramp of 0.5 ppm/s passes.
4. A synthetic run whose drift estimate moves by 100 ppm at 0.5 ppm/s while the correction term stays within 10 ppm and 1 ppm/s passes C-60 and C-61 (C-119); the same run with a correction term reaching 12 ppm fails C-60.
5. `wander_verdict` fails a series with MTIE(1 s) = 11 µs and passes one with MTIE(1 s) = 9 µs and overall MTIE ≤ 50 µs; with the Linux target, a series whose overall MTIE is 45 µs passes even when its own 99.9th-percentile emission error is 30 µs (C-149).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The analysis code runs in CI on recorded capture files (no audio hardware needed)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Whether the two-point MTIE mask is sufficient (020-19).
- Spectral criterion (020-20).
- Emission-error percentiles and slip (020-06).
