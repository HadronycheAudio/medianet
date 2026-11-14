# 027-09 — Build and document the rig
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-08, epic 017, epic 018, epic 020, epic 026

## Context
- L3 compares the physical outputs of two certified sinks on one capture clock. The rig must be documented well enough to rebuild.
- Rig parts fixed by the epic:
  - One Linux host: clock holder, reference source module emitting a marker stream (epic 017), and the certified ALSA reference sink (epic 018, certified at L2 by epic 020).
  - One embedded reference device (epic 026) on the same switch, with a switchable power supply (outlet or PoE port) for `power-cycle-device` (027-13).
  - One two-channel capture device at 96 kHz, independent of both sinks, with a known inter-channel skew; **Linux output on channel 1, device output on channel 2**.
  - A second Linux host on the same switch as the new clock holder for the handover scenario (027-14). C-212 makes it a required rig part: the second host must be holder-eligible and locked when the current holder stops answering. Only a holder-eligible module with usable time (Locked, or in holdover with its claim not lapsed) may take over, keeping the epoch (C-213).
- Network: a Hadronyche Medianet is a wired, isolated LAN whose only traffic is the runtime's `[decision]`. IPv4 only, multicast with TTL 1 (C-100): all rig hosts on one switch and one subnet.
- Load: one default stream is 6 channels, about 1 320 bytes per packet on the wire and about 21.1 Mbit/s (C-04, C-20, C-21); the device's 100 Mbit port carries about 19 channels (C-26), so the marker stream fits.
- The wire carries only the domain rate, 96 kHz (C-01); the device's 48 kHz hardware converts 2:1 at the edge (C-80). The capture rate is 96 kHz.
- Why one capture device: one capture clock means skew is read directly as a time difference between two channels, with no clock comparison and no capture-delay calibration; any capture delay is common to both channels, provided the inter-channel skew is known and small.
- The capture device's inter-channel skew at 96 kHz is an open question owned by this epic: if larger than 2 µs and unstable, choose another capture device (measured in 027-10).
- The audition offset A has no default and is one number per presentation group (C-55). For ladder runs the harness computes A with the commissioning rule and records it with the results (C-148, ticket 027-23); the rig applies the computed value.
- Every rig run feeds the constants table, so it runs in strict mode (C-147), which the operator policy of the commissioning configuration sets (C-165). Strict mode is hosted-only; the device stamps real-time scheduling and memory lock present by construction.
- Both sinks must be in certification state commissioned, the only state that gives a numeric E (C-140).
- Where documents live (C-130): rig documentation and `l3-results.md` in `doc/design/reports/`.

## Work
1. Assemble the rig: Linux host, embedded device with switchable power, one switch with port control (for 027-16), capture device (96 kHz two-channel), second holder-eligible Linux host (C-212). The capture host may be a separate machine so capture load does not disturb the sink host; record which.
2. Parts list: make, model, firmware/kernel versions, cable types and lengths, switch model, power switching hardware, capture device model and settings (rate, bit depth, input level), IPv4 addressing.
3. Wiring diagram (SVG or similar, committed) showing network links, analogue links, power control, and channel assignment (ch1 = Linux, ch2 = device). Photograph if possible.
4. Rebuild procedure: step-by-step setup, including where the group's audition offset A, the group's required E, the operator policy with strict mode (C-165) and the stream configuration are set, how the Linux host grants real-time scheduling and memory lock (C-127), and how the current clock holder is made to stop answering for `handover` (C-212, 027-14) and the holder removed with no eligible replacement for `holdover` (027-18); for each scenario, which modules are holder-eligible, so that only the second Linux host can take over in `handover` and no module can in `holdover` (C-213).
5. Smoke test: marker stream plays on both sinks, capture shows markers on both channels, `hadronyche-l3 analyze` yields a skew series.
6. Commit the documentation in `doc/design/reports/` (C-130), next to where `l3-results.md` will live (the rig section of 027-20 references it).

## Acceptance criteria
1. Parts list covers every device and cable in the rig, with model and version.
2. Diagram shows channel 1 = Linux sink, channel 2 = embedded device, and all hosts on one switch and subnet.
3. Smoke test: a 60 s capture at 96 kHz produces at least one paired marker per second of markers planned, and `unpaired_markers` = 0.
4. A second engineer can rebuild the rig from the document alone (review sign-off on the PR).

## Definition of done
- [ ] The rig is photographed or diagrammed and its parts listed
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Tests green in CI, unprivileged" — no code; "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — none; "Design documents updated where behaviour differs" — documents the rig, changes no design behaviour; "No new open item" — opens none; "The skew analysis runs in CI on recorded two-channel files" — 027-08; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Measuring the capture device's inter-channel skew (027-10).
- Any scenario runs (027-11 onward).
