# ADR 18 — DSCP classes

## Status

Accepted

## Context

A medianet carries four kinds of runtime traffic: clock sync exchanges, media slices, measurement messages, and control (discovery, election, admission). Marking them lets a switch that is configured for it treat them differently. On a medianet the only contention is between the runtime's own classes, typically sync against media bursts.

The POC marks one configurable value, default 46, on clock and data sockets alike `[code]`.

The open question was where sync belongs. One argument puts sync in the media class "because the calibration target is the media path". That confuses two measurements. The servo estimates a clock offset from request/reply exchanges; its error is half the asymmetry between the two directions, so it wants the smallest and most symmetric delay available. The latency of the media path is measured separately, by slice timestamps.

Platforms also differ in whether marking can be set or trusted at all.

## Decision

**Three classes** (C-13):

| Traffic | Class | Value |
|---|---|---|
| Clock sync exchanges | CS6 | 48 |
| Media slices and one-way probes | EF | 46 |
| Control: discovery, election, admission, receiver reports | CS5 | 40 |

- Sync rides its own class above media.
- One-way probes ride the media class because they measure the media path.
- **Marking is verified by the receiving peer**, which reads the received byte and reports it in the receiver report. No platform is trusted to self-report.
- The classes and the peer-observed values are in the regime stamp; where marking cannot be confirmed the stamp says "unverified".
- Switch configuration to honour the classes is a commissioning item.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Sync in the media class | The servo wants the least and most symmetric delay, not a representative one; sync queued behind media bursts sees direction-dependent delay, which biases the offset. The media path is measured separately by slice stamps |
| One class for everything (POC) | Sync and control queue behind media bursts |
| Self-reported marking | On Windows an arbitrary value goes through a QoS API and may need administrator rights or policy `[memory]`; a platform can believe it marked when it did not |
| Per-deployment values with no defaults | Every commissioning would start from nothing; the defaults are configurable |
| No marking | Marking is nearly free and is the first thing that helps on a congested port |

## Consequences

Positive

- Sync exchanges are not delayed by media bursts on a switch that honours the classes.
- The stamp states what was actually observed on the wire, per connection.
- Control traffic cannot starve media or sync.

Negative

- Sync in a better queue than media makes the measured sync delay flatter than the media path. The stamp records the classes for that reason; the sync path delay must never be read as the media path delay.
- On an unmanaged switch the marking does nothing.
- On Windows the marking may be report-only.

Neutral

- Interrupt coalescing on the NIC biases software receive stamps on every hosted platform regardless of class; commissioning disables it on medianet interfaces.

## Falsifier

- L1 (task 019): the receiver report does not show sync 48, media and probes 46, control 40 and the stamp does not say "unverified" (AC-34).
- L1 measurement on a configured switch shows the offset estimate is no better, or worse, with sync in its own class than in the media class.

## Links

- [01-architecture.md §3.5 DSCP](../01-architecture.md#35-dscp), [§1.2 Protocol stack](../01-architecture.md#12-protocol-stack), [§3.6 Measurement path](../01-architecture.md#36-measurement-path), [§10.3 Porting traps](../01-architecture.md#103-porting-traps)
- [02-constants.md](../02-constants.md): C-13
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-34
- Related: [10-Servo-redesign.md](10-Servo-redesign.md), [24-Numbers-governance.md](24-Numbers-governance.md), [23-macOS-and-Windows-edge-roles.md](23-macOS-and-Windows-edge-roles.md)
