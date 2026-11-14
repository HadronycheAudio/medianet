# 026b-05 — Resampler CPU budget
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** S · **Depends on:** 026b-03

## Context

Open sub-question owned by this epic: **do the fixed 2:1 resampler node and the steerable rate actuator fit the device's CPU budget at 2 000 slices/s, including the soft-float cost of `f64` ratio arithmetic on a part without a double-precision FPU?** Falsifier: it does not fit — shorten the kit actuator's filter (quality cost; owner epic 024a, which builds both resamplers, C-197), change the ratio type (owner epic 024a, which defines `RateActuator`, C-210), or require hardware clock trimming earlier.

Facts: slices are 48 frames = 0.5 ms (C-02) at 96 kHz, the only rate on the wire (C-01), 2 000 per second; the 2:1 conversion to 48 kHz (C-80) is done by the kit's resampler node (epic 024a, C-197), fixed-ratio and offered for 2:1 because its delay is a whole number of samples (C-105), inside the edge (C-134); the rate actuator is a separate steerable resampler whose ratio varies by parts per million (C-189), also the kit's (epic 024a, C-197; adapted in 026a-09), and the `f64` ratio arithmetic belongs to it. If the actuator's filter is shortened, the change in group delay changes D (C-134, C-170) and is reported through 026b-14. The device always receives a whole 6-channel stream (C-04); it may resample only the channels it plays. Both resamplers share the CPU with the cooperative `poll` loop, whose control handling must stay bounded. No libm is allowed (026-AC-11).

## Work

1. On the board, measure cycles per slice for the fixed node and for the rate actuator, each for 6 channels and 2 channels, with a cycle counter, and separately the cycles spent in the actuator's `f64` ratio arithmetic.
2. Measure total CPU load of the firmware while receiving one 6-channel stream and playing out, locked to a holder.
3. If load leaves < 25 % idle (or the poll worst case is threatened), shorten the actuator's filter via the const parameter from 026a-09, or request a shorter filter from epic 024a if the kit does not offer it (C-197), and record the quality cost (passband ripple / stopband attenuation before and after).
4. Record the answer to the open sub-question in `doc/design/reports/` (C-130).

## Acceptance criteria

1. Cycles per slice (fixed node and rate actuator separately), `f64` share and CPU load % recorded `[measured]` for 2 and 6 channels.
2. A recorded yes/no answer to the sub-question; if "no", the chosen remedy (shorter filter with its quality figures, filed with epic 024a if the kit must change, a ratio-type change filed with epic 024a (C-210), or a request to bring hardware trimming forward) is recorded with an owner.
3. No underruns over 10 minutes at the chosen filter length.

## Definition of done

- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope

- Hardware clock trimming implementation (later, behind the same trait).
- Memory fit (026b-04).
- The calibration factor for DSL modules (026b-19).
