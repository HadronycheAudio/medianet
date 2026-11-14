# 020-21 — Linux reference rig build and documentation
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-09, 020-11, 020-12, 020-28, epic 016

## Context
The L2 rig `[decision]`: one machine; the edge under test (Linux ALSA reference sink, epic 018); a reference module in the same process acting as clock holder with injection; a physical loop from the device output to a capture input — the same device read raw, or an independent capture device. The capture-side delay is calibrated once per rig (method chosen in 020-11, procedure in 020-12). For the handover the rig adds a second holder-eligible Linux node running the holder that takes over (C-212, 020-28).

Definition of done of the epic: **the rig (devices, cables, loop type, calibration value) is documented well enough to rebuild.** Epic criterion 020-A6: **capture-side delay within ±1 sample over 10 calibrations** (1 sample = 10.4 µs at 96 kHz, C-01). Epic criterion 020-A7 requires the certification record to name the reference hardware.

The machine uses the real-time setup of epic 016.

## Work
- Assemble the Linux reference rig; pick the reference sound card and capture path, run at 96 kHz, the domain rate (C-01). A card running at 48 kHz would put the edge's 2:1 resampler (C-80) in the certified path, and its conversion delay would be part of the edge's D and never counted in L (C-134, C-170, C-207); either way the device rates go into the rig file.
- Write the committed rig file `rig.toml` (format of 020-09) and a rig document (alongside `l2-results-linux.md` in `doc/design/reports/`, C-130) with: machine model, kernel and OS build, real-time setup reference (epic 016), output device, capture device, driver names and versions, cable types and lengths, loop type, connection diagram, ALSA device names, buffer sizes, calibration method and value.
- Add the second holder-eligible Linux node (C-212): machine model, kernel and OS build, real-time setup, network connection to the first machine; enter it as `holder_node` in `rig.toml` and in the rig document.
- Run `hadronyche-l2 calibrate --repeat 10` and record all 10 values.
- Propose the rig's (OS, driver, device) combination as Linux reference hardware; once the product owner approves it, record it in the constants table `doc/design/02-constants.md` (C-172) and add it to the reference list of `report.rs` (020-13).

## Acceptance criteria
1. The 10 calibration values are within ±1 sample (10.4 µs) of their mean (epic criterion 020-A6; repeatability is judged against the mean of the 10 calibrations, C-191; all 10 values are committed) `[measured]`.
2. A second engineer, using only the rig document, can list every item needed to rebuild the rig, the second holder node included (checked by review).
3. The rig's combination is recorded in the constants table with the product owner's approval (C-172), and the reference list contains exactly that combination for Linux.
4. `rig.toml` names the second holder-eligible Linux node, and `hadronyche-l2 holder` (020-28) runs on it and is reached from the first machine (C-212).

## Definition of done
- [ ] The rig (devices, cables, loop type, calibration value) is documented well enough to rebuild
- [ ] Every new factual claim carries an evidence tag
- [ ] Regime stamp fields touched are complete and documented — not applicable (C-125): this ticket adds no regime stamp field; it only adds the rig's combination to the reference list

## Out of scope
- Certification runs (020-22, 020-23).
- macOS / Windows rigs (epics 021, 022).
