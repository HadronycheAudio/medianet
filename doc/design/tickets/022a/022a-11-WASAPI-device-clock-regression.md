# 022a-11 — `DeviceClock` from (device position, QPC) pairs
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** M · **Depends on:** 022a-02, 022a-10, epic 018

## Context
- Part of epic 022a (Windows edge backend), `wasapi/device.rs`. Deliverable: `impl DeviceClock for WasapiDevice { /* nominal_rate, declared_jitter_ns */ }`.
- Device clock S on Windows `[memory]`: the device clock interface returns a device position with a QPC timestamp. Check: regression residual.
- **Two-loop timebase, shared core code** (epic 018) `[decision]`: loop 1 maps host clock H (QPC) to the domain line through sync exchanges; loop 2 maps the device sample counter S to H by regression over (device position, QPC time) pairs; the ratio of the kit's steerable rate actuator (epic 024a, C-189, C-197) is the quotient of the slopes; rate-loop bandwidth well below 1 Hz (C-68). This ticket only feeds loop 2 with pairs; the regression code itself is shared core.
- Timeline: `position = (network_time − epoch_origin) × rate`; `T(p)` is the network time of position p.

## Work
1. In each audio callback (and optionally between callbacks), read (device position, QPC timestamp) from the WASAPI device clock interface; convert QPC to ns with `qpc_ticks_to_ns` (022a-02); convert position to frames at the device rate.
2. Feed the pairs to the epic-018 loop-2 regression; expose its slope, intercept and residual.
3. `nominal_rate` = negotiated device rate (022a-10); `declared_jitter_ns` = derived from the observed regression residual (document the rule, e.g. 99.9th percentile of residual over the warm-up window).
4. Handle device position wrap/reset (stream restart, glitch) by restarting the regression and flagging a discontinuity.
5. Tests: unit test with synthetic pairs (known slope + noise); manual run on reference hardware recording residual.

## Acceptance criteria
1. With synthetic pairs at 48 000 Hz × (1 + 50 ppm) plus 20 µs uniform noise, the loop-2 slope estimate is within 1 ppm of truth after 60 s of simulated input.
2. A position reset in the input produces a discontinuity flag and a fresh regression, not a slope excursion.
3. On reference hardware, the regression residual over 10 minutes is recorded `[measured]` (50th, 99.9th, max).
4. `WasapiDevice` implements `DeviceClock` from the epic-018 surface.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The device-clock fact is confirmed `[measured]` or corrected, for 022b-08

## Out of scope
- The regression algorithm and loop 1 (shared core, epic 018).
- The deadline clock (022a-12).
