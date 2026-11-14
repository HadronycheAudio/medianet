# 020-04 — Marker detection with sub-sample interpolation
**Epic:** 020-L2-certification-harness · **Effort:** M · **Depends on:** 020-02, 020-03

## Context
The L2 harness reads raw capture frames with their hardware frame counts (`RawBlock { frames, first_frame_count, host_ns }`, 020-02) and locates each position marker (020-03) in them `[decision]`. The result per marker is a fractional **capture frame count**: the hardware frame counter value at which the marker's reference instant arrived, with sub-sample precision.

Precision matters: at 96 kHz (C-01) one sample is 10.4 µs `[decision]`, the Linux E target is 50 µs (C-70) and its D share only 10 µs, and the harness self-check (epic criterion 020-A5, ticket 020-08) must recover a known 30 µs error within 2 µs. Detection therefore needs an interpolation error well below 2 µs, i.e. below about 0.2 samples. The capture device rate is a rig property (a device may run at 48 kHz, C-80); timing errors are therefore judged in µs and in domain-rate samples, never in device samples.

## Work
- In `marker.rs`, implement detection over a stream of `RawBlock`s:
  - matched filter / correlation against the marker template;
  - peak picking with sub-sample interpolation (for example parabolic or sinc interpolation around the correlation peak);
  - decoding of the marker identifier to its position p;
  - output: a sequence of `(position p, capture_frame: f64, quality metric)`.
- Handle markers that straddle block boundaries and gaps in `first_frame_count` (report a capture overrun instead of silently mis-timing).
- Reject detections below a quality threshold and count them.

## Acceptance criteria
1. On a synthetic capture with markers placed at known fractional frame offsets (at least 100 offsets uniformly spread over [0, 1) sample), the absolute timing error is at most 0.1 sample (about 1 µs at 96 kHz) for every marker at a signal-to-noise ratio of 40 dB.
2. Markers straddling a block boundary are detected with the same accuracy as criterion 1.
3. A gap in `first_frame_count` between consecutive blocks is reported as an overrun event, and no marker spanning the gap is reported.
4. Every detected marker is mapped to its correct position p; on a 1-hour synthetic stream at 96 kHz the mis-identification count is 0.
5. The detector processes 1 hour of 96 kHz mono capture in less than 10 minutes on a CI machine.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The analysis code runs in CI on recorded capture files (no audio hardware needed)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Conversion of capture frames to network time and the emission-error formula (020-05).
- Statistics and slip detection (020-06).
- Marker waveform design (020-03).
