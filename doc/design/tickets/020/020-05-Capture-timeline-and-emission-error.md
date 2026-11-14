# 020-05 — Capture timeline and emission error
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-04, epic 018

## Context
Each edge has two loops `[decision]`: loop 1 maps the host clock to the domain line (the servo); loop 2 maps the device sample counter to the host clock by regression over (sample count, host time) pairs from the audio API. The shared two-loop core comes from epic 018.

For capture, the harness builds the **capture side's device line**: a regression of capture `first_frame_count` against `host_ns` (from `RawBlock`), then host time to network time through loop 1. With it, a detected marker's capture frame (020-04) is converted to the network time at which the marker was captured `[decision]`.

The emission error for a marker at position p is `[decision]`:

`emission_error(p) = (network time at which it was captured − capture-side delay) − (T(p) + A)`

where `T(p)` is the network time of position p (`position = (network_time − epoch_origin) × rate`), A is the presentation group's audition offset, and the capture-side delay is calibrated once per rig (020-11, 020-12). The capture-side delay is the capture mirror of D: from acoustic instant to the sample being available (C-170).

## Work
- In `analysis.rs`, build the capture device line from the `RawBlock` stream using the two-loop core of epic 018 (loop 2 regression; loop 1 host-to-network mapping, from the in-process clock holder or from recorded ground truth).
- Implement `capture_network_time(capture_frame: f64) -> network_ns`.
- Implement the emission-error computation per marker, taking `epoch_origin`, rate, A and the calibrated capture-side delay as inputs (the rate is the domain rate, 96 kHz, the only rate on the wire, C-01; A has no default, C-55; for ladder runs the runner computes it with the commissioning rule, C-148, ticket 020-26; it is written into the output).
- Handle timeline-epoch changes: an announced step larger than 2 ms (C-67) changes `epoch_origin`; T(p) must use the epoch in force for p.
- Output a series `(p, network_time_captured, emission_error_ns)`.

## Acceptance criteria
1. On a synthetic recording where the capture counter runs 20 ppm fast against host time and the emission error is a constant 0, the computed emission error is within ±1 µs for every marker over 1 hour.
2. With a constant injected emission error of +30 µs, the mean computed error is 30 µs ± 1 µs.
3. Across a synthetic announced epoch change of 5 ms (larger than C-67's 2 ms), the computed emission error shows no step larger than 1 µs.
4. Changing A or the capture-side delay by x shifts every computed error by exactly −x (A) or −x (delay), verified by test.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The analysis code runs in CI on recorded capture files (no audio hardware needed)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Percentiles, offset series and slip detection (020-06); ratio and MTIE (020-07).
- How the capture-side delay is obtained (020-11, 020-12).
- Changes to the two-loop core (epic 018).
