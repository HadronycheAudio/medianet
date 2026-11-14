# ADR 20 — 10 GbE for wide nodes

## Status

Accepted. Multicast fan-out is Deferred-with-reservation.

## Context

Fan-out in v1 is unicast: one copy of each stream per consumer. A 96-channel source is 338 Mbit/s on the wire (C-22). Each endpoint link is admitted up to 70 % of its rate (C-23).

On gigabit Ethernet that gives a wall of about 198 channel-copies per port (C-24): 8 × 24 channels, 4 × 48 channels or 2 × 96 channels. Concretely:

- a 96-channel source can feed two consumers by unicast, not three;
- a mixer can take two 96-channel sources, not three.

The limit is per endpoint port, not per medianet: any number of wide groups coexist on a non-blocking switch as long as no port exceeds the wall.

Two remedies exist for a port beyond the wall: a faster port, or one-to-many delivery (multicast) so that fan-out does not multiply egress.

## Decision

- **Any port carrying more than about 198 channel-copies runs 10 GbE** (C-25). The trigger is channel-copies per port, source or sink, not the number of bundles a node sources; one bundle fanned out to three consumers is also beyond the wall.
- A request that would exceed a port's ceiling is **refused at admission** with a named reason.
- **Multicast fan-out stays deferred.** There is no v1 reconsideration trigger except an explicit product change.

Reservations already in the wire format, so multicast can be added without a wire break:

- streams are identified by source and stream number, never by destination;
- forward packets carry no per-receiver state;
- the admission reply names the address to listen on;
- the per-source pacing phase offset is a property of the source, so it survives one-to-many delivery unchanged.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Receiver-joined multicast streams in v1 | On switches without group snooping it floods every port; v1 already carries packetisation, fan-in alignment, admission and the connection state machine. Deferred with the reservations above |
| Leave wide fan-out unspecified | A third unicast copy of 96 channels silently oversubscribes the port |
| Raise the admission ceiling above 70 % | The margin covers pacing bursts and residual chatter; it is not spare capacity |
| 3-byte samples to gain density | Rejected (one wire sample format); it would move the wall, not remove it |
| Trigger on "nodes sourcing more than two bundles" | Wrong unit: the wall is channel-copies per port and applies to sinks and to fan-out of a single bundle as well |

## Consequences

Positive

- The capacity rule is one number per port, checkable at admission and at commissioning.
- No multicast machinery in v1: no group management, no snooping dependence, no per-group path-MTU minimum.
- On 10 GbE the port wall moves far out.

Negative

- Wide nodes need 10 GbE ports and a switch that provides them.
- On 10 GbE the first wall becomes the consumer's receive cost per slice (guessed at 2 to 4 µs per packet, C-93), which is unmeasured.
- A deployment that must fan 96 channels out to three or more consumers on gigabit hardware cannot do so in v1.

Neutral

- A 100 Mbit device port holds about 19 channels (C-26); devices are narrow sinks or sources that subscribe to individual bundle members.
- Memory is not a wall (about 370 KB of alignment buffer per 16-member bundle, C-27), nor is clock serving, which is per process and independent of streams.

## Falsifier

- An explicit product requirement for three-way or wider fan-out of 96 channels on gigabit Ethernet. Multicast then returns to scope.
- L1 (task 019): the two-bundle test on one GbE port fails (C-92), which would mean the usable wall is lower than 198 channel-copies.
- L1 on 10 GbE shows the receive-cost wall at a channel count too low to be useful (C-93).

## Links

- [01-architecture.md §8 Capacity map](../01-architecture.md#8-capacity-map), [§8.2 The walls](../01-architecture.md#82-the-walls), [§6.3 Ceilings](../01-architecture.md#63-ceilings), [§3.2 Wire layout](../01-architecture.md#32-wire-layout)
- [02-constants.md](../02-constants.md): C-22, C-23, C-24, C-25, C-26, C-27, C-92, C-93
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-12, AC-13
- Related: [02-Channel-bundling.md](02-Channel-bundling.md), [08-Two-sided-admission.md](08-Two-sided-admission.md), [19-Six-channels-per-stream.md](19-Six-channels-per-stream.md)
