# ADR 09 — Domain line with epochs

## Status

Accepted

## Context

Fan-in alignment (ADR 05) keys on media position. That requires an answer to: what is a position, who defines it, and what happens when network time is discontinuous.

In the POC the media clock is the sender's local crystal: the data-plane ticker runs on local monotonic time, not network time `[code]`. Two sources would drift apart by their ppm difference. This does not matter in the POC because its consumers are arrival-driven and there is only one source.

A design with one position line per stream fails as soon as a mixer combines two independent sources: the mixer has no single timeline to stamp its output on, and an "audition offset" for a group of sinks is not actually one number.

## Decision

**One timeline per sample rate for the whole domain.** Position is network time counted in samples from a domain epoch:

`position = (network_time − epoch_origin) × rate`

- Streams have no private origins. Two independent sources at the same rate share one position space.
- Sources pace position from network time.
- A capture edge stamps each sample with the position of its acoustic instant.
- Rates other than the domain default (96 000 Hz, C-01) share the epoch; a 48 kHz position is half the 96 kHz position.
- **The map is affine per timeline epoch and never extrapolated across epochs.**

**Timeline epochs.** A 16-bit counter (C-10), distinct from the election epoch. A continuous clock handover does not change it; a discontinuity does.

- The epoch is carried in every sync reply and every slice header.
- The holder is a clock server, not a stream source, so it cannot announce in-band. Each **source** learns the new line from sync, announces in its own streams the future position at which it switches (lead 64 slices, C-43), and restarts its session there.
- A fan-in treats an input still on another epoch as a gap.
- A step larger than 2 ms (C-67) is never slewed; it is an epoch change.
- A holder that detects its own stall (the host was suspended) bumps the epoch.
- If no locked module is available to take the clock, the new holder bumps the epoch.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| One position line per stream | Breaks mixed-source stamping: a mixer of two independent sources has no single output timeline, and one audition offset per group is not one number |
| Media clock on the source's local crystal (POC) | Independent sources drift apart; fan-in by position is impossible without resampling at every mixer |
| Position carried with an explicit presentation timestamp per slice | Redundant once position is network time in samples |
| Holder announces discontinuities in-band | The holder emits no media streams |
| Slew across a discontinuity | A large step slewed at the allowed rate takes far too long and silently misaligns everything meanwhile; steps must be explicit |
| Reuse the election epoch as the timeline epoch | A continuous handover changes the election epoch without any discontinuity of time |
| Extrapolate the old line across an epoch change | Would align samples that belong to different timebases |

## Consequences

Positive

- Fan-in across independent sources is trivially aligned.
- With capture stamped at the acoustic instant, the audition offset A is exactly the mouth-to-ear latency of the medianet.
- Discontinuities are always explicit, per stream, at a named position.
- A 64-bit position at 96 kHz never wraps in practice.

Negative

- The servo is now the pacing clock of every source and reaches hardware edges as resampler ratio; its smoothness becomes a hard requirement (ADR 10).
- A source with its own hardware clock (a converter with its own crystal) must resample at the edge (ADR 11).
- During an epoch transition a fan-in gaps inputs that have not yet switched; the transition is audible.

Neutral

- Non-live sources (file playback, synthesis) stamp at generation time on the same line.

## Falsifier

- AC-18 fails: samples from two epochs are combined at a fan-in.
- AC-36 fails: after a healed partition the losing side steps silently instead of receiving an epoch change at an announced position.
- Spec drafting (task 006) shows a 16-bit epoch counter can wrap within a plausible uptime (C-10).

## Links

- [01-architecture.md §2.1 The domain line](../01-architecture.md#21-the-domain-line), [§2.2 Timeline epochs](../01-architecture.md#22-timeline-epochs), [§4.5 In-band announcements](../01-architecture.md#45-in-band-announcements), [§9.1 One formula](../01-architecture.md#91-one-formula)
- [02-constants.md](../02-constants.md): C-01, C-10, C-43, C-67
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-18, AC-36
- Related: [05-Position-invariant-fan-in-alignment.md](05-Position-invariant-fan-in-alignment.md), [10-Servo-redesign.md](10-Servo-redesign.md), [12-Presentation-groups-with-required-E.md](12-Presentation-groups-with-required-E.md)
