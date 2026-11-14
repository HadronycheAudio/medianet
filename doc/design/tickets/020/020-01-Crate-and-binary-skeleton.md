# 020-01 — Crate and binary skeleton
**Epic:** 020-L2-certification-harness · **Effort:** XS · **Depends on:** none

## Context
Epic 020 builds the L2 certification harness: a tool that certifies a *physical edge* (a sink or capture module on a real audio device) by playing position markers through the device, capturing them back raw, and measuring the emission error E and the plumbing delay D. Linux is certified first; the same harness is reused by the macOS (epic 021) and Windows (epic 022) tasks with a per-platform audio backend.

The epic fixes the packaging: crate `hadronyche-tools`, binary `hadronyche-l2`, with the modules `marker.rs`, `rig.rs`, `run.rs`, `analysis.rs`, `report.rs` `[decision]`. This ticket creates that frame so the other tickets can land independently. Harness and tool binaries live in the `hadronyche-tools` crate of the design set's repository, and all protocol types (timeline, slices, stamps, control messages) in `hadronyche-core` (C-133).

## Work
- Create (or, if it already exists, extend) the crate `hadronyche-tools` and add the binary target `hadronyche-l2`.
- Depend on `hadronyche-core` for timeline, slice and stamp types; the harness defines no protocol type of its own (C-133).
- Add empty modules `marker`, `rig`, `run`, `analysis`, `report` (files `marker.rs`, `rig.rs`, `run.rs`, `analysis.rs`, `report.rs`), each with a module doc comment stating its role:
  - `marker.rs` — marker generation and detection in raw capture (sub-sample interpolation).
  - `rig.rs` — rig description and calibration procedure.
  - `run.rs` — scenario runner (`steady`, `restarts`, `loss`, `handover-50ppm`, `step`, `drift-ramp`).
  - `analysis.rs` — emission-error statistics, marker-offset series, slip detection, ratio series, MTIE.
  - `report.rs` — certification record.
- Add a command-line skeleton for `hadronyche-l2` with subcommands `calibrate`, `run <scenario>`, `analyse <recording>`, `report`; each unimplemented subcommand exits with a non-zero status and a clear "not implemented" message.
- Add a CI job that builds the crate and runs its tests as an unprivileged user with no audio hardware.

## Acceptance criteria
1. `cargo build -p hadronyche-tools` produces the binary `hadronyche-l2`.
2. `hadronyche-l2 --help` lists the 4 subcommands `calibrate`, `run`, `analyse`, `report`.
3. `hadronyche-l2 run steady` (any unimplemented path) exits with a non-zero status and prints "not implemented".
4. The CI job runs `cargo test -p hadronyche-tools` without root and without an audio device, and passes.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Any real implementation of the modules (020-02 onward).
- The `RawCapture` trait (020-02).
- Platform backends (020-10 for Linux; epics 021 and 022 for macOS and Windows).
