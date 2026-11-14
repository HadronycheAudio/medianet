# ADR 19 — Six channels per stream

## Status

Accepted

## Context

With the single-packet default (ADR 01), the number of channels a stream carries is fixed by arithmetic.

- The wire sample format is 4 bytes per sample, f32 or Q1.31, with no 3-byte variant (C-03).
- A slice is 48 frames (C-02), so one channel costs 192 bytes per slice including its channel-id byte (C-04).
- A 1 500-byte path leaves 1 472 bytes of UDP payload.

Seven channels use 1 351 bytes and leave 121 bytes. Six channels use 1 152 bytes and leave about 320 bytes (C-04). Those remaining bytes must hold the packet header, the slice header, the slice checksum, the optional measurement extension with its lineage fields, and any in-band announcements.

The choice also fixes the stream count of wide sources and therefore packet rates.

## Decision

**At defaults a stream carries 6 channels** (C-04): 1 152 bytes of audio per slice, about 320 bytes left for headers and in-band content.

Derived figures:

| Quantity | Value | Constant |
|---|---|---|
| Wire size of one default packet | about 1 320 bytes including all Ethernet overhead | C-20 |
| Wire rate per stream | about 21.1 Mbit/s, about 3.5 Mbit/s per channel | C-21 |
| 24-channel source | 4 streams, 84 Mbit/s, 8 000 packets/s | C-22 |
| 48-channel source | 8 streams, 169 Mbit/s, 16 000 packets/s | C-22 |
| 96-channel source | 16 streams, 338 Mbit/s, 32 000 packets/s | C-22 |

The wire sample format is closed at 4 bytes per sample (C-03); the density cost is priced into these figures.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| 7 channels per stream | Only 121 bytes remain for all headers and in-band content; too tight for header, checksum, lineage extension and announcements together |
| 3-byte samples to fit more channels | Rejected: one sample format on the wire; fixed-point processing uses the full 32 bits |
| Shorter slices to fit more channels per packet | Raises packet rate and wake-ups for every stream, including narrow ones |
| Wider streams in the multi-packet regime by default | Makes reassembly the normal case (ADR 01) |
| Jumbo frames by default | Depends on every hop; path MTU is not interface MTU |
| Derive channels per stream from the path MTU per connection | The fallback path MTU of 1 280 bytes (C-06) already forces a narrower stream on such a path; the default must be one predictable number for capacity planning |

## Consequences

Positive

- Clean stream counts for common widths: 4, 8 and 16 streams for 24, 48 and 96 channels.
- Room for the measurement extension and announcements in every packet without a second packet.
- Capacity planning has one number per channel: about 3.5 Mbit/s.

Negative

- 16 streams and 32 000 packets per second for a 96-channel source; two such bundles are 64 000 packets per second at one consumer (C-92). Six channels per packet is cheap on the wire and expensive in system calls.
- Header overhead is about 15 % over raw samples.
- On a path at the fallback MTU (C-06) six channels do not fit; such a connection carries fewer channels per stream or enters the multi-packet regime, and the stamp says which.

Neutral

- A 96-channel source is 338 Mbit/s on the wire; raw samples alone are most of that. The POC's often-quoted 150 Mbit/s was for 48 channels (P-01).

## Falsifier

- Spec drafting (task 004): the final header layout plus in-band content exceeds the roughly 320 bytes left (C-04), or changes the packet size by more than 5 % (C-20).
- L1 (task 019): per-packet receive cost makes the two-bundle test fail (C-92, C-93), which would argue for fewer, larger packets.

## Links

- [01-architecture.md §3.1 Slices and packets](../01-architecture.md#31-slices-and-packets), [§3.2 Wire layout](../01-architecture.md#32-wire-layout), [§8.1 Rates](../01-architecture.md#81-rates), [§7 Channel bundles](../01-architecture.md#7-channel-bundles)
- [02-constants.md](../02-constants.md): C-02, C-03, C-04, C-06, C-20, C-21, C-22, C-92, C-93, P-01
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-13
- Related: [01-Single-packet-slice-default.md](01-Single-packet-slice-default.md), [02-Channel-bundling.md](02-Channel-bundling.md), [20-Ten-GbE-for-wide-nodes.md](20-Ten-GbE-for-wide-nodes.md)
