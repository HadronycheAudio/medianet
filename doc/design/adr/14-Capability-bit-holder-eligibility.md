# ADR 14 — Capability-bit holder eligibility

## Status

Accepted

## Context

One module holds the clock; its monotonic clock defines network time for the whole medianet. A medianet mixes profiles: Linux hosts, macOS and Windows edges, embedded devices. Not every node should be a candidate.

In the POC every module is a candidate, the holder rotates after 2 mandates (P-05), and eligibility is only "the machine is not under I/O pressure" `[code]`.

An obvious rule would exclude profiles: "embedded devices have no kernel timestamps, so they cannot hold the clock". That rule is wrong in its reason. A device with MAC-level hardware timestamping stamps better than a Linux host with software stamps `[memory]`.

Since sources pace from network time, every holder change is a frequency event for every stream, and handover must be frequency-continuous (ADR 10).

## Decision

**Eligibility to hold the clock is a capability bit in the hello, not a rule about profiles.**

A candidate sets the bit only if it:

- is **locked** (so it has a rate estimate to make handover frequency-continuous);
- runs on a host not under I/O pressure;
- can **serve up to 64 followers** (C-35) at 50 requests per second each (C-36).

Further rules:

- Embedded, macOS and Windows nodes do not set the bit in v1.
- The holder changes only on loss or pressure. Rotation is retired.
- Holder loss is judged on sync replies only: election after 1.5 s of reply silence (C-37).
- A follower beyond the cap is refused with a named event (C-35). A tiered sync relay is the documented future answer and is not built.
- If no locked candidate exists, the new holder bumps the timeline epoch, which makes it an announced discontinuity.
- Every module computes the same winner from the same candidate set; higher election epoch wins on merge.
- **A medianet needs at least one holder-capable node, or there is no audio.**

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Exclude embedded by profile ("no kernel timestamps") | Wrong disqualifier: MAC-timestamping devices stamp better than software-stamping hosts. The real reasons are serving load, availability and oscillator quality |
| Every module is a candidate (POC) | Devices get power-cycled; a laptop edge sleeps; serving dozens of followers competes with audio on a small device |
| Holder rotation (POC) | Each handover is a frequency event on every stream |
| Unlocked candidates eligible with a rate step at handover | Violates frequency-continuous handover; allowed only as an explicit timeline epoch change |
| No follower cap | Serving load at the holder grows with followers and processes with nothing to bound it |
| Rank compared across election epochs on merge | Rank is a stale snapshot; comparing it across epochs breaks monotonic convergence |

## Consequences

Positive

- The rule is about what a node can do, so a future device with hardware timestamping, a good oscillator and capacity can become eligible by setting one bit.
- Holder changes become rare events.
- Handover has a rate estimate to start from, by construction.

Negative

- A medianet of only edges and devices has no clock.
- The cap of 64 limits the medianet's size together with the 64-module bound (C-34), and it counts processes, since each process runs its own servo.

Neutral

- Cost of the reservation: one bit in the hello. No v1 device sets it.
- Embedded devices still run the adopt side of the election state machine, which lives in core for both profiles.

## Falsifier

- L1 / L3 (tasks 019, 027): holder serving lateness grows with followers below 64 (C-35, AC-25).
- A deployment requirement for medianets with no Linux host, which would force a holder-capable device profile.
- Simulation (task 008): a locked new holder still produces a rate step above 1 ppm/s at handover (C-61).

## Links

- [01-architecture.md §2.3 Holder and election](../01-architecture.md#23-holder-and-election), [§2.2 Timeline epochs](../01-architecture.md#22-timeline-epochs), [§10.1 Capability table](../01-architecture.md#101-capability-table), [§11.1 Regime stamp](../01-architecture.md#111-regime-stamp)
- [02-constants.md](../02-constants.md): C-34, C-35, C-36, C-37, C-61, P-05
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-5, AC-25, AC-36
- Related: [10-Servo-redesign.md](10-Servo-redesign.md), [07-Control-plane-invariants.md](07-Control-plane-invariants.md), [23-macOS-and-Windows-edge-roles.md](23-macOS-and-Windows-edge-roles.md)
