# 026b-07 — Timestamp source and jitter
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** S · **Depends on:** 026b-02

## Context

Open sub-question owned by this epic: **timestamp source on the reference board (MAC hardware or interrupt) and its jitter.** Falsifier: clock lock bound of 25 µs unreachable with interrupt stamps.

Facts: receive timestamps come from MAC hardware timestamping if the part has it, else from the receive interrupt; the source is reported with a C-131 value (mac-hardware, interrupt, or userspace) in one of the four C-129 capability states. `DeviceTime::declared_jitter_ns()` is declared per device. Lock bound at most 25 µs (C-62, pending simulation; owner of that value: the servo simulation). Correction term at most 10 ppm (C-60) and its rate of change at most 1 ppm/s (C-61), bounding the correction, not the drift estimate (C-119). A device with MAC-level hardware timestamping can stamp better than a Linux host `[decision]`.

## Work

1. On the board, measure rx timestamp jitter against a reference (e.g. a Linux host with hardware timestamping sending at known times, or a GPIO toggled at rx stamp time captured by a logic analyser). The reference sender is a tool binary in `hadronyche-tools` (C-133).
2. Measure under load (one 6-channel stream at 2 000 packets/s plus control traffic).
3. Set `declared_jitter_ns` from the measured p99.99 or max; record.
4. Lock the board to a Linux holder for 1 hour; record offset error distribution vs the C-62 bound.
5. Record results in `doc/design/reports/` (C-130).

## Acceptance criteria

1. Timestamp source (C-131 value and C-129 state) recorded and shown in the stamp.
2. Jitter distribution (p50, p99, max) recorded `[measured]`, idle and loaded.
3. 1-hour lock: max offset error recorded; pass if ≤ C-62 (25 µs at the time of writing); else the falsifier is recorded with an owner.
4. `declared_jitter_ns` in firmware equals the recorded figure.

## Definition of done

- [ ] The reference board, its timestamp source and its oscillator are named in the docs
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] Regime stamp fields touched are complete and documented

## Out of scope

- Holdover behaviour (026b-15).
