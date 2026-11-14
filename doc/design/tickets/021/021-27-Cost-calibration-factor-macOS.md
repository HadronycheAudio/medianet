# 021-27 — Cost calibration factor for the macOS target
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-01, 021-31, epic 018, epic 025
## Context
The compiler's cost bound is counted in abstract operations per slice, identical on every machine. Each target has a calibration factor (time per operation), measured once and recorded in the regime stamp; admission converts the cost bound to time with it (C-106) [decision].

- The macOS edge runs the kit's resampler node when the device is not at 96 kHz (C-80); the kit (epic 024a) builds it and this epic builds none (C-197), so admission on a macOS target needs that target's factor.
- Calibration procedure [decision, C-176]: the DSL conformance suite (epic 025) runs a reference workload once per listed target and records time per operation; the value is carried in the regime stamp. This ticket makes sure macOS is a listed target, runs the suite's calibration on each reference Mac (021-31) and wires its result into the stamp.
- Harness and tool binaries live in the `hadronyche-tools` crate (C-133); results go to `doc/design/reports/` (C-130).
## Work
- Run the epic 025 conformance suite's calibration workload on each reference Mac and record the factor `[measured]` with the OS build and CPU model.
- Expose the factor to the regime stamp (021-20) and to core admission through the hosted trait surface.
- Add the procedure to the crate's manual hardware-test document.
## Acceptance criteria
1. A calibration factor for each reference Mac is recorded `[measured]` in `doc/design/reports/` with the conditions of the run.
2. The `macos-std` stamp carries the factor of the running machine; a machine without a measured factor stamps it with capability state *unverified* (C-129).
3. Unit test: admission of a node with a known operation count converts to time using the stamped factor.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
- [ ] Every new factual claim carries an evidence tag
## Out of scope
Defining the cost unit or the calibration workload (epic 025). The resampler node (the kit, epic 024a, C-197).
