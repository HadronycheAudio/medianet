# Wire specification

Field-level byte formats of the Hadronyche Medianets protocol. This file fixes the conventions every later ticket of epic 004 writes against — byte order, alignment, bit numbering, sample encodings, field-table format, layer overview and the constants this epic works from — and holds the headed sections that the later tickets fill. The conventions and the section headings are fixed by ticket 004-01; filling tickets write under the existing headings and do not renumber them `[decision]`.

## Scope and sources

This document specifies every message on the wire: media packets (packet header, slice header, in-band announcements, measurement extension, content messages, checksum) and control messages (hello, election, admission handshake with the manifest and the path-MTU probe pair, regime stamp sections and stamp request, commissioning window messages), the sync request and reply, and the measurement messages (receiver report, one-way probe), with exact offsets, widths, byte order, versioning, reserved fields and the required outcome of every malformed case. It lives in `doc/design/spec/` (C-130), beside the architecture specification ([01-architecture.md](../01-architecture.md)).

Numbers are the constants table's, not this file's: [02-constants.md](../02-constants.md) is the single source of truth and wins on any difference; the copy in [Constants](#constants) at the end of this file exists for the reader's convenience. Evidence tags are those of [00-context.md](../00-context.md); a figure cited from a published standard is tagged `[standard]` with the standard named and its clause (C-132).

## Byte order and alignment

### Byte order

Every multi-byte integer on the wire — every field of 2, 4, 8 or 16 bytes and every 4-byte sample — is little-endian: the least significant byte is sent first. The wire's byte order is little-endian, one order for all multi-byte integers, and no message and no field is excepted `[decision]`.

Chosen from the protocol's own needs `[decision]`. The two implementations share one `no_std` core crate and must parse without allocation and with bounded parsing time, so the cheapest correct parsers are the ones that move integers between wire and memory without byte swaps. Every platform the design names is little-endian: the AMD64 ABI fixes little-endian storage in its data-representation clause — System V Application Binary Interface, AMD64 Architecture Processor Supplement, § 3.1.2 Data Representation (Fundamental Types): `__int128` and `_BitInt(N)` are stored in little-endian order in memory `[standard]` — and the AArch64 call standard defines the little-endian memory view used here — Procedure Call Standard for the Arm 64-bit Architecture, § 5.9 Byte order (Endianness): the least significant byte of a data object is at the lowest byte address the object occupies in memory `[standard]`. That the 64-bit ARM machines of the pair (C-151) in fact use that little-endian view is recalled, clause not yet verified: the AArch64 default byte order is fixed by the Arm Architecture Reference Manual, which needs an account to fetch; ticket 004-22 pins the clause before anything depends on it `[memory]`. With every platform the design names little-endian, a little-endian field is exactly the core crate's in-memory integer on every supported target. The media layer dominates the byte stream — up to 338 Mbit/s per source (C-22) — and its bytes are almost entirely 4-byte samples (C-03, C-04), so a per-sample swap on encode and decode would be avoidable work on the data-plane hot path; this order avoids it everywhere. The retired POC used the same order, but that is coincidence, not the argument: its 88-byte header is retired `[code]`, and this decision stands on the reasons above.

### Alignment and padding

The format is packed: a field sits at exactly the offset its field table states, and no padding is inserted anywhere for natural alignment `[decision]`. The guarantees per field width are:

| Field width | Alignment guarantee |
|---|---|
| 1 byte | Byte alignment holds by construction: every whole-byte field starts at a whole-byte offset |
| 2 bytes | No natural alignment is guaranteed: a 2-byte field may start at any byte offset its table states |
| 4 bytes | No natural alignment is guaranteed: 4-byte fields and samples (C-03) start at their stated offsets, even or odd |
| 8 bytes | No natural alignment is guaranteed: 8-byte fields such as the module id and instance nonce (C-11) start at their stated offsets |
| 16 bytes | No natural alignment is guaranteed: 16-byte fields such as the 128-bit tag reservation (C-12) start at their stated offsets |
| sub-byte bit fields | No alignment beyond the containing byte: a bit field occupies stated bits of one stated byte |

A reader therefore never assumes a 2-byte field is at an even offset or a 4-byte field at a 4-byte offset: implementations read fields byte-wise or with unaligned-safe instructions on targets that trap on unaligned access, and a received buffer is never cast to an aligned struct `[decision]`. A writer inserts no padding; reserved space is a reserved field (004-05), not alignment filler.

### Signed integers

Every signed integer on the wire is two's complement, in the width its field table states; there is no sign-magnitude, biased or offset encoding `[decision]`. The Q1.31 sample format below uses the same representation, so integers and samples share one signed encoding.

### Bit-field numbering

Bits within a byte are numbered 7 down to 0, bit 7 being the most significant. A sub-byte field is written bits [hi:lo] of a named byte, and bit hi is the field's most significant bit. Sub-byte fields never straddle a byte boundary; anything wider than 8 bits is a multi-byte field read in the little-endian order above `[decision]`.

### Bit-level encoding of samples

Samples are 4 bytes each, f32 or Q1.31; there is no 3-byte variant (C-03) `[decision]`. There is one sample rate on the wire, 96 000 Hz: every stream runs at the domain rate, one timeline per domain, so no rate field exists on the wire (C-01) `[decision]`. The two encodings below are fixed bit for bit; which of them a stream uses is declared where the manifest tickets fix it (004-26), not on every packet.

#### f32 samples

An f32 sample is 4 bytes (32 bits). Read as a little-endian 32-bit word, the bits are an IEEE 754 binary32 — the 32-bit basic format of IEEE 754-2008, clause 3 (floating-point data) `[standard]` — with bit 31 the sign, bits 30..23 the exponent (bias 127) and bits 22..0 the mantissa (fraction). A sender emits the binary32 encoding of the sample's value and a receiver recovers that value directly. The format never normalises a sample's bits: sign, exponent, mantissa and any NaN payload are carried exactly as produced, so the encoding is bit-exact end to end `[decision]`.

#### Q1.31 samples

A Q1.31 sample is 4 bytes (32 bits): a two's complement fixed-point value with 1 sign bit and 31 fractional bits. The little-endian 32-bit word N represents the value N × 2^-31; the fractional scale is one least significant bit = 2^-31. The representable range is [−1, +1): 0x80000000 is −1 exactly, 0x00000000 is zero and 0x7fffffff is 1 − 2^-31, the largest value below +1, which is itself not representable `[decision]`.

## Field-table format

Every message and header in this specification is described by one field table with exactly these five columns `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| 0 | 2 | u16 | *Illustrative row, not a real field* — the real tables arrive with 004-06 onward | 0 to 65535 |

- **Offset** — byte distance from the first byte of the message (the first byte of the UDP payload), counted from 0.
- **Width** — size in whole bytes; a sub-byte field states its bits and its containing byte, as Bit-field numbering fixes.
- **Type** — the primitive: u8/u16/u32/u64/u128, i8 to i128, f32, Q1.31, or a byte blob of stated elements.
- **Meaning** — what the field carries.
- **Valid range** — the values a v1 receiver accepts; any other value is a malformed case, handled under Rejection reasons (004-03) and the section's own validity rules.

Rules that every later section inherits `[decision]`:

1. Every field has a fixed width at a fixed offset; nothing on the wire is implicit.
2. Every variable-length part — a list, a string, a byte blob — is a fixed-width count field followed by exactly that many elements, and the count's Valid range states a fixed maximum count. Every message therefore has a fixed maximum size, and a parser bounds its work before it starts, as the no-allocation, bounded-parsing requirement demands.
3. A string is a byte count followed by that many bytes, with the maximum stated in its Valid range; the byte encoding of each string field is fixed by the ticket that owns the field.
4. No alignment padding exists (Alignment and padding); reserved space is a reserved field with the values 004-05 fixes.
5. Control messages fit the 1 252-byte UDP-payload bound (C-17); the path-MTU probe pair is the only exemption and is sent at the intended media packet size (C-116).

## Layer overview

The protocol's messages travel on four layers:

| Layer | Carries | Transport | DSCP (C-13) |
|---|---|---|---|
| Discovery and control | hello, election, admission handshake with the manifest and the path-MTU probe pair, regime stamp sections and the stamp request | UDP over IPv4 (C-100): multicast with TTL 1, and unicast | CS5 (40) |
| Clock sync | request / reply exchanges with the holder | UDP unicast | CS6 (48) |
| Media | slices and in-band announcements | UDP unicast, one packet per slice by default | EF (46) |
| Measurement | receiver reports; one-way probes | UDP unicast, on the reverse port given in the handshake | reports CS5 (40); probes EF (46) |

- IPv4 only in v1; discovery is multicast with TTL 1 (C-100) `[decision]`. A 1 500-byte path MTU leaves 1 472 bytes of UDP payload — the no-options IPv4 header is 20 bytes (RFC 791, § 3.1 Internet Header Format, IHL: the minimum correct header is 5 words of 32 bits) and the UDP header is 8 bytes (RFC 768, Format and Fields, Length: the minimum datagram is 8 octets including the header) `[standard]` — and the 1 280-byte fallback (C-06) leaves 1 252 bytes of UDP payload (C-17) `[decision]`.
- The DSCP assignment is C-13's `[decision]`: control CS5 (40), clock sync CS6 (48), media and one-way probes EF (46), measurement reports CS5 (40) with probes EF (46); the class-selector codepoints are the 'xxx000' set (RFC 2474, § 4.2.2.1 The Class Selector Codepoints) and EF is codepoint 101110 (RFC 3246, § 2.7 Recommended codepoint for this PHB) `[standard]`.
- Every message defined below belongs to exactly one layer; the per-message sections follow the layer order of the table.

## Rejection reasons

To be filled by 004-03 (rejection-reason enumeration: the closed set of reject reasons and the counting key for traffic that matches no admitted input).

## Versioning

To be filled by 004-04 (versioning rules, magic and version fields).

## Reserved fields

To be filled by 004-05 (reserved fields and their required values, including the copy index C-07 and the authentication reservation C-12 with the v1 zeroing of scheme and tag, C-115).

## Media packet header

To be filled by 004-06 (media packet header layout).

## Slice header

To be filled by 004-08 (slice header and send-timestamp format).

## Announcements

To be filled by 004-09 (in-band announcement layouts, including the re-alignment and transport-reference-change payloads, C-208).

## Measurement extension

To be filled by 004-10 (media measurement extension layout).

## Content messages

To be filled by 004-11 (content messages: audio and dense controllers) and 004-12 (content messages: sparse keyed controllers and transport state).

## Checksum

To be filled by 004-13 (slice checksum algorithm, width and placement).

## Size budget

To be filled by 004-14 (media packet size budget at 1 472 and 1 280 bytes, with the 100-byte header cap of C-137).

## Hello

To be filled by 004-16 (hello message layout and 32-member size bound).

## Election messages

To be filled by 004-17 (election message layouts, updated for C-213).

## Admission messages

To be filled by 004-18 (admission handshake messages and refusal reasons).

## Manifest

To be filled by 004-26 (manifest encoding in the admission handshake).

## Path-MTU probe pair

To be filled by 004-27 (path-MTU probe pair, exempt from C-17 per C-116).

## Regime stamp

To be filled by 004-24 (regime stamp message: sections, version and fetch request), 004-25 (regime stamp section types named by the constants table) and 004-28 (regime stamp sections added after the split).

## Commissioning window messages

To be filled by 004-29 (commissioning window announcement and close messages, C-153).

## Sync request and reply

To be filled by 004-19 (sync request and reply layouts).

## Receiver report

To be filled by 004-20 (receiver report layout, including the consumer's current A, C-198).

## One-way probe

To be filled by 004-20 (one-way probe layout).

## Validity rules

To be filled by 004-21 (validity rules completeness audit, consolidating the per-message rules of 004-06 to 004-20 and 004-24 to 004-29).

## Traceability table

To be filled by 004-22 (traceability table and review against the architecture specification).

## Constants

The values below are copied from [02-constants.md](../02-constants.md). The constants table is the single source of truth: if a value here differs from the table, the table wins and this copy is a bug. Later sections reference these constants; they never redefine them `[decision]`.

| ID | Constant | Value |
|---|---|---|
| C-01 | Domain sample rate | 96 000 Hz, and it is the only rate on the wire: every stream on a medianet is at the domain rate. An edge whose hardware runs at another rate (typically 48 kHz) converts at the edge |
| C-02 | Slice length (default) | 48 frames = 0.5 ms = 2 000 slices/s |
| C-03 | Wire sample format | 4 bytes per sample, f32 or Q1.31; no 3-byte variant |
| C-04 | Channels per stream at defaults | 6 (192 bytes per channel per slice, 1 152 bytes of audio; no per-channel id byte, the channel map is in the manifest) |
| C-05 | Default slice size | Fits one packet at the path MTU; multi-packet slices are an explicit, stamped regime |
| C-06 | Path-MTU fallback | 1 280 bytes |
| C-07 | Copy index | 2 bits in the packet header |
| C-08 | Members per bundle | at most 32, the controller member included (C-209) |
| C-09 | Streams per source | at most 256 |
| C-10 | Stream number / session / timeline epoch widths | 16 / 32 / 16 bits |
| C-11 | Module id / instance nonce / domain id | 64 / 64 / 16 bits; domain 0 = unconfigured |
| C-12 | Authentication reservation in control messages | scheme 8 bits (0 = none) + message counter 64 bits + tag 128 bits |
| C-13 | DSCP classes | sync CS6 (48); media and one-way probes EF (46); control CS5 (40) |
| C-17 | Control message size bound | must fit one packet at the fallback MTU: at most 1 252 bytes of UDP payload |
| C-18 | Regime stamp transport | a dedicated control message split into sections that each fit C-17. Multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch. The hello carries only a stamp version number |
| C-19 | Session number ownership | assigned by the source, one per stream; bumped on restart and on timeline epoch change; every consumer of a stream sees the same value |
| C-20 | Wire size of one default packet | about 1 320 bytes including all Ethernet overhead |
| C-21 | Wire rate per stream | about 21.1 Mbit/s (about 3.5 Mbit/s per channel) |
| C-22 | Wire rate per source | 24 ch: 4 streams, 84 Mbit/s, 8 000 packets/s. 48 ch: 8 streams, 169 Mbit/s, 16 000 packets/s. 96 ch: 16 streams, 338 Mbit/s, 32 000 packets/s |
| C-34 | Active modules per medianet | at most 64; commissioning refuses more |
| C-40 | Receiver report | one per bundle every 100 ms |
| C-43 | In-band announcement lead | 64 slices (32 ms at defaults), flat |
| C-44 | Announcement re-issue | every 16 slices until the named position passes, in every bundle member |
| C-100 | IP version | IPv4 only in v1; multicast with TTL 1. The 1 280-byte fallback (C-06) is a conservative path MTU giving 1 252 bytes of UDP payload |
| C-137 | Header cap | a default slice must fit 1 252 bytes of UDP payload: 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum. In-band announcements and the measurement extension fit inside that cap or are omitted on a fallback path |
