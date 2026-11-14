# 027-25 — Per-sink frequency continuity at takeover
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-03, 027-06

## Context
- L3 rig: Linux sink on capture channel 1, embedded device on channel 2, one capture clock (027-03). The skew series is the difference of the two channels, so it cannot show a rate change or phase step common to both sinks.
- C-212 applies to L2 and L3 explicitly: the handover run asserts frequency continuity at takeover — each sink's rate change within C-61 (at most 1 ppm per second) and no phase step. A holder-eligible module with usable time takes over with frequency continuity and the epoch kept (C-213). At L3 that per-sink frequency continuity and the skew within the sum of the two E values through holdover and takeover are the whole handover assertion; per-sink emission error against the timeline is an L2 assertion only, which the L3 rig cannot observe (C-220).
- Each channel's marker arrivals, located with sub-sample interpolation by 027-03, give that sink's emission timeline against the capture clock: a linear fit of capture time against marker position gives the sink's rate relative to the capture clock and its phase. The capture clock is independent of both sinks; its own rate is common to both channels and cancels in a rate change, provided it is stable over the windows used.
- "No phase step" per sink uses that sink's own no-step tolerance at handover: ±1 sample (10.4 µs) for the Linux sink (C-166), the device's declared restart spread for the device (C-166, C-191). Comparison windows are 10 s (C-166) and start when the event ends (C-191); here the event is the takeover.
- Consumer: the handover scenario (027-14, 027-AC-3). The holdover interval (about 2 s, C-37 plus C-33) is short, so the fit across it uses the samples available and states their count.

## Work
1. Extend `measure_skew_detailed` (027-03) to return, for each paired marker, the sub-sample arrival time in each channel; `measure_skew` keeps its exact signature.
2. In `report.rs`: `pub fn channel_rate_ppm(arrivals, rate, window) -> Vec<RateSample>` — sliding linear fit of capture time against marker position (converted to seconds at the domain rate) per channel, giving the sink's rate relative to the capture clock in ppm.
3. `pub fn rate_change_at(rates, takeover, window) -> RateChange` — the largest |change of rate| per second across the takeover, per sink, with pass if within C-61 (1 ppm per second).
4. `pub fn phase_step_at(arrivals, takeover, holdover_start, window, tolerance) -> PhaseStep` — per sink, the phase at the takeover instant from a fit over the holdover interval compared with that from a fit over the 10 s window starting at the takeover (C-166, C-191); pass if within ±`tolerance`, the sink's own no-step tolerance (C-166, C-191).
5. Record in the result the windows, the number of markers in each fit, both sinks' tolerances and the measured resolution of the rate estimate on the rig's marker plan.
6. Unit tests on synthetic two-channel series (027-04 generator or in-test) with a known rate ramp and a known phase step in one channel only and in both channels together.

## Acceptance criteria
1. **027-AC-3 (C-212).** On a synthetic series whose channel-2 rate ramps at a test value above C-61 across the takeover, `rate_change_at` fails for channel 2 and passes for channel 1; a ramp at a test value below C-61 passes.
2. **027-AC-3 (C-212).** A phase step of 20 µs (test value) in both channels together at the takeover is detected in each channel by `phase_step_at` and fails against a one-sample tolerance, although the skew series shows no step; a 5 µs step passes.
3. A common rate offset in both channels (capture clock offset, test value) leaves `rate_change_at` passing.
4. The measured resolution of the rate estimate is stated `[measured]` in the result; if it is coarser than C-61 the check reports "unresolved" instead of pass.
5. Tests run in CI without network or privileges.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag (the rate resolution is `[measured]`)
- [ ] No new open item without an owner and a falsifier (whether the rate resolution suffices for C-61 is owned by this ticket; falsifier: resolution coarser than C-61, or apparent rate changes common to both channels in a steady or calibration capture, meaning the capture clock is not stable enough)
- Not applicable (C-125): "Every parser touched is fuzzed" — reads in-memory arrivals, no parser; "Regime stamp fields touched" — none; "Design documents updated" — implements C-212 and C-61 as written; "The skew analysis runs in CI on recorded two-channel files" — 027-08 (fixture from 027-14); "The rig is photographed or diagrammed" — no rig work; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Running the handover scenario (027-14).
- A single sink's emission error against the timeline during holdover: an L2 assertion only, not made at L3 (C-220).
- Changing C-61 or the C-166 and C-191 parameters.
