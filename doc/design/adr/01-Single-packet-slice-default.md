# ADR 01 — Single-packet slice default

## Status

Accepted

## Context

A slice is the content of one stream for one stretch of media time: 48 frames, 0.5 ms, 2 000 slices per second at defaults (C-02).

The proof of concept sends one UDP datagram per slice regardless of size. At its default feed of 48 channels that datagram is about 9.6 KB, which the IP layer cuts into 7 fragments at MTU 1500 (P-01) `[code]`. Fragmentation was therefore the POC's default operating mode, and it was invisible to the program:

- Losing any one fragment loses the whole datagram, so datagram loss is roughly the fragment count times the link loss, and the link loss itself cannot be observed.
- The kernel reassembly queue has a long timeout and a memory ceiling shared by the whole host `[memory]`.
- A reassembled datagram carries one receive timestamp, so the time spent serialising the fragments is hidden inside the latency figure `[memory]`.
- Loopback interfaces have a very large MTU, so loopback tests never exercise the fragmented regime at all `[measured]`.

The runtime needs a default whose loss and latency figures mean what they say.

## Decision

- **By default a slice is exactly one packet at the path MTU** (C-05).
- **IP fragmentation is forbidden.** Every packet is sent with don't-fragment set.
- The default stream width is chosen so that this holds: 6 channels per stream (C-04). Wider sources use channel bundles (ADR 02).
- A slice larger than the path MTU is still possible, but only as an **explicit multi-packet regime** that the runtime packetises itself (ADR 03) and that is recorded in the regime stamp. The operator enters it knowingly.
- The path MTU is established per connection by a probe pair at admission, with a fallback of 1 280 bytes (C-06).

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Keep one datagram per slice and document IP fragmentation as a supported constraint | Leaves the runtime blind: loss is multiplied by the fragment count with no way to observe the underlying rate; the reassembly queue is a host-wide resource with a long timeout; nothing can be recovered or concealed per fragment |
| Keep the POC's 48-channel default feed and make multi-packet slices the normal case | The default configuration of the runtime should never need reassembly; every default-path measurement would then include a size-dependent serialisation floor |
| Per-channel streams | Maximises packet rate and system calls for no gain over a stream that fills one packet |
| Shorter slices to fit more channels per stream | Raises packet rate and wake-ups; rejected as a default, left to configuration |

## Consequences

Positive

- Packet loss equals slice loss on the default path; the loss figure is directly a link figure.
- One kernel receive timestamp per slice with no hidden serialisation time.
- No reassembly state, no reassembly deadline and no partial-slice handling on the default path.
- Loopback and wire tests run in the same regime.

Negative

- A wide source becomes many streams (16 for 96 channels), which raises packet rate to 32 000 packets per second per 96-channel source (C-22) and makes per-packet receive cost a capacity wall (C-93).
- "All content for a time stretch arrives together" now holds per stream, not per source (see ADR 02).

Neutral

- The multi-packet regime still exists and must be implemented and tested (reassembly deadline C-15), but it is no longer the common case.

## Falsifier

- Final header layout (task 004) leaves too little room: headers plus in-band content exceed the roughly 320 bytes left beside 1 152 bytes of audio in a 1 472-byte payload (C-04). The default width would then drop.
- L1 (task 019) shows hosts cannot keep up with the resulting packet rates (C-92 fails). The trade between packet count and packet size would be reopened.

## Links

- [01-architecture.md §3.1 Slices and packets](../01-architecture.md#31-slices-and-packets), [§3.3 Path MTU](../01-architecture.md#33-path-mtu), [§8 Capacity map](../01-architecture.md#8-capacity-map)
- [02-constants.md](../02-constants.md): C-02, C-04, C-05, C-06, C-15, C-22, C-92, C-93, P-01
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-13, AC-19, AC-24
- Related: [02-Channel-bundling.md](02-Channel-bundling.md), [03-Runtime-owned-packetisation.md](03-Runtime-owned-packetisation.md), [19-Six-channels-per-stream.md](19-Six-channels-per-stream.md)
