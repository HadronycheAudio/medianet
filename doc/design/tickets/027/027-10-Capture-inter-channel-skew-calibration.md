# 027-10 — Capture inter-channel skew calibration
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-03, 027-07, 027-09

## Context
- The L3 rig records the Linux sink on channel 1 and the embedded device on channel 2 of one capture device. Any capture delay is common to both channels **provided the device's inter-channel skew is known and small**.
- **027-AC-9:** the capture device's own inter-channel skew is measured (same signal into both channels) and subtracted; the value is in the report.
- Open sub-question owned by this epic: "Inter-channel skew of the capture device at 96 kHz" — falsifier: larger than 2 µs and unstable ⇒ choose another capture device.
- Repeatability is judged against the mean of 10 calibrations (C-191, proposal pending first runs).
- Skew convention: channel 2 minus channel 1, in ns. The capture runs at 96 kHz, the domain rate (C-01); one sample = 10.4 µs.

## Work
1. Subcommand `hadronyche-l3 calibrate --capture <file> --markers <plan> --out <json>`: same marker signal split (passive Y-cable or buffered splitter, documented) into both channels; runs `measure_skew` and records median, p99.9 |·| and max of the inter-channel skew.
2. Run 10 calibrations across a power cycle of the capture device and over 1 hour; compute their mean and each calibration's deviation from it, to judge repeatability (C-191).
3. `analyze` and all scenarios subtract the calibrated median from every `skew_ns` before statistics, and write `capture_interchannel_skew_ns` into the report (027-07 field).
4. Decision recorded: if |median| > 2 µs **and** any of the 10 calibrations deviates from their mean by more than 2 µs (C-191), the capture device is rejected and replaced (re-do the 027-09 parts list).
5. Store one short calibration recording as a CI fixture (027-08).

## Acceptance criteria
1. Calibration result committed in `doc/design/reports/` (C-130) with each of the 10 calibrations, their mean, the largest deviation from the mean (C-191), and rate (96 kHz), each tagged `[measured]`.
2. The capture device in use has every one of the 10 calibrations within 2 µs of their mean (C-191); otherwise this ticket records the replacement and re-measures.
3. On a synthetic file with 37 µs true skew plus a 3 µs simulated inter-channel offset, `analyze --interchannel-ns 3000` reports 37 µs ± 2 µs.
4. Every scenario report shows a non-empty `capture_interchannel_skew_ns`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] The skew analysis runs in CI on recorded two-channel files (calibration fixture)
- Not applicable (C-125): "Every parser touched is fuzzed" — no new parser; "Regime stamp fields touched" — none; "Design documents updated" — the result is written up by 027-20; "The rig is photographed or diagrammed" — 027-09 (re-done only if the device is replaced); "`l3-results.md` is committed" — 027-20.

## Out of scope
- Rig assembly (027-09).
- Sink-to-sink scenarios (027-11 onward).
