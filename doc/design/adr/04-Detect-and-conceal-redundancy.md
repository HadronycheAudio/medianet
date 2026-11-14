# ADR 04 — Detect-and-conceal redundancy

## Status

Accepted. Duplication (second path, or delayed on the same path) is Deferred-with-reservation.

## Context

Media travels over UDP. Every hop adds its own loss, so a path of depth five has roughly five times the loss of one edge, and each gap marks everything downstream as degraded. The design needs a stated answer to loss.

The premises on a medianet differ from a general LAN. The network is wired, isolated and carries only admitted traffic; slices are single packets; egress is paced. Under those premises steady-state loss does not approach zero, it approaches the bit-error floor: at a bit error rate of 1e-12, about 1.1e-8 per 1 500-byte packet, one loss per stream per link about every 13 hours at 2 000 packets per second (C-90) `[memory]`. What remains above that floor is faults (cable, port, NIC) and host drops.

## Decision

**v1 detects, classifies, conceals and accounts. It does not recover.**

- Detection and classification: the four gap classes at the consumer's deadline (marked, late, missing, dead), with the degraded flag propagating downstream.
- Concealment: runtime defaults per content type (audio zero-fill, dense controllers hold last value, sparse keyed controllers repaired by state refresh), overridable per connection.
- Accounting: the per-socket overflow counter `[kernel-doc]`, interface drop counters, and measurement-buffer overflow are in every regime stamp and receiver report.
- **The expected loss floor is published (C-90) and an alarm is raised above the threshold measured on a clean network (C-91).** Loss rate becomes a cable-health diagnostic with a physical threshold.
- **Reservation:** a 2-bit copy index in the packet header (C-07). Dedup rule: by stream, position and packet index; first valid copy wins. The same field later serves disjoint-path duplication and same-path delayed duplication.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Bounded retransmission (ARQ) | Puts per-receiver state and inbound requests on the source's real-time path; emits traffic outside the admitted rate; resends over the same path that just failed. (Not rejected on latency: on a wired LAN recovery would cost only two or three slice periods) |
| Forward error correction | With single-packet slices, per-slice parity is duplication; parity across slices costs latency |
| Per-slice parity hook reserved in the format | Withdrawn for the same reason; the copy index subsumes it |
| Dual-path duplication in v1 | The only strategy that protects against faults rather than loss processes, at zero latency cost, but it needs clock sync and gossip on both networks and a consumer offset that covers the slower path; deferred with the reservation above |
| Received-positions bitmap so a replica applies the same gaps | Deferred; the exact tier excludes windows touched by loss instead |

## Consequences

Positive

- No per-receiver state at sources; all traffic stays within admitted rates.
- Zero added latency.
- Loss becomes a measured, attributable quantity with a physical reference.
- Duplication can be added later without a wire break.

Negative

- Every residual loss is audible unless concealed: zero-fill can click. Click-free behaviour is guaranteed only for announced transitions.
- "Reliable" in v1 means reliable detection and accounting, not delivery.
- Loss still compounds with graph depth, even if from a very small base.

Neutral

- When dual path is built, two costs are already known: sync and gossip must run on both networks, and the consumer's offset must cover the slower path or failover arrives late.

## Falsifier

- L1 (task 019): loss on a clean medianet well above the floor (C-90), in particular host or NIC drops at 32 000 to 64 000 packets per second (C-92). Then detect-and-conceal is not enough and redundancy returns to v1.
- The pre-implementation check of the published bit-error figure (C-90) differs materially from the value assumed.

## Links

- [01-architecture.md §3.7 Redundancy](../01-architecture.md#37-redundancy), [§4.6 Gap contract](../01-architecture.md#46-gap-contract), [§11.4 Loss floor](../01-architecture.md#114-loss-floor), [§16 Verification tiers](../01-architecture.md#16-verification-tiers)
- [02-constants.md](../02-constants.md): C-07, C-90, C-91, C-92
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-15, AC-16, AC-24, AC-37
- Related: [03-Runtime-owned-packetisation.md](03-Runtime-owned-packetisation.md), [13-Interrupted-and-dead-thresholds.md](13-Interrupted-and-dead-thresholds.md), [24-Numbers-governance.md](24-Numbers-governance.md)
