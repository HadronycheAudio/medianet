# 020-09 — Rig description
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-01

## Context
An L2 measurement rig is `[decision]`: one machine, the edge under test, a reference module in the same process acting as clock holder with injection available, and a physical loop from the device output to a capture input — either the **same device, read raw**, or an **independent capture device**. The capture-side delay is calibrated once per rig. For the handover scenario the rig also needs a second holder-eligible Linux node, which runs the holder that takes over (C-212, 020-28).

`rig.rs` holds the rig description: output device, capture device, loop type, calibrated capture delay `[decision]`. The record of a certification (020-13) names the rig, and the rig must be documented well enough to rebuild. Device rates are part of the rig: the wire carries only the 96 kHz domain rate (C-01), and a device running at 48 kHz converts 2:1 inside the edge (C-80). The resampler node exists only inside the edge module, and its conversion delay is part of the edge's D (C-134, C-170).

## Work
- In `rig.rs`, define `Rig { output_device, output_rate, capture_device, capture_rate, loop_type: LoopType { SameDeviceRaw, IndependentCapture }, cables: String, capture_delay: Option<CaptureDelay { value_ns, method, uncertainty_ns, calibrated_at, calibration_count }>, holder_node: Option<HolderNode { host, address, machine_model }> }` plus free-text fields for hardware model, driver and cabling notes.
- Serialize to and parse from a human-editable text file (e.g. TOML) `rig.toml`; `hadronyche-l2` takes `--rig <file>`.
- Validation: refuse to run a measurement scenario when `capture_delay` is missing (calibration scenarios excepted), and warn when `loop_type = SameDeviceRaw`, recording in the rig that both directions share one crystal (the shared-crystal trap requires raw capture `[decision]`).
- Validation: refuse `handover-50ppm` when `holder_node` is missing, with the message "rig has no second holder node" (C-212); the other scenarios do not need it.
- Fuzz the rig file parser (cargo-fuzz target `rig_parse`) with the budget of C-95: 60 s on every change and 1 hour nightly, with the corpus kept (C-95); the CI jobs are 020-25.

## Acceptance criteria
1. A rig file round-trips: parse → serialize → parse gives an equal `Rig`.
2. `hadronyche-l2 run steady --rig r.toml` with no `capture_delay` exits non-zero with the message "rig not calibrated".
3. The fuzz target `rig_parse` runs 60 s per change and 1 hour nightly with zero panics (C-95, jobs from 020-25); a seed corpus of at least 3 rig files is committed and the corpus is kept.
4. Both loop types are covered by a test.
5. `hadronyche-l2 run handover-50ppm --rig r.toml` with no `holder_node` exits non-zero with the message "rig has no second holder node" (C-212).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The calibration method and procedure (020-11, 020-12).
- The physical rig and its documentation (020-21).
