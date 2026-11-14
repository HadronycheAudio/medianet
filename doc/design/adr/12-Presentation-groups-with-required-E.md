# ADR 12 — Presentation groups with required E

## Status

Accepted

## Context

Fan-in alignment (ADR 05) is local: each consumer chooses its own offset from its own measurements. Thirty sinks of one stream would therefore play at thirty different delays. Nothing in alignment alone gives a set of physical outputs a common presentation time.

Sinks also differ widely in quality. A Linux reference sink targets an emission error of 50 µs (C-70); a Windows sink targets 1 ms (C-72). Without a rule, a 1 ms sink silently joins an array that needs 50 µs.

The edge needs one formula, one operator number, and honesty requirements per sink.

## Decision

**One formula.** For a sink in a presentation group with audition offset A:

- Emission: `emit(p) = T(p) + A`
- Hand-off to hardware: `submit(p) = emit(p) − D`

T is the domain line; D is the sink's declared total plumbing delay (driver buffer, resampler, converter). Capture is the mirror: a sample is stamped with the position of its acoustic instant. With both, A is exactly the mouth-to-ear latency.

**A is operator configuration, one per presentation group** (C-55). The default group is the whole domain.

- The commissioning tool proposes the minimum feasible A (largest over sinks of lateness + D + margin).
- The runtime verifies continuously; a sink that cannot meet A alarms.
- A is the ceiling of every fan-in ratchet upstream of the group (ADR 06).
- A has no default number.

**A group carries a required E.** It is enforced in the admission manifest exchange: a sink whose E exceeds the group's required E is refused with a named reason. The commissioning tool certifies membership.

**E and D honesty.**

- E is the 99.9th percentile of |actual emission − (T(p) + A)| over a one-hour certification run, with the maximum reported beside it (C-73). On hosted platforms E is a percentile claim, not a bound; only embedded can claim a bound.
- E decomposes into clock bound + rate-tracking error + uncertainty of D.
- A sink self-monitors the first two continuously. The analogue part of D needs an external commissioning measurement. Certification is two-tier.
- D must be repeatable across restarts (C-74).
- A sink that cannot bound E cannot claim presentation alignment. Certification failures are in the stamp.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Each sink uses its own fan-in offset | Sinks of the same stream play at different delays |
| A computed live by the runtime | Changing A is a domain-wide discontinuity, so it is operator policy; the runtime proposes and verifies |
| One A for the whole domain, no groups | A monitor sink next to the source would pay the deepest path's delay |
| A per sink set by the operator | Loses simultaneity, which is the purpose |
| E as a hard bound on hosted platforms | No hosted platform without a real-time kernel can bound scheduling latency; a percentile over a certification run is what can honestly be claimed |
| Hand-off time as the emission formula (`T(p) + A − D` as playout) | Conflates emission with hand-off; E must be measured against `T(p) + A` |
| No required E per group | A coarse sink silently joins a tight array |

## Consequences

Positive

- Cross-sink simultaneity is one configured number (A) and one honesty requirement (D, E) per sink.
- A sink's fitness for a group is checked at admission, by name.
- A bounds all upstream ratchets, so the operator has one latency budget.

Negative

- Every sink in a group pays the worst member's delay.
- The analogue part of D cannot be self-certified; commissioning measurement is mandatory.
- A group that includes a Windows sink must accept an A of the order of its plumbing delay (typical D of 6 to 20 ms, C-81).

Neutral

- Windows and macOS sinks can still play; they simply cannot join groups whose required E they do not meet.

## Falsifier

- L3 (task 027): skew between a hosted and an embedded sink exceeds the sum of their E values over one hour or across restarts (C-79). This is the proof of cross-sink simultaneity.
- L2 (task 020): restart spread of D exceeds its target (C-74), so D is not a constant a sink can declare.
- AC-30 fails: a sink whose E exceeds the required E goes live in the group.

## Links

- [01-architecture.md §9.1 One formula](../01-architecture.md#91-one-formula), [§9.2 Presentation groups](../01-architecture.md#92-presentation-groups), [§9.3 E and D honesty](../01-architecture.md#93-e-and-d-honesty), [§9.5 Certification ladder](../01-architecture.md#95-certification-ladder)
- [02-constants.md](../02-constants.md): C-55, C-70, C-71, C-72, C-73, C-74, C-79, C-81
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-26, AC-27, AC-28, AC-29, AC-30
- Related: [06-Ratcheted-offset.md](06-Ratcheted-offset.md), [09-Domain-line-with-epochs.md](09-Domain-line-with-epochs.md), [11-Software-resampler-edge.md](11-Software-resampler-edge.md), [23-macOS-and-Windows-edge-roles.md](23-macOS-and-Windows-edge-roles.md)
