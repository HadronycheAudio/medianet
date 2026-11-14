# 020-11 — Capture-delay calibration method
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-09

## Context
The L2 emission error for a marker at position p is `[decision]`:

`(network time at which it was captured − capture-side delay) − (T(p) + A)`

The capture-side delay is calibrated once per rig. It is the capture mirror of D: from acoustic instant to the sample being available (C-170). Calibrating it from the same loop that measures the sink would be circular. This is an open sub-question owned by epic 020 `[decision]`:

| Question | Falsifier |
|---|---|
| How is the capture side's own delay calibrated without circularity (independent capture device with a datasheet figure, an oscilloscope against a network-time pulse, or symmetric-loop halving)? | No method resolves below 10 µs, the D share of the Linux target |

The Linux E target is 50 µs (C-70) = 20 µs clock + 20 µs tracking + 10 µs D `[decision]`. At 96 kHz (C-01) one sample is 10.4 µs. For the reuse by epics 021 and 022, the D shares are 20 µs on macOS and 800 µs on Windows (C-150, proposals). Epic criterion 020-A6 requires the calibration to be repeatable: capture-side delay within ±1 sample over 10 calibrations.

## Work
- Evaluate the three candidate methods against the rig hardware available:
  1. independent capture device with a datasheet latency figure;
  2. oscilloscope comparing the analogue marker against a network-time pulse (e.g. a GPIO or a second audio channel driven at a known network time);
  3. symmetric-loop halving.
- For each: expected resolution, sources of circularity, equipment needed.
- Where hardware allows, measure the resolution of at least the chosen method `[measured]`.
- Write a short decision note (in the `rig.rs` module doc and in the rig documentation of 020-21) naming the chosen method with its reason `[decision]`, or, if the falsifier holds (no method below 10 µs), state that and the consequence for the D share of C-70 (to be carried into 020-24).

## Acceptance criteria
1. The note compares all 3 methods with a stated resolution for each (measured where possible, otherwise tagged with its source).
2. One method is chosen, and its expected resolution is at most 10 µs, or the falsifier is declared met with its measured figures.
3. The chosen method's procedure is specified step by step precisely enough for 020-12 to implement it.

## Definition of done
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Implementing the calibration in `rig.rs` and the 10-run repeatability check (020-12).
- The physical rig documentation (020-21).
