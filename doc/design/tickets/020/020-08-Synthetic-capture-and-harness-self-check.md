# 020-08 — Synthetic capture and harness self-check
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-03, 020-06, 020-07

## Context
Before the harness certifies anything it must prove it measures correctly. Epic criterion **020-A5 (harness self-check)** `[decision]`: *with a synthetic capture stream containing a known 30 µs emission error and a known 1-sample slip, the analysis reports both within 2 µs and flags the slip.*

At 96 kHz (C-01) one sample is 10.4 µs `[decision]`. The analysis must run in CI on recorded capture files with no audio hardware.

## Work
- Add a synthetic recording generator (test utility, also callable as `hadronyche-l2 synth`) that writes a recording in the 020-02 format containing:
  - the marker stream of 020-03 at 96 kHz, delayed by a configurable emission error, a configurable capture-side delay, a capture-counter drift in ppm, additive noise and a configurable slip (an extra or missing whole number of samples at a given time);
  - the side channels the analysis needs (marker schedule, ground truth, a ratio series).
- Add an end-to-end CI test: generate the criterion 020-A5 stream (30 µs constant emission error, a 1-sample slip at a known time, noise at 40 dB SNR), run the full analysis (020-04 detection, 020-05 error, 020-06 statistics and slip, 020-07 MTIE) via `hadronyche-l2 analyse <recording>` (this ticket implements that subcommand, which prints the analysis results as JSON), and check the results.
- Commit the generated recording (or its generator seed) as a CI fixture.

## Acceptance criteria
1. Before the slip, the reported emission error is 30 µs within ±2 µs (epic criterion 020-A5).
2. The slip is reported with size 1 sample = 10.4 µs within ±2 µs, and a slip event is flagged at the correct marker (epic criterion 020-A5); its Linux C-76 verdict is pass (C-166).
3. After the slip, the reported emission error is 30 + 10.4 µs (or 30 − 10.4 µs for a missing sample) within ±2 µs.
4. A control stream with the same parameters and no slip raises no slip event.
5. The test runs in CI unprivileged, without audio hardware, in under 5 minutes.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The analysis code runs in CI on recorded capture files (no audio hardware needed)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Recordings from real hardware (committed by 020-22).
- The analysis algorithms themselves (020-04 to 020-07).
