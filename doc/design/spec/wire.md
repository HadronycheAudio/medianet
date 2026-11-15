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

Every malformed input has one required outcome: the packet is dropped and counted under a named reason of the closed `Reject` enumeration below, and nothing is dropped silently `[decision]`. One reason additionally raises a named event: `VersionMismatch` — a version mismatch is never a silent drop — which replaces the retired POC's `Header::decode`, which returned `None` silently on a version mismatch `[code]`. System criterion AC-23 requires exactly this for malformed, wrong-version and foreign-domain traffic: no crash, no state change, and every rejected packet counted per input and per reason. Each reason below is decidable from the packet's own bytes in a single pass, linear in packet length, inside the 10 µs-per-packet bound on processing one bad input (C-164) `[decision]`.

The enumeration — one row per variant, with its `u8` value, its meaning, and the layers whose parsers can produce it `[decision]`:

| Value (u8) | Name | Meaning | Layers |
|---|---|---|---|
| 0 | TooShort | fewer bytes than the message's fixed minimum: dropped before any field is read | control, sync, media, measurement |
| 1 | BadMagic | the leading bytes are not the magic of any message of this major version; the magic is fixed by 004-04 | control, sync, media, measurement |
| 2 | VersionMismatch | the version field is not the version of this major version: counted like every rejection and also raises a named event (never a silent drop); the version travels in the hello, so the mismatch is found before a connection exists | control, sync, media, measurement |
| 3 | ForeignDomain | the domain id is a non-zero domain other than this module's (C-11): dropped and counted | control, sync, media, measurement |
| 4 | UnconfiguredDomain | the domain id is 0, the unconfigured domain (C-11): the packet is ignored and counted, not refused, and is never a `ForeignDomain` (C-173) | control, sync, media, measurement |
| 5 | UnknownStream | the stream number names no stream this consumer admits | media |
| 6 | UnadmittedSource | the source's module id has no admitted connection at this consumer | media |
| 7 | BadLength | a declared length or count disagrees with the message's own size rules: a count above its fixed maximum, or a body that does not fill the fixed size | control, sync, media, measurement |
| 8 | BadChecksum | the checksum does not match the recomputed value; the checksum arrives with 004-13 | media |
| 9 | BadPacketIndex | the packet index lies outside the packet count its slice declares | media |
| 10 | WrongEpoch | the timeline epoch is not the consumer's current epoch (C-10): counted under this reason — the per-connection other-epoch counter (C-135, C-159) — and never a miss (C-102) | media |
| 11 | OversizeControl | a control message exceeds the 1 252-byte UDP-payload bound (C-17) | control |
| 12 | ReservedNonZero | a field this specification fixes at zero carries a non-zero value; the fields and their required zeros are fixed by 004-05 | control, sync, media, measurement |

`VersionMismatch` is the only reason that raises a named event, in addition to being counted; every other reason only counts `[decision]`. The event is what stops a wrong-version module from joining: it is raised at detection, which happens on the hello, before any connection exists.

### Counting model

Every rejection increments one counter, keyed by (input, reason) `[decision]`. An input is the design's input: an admitted stream at a consumer, the unit of the bounded drain (C-16), and a unit that can be Demoted (C-109). A packet that matches an admitted input is counted on that input, and the reason names the other half of the key, so the per-input, per-reason counting AC-23 requires is one counter per (input, reason) pair.

Traffic that matches no admitted input — foreign-domain, unconfigured-domain, bad-magic, unknown-stream and unadmitted-source packets — still needs a key, and this ticket fixes it `[decision]`: such traffic is counted per socket, on the receiving socket of the layer it arrived on. Per socket is chosen over per interface and per source address because the socket set is bounded — a module opens at most one receiving socket per layer, and the layer overview names four layers, so at most 4 — while an interface key cannot say which layer's socket received the junk, and a source-address key is unbounded (any number of spoofed senders) and would break the fixed bound below. The no-match key is therefore (socket, reason).

The counter set has a fixed numeric bound: at most 213 044 counters per module — at most 16 384 inputs (at most 64 modules, C-34, each with at most 256 streams, C-09) times 13 reasons, plus at most 4 sockets times 13 reasons `[decision]`.

### Closed enumeration

The enumeration is closed within a major version: no variant is added, renamed or renumbered by a minor version or a patch, and every wire value outside 0..=12 is unused. Values 13..=255 are reserved for a future major version: they stay unused — no packet, counter or event carries them — until a major version bump assigns them, so the names and values above are stable across every minor version `[decision]`.

The refusal reasons of C-185 — the closed set of 18 named there, for example `RequiredEExceeded` (C-103) — are a separate enumeration owned by ticket 004-18: they are outcomes of discovery or of the admission handshake, not packet rejections. `VersionMismatch` and `ForeignDomain` are names in both; the two enumerations keep separate numeric values, and a refusal never increments a rejection counter `[decision]`.

Changes from the starting shape: all twelve starting names are kept; the one change is an addition — `UnconfiguredDomain`, value 4, inserted after `ForeignDomain` so the two domain-related rejection reasons sit together — which shifts the eight later starting names up by one: UnknownStream 4 to 5, UnadmittedSource 5 to 6, BadLength 6 to 7, BadChecksum 7 to 8, BadPacketIndex 8 to 9, WrongEpoch 9 to 10, OversizeControl 10 to 11, ReservedNonZero 11 to 12; the first four names keep 0 to 3. `UnconfiguredDomain` is needed because a hello with domain 0 is ignored and counted, not refused, and so needs its own reason rather than `ForeignDomain` (C-173, C-11) `[decision]`. The crate and module that define the Rust `Reject` type are chosen by the implementing tasks 009 to 014; this specification fixes only the names and the values.

## Versioning

Every message — media, control, sync and measurement — starts with the same two leading fields, a magic and a version, and the version also travels in the hello, so a peer's version is known before any connection exists [decision]. This section fixes the two fields and their offsets, the version scheme, what a minor and a major change are, and what a receiver does with every case; the media packet header's remaining fields arrive with 004-06 and the hello's layout with 004-16.

### The magic and the version fields

Every message of this protocol starts with the 2-byte magic 0x4d4e, the ASCII letters M and N, a value a receiver compares without knowing anything else about the packet [decision]. The retired POC used the 16-bit magic 0x4844 ('HD'), which v1 does not reuse; the two values differ in both bytes, so no packet of the old protocol satisfies this magic and no v1 packet satisfies the old one [code].

The magic is followed immediately by the version, a u16 (2 bytes) whose major is the high byte, bits 15..8, and whose minor is the low byte, bits 7..0 [decision]. The first version is v1: 1.0, packed in the u16 field as the value 0x0100 [decision]. A receiver reads the major first and treats it as the whole compatibility question: the major says which wire layout follows, the minor says which optional additions that layout may carry [decision].

The magic and the version sit at fixed offsets — the magic at offset 0 and the version at offset 2 — that no future version may move: offsets 0 and 2 are fixed for every later version, whatever else changes [decision].

The magic and the version are checked before any other field is read — no length, no type, no other constant — so those leading bytes classify every message, whatever its version [decision]. A future major version may therefore change every field after the version, because no parser of this protocol reads those fields to classify the packet [decision]. This is the ticket's falsifier answered: a receiver can tell a POC packet or a future-major packet from a v1 packet on the magic and the version alone, before it reads any other field — the old magic differs in both bytes and a different major sits in the version's high byte, so no packet of another version is mistaken for v1 [decision].

### Minor and major changes

A minor change adds to the wire without moving or reinterpreting anything an existing receiver already parses; it never changes an existing field's offset, width or meaning, and an older receiver either knows the new part or can skip it by its length [decision]. A minor change adds, for example:

- a new optional in-band announcement type (announcements are optional and are skipped by length by a consumer that does not know them, C-43);
- a new content message kind for a new controller or transport-state payload (004-11, 004-12), skipped by its length by a receiver that does not know it;
- a new regime stamp section type (C-18): the regime stamp is extended by other epics as they stamp new state, and a consumer that does not know a section type skips the section by its length [decision];
- a new optional measurement-extension entry (the extension fits inside the header cap or is omitted, C-137).

A major change is any change to an existing field's offset, width or meaning; every major change breaks every receiver of the previous major, because the old parser can no longer find or interpret the fields it knows [decision]. A major change is, for example:

- moving an existing field: the stream number (C-10) leaving its offset would shift every later field for a receiver of the previous major;
- widening an existing field: the session number (C-10) growing from 32 bits to 64 would make a previous parser read a different value;
- changing an existing field's meaning: the 2-bit copy index (C-07) reinterpreted as a hop limit would make a previous parser apply the wrong rules.

The ticket's example list names one more minor candidate, a new reason value in a reserved range; this section does not adopt it, because the rejection enumeration is closed within a major version and its values 13..=255 are reserved for a future major version (Rejection reasons, Closed enumeration), so a new rejection reason arrives only with a major version bump [decision].

### Receiver behaviour

A receiver reads the magic and the version first and classifies every packet on them alone; the five cases below cover every classification, and each row's outcome is the required one [decision]. The same rules apply at every layer, because every message starts with the same two fields [decision].

| Case | Outcome |
|---|---|
| same version | accept: parse the message normally |
| same major, higher minor | accept: parse the known parts, and skip any unknown optional part by its length |
| same major, lower minor | accept: an older minor's packet uses only parts this receiver already knows |
| different major | drop, count under VersionMismatch, and raise the VersionMismatch event (AC-23: a version mismatch is never a silent drop) |
| bad magic | drop and count under BadMagic |

The lower-minor row is this section's choice: a minor change only adds optional parts, so a packet of an older minor carries nothing a newer receiver misreads, and accepting it is the correct outcome [decision]. The hello carries the protocol version as well, so the check runs at the first message of discovery: a hello whose major differs raises the VersionMismatch event before any connection exists, and a wrong-version module is never admitted to a bundle on a silent path (AC-23) [decision].

## Reserved fields

A reserved field is wire space of a fixed width that this specification fixes at a required value, so a later version can give the space a meaning without moving any other field: reserved space is a named field with a required value, never alignment filler (Alignment and padding). Two reserved fields are fixed now — the copy index of the media packet header (C-07) and the authentication reservation of the control messages (C-12) — and the general rule below covers every padding or reserved field a later ticket adds. A reserved value that does not conform is a malformed case like any other: the receiver drops the packet and counts it under `ReservedNonZero` (Rejection reasons, value 12), the reason that enumeration reserves for exactly this case `[decision]`.

| Field | Location | Width | Required value in v1 | Receiver outcome |
|---|---|---|---|---|
| Copy index (C-07) | media packet header (004-06 places it) | 2 bits | 0 — v1 senders emit 0 only | v1 receivers accept all four values 0 to 3 [decision]; no copy-index value is a malformed case in v1 |
| Authentication scheme (C-12) | control messages (004-16 to 004-18 place it) | 8 bits | 0 (none) | a non-zero scheme: drop and count as ReservedNonZero |
| Message counter (C-12) | control messages | 64 bits | live — the liveness counter (C-115); any value | never ReservedNonZero: the counter is not a reserved zero field |
| Authentication tag (C-12) | control messages | 128 bits | 128 zero bits | a non-zero tag: drop and count as ReservedNonZero |

### Copy index (C-07)

The copy index is 2 bits in the media packet header (C-07), placed by 004-06. It is wire space kept for the deferred duplication feature — retransmitting a slice as numbered copies — not a flag, not a count and not alignment filler `[decision]`.

Required value in v1: senders emit only the value 0. The three non-zero values stay unused until duplication assigns them a meaning `[decision]`.

Receiver rule `[decision]`: v1 receivers accept all four copy-index values, 0 to 3. Accepting every value is the deliberate choice: the copy index exists so a later version can send duplicated copies without a major version, and a v1 receiver that rejected non-zero values would force exactly that bump; no copy-index value is a malformed case in v1.

Dedup rule (media rule 9, restated): copies are deduplicated by stream, position and packet index — the first valid copy wins; there is no separate sequence number, and nothing about a copy's content depends on its index `[decision]`.

When duplication arrives — a later version retransmitting a slice as copies 1 to 3 — v1 receivers already accept all four copy-index values, so the feature ships as a minor version with no new wire value: no value that version needs is rejected today, and no major version is forced `[decision]`.

### Authentication reservation (C-12)

Every control message carries the authentication reservation (C-12): scheme 8 bits (0 = none) + message counter 64 bits + tag 128 bits — 200 bits in all — placed by 004-16 to 004-18. The reservation is wire space for an authentication scheme a later version may define; v1 defines none `[decision]`.

In v1 (C-115) the scheme is 0 and the tag is 128 zero bits, while the 64-bit message counter is live: it is the liveness counter, it increments with every control message a source sends, and it carries any value — it is not a reserved zero field, so no rule of this section applies to it `[decision]`.

Receiver outcomes: a scheme other than 0 is a malformed case — the receiver drops the packet and counts it under ReservedNonZero (12); a non-zero tag arriving with scheme 0 is likewise dropped and counted under ReservedNonZero. The counter never produces ReservedNonZero: it is not fixed at zero `[decision]`.

### General rule for later tickets

Any padding or reserved bits a later ticket adds to any message follow one rule: the sender sets them to 0 — every reserved bit is written zero on transmit, whatever meaning a later version gives it — and a receiver that sees a non-zero value drops the packet and counts it under ReservedNonZero, exactly as for the named reserved fields above. No ticket may give a reserved field a non-zero v1 value; assigning one a meaning is a version change, never a silent reinterpretation `[decision]`.

## Media packet header

A media packet carries the content of one stream for one stretch of media time — the slice content, one packet per slice by default — and travels by UDP unicast at DSCP EF 46 (C-13) `[decision]`. Its header is 16 bytes (128 bits) long and counts against the header cap of C-137 (a default slice fits 1 252 bytes of UDP payload: 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum), leaving 84 bytes of the cap for the slice header, the in-band announcements, the measurement extension and the checksum that 004-08, 004-09, 004-10 and 004-13 place after it; those tickets extend this message and grow its declared maximum size, and this section's table does not move [decision]. The header carries no sample-rate field: the wire has one rate, the domain's 96 000 Hz (C-01), so the rate is a property of the domain, not of a packet [decision].

### Field table

Little-endian (Byte order), packed with no padding (Alignment and padding), bits numbered 7..0 within a byte (Bit-field numbering); offsets are byte distances from the first byte of the UDP payload [decision]:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| 0 | 2 | u16 | Magic 0x4d4e, the ASCII letters M and N, fixed by 004-04 | 0x4d4e only |
| 2 | 2 | u16 | Protocol version, major in the high byte (bits 15..8), minor in the low byte; v1 packs 1.0 as 0x0100 (004-04) | 0x0100 in v1 |
| 4 | 2 | u16 | Domain id (C-11); 0 is the unconfigured domain | 1 to 65535; 0 = unconfigured |
| 6 | 2 | u16 | Stream number at the source (C-10); the stream is identified by the source address pinned at admission plus this number, never by a destination (rule 7) | 0 to 255 (at most 256 streams per source, C-09) |
| 8 | 4 | u32 | Session number (C-19): assigned by the source, one per stream, bumped on restart and on timeline-epoch change, and every consumer of a stream sees the same value — a stream property, not a per-receiver field | any 32-bit value |
| 12 | 2 | u16 | Timeline epoch: the epoch the packet's media-time positions are counted in (C-10) | the consumer's current epoch |
| 14 | 1 | u8 | Flags: gap (bit 7), degraded (bit 6; C-122: the flag follows the content — an output at position p is degraded if its input at p − L was gapped), announcement present (bit 5), measurement extension present (bit 4), bits 3..0 reserved | see the flag bits below |
| 15 | 2 bits | uint | Copy index (C-07), bits 1..0 of byte 15: wire space for the deferred duplication feature, not a flag and not a count (Reserved fields) | 0 to 3 in v1: senders emit 0, receivers accept all four values |
| 15 | 6 bits | uint | Reserved, bits 7..2 of byte 15: senders write zero (004-05 general rule) | 0 only; any value set here is counted under ReservedNonZero |

The fields tile the 16 bytes contiguously with no gap and no overlap: bytes 0 to 13 are the six multi-byte fields above, byte 14 is the flags, and byte 15 is shared by the 2-bit copy index (bits 1..0) and the 6 reserved bits (bits 7..2) [decision].

### The flags byte

Bit 7 first (Bit-field numbering); every flag describes the content the packet carries, never the receiver [decision]:

| Bit | Name | Meaning |
|---|---|---|
| 7 | gap | the packet carries a gap slice: header-only, marking positions the sender did not produce or receive (rule 10) |
| 6 | degraded | the slice was computed from a gapped input: the output at position p is degraded if the input at p − L was gapped, the flag travelling with the content, delayed by L (C-122) |
| 5 | announcement | an in-band announcement follows the header (placed by 004-09); a consumer that does not know an announcement type skips it by length (C-43) |
| 4 | measurement | the measurement extension follows the header (placed by 004-10); it fits inside the header cap or is omitted on a fallback path (C-137) |
| 3..0 | reserved | sent zero (004-05 general rule); a receiver that sees one set drops the packet and counts ReservedNonZero |

### What the header does not carry

Field by field, the header carries no module id, no destination address and no per-receiver acknowledgement (rules 7 and 8) [decision]. No field of the table names a module: streams are identified by the source address pinned at admission plus the stream number, and the 64-bit module id is not in media packets (rule 7). No field names a destination or is per-receiver: the receiver demultiplexes on the source address and the stream number, and every field of the table — domain, stream number, session, timeline epoch, copy index, flags — describes the stream and its content, so every consumer of a stream reads the same values. No field acknowledges anything: forward packets carry no per-receiver state, and round-trip closure uses the measurement path (rule 8); the POC's per-receiver `F_ACK` flags and its direction-overloaded `ack_seq`, `ack_t4` and `path_delay` fields are all retired `[code]`. The session is per stream, one value for every consumer of the stream (C-19) `[decision]`.

### Don't fragment

Don't-fragment (rule 11): the sender must set the IPv4 DF bit on every media packet it sends (C-100: IPv4 only in v1); IP fragmentation is forbidden. The DF bit is a sender obligation on the IP header, outside this header — the header carries no fragmentation field of its own `[decision]`.

### Validity rules

Every malformed header is dropped — or ignored, for the unconfigured domain — and counted under one member of the closed Rejection reasons enumeration; nothing is dropped silently (Rejection reasons) `[decision]`. The header's invalid forms and their required outcomes:

| Invalid form | Required outcome |
|---|---|
| shorter than the 16-byte header | drop and count under TooShort: dropped before any field is read |
| bad magic (the leading bytes are not 0x4d4e) | drop and count under BadMagic |
| version mismatch (the version field is not 0x0100) | drop and count under VersionMismatch, and raise the VersionMismatch event — a version mismatch is never a silent drop (Versioning, Receiver behaviour) |
| foreign domain (a non-zero domain other than the local domain, C-11) | drop and count under ForeignDomain (C-173: a different non-zero domain) |
| domain 0 (the unconfigured domain, C-11) | ignore and count under UnconfiguredDomain — never ForeignDomain (C-173) |
| stream number unknown for the pinned source (no such stream at this consumer) | drop and count under UnknownStream |
| source not admitted (no admitted connection for the source at this consumer) | drop and count under UnadmittedSource |
| wrong epoch (the timeline epoch is not the consumer's current epoch, C-10) | drop and count under WrongEpoch — an other-epoch slice is never a miss (C-102, C-135) |
| reserved bits set (any reserved bit of byte 15 or of the flags byte is non-zero) | drop and count under ReservedNonZero (Reserved fields, 004-05) |
| gap flag with a non-empty payload (a gap slice is header-only, rule 10, so the packet's length is wrong for its declared kind) | drop and count under BadLength |
| invalid flags combination (a presence bit — announcement or measurement — set on a gap slice, which carries no optional part; the declared shape disagrees with the header-only rule) | drop and count under BadLength [decision] |

### Timeline epoch width and wrap

The timeline epoch is 16 bits (C-10): 65 536 values, then the field wraps to its first value again. The falsifier to answer is an epoch-bump rate that wraps the field in under a year of operation, one year being 31 557 600 s; this section computes the wrap time at both rates and answers it `[decision]`. Three events bump the epoch: the in-band epoch-switch announcement (004-09); an election won by a candidate without usable time, who cannot take over with frequency continuity and so restarts the timeline (C-113 — a candidate with usable time takes over with continuity and keeps the epoch, C-213); and a holder stall (C-136). No constant states a bump rate, so both rates below are decisions, and epic 005 is asked to confirm the second of them.

**Pathological ceiling, election-driven bumps.** The election timers bound how fast elections can follow one another: the reply-silence trigger 1.5 s (C-37) plus the election window 400 ms (C-33) is 1.9 s per election cycle — epic 005's attribution: the silence trigger, not discovery, opens the cycle — and the discovery listen-before-claim (1.5 s, C-32) gates a newcomer's first claim without shortening the cycle. Election-driven bumps are therefore floored at one bump every 1.9 s `[decision]` — back-to-back elections in which no candidate ever has usable time (C-113), each bumping the epoch. 65 536 × 1.9 s = 124 518 s, about 1.44 days: election bumps alone wrap the field in a day and a half `[decision]`. This floor is scoped to elections; it says nothing about the other two bump paths, which the next two paragraphs bound or defer `[decision]`.

**Holder-stall cadence (C-136).** The stall path has no election behind it: the holder bumps the epoch itself (C-136) when its monotonic clock diverges from the suspend-proof clock by more than the step threshold (2 ms, C-67), so the 1.9 s floor does not bound it. What bounds it is its cause: a stall requires a suspend or a resume of the holder's host, and suspend/resume is a human or operator action — nothing on the wire and no timer of this protocol can trigger it at machine speed, so its pace is set by people, not by the medianet `[decision]`. A conservative bound is twelve stalls per hour, one stall per 5 minutes — well above any real pattern of a machine being suspended and resumed, which runs minutes to hours per event by nature `[decision]`. Folded into the wrap: elections at the 1.9 s floor plus stalls at one per 5 minutes give a combined floor of one bump every 1.89 s, so the shortest full wrap the two bounded paths can produce is 65 536 × 1.89 s ≈ 123 900 s, about 1.43 days — within one percent of the election-only figure `[decision]`.

**Announced switches: deferred to epic 006.** The third path, the epoch-switch announcement (004-09), has no cadence bound in this section: how often a switch may be announced belongs to announcement ordering and loss, which epic 006's 006-12 specifies. Until that section bounds the cadence, the announced path is carried by the open question at the end of this section: a specified announcement cadence faster than one bump every 10 minutes reopens C-10's width exactly as a measured rate would `[decision]`.

**Plausible worst-case operating rate.** Bumps are event-driven, not periodic: an election bumps the epoch only when no candidate has usable time (C-113; a candidate with usable time takes over with frequency continuity and keeps the epoch, C-213), a holder stall (C-136) needs a suspend-class clock divergence above the 2 ms step threshold (C-67), and an epoch switch is a deliberate reconfiguration act; in a healthy medianet all three are rare `[decision]`. As a deliberately generous plausible worst-case operating rate, sustained around the clock, I take one bump every 10 minutes `[decision]`. 65 536 × 10 minutes = 39 321 600 s, about 455 days, or 1.25 years `[decision]`. No bump rate is measured anywhere in v1, so the figure is a choice and not a measurement `[memory]`; epic 005 confirms it against a real medianet before commissioning.

**Verdict against the falsifier.** The falsifier asks for an epoch-bump rate that wraps 65 536 epochs in under a year. At the plausible worst-case operating rate the wrap takes 39 321 600 s — 1.25 years — so a year of operation does not exhaust the epoch space: C-10 is not falsified at the plausible rate `[decision]`. The pathological ceiling is answered explicitly: at one bump every 1.9 s the wrap takes 124 518 s, about 1.44 days, far under a year `[decision]`. Sustaining that ceiling for 1.44 days means an election every 1.9 s in which no candidate ever has usable time (C-113, C-213) — a medianet that has had no usable time source for a day and a half, whose timeline is already dead for every consumer; that is the domain-dead case, which no epoch width makes healthy, and recovery from it is re-establishment through the sync request and reply, not a wider field `[decision]`. The verdict is therefore that C-10 stands at the plausible rate, and the ceiling's sub-year wrap is the signature of a dead domain, not an operating rate `[decision]`. The stall path is bounded by its operator-paced cadence and the announced path is deferred to 006-12; the comparison rule's fail-safe below does not depend on any of these bounds holding `[decision]`.

**Current versus stale: the comparison rule at wrap.** A receiver decides current versus stale by comparing a packet's epoch with its current epoch, and equality alone is unsafe across a wrap: after 65 536 bumps the same 16-bit value recurs, so the live epoch and a long-dead epoch are indistinguishable by value `[decision]`. The rule is therefore equality inside a bounded tenure window. The receiver holds its current epoch together with the network time at which it adopted that epoch — from a sync reply, an epoch-switch announcement, or admission — and treats a packet epoch e as current only if e equals the current epoch and the tenure of the current epoch is younger than the wrap-ambiguity span W = 65 536 × 1.89 s ≈ 123 900 s, about 1.43 days — the shortest full wrap the two bounded paths can produce: election-driven bumps at the C-37 + C-33 floor plus holder stalls (C-136) at their operator-paced bound (holder-stall cadence above) `[decision]`. Inside the window equality is exact for those two bounded paths: fewer bumps than a full wrap can have happened since adoption, so an equal epoch is the same epoch `[decision]`. The announced-switch path is the one residual inside W until 006-12 bounds its cadence, and the open question at the end of this section carries it `[decision]`. An epoch that differs from the current epoch is stale without ambiguity: drop and count under WrongEpoch (C-10) `[decision]`. Tenure beyond W is reachable only for a receiver that has heard no sync traffic for days; such a receiver cannot tell whether 65 536 bumps have passed, so it stops trusting equality — every packet epoch counts as stale until a sync reply re-establishes the current epoch, and the receiver rejoins the timeline from that reply, never from packet epochs alone `[decision]`. The fail-safe does not depend on the bounds behind W: whichever path produced the bumps — elections, stalls, or announced switches — equality beyond W is never trusted, so the rule stays correct even if a path outruns its bound `[decision]`. Serial-number arithmetic was considered and declined: it orders two epoch values but cannot rescue equality across a wrap either, and the tenure rule is simpler and total `[decision]`. The line and epoch state that consumes this rule is timeline.md's (006-03 onward).

**Open item.** The plausible rate is a decision, not a measurement, and it stays open until epic 005 confirms it. Owner: epic 005, which owns the bump-rate evidence on all three paths. Falsifier: a measured or specified bump rate on any path — election-driven (C-113), holder stall (C-136), or announced epoch switch (004-09, cadence owned by 006-12) — faster than one bump every 10 minutes, which would wrap 65 536 epochs in under a year and reopen the width question of C-10 `[decision]`.

## Slice header

Every media packet carries a slice header directly after the packet header: the address of the content the packet carries, the slice's packet layout, and the send timestamp of this packet (media rule 2). A slice is the content of one stream for one stretch of media time, addressed by position — network time counted in samples from the domain epoch, in the timeline epoch the packet header carries (C-10) — and the slice header is 22 bytes long, bytes 16 to 37 of the UDP payload, counting against the header cap of C-137 together with the packet header's 16 bytes [decision].

### Field table

Little-endian (Byte order), packed with no padding (Alignment and padding); offsets are byte distances from the first byte of the UDP payload and continue the packet header's 16 bytes (004-06) [decision]:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| 16 | 8 | u64 | position: the content position of the slice's first frame on the domain line — samples at 96 000 Hz (C-01) counted from the domain epoch in the packet header's timeline epoch (C-10); the position half of the dedup key (rule 9) | any 64-bit value |
| 24 | 2 | u16 | frames: how many frames of media time the slice spans | 1 to 65 535 [decision]; 48 in the default regime (C-02) |
| 26 | 2 | u16 | packet index: this packet's index within its slice, 0-based | 0 to packet count − 1 (packet index < packet count) |
| 28 | 2 | u16 | packet count: into how many packets the slice is split | 1 in the default regime (C-05); 2 to 65 535 in the stamped multi-packet regime |
| 30 | 8 | u64 | send timestamp of this packet (Send timestamp) | any 64-bit value: nanoseconds of the sender's local monotonic clock |

The five fields tile bytes 16 to 37 contiguously with no gap and no overlap — 22 bytes for the slice header, 38 bytes including the packet header: position 16 to 23, frames 24 to 25, packet index 26 to 27, packet count 28 to 29, send timestamp 30 to 37 [decision]. A gap slice is header-only (rule 10): its packet carries exactly these 38 bytes, and any payload beyond them is the gap-with-payload case the packet header's validity rules already reject under BadLength.

### Send timestamp

The send timestamp is 64 bits of nanoseconds read from the sender's local monotonic clock — the clock the sender paces its stream from, the same basis the retired POC's t1 was read from (`module.rs`) [code]. The unit is nanoseconds, one tick 1 ns [decision]. This answers the open sub-question epic 004 owned: the unit resolves one sample period at 96 000 Hz — 10 416 2/3 ns — with four orders of magnitude to spare, so no sample boundary is ever blurred by the stamp's granularity, and the clock is the one the sender's pacing already depends on, so stamping adds no new clock and no new synchronisation requirement [decision]. The wrap period is 2^64 ns, about 584 years: vastly longer than a reassembly span — 2 slice periods (C-15), 1 ms at the C-02 defaults — and longer than any alignment span (C-110, the consumer's own deadline for the position), both of which the consumer bounds by deadlines it enforces itself, so no consumer ever observes a wrapped send timestamp on a slice it is still reassembling or aligning, and no wrap handling exists anywhere in v1 [decision].

### Multi-packet regime

The default regime is one packet per slice: packet count = 1 (C-05). The multi-packet regime is the media side's explicit stamped regime: packet count > 1 in the slice header, the source splitting a slice that does not fit one packet into ordered packets that share the slice's position and frames and each carry their own packet index, packet count and send timestamp [decision]. A consumer reassembles and deduplicates by stream, position and packet index (rule 9); the reassembly deadlines are behaviour, not wire fields — 2 slice periods for arrival-driven inputs (C-15) and the consumer's own deadline for aligned inputs (C-110).

The regime is declared only by the regime stamp (C-18): a stream uses the default regime until a regime stamp section stamps its multi-packet regime, and the hello carries only the stamp's version number. This ticket hands the following values to 004-25, which lays the multi-packet regime section out (Regime stamp): the stream number the regime applies to, the packets per slice (the slice header's packet count for that stream), the frames per slice (the slice length, which steps the stream's position grid), and the timeline epoch the stamped positions are counted in. That set is complete and documented here as the media side's requirement: with the slice header plus these stamped values, every consumer of the stream can reassemble, deduplicate and place every packet of the stream, and no further wire field carries regime state [decision].

### Validity rules

Every malformed slice header is dropped and counted under one member of the closed Rejection reasons enumeration; nothing is dropped silently [decision]. The slice header's invalid forms and their required outcomes:

| Invalid form | Required outcome |
|---|---|
| packet index ≥ packet count (the index lies outside the packet count its slice declares — BadPacketIndex's own definition) | drop and count under BadPacketIndex |
| packet count = 0 (a slice cannot consist of zero packets: the packet carrying the count is itself at least one packet of the slice, so the declared count disagrees with the message's own size rules) | drop and count under BadLength |
| frames = 0 or frames above 65 535 (a slice spans at least one frame, and the wire never carries a slice longer than the frames field can express) | drop and count under BadLength |
| frames inconsistent with the payload length (the content after the headers does not fill frames × channels × 4 bytes at the stream's channel count, C-03, C-04) | drop and count under BadLength |
| packet count inconsistent across the packets of one slice (the packets disagree on the slice's declared shape, so at least one packet's index lies outside the count another packet of the same slice declares) | drop and count under BadPacketIndex |

No position-grid rule applies at packet intake in v1: the position is a raw sample count, and whether it sits on the stream's position grid — the slice length the regime stamp declares — is checked where the slice is consumed (reassembly and alignment, C-15, C-110), not at intake; no member of the closed enumeration names a misaligned position, so inventing an outcome here would extend the enumeration, which only a major version may do [decision].

### Receive timestamps

The slice header carries no receive timestamp: when a packet arrived is a property of the receiver, not of the content. The receive-timestamp source is a per-module capability, one of kernel (hosted OS kernel stamp), userspace (clock read after receive), mac-hardware (Ethernet MAC timestamp register, embedded) or interrupt (stamped in the receive interrupt, embedded) — the four values of C-131 — and each is reported with one of the four capability states of C-129: present, absent, unverified, not requested. The source and its capability state travel in the regime stamp (004-25's receive-timestamp section, C-18), not in the slice header; the per-OS detail is completed from epic 001's report on receive timestamps and don't-fragment per OS, which is non-blocking and affects only these notes [decision].

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
