# 018-11 — Declared delay D and commissioning persistence
**Epic:** 018-Linux-ALSA-edge · **Effort:** S · **Depends on:** 018-10, epic 024a

## Context
- **D** is the sink's declared total plumbing delay from hand-off to emission: **driver buffer + resampler group delay + converter** (C-170); the edge's conversion delay is part of D (C-134) and is never counted in the structural latency L (C-207). `submit(p) = emit(p) − D`. Capture mirrors it with D_capture, from acoustic instant to the sample being available (C-170): a sample is stamped `T⁻¹(capture_time − D_capture)`.
- Typical D on Linux: about **1 ms** (C-81, pending measurement).
- The analogue part of D needs an **external commissioning measurement**; certification is two-tier (*self-monitored* vs *commissioned*).
- D must be repeatable across restarts: spread at most **25 µs over 20 restarts** on Linux (C-74). Whether this holds on ALSA with the chosen device is an open sub-question owned by epic 020 (falsifier: spread above 25 µs over 20 restarts).
- E budget at 50 µs (C-70) allows **10 µs** for the uncertainty of D.
- Resampler delay has two parts (C-189): the rate actuator's group delay from `group_delay_frames()` of the kit's rate-actuator trait (defined by epic 024a, C-210), and, on a 48 kHz device, the delay of the fixed 2:1 (sink) or 1:2 (capture) resampler node (C-80); both resamplers are the kit's, built by epic 024a (C-197), a whole number of samples (C-105). Both are part of D (C-134, C-189); each is converted at the rate at which it runs. This resampler part is also the conversion delay that the manifest reports separately on the device-facing port (C-207, ticket 018-27).
- The epic settles persistence: the commissioning-measured part is kept per (device, driver, device rate) in a versioned file in the module's state directory, and is **written only inside a declared commissioning window** that the module stamps; outside a window the module is read-only to the commissioning tool (C-107). The window is announced by the tool on the control group with an expiry time and closes at expiry or on explicit close (C-153). **C-178:** the commissioning tool's own measurements taken in a window (D, restart spread) are valid commissioning results, each recorded with the window it was taken in; it is the ordinary production figures in the stamp that are unqualified while a window is open (018-18, 018-23, 018-24).
- Fuzz budget: 60 s on every change and 1 hour nightly, corpus kept (C-95).
- File: `hadronyche-os-linux/src/alsa/declared.rs`.

## Work
1. `DeclaredDelay { driver_ns, resampler_ns, converter_ns, method }` with `total_ns()`; `method` records per component whether it is self-reported (driver, resampler) or commissioned/configured (converter).
2. Driver part from the PCM status delay/buffer configuration (018-10); resampler part = actuator `group_delay_frames()` plus the fixed node's delay when the device runs at 48 kHz, each converted at its own rate (C-189); converter part from configuration, or from the persisted commissioning measurement when present.
3. Persistence of the commissioning-measured part: a small versioned text file per (device, driver, device rate) key, in the module's state directory; load at start, write via a commissioning API that refuses any write outside a declared commissioning window (C-107) with a named reason; each persisted value is recorded with the window it was taken in (C-178).
4. Parser for that file: strict, versioned, rejects unknown keys; fuzz target run at the C-95 budget.
5. Log D and components at every start so restart spread can be computed (input to epic 020).

## Acceptance criteria
1. `total_ns()` equals the sum of components in a table-driven test, including a 48 kHz device whose resampler part is the actuator delay plus the fixed node's whole-sample delay (C-105, C-189), and a 96 kHz device where it is the actuator delay alone.
2. A persisted commissioning value round-trips exactly; a malformed or version-mismatched file is rejected with a named error and the sink falls back to *self-monitored*.
3. A commissioning write outside a declared commissioning window, including one after the window's expiry (C-153), is refused with a named reason and leaves the file unchanged; inside a window it succeeds, the persisted value is recorded with that window and reloaded with it (C-178), and the window is visible in the stamp.
4. The persistence parser's fuzz target runs 60 s in the per-change CI job and 1 hour nightly without crash, with its corpus committed (C-95).
5. On the reference setup, D logged across 20 restarts is collected by a script that prints the spread as max − min (C-166); pass/fail at 25 µs is judged by epic 020.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Measuring the analogue converter delay (commissioning tool, not this epic).
- Declaring and stamping the commissioning window itself (hosted runtime, epic 015).
- Documenting the D method (018-20).
- Writing D to the regime stamp (018-18).
