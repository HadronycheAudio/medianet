# ADR 02 — Channel bundling

## Status

Accepted

## Context

The product must absorb sources of up to 96 audio channels, and several such sources on one medianet. With the single-packet default (ADR 01) a stream carries 6 channels (C-04), so a wide source cannot be one stream.

Three mechanisms can carry a wide source: larger frames on the wire (jumbo), one-to-many delivery (multicast), or several narrow streams presented as one logical unit (a bundle). They answer different problems: frame size, fan-out, and width.

## Decision

**Streams stay narrow. A wide source emits a bundle of aligned streams, and a fan-in reassembles the bundle as one logical input.**

- A bundle is a group id, a member list and a channel map, declared in the module manifest. A 96-channel source is 16 members at defaults (C-22).
- **Members are independent on the wire.** Gaps, interrupted and dead are per member. The bundle is dead when all members are.
- **One offset for the whole bundle is not a choice.** Every aligned input of a consumer shares that consumer's firing time; that is what alignment by position means (ADR 05).
- **One admission handshake** lists the members wanted. A narrow sink subscribes to only the members it needs.
- **One socket per bundle** at the consumer, demultiplexed by stream number. Batch receive is required.
- **One receiver report per bundle** every 100 ms (C-40), with per-member counters.
- In-band announcements are re-issued in every member (C-44).
- Controllers ride in their own member.
- Bounds: at most 32 members per bundle (C-08) and 256 streams per source (C-09).

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Atomic bundle: a slice of the bundle is delivered only if all members arrived | Multiplies the weakest member's loss into the whole group: with 16 members the bundle's loss is about 16 times one member's |
| Jumbo frames as the mechanism for width | Depends on every NIC and switch on the path being configured for it; a 96-channel slice does not fit one jumbo frame either; path MTU is not interface MTU |
| Multi-packet slices as the mechanism for width | Makes reassembly the normal case and delivery atomic across many packets; kept only as an explicit regime (ADR 01, ADR 03) |
| Multicast streams | Solves fan-out, not width; deferred (ADR 20) |
| One socket per member | 16 sockets and 16 poll entries per bundle for no benefit, since admission and accounting are per bundle |
| One receiver report per member | Ten times sixteen reports per second per bundle for information that fits one message |

## Consequences

Positive

- Selective subscription: an embedded speaker takes one member out of 96 channels. This is the strongest argument for independent members.
- Loss in one member degrades only that member's channels.
- Alignment across members is sample-exact by position, with no extra mechanism.
- Width scales by adding members, without changing packet size or the wire format.

Negative

- "All content for a time stretch arrives together or not at all" now holds **per member**, not per source. A lost controller slice means hold-last-value while audio plays.
- Packet rate grows with width: 32 000 packets per second for a 96-channel source (C-22). Receive cost per packet becomes a capacity wall (C-93).
- Fan-out is still unicast, so a GbE port holds about 198 channel-copies (C-24).

Neutral

- Group-level operations (admission, session, channel map) need a group id in the control plane but nothing extra in media packets: the stream number suffices.
- Alignment buffer memory is about 370 KB per 16-member bundle at a 10 ms skew budget (C-27); not a constraint on hosts.

## Falsifier

- L1 (task 019): one Linux consumer on one GbE port cannot align two 96-channel bundles for one hour with zero late slices (C-92). Then either streams must get wider (larger packets) or batch receive is insufficient.
- A product requirement for source-level atomicity (all 96 channels or none) that hold-last-value and per-member gaps cannot satisfy.

## Links

- [01-architecture.md §7 Channel bundles](../01-architecture.md#7-channel-bundles), [§8 Capacity map](../01-architecture.md#8-capacity-map), [§4.2 Fan-in alignment](../01-architecture.md#42-fan-in-alignment)
- [02-constants.md](../02-constants.md): C-04, C-08, C-09, C-22, C-24, C-27, C-40, C-44, C-92, C-93
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-13, AC-16
- Related: [01-Single-packet-slice-default.md](01-Single-packet-slice-default.md), [05-Position-invariant-fan-in-alignment.md](05-Position-invariant-fan-in-alignment.md), [19-Six-channels-per-stream.md](19-Six-channels-per-stream.md), [20-Ten-GbE-for-wide-nodes.md](20-Ten-GbE-for-wide-nodes.md)
