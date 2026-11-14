# 020-12 — Rig calibration procedure
**Epic:** 020-L2-certification-harness · **Effort:** M · **Depends on:** 020-04, 020-05, 020-09, 020-10, 020-11

## Context
`rig.rs` holds the rig description and the **calibration procedure** for the capture-side delay `[decision]`. The capture-side delay enters every emission error: `(network time at which it was captured − capture-side delay) − (T(p) + A)` `[decision]`. The method was chosen in 020-11 (one of: independent capture device with a datasheet figure, oscilloscope against a network-time pulse, or symmetric-loop halving).

**Epic criterion 020-A6: rig calibration is repeatable — capture-side delay within ±1 sample over 10 calibrations.** At 96 kHz (C-01) one sample is 10.4 µs `[decision]`.

## Work
- Implement `hadronyche-l2 calibrate --rig <file>` running the method chosen in 020-11, using marker generation and detection (020-03, 020-04) and the capture timeline (020-05) as needed.
- Write the result into the rig file's `capture_delay` (value, method, uncertainty, timestamp, number of runs).
- Add `--repeat N` (default 10) that runs the calibration N times, reopening the devices each time, and reports mean, minimum, maximum and spread.
- CI test of the calculation path on a recorded or synthetic calibration capture.

## Acceptance criteria
1. `hadronyche-l2 calibrate --repeat 10` on a Linux development machine with a physical loop produces 10 values whose maximum deviation from their mean is at most 1 sample (10.4 µs at 96 kHz) `[measured]` (calibration repeatability is judged against the mean of the 10 calibrations, C-191; all 10 values are kept) (pre-check of epic criterion 020-A6; the reference measurement on the documented rig is 020-21).
2. The rig file is updated with the mean value, the method name and the uncertainty.
3. The calculation path passes a CI test on a recorded calibration capture with a known delay, recovering it within 0.1 sample.
4. If criterion 1 fails, the ticket is not closed; the measured spread is recorded and 020-11's decision revisited.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The analysis code runs in CI on recorded capture files (no audio hardware needed)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Choice of method (020-11).
- Documentation of the physical rig (020-21).
