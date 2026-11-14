# ADR 03 — Runtime-owned packetisation

## Status

Accepted

## Context

The POC hands each slice to the kernel as one datagram and lets IP split it (P-01) `[code]`. Even with a single-packet default (ADR 01), the runtime must decide who is responsible for the mapping between a slice (what a handler sees) and packets (what the wire carries), for the cases where a slice exceeds the path MTU and for every case where the wire format itself matters: identity, duplication, gaps, measurement.

The POC wire format has further problems `[code]`:

- Header fields are overloaded by direction (one field is an acknowledged sequence one way and a module id the other).
- Forward packets carry per-receiver state (the acknowledgement of that receiver's last echo).
- Timing measurements live in the stream header.
- A protocol version mismatch is a silent drop, so a mixed-version network simply looks dead.

## Decision

**The runtime owns packetisation and reassembly. The handler sees a slice whole or not at all.**

- Every packet is bounded to the path MTU and sent with don't-fragment set.
- A slice larger than the path MTU is split by the runtime into ordered packets, each carrying the slice identity, its index, the packet count and its own send timestamp. The reassembly deadline for an arrival-driven multi-packet input is 2 slice periods (C-15); for aligned inputs it is the consumer's deadline.
- **Streams are identified by source and stream number** (16 bits, C-10), never by destination. The receiver demultiplexes on the source address pinned at admission plus the stream number.
- **Forward packets carry no per-receiver state.** Round-trip closure moves to the measurement path.
- The 64-bit module id (C-11) stays in the control plane; media packets carry domain, stream number, session and timeline epoch (C-10, C-11).
- A **copy index** of 2 bits (C-07) is reserved; dedup is by stream, position and packet index, first valid copy wins.
- A **gap slice** (header only) and a **degraded flag** are part of the format.
- One **checksum over the slice**.
- Timing measurements live in an **optional measurement extension**, not in the stream header.
- A version mismatch is a named event.
- Path MTU is found by a **probe pair** at admission (padded don't-fragment packet plus acknowledgement); fallback 1 280 bytes (C-06); re-probe when large packets vanish while liveness holds.

Exact field offsets are spec drafting (task 004).

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| IP fragmentation, documented | No visibility of fragment loss, host-wide reassembly queue with a long timeout, one timestamp for the whole datagram |
| Per-channel or per-stream messages with no slice unit | Breaks the property event semantics rest on: content that "takes effect over the audio of the slice that brought it" needs audio and events to be one unit |
| Destination-addressed streams | Incompatible with later multicast or duplication; the receiver needs a source filter anyway |
| Per-receiver acknowledgement in forward packets (POC) | Incompatible with one-to-many delivery; mixes measurement with content |
| Per-slice parity packet reserved in the format | Withdrawn: with single-packet slices per-slice parity is duplication (see ADR 04) |
| Path MTU from the interface MTU, or from a send error alone | A too-small hop on a layer-2 path drops silently and returns no error; silence is only an answer if an acknowledgement was expected |

## Consequences

Positive

- Packet loss and slice loss are both observable and reported separately in the multi-packet regime.
- Every packet has its own receive timestamp, so first-packet and slice-complete arrival are both known and the serialisation floor becomes a measured difference.
- The format is ready for duplication and one-to-many delivery without a wire break.
- Production streams can omit measurement fields.

Negative

- The runtime must implement and test reassembly, deadlines and dedup.
- Removing per-receiver state from forward packets removes the POC's in-band round-trip closure; a separate unicast measurement path is needed (receiver reports, one-way probes).

Neutral

- "Travels together" moves from a property of the wire to a guarantee of the API.

## Falsifier

- Spec drafting (task 004) shows the header set does not fit beside 6 channels of audio (C-04).
- L1 (task 019), multi-packet regime: first-packet and slice-complete latency do not differ by the serialisation time, which would mean the per-packet stamps are not what they are claimed to be.
- The probe pair fails to detect a real size-limited hop on a physical switch (AC-19).

## Links

- [01-architecture.md §3.1 Slices and packets](../01-architecture.md#31-slices-and-packets), [§3.2 Wire layout](../01-architecture.md#32-wire-layout), [§3.3 Path MTU](../01-architecture.md#33-path-mtu), [§3.6 Measurement path](../01-architecture.md#36-measurement-path)
- [02-constants.md](../02-constants.md): C-04, C-06, C-07, C-10, C-11, C-15, P-01
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-1, AC-19, AC-23, AC-24
- Related: [01-Single-packet-slice-default.md](01-Single-packet-slice-default.md), [04-Detect-and-conceal-redundancy.md](04-Detect-and-conceal-redundancy.md), [24-Numbers-governance.md](24-Numbers-governance.md)
