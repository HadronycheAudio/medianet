# ADR 11 — Software-resampler edge

## Status

Accepted. Hardware clock steering is Deferred-with-reservation.

## Context

A sink's or capture device's crystal is not the network's. Positions are network time in samples (ADR 09), so at every hardware edge something must reconcile the device's sample clock with the domain line. Only at this point does servo behaviour become audible: inside the network, alignment is by position.

Two actuators exist. A software resampler changes the sample stream to fit the device clock. Hardware steering trims the converter's clock (a PLL or fractional divider) to follow the network. The first works on any device; the second is bit-transparent but depends on hardware that exposes it.

The edge must run on four profiles: Linux, macOS, Windows and embedded.

## Decision

**v1 ships one blessed edge mechanism on every profile: a software resampler whose ratio is steered continuously.**

- **Two loops, both shared core code.** Loop 1 is the core servo: host clock to domain line, through sync exchanges. Loop 2 is a regression: device sample counter to host clock, through the (sample count, host time) pairs every audio API supplies. The resampler ratio is the quotient of the two slopes.
- A platform supplies two traits only: a host clock and a device clock, each with a declared jitter.
- The rate loop is a second servo cascaded on the first, with bandwidth well below 1 Hz (C-68), so it filters servo jitter.
- The published quality number is clock bound plus regression residual.
- Steps occur only as announced discontinuities (timeline epoch changes).
- Edge modules resample and are **never exact-tier**.
- Embedded devices in v1 run a 48 kHz device clock through the arithmetic kit's rational-ratio resampler node (C-80).
- **Reservation:** the embedded HAL defines a rate-actuator trait with one v1 implementation (software ratio). The edge contract is expressed only in E, D and wander, so hardware clock trimming later needs nothing on the wire.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Converter PLL steering in v1 | Adds a hardware dependence: works only on devices that expose a trimmable clock; unavailable on ordinary sound cards and laptop audio |
| Both actuators in v1 | Two certification paths for the first release; the trait reserves the second at the cost of one interface |
| No reconciliation (let the device free-run and drop or repeat samples) | Periodic slips are audible and break the zero-slip criterion (C-76) |
| Per-platform servo implementations | Three or four implementations of the same loops; replaced by shared core code with two small traits |
| Loop back through the same sound card to test the servo | Shares one crystal, so two resamplers cancel each other's error; and a lone host is its own holder, so the servo is idle unless drift is injected |

## Consequences

Positive

- One mechanism, one certification procedure, any device.
- The servo's slew and wander requirements finally have a consumer and can be measured (C-77).
- Laptop devices that run at 48 kHz are absorbed by the same resampler.

Negative

- CPU cost on small devices.
- No bit-transparent path, even where the hardware could offer one.
- The resampler's group delay adds to D, the sink's plumbing delay.
- Every edge module is bounded-class for verification.

Neutral

- Frequency-continuous handover (ADR 10) matters more to the edge than short-term slew, because the rate loop's low bandwidth filters short-term jitter but follows a rate step.

## Falsifier

- L2 (task 020): the reference sink cannot hold E within target (C-70, C-71, C-72), the marker offset within ±2 samples (C-75), zero slip (C-76) or the wander mask (C-77) with a software resampler.
- L3 (tasks 026, 027): the resampler's cost or quality does not fit the reference embedded device.
- The edge passes servo jitter into the ratio despite the low loop bandwidth (C-68).

## Links

- [01-architecture.md §9.4 Two-loop timebase](../01-architecture.md#94-two-loop-timebase), [§9.3 E and D honesty](../01-architecture.md#93-e-and-d-honesty), [§9.5 Certification ladder](../01-architecture.md#95-certification-ladder), [§10.2 Scope on macOS and Windows](../01-architecture.md#102-scope-on-macos-and-windows)
- [02-constants.md](../02-constants.md): C-68, C-70, C-71, C-72, C-75, C-76, C-77, C-80
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-26, AC-27, AC-28, AC-29
- Related: [10-Servo-redesign.md](10-Servo-redesign.md), [12-Presentation-groups-with-required-E.md](12-Presentation-groups-with-required-E.md), [23-macOS-and-Windows-edge-roles.md](23-macOS-and-Windows-edge-roles.md)
