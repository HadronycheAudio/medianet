# 020-14 — Scenario runner, steady scenario and idle-servo control
**Epic:** 020-L2-certification-harness · **Effort:** M · **Depends on:** 020-03, 020-05, 020-06, 020-07, 020-09, 020-10, 020-13, 020-26, epic 017, epic 018

## Context
`run.rs` is the L2 scenario runner. Scenarios `[decision]`: `steady` (1 h), `restarts` (20), `loss`, `handover-50ppm`, `step`, `drift-ramp` (injected drift swept ±100 ppm). This ticket builds the runner core and `steady`; the others are 020-15 to 020-18.

Setup `[decision]`: one machine; the edge under test (the Linux ALSA sink of epic 018); a reference module **in the same process** acting as clock holder with the ground-truth injection API (epic 017); a physical loop from the device output to a capture input, read raw (020-10); only the handover adds a second holder-eligible Linux node (C-212, 020-17, 020-28). The sink plays a marker stream (020-03); the harness reads raw capture, locates markers and computes emission error (020-05). Run for one hour per condition. Record the steered resampler ratio throughout.

**Trap 1, the idle servo** `[decision]`: a lone host is its own clock holder and its servo does nothing. To exercise the servo on one machine, the sink must *follow another in-process module* whose clock has **injected drift and steps**; the injected values are the ground truth. **Epic criterion 020-A3: a control run with no injection shows a servo that is idle (ratio tracks only the device crystal); the certified run shows the servo active.**

Servo requirements whose effect is measured `[decision]`: published time continuous while locked (zero phase step); slew at most 10 ppm from the holder's rate (C-60); rate change at most 1 ppm/s (C-61); lock bound at most 25 µs (C-62); acquisition to Locked within 10 s of the first exchange (C-63); rate loop bandwidth well below 1 Hz (C-68). C-60 and C-61 bound the servo's correction term, not the drift estimate it tracks (C-119). Linux pass numbers: E ≤ 50 µs (C-70; E = 99.9th percentile over 1 h with maximum, C-73); marker offset within ±2 samples of the median of the first 60 s (C-75, C-166; 1 sample = 10.4 µs at 96 kHz, C-01); MTIE ≤ 10 µs for τ ≤ 1 s and ≤ the 50 µs target overall (C-77; the target E, never the E measured in the run, C-149).

A for ladder runs is computed by the harness with the commissioning rule and recorded (C-148, ticket 020-26). Runs whose numbers enter the constants table run in strict mode (C-147, ticket 020-27); CI runs on recorded files do not.

## Work
- In `run.rs`, a runner that: loads the rig (020-09, must be calibrated); opens the raw capture and refuses if a resampler is reported (020-10); obtains the audition offset A from the commissioning-rule computation of 020-26 (C-148) and records it with its inputs; instantiates in-process the reference clock holder with injection and the sink under test following it; waits for Locked (C-63) and the 4 s Aligning phase (C-50) before measuring; starts the marker stream; records capture, ground truth, injected values, the steered ratio, and the servo's drift estimate and correction term to a recording (020-02); at the end runs the analysis and emits a certification record fragment (020-13).
- Scenario `steady`: 1 hour (duration overridable for testing), with a constant injected drift on the holder (default value documented) so the servo is active. The drift is not limited to 10 ppm: C-60 bounds the correction term, not the drift the servo tracks (C-119).
- Scenario option `--no-injection` (control run): same as `steady` with injection disabled.
- Servo-activity metric: the variance of the loop-1 (servo) slope correction, or equivalently the part of the steered ratio not explained by the loop-2 device-crystal slope; report it for both runs. The threshold separating "idle" from "active" is an open sub-question of the epic owned by this task and settled by measurement of the control run (020-22); set a provisional threshold here and document it.
- `hadronyche-l2 run steady [--no-injection] [--duration]`.
- A CI test running a 60-second `steady` against `FileCapture` / a simulated device (no hardware).

## Acceptance criteria
1. `hadronyche-l2 run steady` completes a 1-hour run on Linux hardware, writes a recording and a record fragment with E (99.9th percentile and maximum), marker-offset verdict (C-75), MTIE verdict (C-77) and ratio series.
2. The control run (`--no-injection`) reports servo activity below the idle threshold, and its ratio matches the loop-2 device-crystal slope within the documented tolerance (epic criterion 020-A3).
3. The injected run reports servo activity above the active threshold, and the servo's tracked drift equals the injected drift within 0.1 ppm (epic criterion 020-A3).
4. The runner refuses an uncalibrated rig and a resampling capture path (non-zero exit).
5. The 60-second CI test passes unprivileged without audio hardware.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The analysis code runs in CI on recorded capture files (no audio hardware needed)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- `restarts` (020-15), `loss` and `step` (020-16), `handover-50ppm` (020-17), `drift-ramp` (020-18).
- The certification runs themselves and their record (020-22, 020-23).
- The injection API and the sink (epics 017, 018).
