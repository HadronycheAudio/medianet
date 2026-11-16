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
| 5 | UnknownStream | the stream number names no stream this consumer admits; on the measurement layer, a receiver-report row whose stream number is not the member admitted to the reporting consumer at that row (Receiver report) | media, measurement |
| 6 | UnadmittedSource | the source's module id has no admitted connection at this consumer; on the sync layer, a reply whose holder module id is not the holder the follower follows (Sync request and reply); on the measurement layer, a receiver report whose bundle identity is not the receiving producer's module id or whose reporter identity names no module with an admitted connection for the bundle its rows name, a probe request from a module that shares no admitted connection with the responder, or a probe reply without matching fields (Receiver report, One-way probe) | media, sync, measurement |
| 7 | BadLength | a declared length or count disagrees with the message's own size rules: a count above its fixed maximum, or a body that does not fill the fixed size | control, sync, media, measurement |
| 8 | BadChecksum | the checksum does not match the recomputed value; the checksum arrives with 004-13 | media |
| 9 | BadPacketIndex | the packet index lies outside the packet count its slice declares | media |
| 10 | WrongEpoch | the timeline epoch is not the consumer's current epoch (C-10): counted under this reason — the per-connection other-epoch counter (C-135, C-159) — and never a miss (C-102); on the sync layer, a reply whose epoch differs from the follower's current epoch and that answers a request sent no later than the follower adopted that epoch, counted per socket (Sync request and reply) | media, sync |
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
- a new optional measurement-extension entry (the extension fits inside the header cap or is omitted, C-137; no v1 sender omits it for size, Size budget).

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

A reserved field is wire space of a fixed width that this specification fixes at a required value, so a later version can give the space a meaning without moving any other field: reserved space is a named field with a required value, never alignment filler (Alignment and padding). Two reserved fields are fixed now — the copy index of the media packet header (C-07) and the authentication reservation of the control and sync messages (C-12) — and the general rule below covers every padding or reserved field a later ticket adds. A reserved value that does not conform is a malformed case like any other: the receiver drops the packet and counts it under `ReservedNonZero` (Rejection reasons, value 12), the reason that enumeration reserves for exactly this case `[decision]`.

| Field | Location | Width | Required value in v1 | Receiver outcome |
|---|---|---|---|---|
| Copy index (C-07) | media packet header (004-06 places it) | 2 bits | 0 — v1 senders emit 0 only | v1 receivers accept all four values 0 to 3 [decision]; no copy-index value is a malformed case in v1 |
| Authentication scheme (C-12) | control messages (004-16 to 004-18 place it) and sync messages (Sync request and reply) | 8 bits | 0 (none) | a non-zero scheme: drop and count as ReservedNonZero |
| Message counter (C-12) | control messages and sync messages (Sync request and reply) | 64 bits | live — the liveness counter (C-115) in control messages; in sync messages it counts the sender's sync messages and is live without being the liveness counter; any value | never ReservedNonZero: the counter is not a reserved zero field |
| Authentication tag (C-12) | control messages and sync messages (Sync request and reply) | 128 bits | 128 zero bits | a non-zero tag: drop and count as ReservedNonZero |

### Copy index (C-07)

The copy index is 2 bits in the media packet header (C-07), placed by 004-06. It is wire space kept for the deferred duplication feature — retransmitting a slice as numbered copies — not a flag, not a count and not alignment filler `[decision]`.

Required value in v1: senders emit only the value 0. The three non-zero values stay unused until duplication assigns them a meaning `[decision]`.

Receiver rule `[decision]`: v1 receivers accept all four copy-index values, 0 to 3. Accepting every value is the deliberate choice: the copy index exists so a later version can send duplicated copies without a major version, and a v1 receiver that rejected non-zero values would force exactly that bump; no copy-index value is a malformed case in v1.

Dedup rule (media rule 9, restated): copies are deduplicated by stream, position and packet index — the first valid copy wins; there is no separate sequence number, and nothing about a copy's content depends on its index `[decision]`.

When duplication arrives — a later version retransmitting a slice as copies 1 to 3 — v1 receivers already accept all four copy-index values, so the feature ships as a minor version with no new wire value: no value that version needs is rejected today, and no major version is forced `[decision]`.

### Authentication reservation (C-12)

Every control message carries the authentication reservation (C-12): scheme 8 bits (0 = none) + message counter 64 bits + tag 128 bits — 200 bits in all — placed by 004-16 to 004-18. The reservation is wire space for an authentication scheme a later version may define; v1 defines none `[decision]`. Measurement messages do not carry it: the receiver report and the one-way probe are data-plane messages, which ADR 22 leaves outside the reserved scheme (Receiver report, Authentication reservation) `[decision]`.

In v1 (C-115) the scheme is 0 and the tag is 128 zero bits, while the 64-bit message counter is live: it is the liveness counter, it increments with every control message a source sends, and it carries any value — it is not a reserved zero field, so no rule of this section applies to it `[decision]`.

Receiver outcomes: a scheme other than 0 is a malformed case — the receiver drops the packet and counts it under ReservedNonZero (12); a non-zero tag arriving with scheme 0 is likewise dropped and counted under ReservedNonZero. The counter never produces ReservedNonZero: it is not fixed at zero `[decision]`.

### General rule for later tickets

Any padding or reserved bits a later ticket adds to any message follow one rule: the sender sets them to 0 — every reserved bit is written zero on transmit, whatever meaning a later version gives it — and a receiver that sees a non-zero value drops the packet and counts it under ReservedNonZero, exactly as for the named reserved fields above. No ticket may give a reserved field a non-zero v1 value; assigning one a meaning is a version change, never a silent reinterpretation `[decision]`.

## Media packet header

A media packet carries the content of one stream for one stretch of media time — the slice content, one packet per slice by default — and travels by UDP unicast at DSCP EF 46 (C-13) `[decision]`. Its header is 16 bytes (128 bits) long and counts against the header cap of C-137 (a default slice fits 1 252 bytes of UDP payload: 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum), leaving 84 bytes of the cap for the slice header, the in-band announcements, the measurement extension, the content-message headers and the checksum that 004-08, 004-09, 004-10, 004-11 and 004-13 place after it; at the format's maximum those parts would need 103 − 16 = 87 bytes, 3 more than the 84, but a v1 sender carries at most one announcement per packet, which keeps them to 22 + 13 + 9 + 3 + 4 = 51 bytes (Size budget); those tickets extend this message and grow its declared maximum size, and this section's table does not move [decision]. The header carries no sample-rate field: the wire has one rate, the domain's 96 000 Hz (C-01), so the rate is a property of the domain, not of a packet [decision].

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
| 4 | measurement | the measurement extension follows the header (placed by 004-10); it fits inside the header cap or is omitted on a fallback path (C-137); no v1 sender omits it for size (Size budget) |
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
| frames inconsistent with the audio length (over the reassembled slice, the audio message's length is not frames × channels × 4 bytes at the stream's channel count, C-03, C-04; the content-message headers, any controller messages and the checksum are not part of it — Content messages, Audio message) | drop and count under BadLength |
| packet count inconsistent across the packets of one slice (the packets disagree on the slice's declared shape, so at least one packet's index lies outside the count another packet of the same slice declares) | drop and count under BadPacketIndex |

No position-grid rule applies at packet intake in v1: the position is a raw sample count, and whether it sits on the stream's position grid — the slice length the regime stamp declares — is checked where the slice is consumed (reassembly and alignment, C-15, C-110), not at intake; no member of the closed enumeration names a misaligned position, so inventing an outcome here would extend the enumeration, which only a major version may do [decision].

### Receive timestamps

The slice header carries no receive timestamp: when a packet arrived is a property of the receiver, not of the content. The receive-timestamp source is a per-module capability, one of kernel (hosted OS kernel stamp), userspace (clock read after receive), mac-hardware (Ethernet MAC timestamp register, embedded) or interrupt (stamped in the receive interrupt, embedded) — the four values of C-131 — and each is reported with one of the four capability states of C-129: present, absent, unverified, not requested. The source and its capability state travel in the regime stamp (004-25's receive-timestamp section, C-18), not in the slice header; the per-OS detail is completed from epic 001's report on receive timestamps and don't-fragment per OS, which is non-blocking and affects only these notes [decision].

## Announcements

An in-band announcement is a short, self-describing note a media packet may carry after its slice header: it names a future position on the domain line — the slice header's position form, 64 bits — and tells every consumer of the stream what will change there. v1 defines four types: end-of-stream, re-alignment, timeline epoch switch and transport reference change (rule 3) `[decision]`.

### The container

A packet carries announcements when the media packet header's announcement-present flag (bit 5 of the flags byte, 004-06) is set, and the sender sets the flag exactly when the count field below is at least 1 `[decision]`. The container is a 1-byte count followed by that many announcements, each led by a type code and a length. A receiver of the same major version that does not know a type code skips that announcement by its length field and reads on with the next: the per-announcement length prefix is what keeps the type space open to minor changes of the same major (Versioning, Minor and major changes, 004-04) `[decision]`. Each announcement is 12 bytes on the wire in v1 — type code 1, length 1, position 8, payload 2 `[decision]`.

The announcements start at byte 38 of the UDP payload, directly after the 22-byte slice header (004-08). Offsets are byte distances from the first byte of the UDP payload, and the per-announcement offsets are relative to the announcement's own first byte, so the first announcement starts at byte 39. Little-endian (Byte order), packed with no padding (Alignment and padding) `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| 38 | 1 | u8 | announcement count: how many announcements follow the count field | 0 to 4 `[decision]` (Maximums) |
| +0 | 1 | u8 | type code of the announcement: which of the four types follows (Type codes) | 0 to 3 in v1; a same-major receiver skips a value it does not know by length |
| +1 | 1 | u8 | length: the announcement's body length in bytes after this field — the length prefix that makes an announcement skippable without knowing its type | 10 in v1: the 8-byte position plus the 2-byte payload |
| +2 | 8 | u64 | position: the announcement's named future position — samples at 96 000 Hz (C-01) counted from the domain epoch in the packet header's timeline epoch (C-10), the slice header's position form | any 64-bit value; a valid announcement names a position after the packet's slice position (Validity rules) `[decision]` |
| +10 | 2 | u16 | payload: the announcement's type-specific payload, laid out by the per-type tables below; in v1 only the epoch switch fills it | the new timeline epoch for type code 2; 0 for type codes 0, 1 and 3 `[decision]` |

The announcement fields repeat, back to back with no separator, up to the count field's value: the second announcement starts at byte 51, the third at byte 63, the fourth at byte 75, and the container ends at byte 38 + 1 + 12 × count. A count above its maximum of 4, and an announcement whose length runs past the end of the packet, are rejections (Validity rules) `[decision]`. The machine-readable tables carry the same layout: the media packet message grows by the count field and a counted entry region of one 12-byte entry per element, and an announcement message lays one entry out (wire.toml) `[decision]`.

### Type codes

Every announcement starts with its 1-byte type code. v1 defines the four codes below; 4 to 255 are undefined in v1, and a same-major receiver skips an announcement whose code it does not know by the announcement's length field (The container), so a minor version of the same major may define new codes without breaking v1 receivers (004-04) `[decision]`:

| Code | Type | Meaning |
|---|---|---|
| 0 | end-of-stream | the stream's content ends at the named position |
| 1 | re-alignment | the stream's output re-aligns at the named position (C-186, C-208) |
| 2 | timeline epoch switch | the timeline's epoch changes at the named position; the payload carries the new epoch (C-10) |
| 3 | transport reference change | the stream's transport reference changes at the named position |

Codes 4 to 255 stay unused in v1; a minor version of the same major may assign from that range, and a major version may redefine the code space (Versioning) `[decision]`.

### End-of-stream (type code 0)

The sender will address no position of the stream at or after the named position: the stream's content ends there, and a consumer treats the stream as complete from that position on. No further information is needed, so the payload rides zero-filled and every announcement keeps the same 12-byte shape (The container) `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| +0 | 8 | u64 | position: the stream's last position — the end-of-stream boundary of the content the sender will address, counted in the packet header's timeline epoch (C-10) | any 64-bit value; a valid announcement names a position after the packet's slice position (Validity rules) |
| +8 | 2 | u16 | payload: unused by this type in v1 | 0 only |

### Re-alignment (type code 1)

The stream's output re-aligns at the named position: from it the sender's output sits on a new alignment, and a consumer that inherited the old alignment re-enters aligning there (C-186). One announcement serves both causes C-208 names — a ratchet step (C-215) and a changed A — because each fan-in picks its own position, one lead (C-43) ahead of its current output, and announces this type either way; the cause itself is not on the wire `[decision]`. The payload is empty in v1, and whether that proposed minimum lets a receiver act is an open item (Open items) `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| +0 | 8 | u64 | position: the re-alignment position — the first position the sender's output addresses on the new alignment | any 64-bit value; a valid announcement names a position after the packet's slice position (Validity rules) |
| +8 | 2 | u16 | payload: unused by this type in v1 | 0 only |

### Timeline epoch switch (type code 2)

The timeline epoch switch moves the timeline to a new epoch at the named position: from it the sender counts positions in the new epoch, and the packet header's timeline epoch field carries the new value on every packet (C-10). The payload names the new 16-bit epoch so a consumer can prepare before the switch; it does not name the new session — C-19 bumps the session of every stream at the switch, and the packet header's session field carries each stream's new value from its first post-switch packet, so repeating it here would duplicate a field every packet already carries `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| +0 | 8 | u64 | position: the first position the sender counts in the new timeline epoch | any 64-bit value; a valid announcement names a position after the packet's slice position (Validity rules) |
| +8 | 2 | u16 | payload: the new timeline epoch (C-10) | the new epoch; the packet header's timeline epoch field carries it from the switch on |

### Transport reference change (type code 3)

The stream's transport reference change moves the output to a new transport reference — the peer the sender paces and aligns its output against — at the named position. The payload is empty in v1: the reference's identity and the receiver's required action are not on the wire, and whether that proposed minimum lets a receiver act is an open item (Open items) `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| +0 | 8 | u64 | position: the first position at which the output follows the changed transport reference | any 64-bit value; a valid announcement names a position after the packet's slice position (Validity rules) |
| +8 | 2 | u16 | payload: unused by this type in v1 | 0 only |

### Maximums

**Maximums.** The container's own maximum follows from its field widths: the count field is 1 byte and each announcement is 12 bytes, so the container occupies at most 1 + 4 × 12 = 49 bytes `[decision]`. The C-137 header cap leaves 100 − 38 = 62 bytes of a default slice's UDP payload for everything that follows the packet and slice headers, of which the measurement extension takes 9 bytes (004-10) and the checksum takes 4 bytes (004-13) `[decision]`. Of that remainder the announcements take at most 4 announcements per packet — 12 bytes each, so at most 48 announcement bytes per packet — plus the 1-byte count field, which leaves 13 bytes of the 62 for those two shares, exactly 9 + 4 `[decision]`. The cap of 4 is the number of announcement kinds, not a guess at a typical load: a holder change plausibly makes an end-of-stream, a timeline epoch switch, a transport reference change and a fan-in's re-alignment pending together, their first announcements coinciding and their re-issue schedules aligned in every bundle member, so without staggering first issues a smaller cap would force a re-issue step to be dropped exactly when several changes are in flight at once; Size budget's announcement schedule staggers them and puts at most one announcement in a v1 sender's packet, while a receiver still accepts up to 4 `[decision]`. With duplicate types invalid in one packet (Validity rules) and announced positions standing until they pass (Sender obligations) — a kind's next change is first announced only after its pending position has passed — at most one announcement of each kind is pending at any moment by construction, so no constructible combination exceeds the cap of 4 and no announcement is ever dropped from a re-issue step `[decision]`. All four announcements fit inside the 100-byte cap beside the mandatory parts — 16 + 22 + 49 + 3 + 4 = 94 bytes with the audio message's content-message header (004-11) — but not beside the measurement extension as well; with it they come to 103 bytes, a packet no v1 sender builds, because each kind rides only in its own slot class of slices, one announcement per packet, so nothing is omitted on a fallback path and no re-issue step is lost (C-137, C-44; Size budget) `[decision]`.

### Sender obligations

**Sender obligations.** A sender that must announce a future change starts announcing one lead ahead: C-43's lead is 64 slices, flat — 64 × 0.5 ms = 32 ms at the C-02 defaults — so the first announcement of a change goes out at least 64 slices before the named position, as an obligation of the sender, never a receiver courtesy `[decision]`. The sender re-issues the announcement every 16 slices (C-44) until the named position passes, in every bundle member it sends — C-08 bounds a bundle at 32 members, so a pending change rides in up to 32 outputs per re-issue step — and the container never blocks that cadence: the cap of 4 covers every constructible combination of pending kinds (Maximums), no announcement is omitted on any path to meet the 100-byte cap, and every stream carries each kind only in the slices of its own slot class, one announcement per packet, so every pending announcement is re-issued on schedule (Size budget, Announcement schedule and omission rule) `[decision]`. A re-issue inside the lead is valid: C-43's lead binds the sender's first issue of a change only, and the re-issues C-44 requires of a change first announced exactly one lead ahead name a position 48, 32 and 16 slices ahead of the slices that carry them `[decision]`. A receiver cannot tell a first issue from a re-issue, so it does not police the lead: it accepts any announcement whose position lies after the packet's slice position — the position of the slice's first frame (Slice header), so a named position inside the carrying slice, after that frame, is accepted — and rejects only a position that is not in the future (Validity rules, C-43, C-44) `[decision]`. The named position passes once the sender's slice position reaches it, so no announcement rides in a slice whose slice position is at or after the position it names, and for a change first announced exactly one lead ahead the re-issue 16 slices ahead is the last (C-44) `[decision]`.

**Announced positions stand until they pass.** Once a sender has announced a kind's position, that position stands until it passes: the sender never withdraws it, never moves it, and never announces a second position of the same kind while it is pending, so at most one announcement of each kind is pending at any moment (C-43, C-44) `[decision]`. A further change of the same kind is first announced only after the pending position has passed, and like every announcement it names a position at least one lead (C-43, 64 slices) ahead of the slice that first carries it and is re-issued every 16 slices until that position passes (C-44) `[decision]`. Withdrawal is excluded rather than given a lead of its own because a withdrawal would travel the same lossy path as the announcement it withdraws: a consumer that heard the first position and missed the withdrawal would act at a position the sender had abandoned, which breaks the promise C-43 and C-44 make to every consumer — learn of a change one lead ahead, repeated every 16 slices. With positions that stand, any one announcement a consumer hears is enough to act at the right position `[decision]`. A consumer therefore never has to reconcile two positions of one kind; a sender that announces a different position of a kind while that kind's earlier position is still pending violates this obligation, and how a receiver detects and handles that across packets belongs to announcement ordering and loss (006-12), since one packet's container already rejects a duplicate kind (Validity rules) `[decision]`.

**A newer re-alignment cause while one is pending.** C-208 gives a re-alignment two causes, a ratchet step (C-215) and a changed A (C-186), and either can arrive while a re-alignment the fan-in has already announced at position P is pending. The re-alignment announcement carries only the position, never the cause (Re-alignment), so the announcement on the wire does not change with the cause `[decision]`. The fan-in may fold the newer cause into P only when P is still at least one lead (64 slices) ahead of the slice in which the fan-in learns of the newer cause: P is then a position C-208 lets the fan-in pick for the newer cause — one lead ahead, announced in its own output — and every consumer has already had at least a lead's notice of it (C-43, C-208) `[decision]`. When P is less than one lead ahead, P is not far enough ahead for the newer cause, because a position freshly picked for it would have to lie at least a lead ahead: the pending re-alignment still takes place at P as announced, and the newer cause's re-alignment is first announced only after P has passed, at a position at least one lead ahead of that first announcement (C-43, C-208) `[decision]`. A fan-in that cannot apply the newer cause at P for any reason of its own state machine (epic 005) takes the same deferred path, so folding is a permission and deferral is always correct. A deferred re-alignment is first announced in the first slice of its slot class (type code 1; Size budget, Announcement schedule and omission rule) at or after the first slice after P passes, at most 3 slices later, and takes effect at most two leads plus those 3 slices — 64 + 3 + 64 = 131 slices, 65.5 ms at the C-02 defaults — after the fan-in learned of its cause, instead of one lead plus at most 3 slices `[decision]`.

### Validity rules

Every malformed announcement is dropped with its packet — or skipped, for the one form that is not a rejection — and every rejection counts under one member of the closed Rejection reasons enumeration; nothing is dropped silently `[decision]`. The container's invalid forms and their required outcomes:

| Invalid form | Required outcome |
|---|---|
| unknown type code within the same major version (a code outside 0 to 3): a same-major receiver skips the announcement by its length field and reads on with the next; the type space is open within a major version (Versioning, 004-04) | skip by length — the announcement is ignored and nothing is counted; this form is never a rejection `[decision]` |
| length overrun: an announcement's length field runs past the end of the packet, so the body it declares does not fit | drop and count under BadLength |
| named position not in the future: the announced position lies at or before the slice position the packet carries, so it has already passed | drop and count under BadLength `[decision]` — see the note below |
| duplicate type in one packet: the count declares two announcements of the same type code, so the container disagrees with its own rule of one announcement per kind | drop and count under BadLength `[decision]` |
| announcement in a gap slice: cannot occur — the packet header's validity rules already reject the announcement-present flag on a gap slice under BadLength (Media packet header, Validity rules), so no announcement-specific outcome exists | none — the case cannot occur `[decision]` |
| count above the maximum: the count field exceeds 4 | drop and count under BadLength |
| announcement-present flag set with a zero count: the flag promises at least one announcement and the container carries none, so the declared shape disagrees with itself | drop and count under BadLength `[decision]` |

**Why BadLength for a position that is not in the future.** The closed enumeration offers no announcement-specific reason: WrongEpoch is about the packet's own timeline epoch, not the announced position, and no other member describes a value that is well-formed but unusable. BadLength is the enumeration's declared-shape member — the slice header already uses it for a packet count of 0, and the packet header for a presence flag on a gap slice — and a valid announcement declares its own shape: a position in the future. A position at or before the packet's slice position disagrees with what the announcement declares, exactly as a zero packet count disagrees with the slice header's own rules. The lead is not a receiver rule: it binds the sender's first issue of a change (C-43, Sender obligations), and a re-issue (C-44) inside it is valid, so a position that is in the future but less than 64 slices ahead is accepted. The behavioural side of an announcement a receiver had already acted on belongs to the state machines of epics 005 and 006 and is out of scope here `[decision]`.

### Open items

**Open items.** Two payloads are proposed minimums and stay open until confirmed. Open item 1 — the re-alignment payload: in v1 the announcement carries only the named future position, because C-208 makes the picked position the only cause-dependent value a fan-in announces, whether the cause is a ratchet step or a changed A (C-186); what epic 005 must confirm is that a receiver can re-enter aligning from the position alone. Open item 2 — the transport reference change payload: in v1 the announcement carries only the named future position of the change; the reference's identity and the receiver's required action are not on the wire, and epic 005 must confirm what the payload must carry for a receiver to act. Owner and confirming owner for both: epic 005, which owns the re-alignment and transport-reference state machines (epic 004 proposes the minimums here). Falsifier for both: a receiver cannot act on the announcement from its payload alone — if a state machine needs a field the payload does not carry, the proposed minimum is wrong and a minor version of the same major grows the payload, which the length prefix makes a minor change (004-04) `[decision]`.

## Measurement extension

The measurement extension is 9 bytes: three counters — hop count, accumulated residence time and accumulated declared delay — that the processors along a stream's path update as they forward its content, so a consumer can read how many processors the content passed, how long it sat inside them and how much structural delay they declared (rule 4, 01-architecture.md § 4.7) `[decision]`. It is optional: a packet carries it exactly when the measurement-present flag (bit 4 of the flags byte, Media packet header) is set, and production streams may omit it (rule 4) `[decision]`.

### Position

The extension follows the announcement container directly: it starts at byte 38 + 1 + 12 × count of the UDP payload, where count is the announcement count and the container ends (Announcements, The container) — byte 39 in a packet with no announcement, byte 87 in a packet with four. Its offsets below are therefore relative to the extension's own first byte, written +0 onward, as the per-announcement offsets are; what follows the extension is placed by 004-11 (content) and 004-13 (checksum) `[decision]`. Little-endian (Byte order), packed with no padding (Alignment and padding) `[decision]`.

### Field table

The three fields tile the 9 bytes contiguously with no gap and no overlap — hop count at +0, residence time +1 to +4, declared delay +5 to +8 — and the extension has no reserved bits of its own `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| +0 | 1 | u8 | hop count: how many processors have forwarded the content since its source, which emits 0; each processor adds 1 when it forwards. Saturates at 64, never wraps: a processor that receives 64 forwards 64 `[decision]` | 0 to 64: a path that visits each of the at most 64 modules of a medianet (C-34) at most once, the source emitting 0, counts at most 63; nothing forbids a module forwarding content twice along a path, so 64 is the saturated value, read as "64 or more" `[decision]` |
| +1 | 4 | u32 | accumulated residence time: the sum, over the processors that forwarded the content, of the duration each held it — unit nanoseconds (ns) of each processor's own local monotonic clock, the send timestamp's unit (Slice header, Send timestamp) `[decision]`. Saturates at 4 294 967 295 ns, never wraps: a sum that would exceed it is written as 4 294 967 295 `[decision]` | any 32-bit value; 4 294 967 295 is the saturated value, read as "about 4.29 s or more" |
| +5 | 4 | u32 | accumulated declared delay: the sum of the structural latency L each forwarding processor declares in its manifest — whole samples at the domain rate, 96 000 Hz (C-01), each L a whole number of samples (C-105), so the sum is an exact integer. Saturates at 4 294 967 295 samples, never wraps: a sum that would exceed it is written as 4 294 967 295 `[decision]` | any 32-bit value; 4 294 967 295 is the saturated value, read as "about 44 739 s or more" |

The residence unit is nanoseconds rather than samples at the domain rate `[decision]`: residence is wall-clock time, read from the clock the send timestamp already uses, and one sample period at 96 000 Hz is 10 416 2/3 ns, so a sample unit would round away the sub-sample residence differences a consumer's lateness measurement resolves (01-architecture.md § 4.7: residence is self-accounting and appears in downstream lateness). The declared delay stays in whole samples because it is positional, not wall-clock: it is the exact sum of the manifests' L values (C-105), and no conversion to time is needed to add them `[decision]`.

### Forwarding update rule

A source that measures a stream emits the extension with all three fields 0; an edge's resampler delay belongs to its plumbing delay D and never to L (C-207), so an edge adds nothing to the declared-delay field `[decision]`. Each processor that forwards the content updates the extension of its output from the extension of its critical input — at a fan-in the critical input's values propagate (01-architecture.md § 4.7), the critical input being the one epic 014 defines: the input that determined the offset (014-14); which input that is belongs to the module runtime, not to this section — in three steps, each saturating at the field's maximum and never wrapping: (1) increment the hop count by 1, saturating at 64; (2) add its own residence time — the duration on its local monotonic clock from receiving the critical input's slice to the send timestamp of the output packet, in ns; which receive-timestamp source (C-131) marks the receipt is the module's backend's, reported with its capability state in the regime stamp (004-25), not fixed here — saturating at 4 294 967 295; (3) add its own declared delay, its manifest's L in whole samples at the domain rate (C-105), saturating at 4 294 967 295 `[decision]`.

A processor carries the extension on its output only when its critical input's slice carried it; it never starts one of its own, so the counters always start at a source `[decision]`. The values describe the slice's content, so every packet of a multi-packet slice that carries the extension carries the same values `[decision]`. A gap slice carries no extension — it is header-only (rule 10), including the gap slice the hosted backend emits for a module that misses its own deadline (C-144), and the packet header's validity rules already reject the measurement flag on a gap slice under BadLength `[decision]`.

### Byte size and the header cap

The C-137 header cap allows at most 100 bytes for all headers and the checksum; the packet header and the slice header take 38, leaving 100 − 38 = 62 bytes, and a maximal announcement container takes 49 of them (1 + 4 × 12, Announcements, Maximums), leaving 62 − 49 = 13 bytes for the measurement extension and the checksum (004-13) together `[decision]`. The 9-byte extension fits beside a maximal container: 13 − 9 = 4 bytes remain for the checksum that 004-13 sizes and places, enough for a 32-bit checksum; this section does not choose the checksum's algorithm `[decision]`. With the audio message's 3-byte content-message header (004-11) the parts at their format maximum no longer all fit together: 16 + 22 + 49 + 9 + 3 + 4 = 103 bytes. A v1 sender never builds that packet: its announcement schedule puts at most one announcement in a packet, so the extension fits inside the cap in every v1 packet (at most 67 bytes of headers and checksum without controllers), no v1 sender omits it on a fallback path to meet C-137, and it is omitted only when the stream does not measure (rule 4; Size budget) `[decision]`. The media packet's maximum size grows from 87 to 96 bytes (87 + 9) before the checksum (wire.toml), the figure 004-14 budgets `[decision]`.

### What the extension does not carry

No field of the extension identifies a receiver or destination (rule 8): all three are sums over the processors upstream of the packet, the same for every consumer of the stream, and none names a module, an address or an acknowledgement `[decision]`. Forward packets carry no per-receiver state; round-trip closure uses the measurement path (receiver reports and one-way probes, 004-20) `[decision]`.

### Validity rules

Every malformed extension is dropped with its packet and counted under one member of the closed Rejection reasons enumeration; nothing is dropped silently `[decision]`. The extension's invalid forms and their required outcomes:

| Invalid form | Required outcome |
|---|---|
| measurement flag set but the extension's 9 bytes are missing: the packet ends before the extension's last byte, so the body does not fill the shape the flag declares | drop and count under BadLength `[decision]` — not TooShort, which the enumeration reserves for fewer bytes than the message's fixed minimum, dropped before any field is read; here the header has been read and the declared shape is what the bytes disagree with, as for an announcement's length overrun |
| hop count beyond the maximum: the hop count exceeds 64, a value no saturating processor writes | drop and count under BadLength `[decision]` — the declared-shape member, as for an announcement count above its maximum |
| reserved bits set: the extension has no reserved bits, and a set reserved bit of the flags byte (bits 3..0) beside the measurement flag is the packet header's case | drop and count under ReservedNonZero (Reserved fields, 004-05) |
| measurement flag set on a gap slice | drop and count under BadLength, the packet header's rule (Media packet header, Validity rules) |

No value of the residence-time or declared-delay field is a malformed case: both accept every 32-bit value, the maximum being the saturated reading `[decision]`.

### Open items

**Open item.** The widths of the two accumulated fields (32 bits each) and the residence unit (ns) are decisions of this section; no constant gives the largest L a manifest admits, so the saturation margin is computed, not guaranteed. Owner: epic 004. At 32 bits the declared-delay field holds 4 294 967 295 samples, 4 294 967 295 / 96 000 ≈ 44 739 s, which over the maximum hop count of 64 is ≈ 699 s of declared delay per processor — far beyond any plausible structural latency, so saturation is unlikely; the residence field holds ≈ 4.29 s, ≈ 67 ms per processor over 64 hops `[decision]`. Falsifier: the declared-delay field saturates over the maximum hop count at the largest L a manifest admits — if a manifest may declare an L above 4 294 967 295 / 64 ≈ 67 108 864 samples (≈ 699 s), the width is wrong and a major version widens it `[decision]`. Falsifier for the residence width: on a 64-hop path a processor's residence time exceeds 2^32 ns / 64 ≈ 67 ms per hop on average, which saturates the residence field on a real path `[decision]`.

## Content messages

After its optional parts a slice carries its content as content messages (media rule 5): audio, dense controllers, sparse keyed controllers and read-only transport state (01-architecture.md § 3.2). This section fixes the content region that holds them, the common header every content message starts with, the audio message and the dense controller message; 004-12 adds the sparse keyed controllers and the transport state under the same common header `[decision]`.

### Content region

The content region of a packet is the one the checksum section defines: every byte after the measurement extension, or after the announcement container when the measurement flag is clear, up to the end of the UDP payload — in the slice's last packet up to the trailing checksum (Checksum, Covered range) `[decision]`. Content messages sit in it back to back, with no separator and no padding (Alignment and padding), and the last message ends exactly where the region ends `[decision]`. In the multi-packet regime the slice's content is the concatenation of its packets' content regions in packet-index order — the order the checksum covers them — and the messages are laid over that concatenation, so a message may continue from one packet into the next; a receiver parses the content after reassembly, when it verifies the checksum and before it hands the handler the whole slice (01-architecture.md § 3.1) `[decision]`. Offsets in this section's field tables are relative to the message's own first byte, written +0 onward, as the per-announcement offsets are `[decision]`.

Which messages a slice carries is fixed by the stream's role in the manifest (004-26), never by a flag in the packet `[decision]`:

| Stream | Audio message | Controller messages |
|---|---|---|
| audio stream: a single-stream source's stream, or an audio member of a bundle source | exactly one, the first content message | none in an audio member of a bundle source (C-205); in a single-stream source's audio stream only while they fit the 100-byte cap (C-181) |
| controller stream: a bundle source's controller member (C-205, C-209), or a single-stream source's controller second stream (C-181) | none | the stream's controllers |

A gap slice has no content region: it is header-only (rule 10), so it carries no content message of any kind `[decision]`.

### Common content-message header

Every content message starts with the common header, 3 bytes long: a kind, then a length `[decision]`. The starting shape is the POC's 8-byte `Msg` — kind in the high 4 bits, channels, count, length and a signed offset `[code]`. v1 keeps only the kind and the length in the common part, because those two are all a receiver needs to classify a message and to step over it; every other POC field moves into the body of the kind that needs it or off the wire — the audio channel count follows from the length and the manifest's channel map, and an offset belongs to the sparse kinds of 004-12 `[decision]`. The kind takes a whole byte rather than the POC's high 4 bits, so the code space is 256 values and no reserved bits hide in it `[decision]`.

Little-endian (Byte order), packed with no padding (Alignment and padding) `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| +0 | 1 | u8 | kind: which content message follows (Kind codes) | 0 to 3 in v1; a same-major receiver skips any other value by the length |
| +1 | 2 | u16 | length: how many body bytes follow the length field, the common part not included | 0 to 65 535; the body must end inside the content region |

A receiver of the same major version that does not know a kind skips the message by its length: it steps over the kind, the length and the length's count of body bytes, and reads on with the next message; the length prefix is what keeps the kind space open to minor changes of the same major (Versioning, Minor and major changes, 004-04) `[decision]`. The common header has no reserved bits and no version of its own `[decision]`.

### Kind codes

The kind is one byte; v1 assigns four codes `[decision]`:

| Code | Kind | Body |
|---|---|---|
| 0 | audio | Audio message |
| 1 | dense controllers | Dense controller message |
| 2 | sparse keyed controllers | Sparse keyed controller message and Sparse keyed entry |
| 3 | transport state, read-only | Transport state message |

Codes 2 and 3 are assigned to 004-12's kinds here so the code space is settled once; their bodies are 004-12's `[decision]`. Codes 4 to 255 are undefined in v1: a minor version of the same major may assign from that range, and a receiver that does not know a code skips the message by its length (004-04) `[decision]`. No future kind may precede the audio message: the audio message stays the first content message of every slice that carries audio, so a v1 receiver finds it at the start of the content region whatever later kinds a minor version adds after it `[decision]`.

In wire.toml the three content messages are declared as media-layer messages of their own, not as sections of the media packet: a content message belongs to the slice's content and may continue from one packet into the next (Content region), so it is not a part of one packet. The choice is also forced for the audio and dense messages, whose maximums exceed the 1 252 bytes the table checker applies to every section; the toml relation is provisional pending a tooling item on that checker rule (004-02) `[decision]`.

### Audio message

Kind 0 carries the slice's audio: for each channel, the slice's frames × 4 bytes, one sample per frame — 48 × 4 = 192 bytes per channel per default slice (C-02, C-03) — and for the six channels of a default stream (C-04) 6 × 192 = 1 152 bytes of audio, C-04's figure unchanged, because the layout adds nothing per channel `[decision]`. The whole message is 3 + 1 152 = 1 155 bytes in a default slice: the common part and the samples `[decision]`.

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| +0 | 1 | u8 | kind: 0, audio | 0 |
| +1 | 2 | u16 | length: frames × channels × 4, the samples that follow | 1 152 in a default slice (48 frames × 6 channels × 4); at most 65 535 |
| +3 | 4 | f32 or Q1.31, per the manifest | one sample; the samples run channel by channel, frames × 4 bytes per channel — 192 per channel per default slice (48 × 4) — channels in channel-map order, with no id byte | length / 4 samples: at most 16 383 (65 532 bytes); 1 to 341 channels at the default 48 frames |

There is no per-channel id byte: the channel map is in the manifest (004-26), and the channels appear in the message in channel-map order, the map's first channel first (C-04) `[decision]`. The POC's per-channel id table, one u8 per channel ahead of the samples, is retired `[code]`.

The samples are planar: each channel is one contiguous run of frames × 4 bytes, channel after channel, and within a run the samples follow frame order, the first at the slice's position (Slice header) `[decision]`. Planar rather than the POC's interleaved frames `[code]`, because a channel's run is the unit C-04 counts and the unit a consumer routes: a consumer that takes some of a stream's channels copies whole runs and never strides across frames `[decision]`.

Every sample is 4 bytes (C-03), f32 or Q1.31 as the stream's manifest declares it (004-26), encoded bit for bit as Bit-level encoding of samples fixes; there is no 3-byte variant `[decision]`. The format is signalled once per stream, in the manifest, and never in the message: the message has no format field, so no packet carries a format value that could be invalid, and no sample value is malformed — every 32-bit pattern is an f32 (NaN payloads carried bit-exact) or a Q1.31 value `[decision]`. There is no sample-rate field either: the wire has one rate, 96 000 Hz (C-01); an edge whose hardware runs at 48 kHz converts 2:1 at the edge (C-80), through a resampler node that may appear only inside an edge module (C-134), so every audio message is at the domain rate `[decision]`.

The message carries no channel-count field: the channel count is the length divided by frames × 4, and it must equal the number of channels in the stream's channel map (Validity rules) `[decision]`. An audio message carries at most 341 channels at the default 48 frames, because the length is a u16 — at most 65 535 body bytes — and 65 535 / 192 = 341 whole channels; in general at most ⌊65 535 / (frames × 4)⌋ channels, and a stream whose frames × channels × 4 would exceed 65 535 has no v1 wire form, which the compiler refuses from the manifest (six channels exceed it from 2 731 frames on) `[decision]`. The slice header's frames-against-length rule (Slice header, Validity rules) compares this message: the audio message's length is what must equal frames × channels × 4, over the reassembled slice, since the content region also holds the common part and, in a source that shares its packet (C-181), controller messages; a valid default packet's 1 155-byte content region therefore passes it `[decision]`.

Maximum size: the length is at most 65 535 and a whole number of 4-byte samples, so an audio message is at most 3 + 65 532 = 65 535 bytes, the bound a parser sizes for; at the default 48 frames the most is 341 channels, 3 + 341 × 192 = 65 475 bytes. In wire.toml the sample run is one 4-byte element counted by the length, `max_count = 16383`, so the declared maximum is the true one `[decision]`.

### Dense controller message

Kind 1 carries dense controllers: controller signals with a value at every frame, or at every stride of frames, in every slice; a dense controller is never absent from a slice its stream carries (01-architecture.md § 3.2) `[decision]`.

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| +0 | 1 | u8 | kind: 1, dense controllers | 1 |
| +1 | 2 | u16 | length: 3 + count × (frames / stride) × 4, the body that follows | 195 for one controller at one value per frame in a default slice; at most 65 535 |
| +3 | 1 | u8 | count: how many dense controllers this message carries | 1 to 255 |
| +4 | 2 | u16 | stride: frames per value; 1 is one value per frame, and a larger stride is the stated sub-rate | 1 to frames, dividing frames: 1, 2, 3, 4, 6, 8, 12, 16, 24 or 48 in a default slice |
| +6 | 4 | f32 or Q1.31, per the manifest | one value; the values run controller by controller, (frames / stride) × 4 bytes per controller — 192 at one value per frame in a default slice | count × (frames / stride) values: at most 16 383 (65 532 bytes); count at most 255 |

Width and format: each value is 4 bytes wide, in the stream's sample format, f32 or Q1.31 as its manifest declares it (004-26, C-03) — one width and one format for audio and controllers, so a single-stream source that shares its packet (C-181) uses one format throughout and a Q1.31 consumer needs no float path `[decision]`. In a Q1.31 stream a value lies in [−1, +1); how a controller's natural range maps onto it is agreed through the manifest, not carried on the wire `[decision]`.

No controller id: the dense messages of a slice carry the stream's dense controllers in the manifest's controller-map order (004-26), each exactly once — the first dense message starts at the map's first dense controller, each following dense message continues where the previous one ended, and together they carry every dense controller of the map `[decision]`. A producer that runs controllers at different rates sends one dense message per run of consecutive controllers sharing a stride `[decision]`.

Sampling: value k of a controller, k from 0, is its value at frame k × stride of the slice — the slice's position plus k × stride samples; stride 1 is one value per frame, and a stride s above 1 is the stated sub-rate of 96 000 / s values per second (C-01), 2 000 per second, one value per slice, at stride 48 in a default slice `[decision]`. What a consumer does between two sub-rate values is the module runtime's (epic 014), not the wire's `[decision]`.

The message carries no initial-value field: each dense controller declares its initial value in the manifest (default 0, C-158, laid out by 004-26), used when a gap precedes any received value `[decision]`.

Maximum count: at most 255 dense controllers per message (the u8 count) and at most 65 535 body bytes (the u16 length); at one value per frame in a default slice a full message is 3 + 255 × 192 = 48 963 body bytes, and a stream with more dense controllers sends further dense messages, which continue the map `[decision]`. In a default slice one controller at one value per frame costs 6 + 192 = 198 bytes, and one at one value per slice 6 + 4 = 10 bytes `[decision]`. The body is 3 + values × 4 ≤ 65 535, so at most 16 383 values and a message of at most 6 + 65 532 = 65 538 bytes, the bound a parser sizes for (wire.toml: one 4-byte element, `max_count = 16383`); at one value per frame in a default slice the 255-controller figure is 6 + 255 × 192 = 48 966 bytes `[decision]`.

### Where controllers travel

The controllers of a bundle source always ride in their own member stream: an audio member stream of a bundle source carries no controller message, and the controller member stream carries no audio (C-205) `[decision]`. The controller member counts towards the 32 members of C-08, so a source with controllers has at most 31 audio members (C-209) `[decision]`.

A single-stream source may always put its controllers in a second stream, and must do so when its headers plus its controller messages exceed the 100-byte cap of C-137; while they fit, it may share the audio packet, its controller messages following the audio message (C-181) `[decision]`. The controller second stream carries controller messages and no audio, and the compiler checks the choice from the manifest (C-181) `[decision]`.

A controller second stream whose controllers would have fitted in the audio packet is valid: no rule of this section rejects a second stream because its controllers would have fitted, and a receiver never compares a second stream's controllers with the room left in the audio packet (C-181, permissive) `[decision]`.

The shared-packet count is the parts the C-181 fit test can rely on, and it applies only to a default-regime packet that carries controller messages beside its audio message: the packet header (16 bytes), the slice header (22), the announcement container, the measurement extension, the audio message's 3-byte content-message header, every controller message whole with its own content-message header, so that no header is counted twice — sparse keyed and transport state messages included (Sparse keyed controllers and transport state) — and the checksum (4) (C-137, C-181) `[decision]`. The compiler counts the container as one announcement's room (1 + 12 = 13) and the extension (9) whenever the stream could carry it; a receiver counts the container as carried with a floor of one announcement, 1 + 12 × max(1, k) for k announcements, and the extension when present, so the two counts agree on every conforming packet, which carries at most one announcement because every stream carries each kind only in its own slot class of slices (Size budget, Shared packets and the controller allowance) `[decision]`. With the measurement extension, 16 + 22 + 13 + 9 + 3 + 4 = 67 counted bytes leave 100 − 67 = 33 bytes for controller messages: one dense message of up to 6 controllers at one value per slice (6 + 6 × 4 = 30 bytes); without it 58 counted bytes leave 42, up to 9 such controllers (6 + 9 × 4 = 42 bytes); a single controller at one value per frame (198 bytes) never shares a default packet `[decision]`. A packet that carries no controller message is never rejected under this count, whatever its announcements and its extension occupy `[decision]`. The count is final and the same on every path; Size budget states it with the allowance table and the proof that a packet within it fits a fallback path `[decision]`.

A controller stream's wire cost is computed like an audio stream's — one packet per slice at 2 000 packets/s, headers up to the 100-byte cap plus the controller bytes per slice — as 004-15 states (C-219) `[decision]`.

### Budget (C-137)

The default packet carries one audio message: 3 + 6 × 48 × 4 = 3 + 1 152 = 1 155 bytes of content (C-04) `[decision]`. Against the 100 bytes C-137 allows for all headers and the checksum, the audio message's 3-byte common header joins the parts already placed: 16 (packet header) + 22 (slice header) + 1 (announcement count, always present) + 9 (measurement extension) + 3 (content-message header) + 4 (checksum) = 55 bytes with no announcement, a 1 207-byte UDP payload; and, as format arithmetic for packets no v1 sender builds since it carries at most one announcement per packet (Size budget), with three announcements 55 + 36 = 91 bytes and with four announcements and no extension 16 + 22 + 49 + 3 + 4 = 94 bytes — each inside the cap `[decision]`. At maximum occupancy — four announcements and the extension — the sum is 16 + 22 + 49 + 9 + 3 + 4 = 103 bytes, 3 bytes over the cap, and 103 + 1 152 = 1 255 bytes of UDP payload, 3 bytes over the 1 252 bytes of a fallback path (C-06, C-17), which a packet sent with don't-fragment (rule 11) cannot cross; on a 1 472-byte path it fits with 217 bytes to spare `[decision]`.

The overflow contradicted four statements written before a content-message header existed — in Announcements, Maximums and Sender obligations, in Measurement extension, Byte size and the header cap, and in Checksum, Placement and the header cap, whose 96 + 4 = 100 bytes left no room for any content-message header — and this section first recorded it without an omission rule of its own `[decision]`. Size budget closes it with option (i) below, the stagger, applied to every stream: each announcement kind rides only in its own slot class of slices, so a packet carries at most one announcement and 16 + 22 + 13 + 9 + 3 + 4 = 67 bytes stay within the 100-byte cap, where the format's maximum of 16 + 22 + 49 + 9 + 3 + 4 = 103 would not; no sender needs to omit the measurement extension or an announcement, on a fallback path or anywhere, and the container's receiver cap of 4 (004-09) stays `[decision]`. Each of the four sections now states the amended rule and cites Size budget, and the Media packet header's account of the 84 bytes gives the 87 they would need at the format's maximum `[decision]`. A packet with no controller message is never rejected for the overflow: the cap row of Validity rules counts only shared controllers' packets (Where controllers travel) `[decision]`.

**Open item (header cap) — closed, answered by Size budget.** The record below is kept as it stood; the answer is option (i), the stagger, applied to every stream as Size budget's announcement schedule, with its final shared-packet count `[decision]`. A default packet of a measuring stream with all four announcement kinds pending needs 103 bytes where C-137 allows 100. Owner: ticket 004-14, which owned the size budget and the 100-byte cap of C-137. Three rules would close the 3-byte gap, and choosing among them was 004-14's: (i) stagger — a packet that carries a content message carries at most three announcements (16 + 22 + 37 + 9 + 3 + 4 = 91 bytes), and when four kinds are pending the fourth rides on its own 16-slice schedule one slice later (C-44), its position picked one lead ahead of the slice that first carries it (C-43); this lowers 004-09's effective cap from 4 to 3 per packet and needs its cap argument restated; (ii) apply the fallback clause 004-09 already states — announcements that do not fit are omitted on the fallback path (C-137) and re-issued from a later slice (C-44); omitting one 12-byte announcement frees 12 ≥ 3 bytes, but the omitted re-issue then misses its 16-slice step on fallback paths, against Sender obligations; (iii) reclaim 3 bytes inside the headers already placed, a change to a settled layout. The same choice settles C-181's fit test: the compiler checks the fit from the manifest (C-181), and counted at maximum occupancy the room for shared controllers is 100 − 103 < 0 bytes, so no controller would ever share an audio packet, while counted without announcements, which C-137 lets a fallback path omit, it is 45 bytes — the count the shared-packet cap row of Validity rules applied until Size budget replaced it with one announcement's room, 33 bytes for a stream that could carry the extension and 42 for one that could not (Where controllers travel). Falsifier: under the rules Size budget states, a v1 sender emits a default packet whose headers, content-message headers and checksum exceed 100 bytes — 1 255 bytes of UDP payload at maximum occupancy, which a fallback path cannot carry — or re-issues a pending announcement less often than every 16 slices (C-44), or first announces a change with less than one lead's notice (C-43) `[decision]`.

Inputs from 004-12 to this item, as the final count settles them: a transport state message is 30 bytes and a sparse keyed controller message 4 + 8 × count bytes, 12 to 1 028; both count as controller messages under the shared-packet count, so the 33 bytes of a stream that could carry the measurement extension hold a transport state message alone or three sparse entries alone, and the 42 bytes of a stream that could not hold a transport state message with one sparse entry beside it, or four alone; the earlier figures of 45 and 54 bytes counted no announcement room (Sparse keyed controllers and transport state; Size budget) `[decision]`.

### Validity rules

Every malformed content message is dropped with its slice and counted under one member of the closed Rejection reasons enumeration — one count per packet of the slice, as for BadChecksum — or skipped, for the one form that is not a rejection; nothing is dropped silently `[decision]`. The content rules run on the reassembled slice after its checksum verifies, so a slice corrupted in transit counts under BadChecksum and a content rejection names a slice its sender built wrong `[decision]`. The invalid forms and their required outcomes:

| Invalid form | Required outcome |
|---|---|
| unknown kind within the same major version (a kind code outside 0 to 3): the receiver steps over the message by its length and reads on with the next | skip by length — the message is ignored and nothing is counted; never a rejection (004-04) `[decision]` |
| message overrunning the slice: fewer than the common header's 3 bytes remain in the content region, or a message's length runs past the end of the content region (in the last packet, into the checksum) | drop and count under BadLength |
| audio length not matching frames × channels × 4: the audio message's length is not a whole number of channel runs of frames × 4 bytes | drop and count under BadLength |
| channel count different from the manifest's channel map for the stream: the length is a whole number of channel runs, but not as many as the channel map holds | drop and count under BadLength `[decision]` — the message has no channel field, so the length is where the count disagrees |
| sample-format value invalid: no content message carries a sample-format field — the format is declared once per stream in the manifest (004-26) | none — the case cannot occur on the wire; a manifest's format value is checked at admission (004-18, 004-26), never per packet `[decision]` |
| audio in a gap slice: a gap slice is header-only (rule 10), so any content in it is the packet header's gap-with-payload case | drop and count under BadLength, the packet header's rule (Media packet header, Validity rules); no content rule is reached |
| a dense controller message in an audio member stream of a bundle source, or an audio message in its controller member stream (C-205) | drop and count under BadLength `[decision]` — the content disagrees with the stream's declared role |
| a default-regime packet carrying controller messages beside its audio message, whose headers plus controller messages exceed the 100-byte cap (C-181, C-137): the shared-packet count of Where controllers travel — the announcement container counted as carried with a floor of one announcement, 1 + 12 × max(1, k) bytes for k announcements, the measurement extension when present — is above 100 (final: Size budget, Shared packets and the controller allowance) | drop and count under BadLength `[decision]` — a packet with no controller message is never rejected under this row |
| audio in a single-stream source's controller second stream (C-181) | drop and count under BadLength `[decision]` |
| no audio message in a non-gap slice of an audio stream, a second audio message, or an audio message that is not the first content message | drop and count under BadLength `[decision]` |
| dense count of 0, stride of 0, or a stride that does not divide the slice's frames | drop and count under BadLength `[decision]` |
| dense length not 3 + count × (frames / stride) × 4 | drop and count under BadLength |
| the dense messages of a slice carry more or fewer controllers than the stream's controller map holds | drop and count under BadLength `[decision]` |

Why BadLength for the role and map forms: the closed enumeration has no content-specific member, and a new one needs a major version (Rejection reasons, Closed enumeration). BadLength is the declared-shape member — the slice header already uses it for a packet count of 0 and the container for a duplicate kind — and a content region that disagrees with the shape the manifest declares for its stream (its role, its channel map, its controller map) is that case `[decision]`.

### Open items

**Open item (dense field set).** No constant gives the field set of dense controllers; this section decides it — per message a count and a stride, then 4-byte values in the stream's sample format, in controller-map order, with no controller id, no initial value (C-158) and no per-controller format — and records it as open `[decision]`. Owner: epic 004 (004-26 lays out the controller map and the initial values the field set relies on). Falsifier: a producer cannot express a value it must send without a major change — for example a controller whose values cannot be mapped into Q1.31's [−1, +1) in a Q1.31 stream, one that needs a format other than its stream's, or a rate that is not a whole divisor of the slice's frames; any such case means the field set is wrong, since a field added to the dense body is a major change, while a new kind code for a richer dense message is a minor one (004-04) `[decision]`.

The header-cap overflow, the second open item of this section, is closed: Size budget answers it (Budget) `[decision]`.

### Sparse keyed controllers and transport state

Two content messages complete the four of media rule 5: the sparse keyed controller message, kind 2, and the read-only transport state message, kind 3 (Kind codes) `[decision]`. Both start with the 3-byte common content-message header, both follow the audio message in a slice that carries audio, and both have a fixed maximum size, so a `no_std` parser bounds its work before it starts and allocates nothing (Field-table format, rule 2) `[decision]`. Offsets are relative to the message's own first byte, written +0 onward, as in the rest of this section; little-endian (Byte order), packed with no padding (Alignment and padding) `[decision]`.

#### Sparse keyed controller message

Kind 2 carries sparse keyed controllers: each entry moves one field of one voice to a target value over the slice, and a (voice, field) pair with no entry in a slice holds its value through that slice `[decision]`. The POC's receiver did the same: it kept a target and a current value per voice and field, set a target only when a part arrived, and otherwise moved each field from its current value to an unchanged target, which holds it (music.rs, `part` and `render`) `[code]`. A sparse keyed message carries at most 128 entries, behind a 1-byte count `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| +0 | 1 | u8 | kind: 2, sparse keyed controllers | 2 |
| +1 | 2 | u16 | length: 1 + count × 8, the body that follows | 9 to 1 025 |
| +3 | 1 | u8 | count: how many entries follow the count field | 1 to 128 |
| +4 | 8 | byte blob of count elements | entries: count entries back to back with no separator, each laid out as the Sparse keyed entry table fixes | count entries, at most 128 (1 024 bytes) |

The sparse keyed message therefore has a fixed maximum size of 3 + 1 + 128 × 8 = 1 028 bytes, the bound a parser sizes for; in wire.toml the entries are one 8-byte element counted by the count, `max_count = 128`, so the declared maximum is the true one `[decision]`.

Why 128: the u8 count would allow 255, but 4 + 255 × 8 = 2 044 bytes exceeds the 1 252 bytes of UDP payload that the 1 280-byte fallback path MTU leaves (C-06, C-100), so such a message could never travel in one packet of a default slice, which fits one packet (C-05) `[decision]`. 128 is the largest power of two whose message fits one such packet whole: 1 028 bytes of message plus the 100 bytes C-137 allows for all headers and the checksum make 1 128 bytes, inside the 1 252-byte payload with 124 bytes to spare `[decision]`. The bound also caps a receiver's per-message work at 128 entry decodes and 128 duplicate tests against its per-stream (voice, field) table, which it already holds to render the ramps, so parsing stays bounded and allocation-free `[decision]`. In a single-stream source's shared audio packet the 100-byte cap binds long before 128 (Where sparse keyed and transport state messages travel) `[decision]`.

A slice may carry several sparse keyed messages, in any order after the audio message; a count of 0 is invalid, because a slice with no change for any (voice, field) simply carries no sparse keyed message `[decision]`. In the default regime a controller stream's slice is one packet (C-05) with 1 252 − 100 = 1 152 content bytes, so beside one full 1 028-byte message the remaining 124 bytes hold a further sparse keyed message of 15 more (4 + 15 × 8 = 124 bytes), or a transport state message (30 bytes) and a further message of 11 more (30 + 4 + 11 × 8 = 122 bytes): 143 sparse targets per default-regime controller slice, 139 beside a transport state `[decision]`. A producer that needs more in one slice uses the explicit, stamped multi-packet regime (C-05, Slice header, Multi-packet regime), where the messages lie over the concatenated content regions `[decision]`.

#### Sparse keyed entry

The entry is 8 bytes: voice key 1, field id 1, start offset 2 and target value 4, tiling bytes +0 to +7 with no gap and no reserved bits `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| +0 | 1 | u8 | voice key: which voice of the stream the entry moves; a key the producer allocates, for example per sounding note | 0 to 255; every value is a valid key |
| +1 | 1 | u8 | field id: which field of the voice the entry moves, an index into the stream's sparse field map in the manifest (004-26) | a field id the stream's sparse field map declares; 0 to 255 at most |
| +2 | 2 | u16 | start offset: the frame of the slice at which the ramp starts, counted from the slice's first frame (Slice header, position) | 0 to frames − 1: 0 to 47 in a default slice (C-02) |
| +4 | 4 | f32 or Q1.31, per the manifest | target value, in the stream's sample format: the value the field reaches at the slice's last frame; the value format is the stream's, declared once in the manifest (004-26, C-03), never per entry | any 32-bit pattern: an f32 (NaN payloads carried bit-exact) or a Q1.31 value in [−1, +1) |

The starting point is the POC's 8-byte musical-event `Part` (proto.rs, `PART = 8`, `struct Part`): type in the high 4 bits of byte 0, a reserved byte 1, voice id byte 2, field byte 3 and a 32-bit value in bytes 4 to 7, taking effect when received `[code]`. v1 keeps the 8 bytes, the 1-byte voice and field and the 4-byte value; the type moves to the common header's kind, and the two freed bytes become the 16-bit start offset, which spans every frame of the longest slice (1 to 65 535 frames, Slice header), so an entry acts at a stated frame instead of on arrival `[decision]`.

Value format: the target is 4 bytes in the stream's sample format, f32 or Q1.31 as its manifest declares (004-26, C-03), the rule the dense controller message already follows, so a stream uses one format for audio and every controller and no entry carries a format value that could be invalid `[decision]`. In a Q1.31 stream a target lies in [−1, +1), and how a field's natural range maps onto it is agreed through the manifest, not carried on the wire `[decision]`.

Widths: the voice key and the field id are one byte each, as in the POC, so a stream addresses at most 256 voices × 256 fields = 65 536 (voice, field) pairs and a receiver's per-stream state table has a fixed bound `[decision]`. The voice key is open within the byte — voices are keys the producer allocates, not entries of a map — while the field id must name a field of the stream's sparse field map (004-26), which also gives each field's initial value, the value a field holds before any entry has reached it `[decision]`.

The ramp: an entry with start offset o and target t moves its (voice, field) by a linear ramp over the rest of the slice, from the value h it holds at the slice's start to t at the slice's last frame `[decision]`. At frame f of a slice of frames F (0 ≤ f < F, each frame one sample at 96 000 Hz, C-01; F = 48 in a default slice, C-02) the field's value is h for f < o, and h + (t − h) × (f − o + 1) / (F − o) for o ≤ f ≤ F − 1, so the ramp reaches t exactly at frame F − 1; with o = 0 the field moves over the whole slice, and with o = F − 1 it steps to t at the slice's last frame `[decision]`. With o = 0 this is the POC renderer's line, `along = (n + 1) / frames` over the block (music.rs, `render`) `[code]`. h is the value the receiver's (voice, field) held at the end of the previous slice it rendered — the previous entry's target once that entry's ramp has completed, or the manifest's initial value before any entry — never a value carried in the entry `[decision]`. The ramp therefore starts from the previous target, not from a start value on the wire, and the rendering is deterministic in the entries a receiver has received: a receiver that missed the previous slice ramps from its own held value and ends at the same t on the same frame as every other receiver `[decision]`. The frame-by-frame values and their rounding in each sample format are the kit's linear ramp (024a-10), started at frame o with current value h, target t and F − o frames, whose fixed-point ramp lands on the target exactly at its last frame `[decision]`.

#### Sparse refresh obligation

The sender refreshes sparse keyed state, and the obligation covers the active pairs only: a (voice, field) becomes active when an entry for it is sent, and every active pair is carried again by an entry at least once in every 16 consecutive non-gap slices the stream sends `[decision]`. An entry that changes a pair counts as its refresh; a refresh entry of an unchanged pair has start offset 0 and the pair's current value as its target, so a receiver that already holds that value renders a flat line, while one that diverged ramps back to it within the slice `[decision]`. No constant gives this interval: 16 non-gap slices — 8 ms at the C-02 defaults — is chosen equal to the announcements' re-issue cadence (C-44), so a receiver repairs a lost announcement and a lost sparse entry on one horizon, and the POC's sender already filled spare room with refreshes of the current values taken in rotation (music.rs, `Queue::fill`) `[code]`. Refresh entries obey every rule of this section — at most one entry per (voice, field) per slice, the 128 per message, the shared-packet cap — and cost the stream's capacity like any controller bytes (C-219): a producer that keeps S pairs active sends at least S / 16 entries per non-gap slice on average `[decision]`.

Retirement: the sender may retire an active pair once its current value equals the initial value the stream's manifest declares for that field (004-26), it has been refreshed at that value at least once after reaching it, and 16 non-gap slices have been sent since the slice whose entry reached it `[decision]`. After retirement the pair is not refreshed until a new entry activates it again, and sender and receivers alike treat it as holding its initial value; a receiver that received the entry that reached the initial value, or any later refresh at it, already holds that value, because every such ramp ends at the initial value on its slice's last frame `[decision]`. A producer that gives voice keys out per sounding note retires a released note's pairs by sending each field's initial value — for a gate whose initial value is 0, the release itself — and refreshing it once more, so its active set is bounded by the notes sounding or released within the last 16 non-gap slices, not by the 256 keys `[decision]`.

After a lost entry a receiver's value diverges from the sender's, is not marked degraded beyond the gapped slice itself (C-122), and the divergence is bounded only by the refresh interval: for an active pair it ends at the last frame of the first slice received with an entry for that (voice, field), no later than 16 non-gap slices after the lost one when no further slice is lost `[decision]`. One divergence remains after retirement: a receiver that lost every slice carrying the pair's entries at its initial value — the slice that reached it and every later slice that refreshed it, at least two slices — still holds a stale value once the sender has retired the pair, until the next entry for that pair activates it again; those losses need not be consecutive, so the interrupted threshold of C-38 (8 consecutive missed slices) does not catch the case, and the stale value is not marked degraded beyond the gapped slices themselves (C-122) `[decision]`. Epic 004 accepts this residue as the price of a bounded active set and owns it through the falsifier below; a producer that wants it smaller refreshes a retiring pair at its initial value more often before retiring it `[decision]`. This is the repair by state refresh that 01-architecture.md § 4.6 gives sparse keyed controllers, and the bound 025a-16 needs to say when normal output returns after a gap `[decision]`.

The per-window capacities for active pairs are stated limits of each regime, not obligations a producer must meet inside one: 48 in a shared audio packet of a stream that could carry the measurement extension (three per slice over 16 non-gap slices) and none there beside a transport state, 64 and 16 (four and one per slice) in one that could not (Size budget), 2 288 in a default-regime controller stream (143 per slice) and 2 224 there beside a transport state (139 per slice); a source whose active pairs exceed its shared-packet capacity carries its controllers in a second stream (C-181) `[decision]`. Falsifier of the refresh rule, owned by epic 004: a producer cannot keep its active pairs refreshed in any regime it is allowed — a shared packet, a controller second stream or controller member (C-181, C-205), or the stamped multi-packet regime — chosen when its stream is compiled, without changing regime mid-stream; or 025a-16 finds that a 16-slice divergence after a loss is audible where its tests require normal output; or the stale value left by a lost retiring window is observed in a conformance or field test where normal output is required; any of these means the interval, the obligation or the retirement rule is wrong, and changing them is a change of this rule `[decision]`.

#### Transport state message

Kind 3 carries read-only transport state: playing or stopped, meter, tempo, the playhead in beats and the time code position, each at the slice's first frame, plus the bar origin from which bar and beat are derived — what 023-08, 023-24 and 023-30 read as "playing, bar, beat, tempo, time code at position p" and 025a-05 as play/stop state, tempo and playhead position `[decision]`. The transport state message is 30 bytes on the wire — the 3-byte common header and a 27-byte body: state 1, numerator 1, denominator 1, tempo 4, beat playhead 8, time code position 8, bar origin 4 `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| +0 | 1 | u8 | kind: 3, transport state, read-only | 3 |
| +1 | 2 | u16 | length: 27, the body that follows | 27 |
| +3 | 1 | u8 | state: 0 stopped, 1 playing | 0 or 1 |
| +4 | 1 | u8 | meter numerator: beats per bar | 1 to 255 |
| +5 | 1 | u8 | meter denominator: the note value of one beat, 4 for a quarter note | 1, 2, 4, 8, 16, 32, 64 or 128 |
| +6 | 4 | u32 | tempo in beats per minute, unsigned fixed point Q16.16: one unit is 1/65 536 beat per minute, so 7 864 320 = 120 beats per minute (C-199) | 1 to 4 294 967 295: 1/65 536 to just under 65 536 beats per minute |
| +10 | 8 | i64 | beat playhead: the transport's position in beats at the slice's first frame — the frame the slice header's position names on the domain line — signed fixed point Q32.32, one unit 2^-32 beat | any 64-bit value: −2^31 to just under 2^31 beats; a negative value is a pre-roll |
| +18 | 8 | i64 | time code position: the transport's timeline position at the slice's first frame, in samples at 96 000 Hz (C-01), advancing one per frame while playing | any 64-bit value; a negative value is a pre-roll |
| +26 | 4 | i32 | bar origin: the beat, in whole beats, at which bar 0 starts under the current meter | any 32-bit value |

The per-stream byte rate is one 30-byte message per non-gap slice: 30 × 2 000 = 60 000 bytes/s, 480 kbit/s, at the C-02 defaults, a cost the capacity figures count like any controller bytes (C-219) `[decision]`.

Derived values, exact and integer, for a receiver that reads the message of a slice and needs a frame n of that slice or, through a gap, a frame n positions after that slice's first frame (n ≥ 0) `[decision]`. Beat playhead: while playing, P(n) = P + ⌊n × T × 65 536 / 5 760 000⌋, P the carried beat playhead, T the carried tempo units and 5 760 000 = 60 × 96 000 (C-01), the product computed exactly in 128-bit unsigned arithmetic, which holds it for every n below 2^64; the floored quotient is reduced modulo 2^64 and added to P with wrapping two's-complement arithmetic, so P(n) is defined for every such n and identical on every receiver, and a wrap means only that the playhead passed ±2^31 beats; while stopped, P(n) = P `[decision]`. Time code position: C + n while playing, also with wrapping two's-complement addition, C while stopped `[decision]`. Bar: B(n) = ⌊(P(n) − R × 2^32) / (N × 2^32)⌋, rounded towards minus infinity, and beat within the bar P(n) − R × 2^32 − B(n) × N × 2^32 in Q32.32, R the bar origin and N the numerator; the difference P(n) − R × 2^32, which can exceed the i64 range, and the division and remainder are computed in 128-bit signed arithmetic, where every intermediate fits `[decision]`. One tempo unit advances the beat playhead by 65 536 / 5 760 000 ≈ 0.011 4 Q32.32 units per frame, 89 478.485… at 120 bpm, never an integer step, which is why the rule floors the product over n rather than accumulating a rounded step per frame: every receiver that applies it gets bit-identical values, and the `TransportState` type that carries them to a module is epic 010's and the gap code that applies them 024b-08's `[decision]`.

Carried, derived or dropped, field by field of the POC's 40-byte `State` (transport.rs, `struct State`), its `Pulse` and its `Report` `[code]`:

- `flags` (`PLAYING`): carried as the state byte; every 023 reader needs playing `[decision]`.
- `num`, `den` (time signature): carried; the numerator is what derives bar and beat-in-bar for the metronome of 023-24, and the denominator gives the beat its note value, without which a 6/8 bar and a 6/4 bar carry the same beat counts `[decision]`.
- `beat` (playhead in beats, signed Q32.32): carried, same unit; beat and tempo are read by 023-24 and 025a-05, and beats cannot be derived from the time code position once the tempo has changed `[decision]`.
- `timeline` (playhead on the timeline, in samples): carried as the time code position; 023-30 reads time code as positions — time code 0 before arrival, advancing by exactly 96 positions over a 96-position gap — and it cannot be derived from beats once the tempo has changed `[decision]`.
- `bar`, `bar_beat` (bar number and the beat it starts at): derived; only bar_beat − bar × num matters, carried as the bar origin, from which bar and beat-in-bar follow by the rule above `[decision]`.
- `tc` (time-code rate index: 24, 25, 29.97 drop-frame or 30 frames/s): dropped; no 023 or 025a reader needs video frames — time code there is a position — and a frame label at a rate is a presentation of the time code position a receiver can compute for any rate it chooses `[decision]`.
- `at` (the media sample from which a state applied): dropped; the slice header's position anchors every message, which applies from its slice's first frame `[decision]`.
- `serial` (serial number of the change): dropped; every non-gap slice carries the whole state, so there is no change to number `[decision]`.
- POC `Pulse` (transport.rs `struct Pulse`, the beat clock: a beat at a media sample plus a fraction): retired; master to follower the POC carried no tempo — the tempo was the spacing of the pulses — and v1 carries the tempo explicitly, because C-141 requires a receiver to advance a playing transport through a gap at its tempo with no pulse to measure, and the beat at any frame then follows from the rule above `[code]`.
- POC `Report` (the follower's report, transport.rs `struct Report`, travelling back with each echo, which carried a tempo in thousandths of a beat per minute among its fields): retired, because transport state is read-only (below) `[code]`.

Tempo encoding: unsigned Q16.16 beats per minute, 65 536 units per bpm, so the C-199 default of 120 beats per minute is 120 × 65 536 = 7 864 320 units, exact `[decision]`. The resolution, 1/65 536 bpm, moves a playing playhead by under 3 × 10^-7 beat per second of a gap away from the true tempo, and the range reaches 65 535 bpm `[decision]`. Tempo 0 is invalid: a playing transport at tempo 0 cannot move, and a stopped one still names the tempo it resumes at, so the field always carries a positive tempo `[decision]`. A negative or non-finite tempo cannot occur, because the field is an unsigned integer `[decision]`. A receiver reads every field afresh in every slice and checks no continuity between slices, so a locate or a tempo change is just a new value, and the transport state machine that produces the values belongs to epics 005 and 006 `[decision]`.

A stream whose manifest declares that it carries transport state (004-26) carries exactly one transport state message in every non-gap slice, so every slice states the whole transport and no receiver depends on an earlier slice for it `[decision]`.

Transport state is read-only on the wire: authoring transport is deferred past v1 (01-architecture.md § 12.7), and a module consumes it as an input `[decision]`. A receiver never writes transport state back: no message of this protocol carries transport state from a receiver towards the stream's source, and the POC's follower report (transport.rs, `T_REPORT`, `struct Report`), which travelled back with each echo, is retired `[code]`.

#### Transport state at the receiver

Before any transport state has arrived on a stream, a receiver assumes the default: stopped, with the beat playhead and the time code position both at 0, tempo 120 beats per minute (C-184, C-199) `[decision]`. This is receiver behaviour, so the message has no unset value: every transport state message carries a real state, and the default lives only in the receiver; C-184 names no meter and no bar origin, so the values a receiver derives bar and beat from before any state has arrived are its gap default's (023-30, 024b-08), not the wire's `[decision]`.

On a gap the receiver holds the last known transport state — state, meter, tempo and bar origin held — and advances the playhead as that state dictates, a stopped transport staying where it is with its beat playhead and time code position unchanged (C-200), a playing one moving on at its tempo with both advanced by the rule above over the positions elapsed, and the output is marked degraded (C-141) `[decision]`. This too is receiver behaviour: a gap slice is header-only (rule 10) and carries no content message, so the transport state message needs no gap value `[decision]`.

#### Where sparse keyed and transport state messages travel

A transport state message counts as a controller message, exactly as a sparse keyed message does, for every rule of Where controllers travel, for the shared-packet cap row of Validity rules and for the Budget (C-137) `[decision]`. The reason: the 100-byte cap and the Budget count every content-message byte beyond the audio message, and an exempt transport state would let a packet exceed the cap without any controller message, or ride in an audio member of a bundle source outside the 55-byte account the Budget states for such a packet `[decision]`.

The controllers of a bundle source always ride in their own member stream (C-205): a sparse keyed or transport state message never appears in an audio member stream of a bundle source, only in its controller member stream `[decision]`. Because a transport state message counts as a controller message, a bundle source that carries transport state needs a controller member stream even when it has no other controller, and that member counts towards the 32 of C-08, so such a source has at most 31 audio members, 186 channels (C-209); the compile-time rejection is 023-32's `[decision]`. A single-stream source may share the audio packet while its headers and controller messages fit the 100-byte cap, and must put them in a second stream once they do not (C-181, C-137) `[decision]`.

Shared-packet arithmetic, on the final count of Where controllers travel (Size budget): a stream that could carry the measurement extension counts 67 bytes and has 33 left for controller messages, so a shared packet carries a sparse keyed message of three 8-byte entries alone (4 + 3 × 8 = 28 bytes) or a transport state message alone (30 bytes), and no sparse entry beside a transport state (30 + 4 + 1 × 8 = 42 > 33 bytes) `[decision]`. Without the extension 58 counted bytes leave 42: four entries alone (4 + 4 × 8 = 36 bytes), or a transport state with one entry beside it (30 + 4 + 1 × 8 = 42 bytes) `[decision]`. A source whose sparse changes and refreshes in a slice can exceed that uses a second stream (C-181), where the 100-byte cap does not bind its controller bytes and a whole 128-entry message fits one default packet (1 028 + 100 = 1 128 ≤ 1 252 bytes, C-06, C-100, C-137) `[decision]`. Neither message changes the Budget's 103-byte maximum-occupancy figure, which counts no controller message; Size budget states the allowance both share `[decision]`.

#### Validity of sparse keyed and transport state messages

Every malformed sparse keyed or transport state message is dropped with its slice — its audio message and every other content message with it — and counted under BadLength, one count per packet of the slice, the model of the section's Validity rules; there is no per-entry skip, because a partly applied set of changes would hide a sender's error behind a slice that renders differently from the one it built `[decision]`. The rules run on the reassembled slice after its checksum verifies `[decision]`. The invalid forms and their required outcomes:

| Invalid form | Required outcome |
|---|---|
| sparse entry count above the maximum: the count field exceeds 128 | drop the slice and count under BadLength |
| sparse entry count of 0: the message declares no entry | drop the slice and count under BadLength `[decision]` |
| sparse length mismatch: the length is not 1 + 8 × count, so the body disagrees with its count | drop the slice and count under BadLength |
| unknown field id: an entry's field id is not in the stream's sparse field map (004-26), including any field id in a stream whose map declares no sparse field | drop the slice and count under BadLength `[decision]` — see the note below |
| start offset beyond the slice's frames: an entry's start offset is at or above the slice's frames | drop the slice and count under BadLength `[decision]` |
| duplicate (voice, field) in one slice: two entries of the slice's sparse keyed messages name the same voice key and field id, so the slice declares two targets for one ramp | drop the slice and count under BadLength `[decision]` |
| a sparse keyed message in an audio member stream of a bundle source (C-205) | drop the slice and count under BadLength `[decision]` — the content disagrees with the stream's declared role |
| transport length mismatch: a transport state message whose length is not 27 | drop the slice and count under BadLength |
| transport state value invalid: the state byte is outside 0 and 1, the tempo is 0, the numerator is 0, or the denominator is not a power of two from 1 to 128 | drop the slice and count under BadLength `[decision]` |
| transport negative or non-finite tempo: the tempo is an unsigned integer | cannot occur — no bit pattern of the field is negative or non-finite, so no outcome exists `[decision]` |
| a transport state message in an audio member stream of a bundle source (C-205) | drop the slice and count under BadLength `[decision]` |
| transport message number wrong: no transport state message in a non-gap slice of a stream whose manifest declares transport state, two in one slice, or one in a stream whose manifest declares no transport state | drop the slice and count under BadLength `[decision]` |

**Why BadLength, and why no skip, for an unknown field id.** The closed enumeration has no content-specific member and a new one needs a major version (Rejection reasons, Closed enumeration), so BadLength, the declared-shape member, names every form above, as it does for 004-11's controller-map forms `[decision]`. An unknown kind and an unknown announcement type are skipped by their length because their code spaces belong to the protocol and are open to minor versions of the same major (Versioning, 004-04, 004-09); the field id space is not a protocol code space: it is fixed per stream by the admitted manifest (004-26), which sender and receiver both hold, and a minor version never adds a field id, so an id outside the map is a sender's error, not a newer sender's addition `[decision]`. An entry also has no length prefix of its own to be skipped by `[decision]`.

No value of the voice key, the target value, the beat playhead, the time code position or the bar origin is a malformed case: each accepts every bit pattern of its width `[decision]`.

#### Open items

**Open item (transport field set).** No constant gives the full field set of read-only transport state; this section decides it — state, meter numerator and denominator, tempo in Q16.16 bpm, beat playhead in Q32.32 beats, time code position in samples and bar origin in whole beats, a 27-byte body — and no stated need of 023-08, 023-24, 023-30 or 025a-05 (playing, bar, beat, tempo, time code at position p; play/stop state, tempo and playhead position) is unexpressible: each is carried or derived exactly by the rules above `[decision]`. Owner: epic 004; the review against the architecture specification in 004-22 confirms or reopens it. Falsifier: a producer cannot express a value it must send without a major change — for example a module that needs a time-code frame rate or a frame-exact SMPTE label, a meter change inside a slice, a bar origin at a fractional beat, a tempo outside 1/65 536 to 65 536 bpm or finer than 1/65 536 bpm, or a playhead beyond ±2^31 beats; any such case means the field set is wrong, since a field added to the transport body changes its fixed 27-byte length, which a v1 receiver drops under BadLength, while a new kind code for a richer transport message is a minor change (004-04) `[decision]`.

**Open item (sparse entry layout).** No constant gives the sparse keyed entry; this section decides it — 1-byte voice key, 1-byte field id, 16-bit start offset and a 4-byte target in the stream's sample format, with the linear ramp ending at the slice's last frame `[decision]`. Owner: epic 004 (004-26 lays out the sparse field map and the initial values the entry relies on; 004-22's review confirms the layout). Falsifier: a producer cannot express a value it must send without a major change — for example more than 256 simultaneous voices or 256 fields in one stream, or a field that must step at a frame inside the slice other than its last; any such case means the layout is wrong, since a change to the entry is a major change, while a new kind code for a richer sparse message is a minor one (004-04) `[decision]`.

## Checksum

The slice checksum is a 32-bit CRC-32C over the reassembled slice — the identity bytes, the announcement container and the content of every packet of the slice — carried once per slice, as the last 4 bytes of the UDP payload of the slice's last packet, so it trails the content as 01-architecture.md § 3.2 draws it (media rule 6; 01-architecture.md § 16: transport integrity, bytes delivered equal bytes sent, per slice) `[decision]`. The fields a sender or a hop writes per packet, per copy or per hop without changing the slice — the send timestamp, the packet index and count, the copy index and the measurement extension — are outside the covered bytes, so stamping, measuring and duplicating never require recomputing it `[decision]`.

### Algorithm

The algorithm is CRC-32C, the Castagnoli polynomial, in the parameterised (Rocksoft) model below; it is the catalogue entry CRC-32/ISCSI, and the check value is the CRC of the nine ASCII bytes "123456789" `[decision]`:

| Parameter | Value |
|---|---|
| Width | 32 bits |
| Polynomial | 0x1EDC6F41 (normal form; 0x82F63B78 reflected) |
| Initial value | 0xFFFFFFFF |
| Input reflection | true (each byte is processed least significant bit first) |
| Output reflection | true |
| Final XOR | 0xFFFFFFFF |
| Check value | 0xE3069283 |

The polynomial and its name are those of iSCSI: RFC 3720, § 12.1 HeaderDigest and DataDigest, lists CRC32C with generator 0x11edc6f41, and its Appendix B.4 CRC Examples gives 32 zero bytes the digest bytes aa 36 91 8a `[standard]`. The parameters above reproduce that vector — 0x8A9136AA, stored little-endian as aa 36 91 8a — and give the check value 0xE3069283: checks/004-13.sh recomputes the check value from the table with a bitwise implementation of these parameters on every CI run (Python 3, machine-independent integer arithmetic) `[measured]`. The 32-bit result is stored as a u32 in the wire's little-endian order (Byte order), so the stored bytes are exactly the RFC's digest bytes `[decision]`.

Why CRC-32C `[decision]`: a 32-bit width is the widest that fits the header cap (Placement and the header cap); among 32-bit CRCs the choice is between the Ethernet polynomial 0x04C11DB7 and Castagnoli's 0x1EDC6F41, and CRC-32C wins on the hosts that carry most of the media load. On x86-64 the SSE4.2 instruction computes exactly this polynomial — Intel 64 and IA-32 Architectures Software Developer's Manual, Volume 2A, instruction CRC32 (Accumulate CRC32 Value), which uses the polynomial 11EDC6F41H `[standard]` — and on 64-bit ARM the CRC32C instructions of the CRC extension do the same, optional in Armv8.0 and mandatory from Armv8.1 `[memory]` (the Arm Architecture Reference Manual clause is pinned by 004-22, as for the byte-order clause), so both machines of the pair (C-151) verify a slice in a fraction of a microsecond (Cost per slice).

Detection over a default slice, computed rather than recalled `[measured]`: the covered bytes of a default slice with no announcement are 14 + 1 + 10 + 1 + 1 152 = 1 178 bytes, and at most 1 178 + 48 = 1 226 bytes with four announcements, so a codeword — covered bytes plus the 32 check bits — is at most 1 230 bytes, 9 840 bits. checks/004-13.sh verifies on every CI run that the full generator 0x11EDC6F41 has an even number of terms, so x + 1 divides it and every error of an odd number of bits — every 1-bit and 3-bit error among them — is detected, and that the residues x^k mod G are pairwise distinct for every k below the codeword length, so no 2-bit error is a multiple of the generator. The checksum therefore detects every error of up to 3 bits and every odd-weight error in a default slice, with or without announcements. The comparison with other polynomials is recalled, not computed: in Koopman's CRC Zoo (32-bit table) Castagnoli's polynomial keeps Hamming distance 4 for every data word up to 2 147 483 615 bits and 6 up to 5 243 bits, while the Ethernet polynomial keeps distance 4 only up to 91 607 bits and lacks the x + 1 factor `[memory]` (read from the published table; 004-22 pins the citation).

The link's own checks do not replace it: the Ethernet frame check sequence covers one link and is recomputed by every switch (IEEE 802.3, clause 3.2.9, Frame Check Sequence (FCS) field), and the UDP checksum covers one datagram and is optional over IPv4 (RFC 768, Fields: an all-zero checksum value means the transmitter generated no checksum) `[standard]`; neither covers a reassembled slice or the buffers of the hosts on either end, which is what media rule 6 asks of this one `[decision]`.

### Covered range

The covered bytes — the checksum input — are, for each packet of the slice in packet-index order, that packet's parts below in the order listed, concatenated; the sender builds them from the packets exactly as it sends them on the path, and the receiver from the packets it reassembled. Every packet header, slice header and container byte is either covered or excluded by a row of this table, and no byte is covered twice `[decision]`:

| Part | Treatment | Reason |
|---|---|---|
| Packet header bytes 0 to 13: magic, version, domain id, stream number, session, timeline epoch | covered, 14 bytes of every packet, as on the wire | the slice's identity: a corrupted stream number, session or epoch would deliver correct content to the wrong stream or epoch, which no other rule detects; magic and version are constant and keep the run contiguous |
| Packet header byte 14, the flags | covered as one byte per packet, the flags byte AND 0xC0: gap (bit 7) and degraded (bit 6) as on the wire, bits 5..0 written zero for the computation | gap and degraded describe the content (C-122, rule 10); the announcement-present bit (5) is redundant with the covered count byte, the measurement-present bit (4) belongs to the excluded extension, and bits 3..0 are reserved zero |
| Packet header byte 15: copy index (C-07) and its 6 reserved bits | excluded | every copy of a slice must carry the same checksum, because nothing about a copy's content depends on its index (Reserved fields, Copy index); the reserved bits are zero by rule, so excluding them loses nothing |
| Slice header bytes 16 to 25: position and frames | covered, 10 bytes of every packet, as on the wire | the slice's address on the domain line and its length; a corrupted position would play correct samples at the wrong time, which no other rule detects |
| Slice header bytes 26 to 29: packet index and packet count | excluded | packetisation, not content: the index differs per packet, and both are already checked (BadPacketIndex, BadLength); the packet-index order of the concatenation carries the index's role |
| Slice header bytes 30 to 37: send timestamp | excluded | per packet: the pacer writes it at the moment of sending, after the checksum exists (Send timestamp; 01-architecture.md § 3.4) |
| Announcement container: the count byte and every announcement | covered, 1 + 12 × count bytes of every packet, as sent on the path | an announcement steers every consumer at a future position (an epoch switch, an end of stream), and a corruption that still passes the announcement's own validity rules — a flipped epoch-switch payload, a shifted end-of-stream position that stays in the future — would otherwise be acted on (Announcements, Sender obligations); covering it closes that gap |
| Measurement extension (004-10) | excluded | mutable per hop: each processor rewrites it from its critical input's values, and its residence time is measured up to the send timestamp, after the checksum exists (Measurement extension, Forwarding update rule) |
| Content region: the audio payload and every content message (004-11, 004-12) | covered, every byte as on the wire | the slice's content: 1 152 bytes of audio in a default slice (C-04) |
| The checksum field itself | excluded | it carries the result |

The content region of a packet is every byte after its measurement extension, or after its container when the measurement flag is clear, up to the end of the UDP payload — in the last packet, up to the checksum, 4 bytes before the end; its layout is 004-11's and 004-12's, and this section covers it byte for byte whatever that layout is `[decision]`. In the default regime the slice is one packet and the covered bytes are that packet's parts in the order above `[decision]`.

### Placement and the header cap

The checksum is placed in the last packet of the slice — the packet whose packet index equals packet count − 1 — and in no other packet; in the default regime the packet count is 1 (C-05), so every default packet is a last packet and carries it `[decision]`. One value per reassembled slice has one place: the receiver verifies after reassembly (Validity rules), so copies in earlier packets would add bytes and a new disagreement case without adding detection `[decision]`.

The checksum trails the content: it is the last 4 bytes of the UDP payload of the last packet, at offset payload length − 4, following the content region, as 01-architecture.md § 3.2 draws it (packet header, slice header, announcements, measurement extension, content, checksum) `[decision]`. A receiver knows the payload length from the datagram before it reads any byte, so the checksum's offset is known up front, and a sender appends it after the content it covers `[decision]`. In wire.toml the checksum is a section message of the media packet, the precedent of the announcement entry, because a trailing field has no fixed offset from the packet's start `[decision]`. A gap slice carries no checksum: it is header-only, exactly 38 bytes (rule 10; Slice header), and has no content to cover `[decision]`.

Before any content-message header was placed, the header cap held with no fallback rule: the packet header (16 bytes) + the slice header (22) + a maximal announcement container (49) + the measurement extension (9) = 96 bytes, and 96 + 4 = 100 bytes, exactly the 100 bytes C-137 allows for all headers and the checksum `[decision]`. The audio message's 3-byte content-message header (004-11) raises maximum occupancy to 103 bytes, 1 255 bytes of UDP payload, so the cap holds on every path by the announcement schedule of Size budget: a v1 sender carries at most one announcement per packet, 16 + 22 + 13 + 9 + 3 + 4 = 67 bytes of headers and checksum, and omits neither the checksum nor an announcement nor the extension `[decision]`. The checksum is never omitted to meet the cap; the media packet's declared maximum stays 96 bytes and the checksum section adds its 4 (wire.toml) `[decision]`.

### Field table

Little-endian (Byte order); the offset is relative to the end of the UDP payload of the slice's last packet `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| payload length − 4: the last 4 bytes of the UDP payload, last packet of the slice only, following the content | 4 | u32 | slice checksum: CRC-32C (Algorithm) of the slice's covered bytes (Covered range), stored little-endian | any 32-bit value; a value that differs from the CRC the receiver recomputes is BadChecksum |

### Who computes and who verifies

The checksum is computed by the sender of the slice — a source, or a processor or edge emitting its own output slice — at packetisation, over the covered bytes of the packets exactly as laid out for the path they are sent on; no hop carries a checksum through `[decision]`. Because the announcement containers are covered and a slice's covered layout is the same on every path — the announcement schedule is the stream's, the same on every path, and no path omits an announcement or the extension (Size budget); the packet count is per stream, fixed by the regime stamp (Slice header, Multi-packet regime), and never differs between paths — the sender computes one CRC per distinct layout, usually one per slice; how it computes it (one pass, or a content CRC combined with the per-layout header bytes) is the implementation's, as long as the result equals the CRC of the covered bytes `[decision]`. A processor verifies each input slice it receives and computes a fresh checksum for each output slice it emits, because its output is new content at a new position (01-architecture.md § 4.1) `[decision]`. Writing an excluded field never requires recomputing: the pacer writes the send timestamp, a processor writes the extension's updated counters, and a future duplicating sender writes the copy index, all after the checksum exists `[decision]`.

### Validity rules

The checksum is verified once per slice, after reassembly — in the default regime, on the single packet — and only after every per-packet rule of the packet header, slice header, announcement container and measurement extension has passed, so a packet rejected for another reason never reaches the checksum and every rejected packet counts under exactly one reason `[decision]`. Its invalid forms and their required outcomes:

| Invalid form | Required outcome |
|---|---|
| the stored checksum differs from the CRC-32C the receiver recomputes over the reassembled slice's covered bytes | drop the slice — every packet of it — and count each of its packets under BadChecksum on the input `[decision]` |
| the last packet's UDP payload is too short to hold its headers, its container, its extension when flagged and the 4-byte checksum: the body does not fill the shape its header declares | drop and count under BadLength, as for a missing measurement extension `[decision]` |
| the packets of one multi-packet slice disagree on an identity byte — session, frames, the degraded flag, or any other covered packet header or slice header byte — that no per-packet rule rejects | no separate comparison: every packet's identity bytes are covered, so the disagreement makes the recomputed CRC differ from the stored one; drop the slice and count each of its packets under BadChecksum `[decision]` |

A mismatch is dropped and counted under BadChecksum — one count per packet of the slice, so the per-input, per-reason counting of AC-23 sees every rejected packet — and never delivered: the handler receives a slice whole or not at all (01-architecture.md § 3.1) `[decision]`. Identity disagreements that a per-packet rule already rejects keep that rule's reason: a domain or epoch other than the consumer's (ForeignDomain, UnconfiguredDomain, WrongEpoch) and an inconsistent packet count (BadPacketIndex); the stream number and position are the reassembly key with the packet index (rule 9), so packets that disagree on them never meet in one slice `[decision]`. A slice that fails the checksum is not a valid copy (rule 9), so a later copy of the same slice may still be accepted `[decision]`. The frames-against-length rule of the slice header compares the audio message's length over the reassembled slice (Content messages, Audio message), so neither the 4 checksum bytes nor the content-message headers enter it `[decision]`. The verification is linear in the slice's length, inside C-164's bound per bad input together with the per-packet rules (Cost per slice) `[decision]`.

### Cost per slice

At the defaults a stream carries 1 152 bytes of audio per slice (C-04) at 2 000 slices/s (C-02): 1 152 × 2 000 = 2 304 000 bytes/s of audio, and with the 26 covered header and container bytes of a packet without announcements 1 178 × 2 000 = 2 356 000 covered bytes/s per stream `[decision]`. The estimates below rest on recalled instruction costs, not on runs `[memory]`:

| Target | Assumption | Per default slice (1 178 bytes) | Load per stream at 2 000 slices/s |
|---|---|---|---|
| x86-64 host with SSE4.2 | one dependent chain of 8-byte CRC32 instructions, 3 cycles each, ≈ 0.375 cycles per byte `[memory]` | ≈ 440 cycles, ≈ 0.15 µs at 3 GHz | ≈ 0.9 Mcycles/s, ≈ 0.03 % of one 3 GHz core; a 96-channel source's 32 000 packets/s (C-22) ≈ 14 Mcycles/s, ≈ 0.5 % |
| 64-bit ARM host with the CRC extension | same order as x86-64, 8 bytes per instruction `[memory]` | well under 1 µs | well under 1 % of one core |
| reference embedded target, software, byte-wise table (1 KiB) | ≈ 8 cycles per byte on a Cortex-M7-class core `[memory]` | ≈ 9 400 cycles: ≈ 20 µs at 480 MHz | ≈ 18.8 Mcycles/s, ≈ 3.9 % of 480 MHz |
| reference embedded target, software, slice-by-8 tables (8 KiB) | ≈ 2.5 cycles per byte `[memory]` | ≈ 2 900 cycles: ≈ 6 µs at 480 MHz | ≈ 5.9 Mcycles/s, ≈ 1.2 % of 480 MHz |
| reference embedded target, hardware CRC unit with a programmable polynomial | one 32-bit word per bus write, ≈ 295 writes, or fed by DMA `[memory]` | ≈ 1 µs or less | negligible |

Per-path computation on the sender `[memory]`: each distinct layout costs one more pass, ≈ 0.15 µs per slice on an x86-64 host; even a slice laid out differently for each of 63 consumers — the most a 64-module medianet has besides the sender (C-34) — costs 63 × 440 × 2 000 ≈ 55 Mcycles/s, ≈ 1.9 % of one 3 GHz core per stream, and the usual case is one layout per slice.

The reference board is not chosen yet (026b-01; 026a-01 builds for a generic Cortex-M target until then), and the reference device is a sink receiving one 6-channel stream (epic 026b), so its load is one row of the table, verification only `[decision]`. The budget this section sets is C-164's: the per-packet rules plus the verification of one default slice must take at most 10 µs on the reference board, which at 2 000 slices/s is at most 2 % of its CPU (10 µs × 2 000 = 20 ms per second); the CRC does not get all of it `[decision]`. On the estimates a byte-wise table alone exceeds the whole budget on any Cortex-M clock (it would need ≈ 940 MHz), slice-by-8 takes ≈ 6 µs at 480 MHz and leaves ≈ 4 µs for the per-packet rules, and a programmable hardware CRC unit leaves nearly all of it; a CRC unit fixed to the Ethernet polynomial cannot compute CRC-32C, so 026b-01 weighs a programmable unit or a clock rate at which slice-by-8 leaves the per-packet rules their share `[decision]`.

### Open items

**Open item.** The embedded cost above is an estimate, not a measurement. Owner: task 026 (026b, on the reference board once 026b-01 names it), which measures, with the implementation the firmware ships, the time to run the per-packet rules on a default packet and to verify one default slice — 1 178 covered bytes. Falsifier: the chosen algorithm is too slow on the reference embedded target — the per-packet rules plus the verification of one default slice exceed 10 µs (C-164), more than 2 % of the device's CPU at 2 000 slices/s, with every implementation the board offers (hardware CRC unit, slice-by-8 tables). A falsified estimate reopens the algorithm, and a different algorithm or width is a major change (Versioning, Minor and major changes) `[decision]`.

### Worked example

One small slice for task 007, chosen to be readable rather than default-sized `[decision]`: stream 3 of domain 1, session 0x12345678, timeline epoch 7, flags 0x70 (degraded, announcement present, measurement present), copy index 0, position 96 000, 2 frames of a 2-channel stream as f32 samples 0.5, −0.5, 0.25, −0.25 (16 bytes of content), packet index 0 of a packet count of 1, send timestamp 1 000 000 000 123 ns, one announcement — a timeline epoch switch (type code 2, length 10) to epoch 8 at position 102 144 — and a measurement extension of hop count 1, residence 1 500 ns and declared delay 48 samples.

Input (54 bytes) — the covered bytes in order, one line per part: packet header bytes 0 to 13, the masked flags byte (0x70 AND 0xC0 = 0x40), slice header bytes 16 to 25, the announcement container (count 1 and the 12-byte announcement), then the 16 content bytes `[decision]`:

```
4e 4d 00 01 01 00 03 00 78 56 34 12 07 00
40
00 77 01 00 00 00 00 00 02 00
01 02 0a 00 8f 01 00 00 00 00 00 08 00
00 00 00 3f 00 00 00 bf 00 00 80 3e 00 00 80 be
```

Expected checksum: 0xD75ABFE5, stored little-endian as e5 bf 5a d7. checks/004-13.sh recomputes it from the stated parameters on every CI run, and it was cross-checked with a second, table-free implementation of the Rocksoft model `[measured]`.

The whole 80-byte packet as sent: headers 0 to 37, the container 38 to 50, the extension 51 to 59, the content 60 to 75, and the checksum trailing at payload length − 4 = 76 `[decision]`:

```
 0: 4e 4d 00 01 01 00 03 00 78 56 34 12 07 00 70 00
16: 00 77 01 00 00 00 00 00 02 00 00 00 01 00 7b 10
32: a5 d4 e8 00 00 00 01 02 0a 00 8f 01 00 00 00 00
48: 00 08 00 01 dc 05 00 00 30 00 00 00 00 00 00 3f
64: 00 00 00 bf 00 00 80 3e 00 00 80 be e5 bf 5a d7
```

The same slice sent as copy 1, with another send timestamp, or with the measurement counters rewritten by a later hop carries the same checksum 0xD75ABFE5, because none of those bytes is covered, while a layout with a different announcement container would have its own checksum (Who computes and who verifies) `[decision]`.

## Size budget

This section adds a default media packet up part by part and states what a sender leaves out when the parts do not fit the path. C-05 makes a default slice one packet, and C-137 bounds it: a default slice must fit 1 252 bytes of UDP payload, 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum, and the in-band announcements and the measurement extension fit inside that cap or are omitted on a fallback path `[decision]`. Two path sizes are budgeted. IPv4 only in v1 (C-100): a 1 500-byte path leaves 1 472 bytes of UDP payload once the 20-byte IPv4 header without options and the 8-byte UDP header are taken (Layer overview: RFC 791, § 3.1; RFC 768) `[standard]`, and the 1 280-byte fallback (C-06) leaves 1 252 bytes (C-17, C-100) `[decision]`. C-137 is read as its two sentences separate it: the 100 bytes bind the headers and the checksum of every default-regime packet sent on a fallback path, the announcements and the extension being the parts a fallback path may leave out, while a packet on a 1 472-byte path is judged against 1 472 bytes, about 320 of them beside the audio (C-04) `[decision]`.

The parts and their sizes are those of the landed field tables and of wire.toml, not new figures `[decision]`: the packet header is 16 bytes (Media packet header); the slice header 22 (Slice header); the announcement container a 1-byte count, always present, plus 12 bytes per announcement, at most 4, so 1 to 49 bytes (Announcements, The container and Maximums); the measurement extension 9, optional (Measurement extension); one 3-byte content-message header, the audio message's, which is the only content message of a packet that shares nothing (Content messages, Common content-message header); the checksum 4, in every default packet (Checksum, Placement and the header cap); and the audio 6 × 48 × 4 = 1 152 bytes, 192 per channel (C-04, C-02, C-03). The mandatory parts, present in every non-gap default packet, are 16 + 22 + 1 + 3 + 4 = 46 bytes; with every optional part at its maximum the bytes beside the audio are 46 + 48 + 9 = 103 `[decision]`.

### Budget on a 1 472-byte path

Bytes per part of one default packet of a 6-channel stream, against the 1 472-byte bound of C-100 `[decision]`:

| Part | Default: mandatory parts only | Worst case: format maximum, all optional parts, with the extension | Worst case: format maximum, all announcements, no extension |
|---|---|---|---|
| Packet header | 16 | 16 | 16 |
| Slice header | 22 | 22 | 22 |
| Announcement container (count byte, 12 bytes per announcement) | 1 | 1 + 4 × 12 = 49 | 1 + 4 × 12 = 49 |
| Measurement extension | — | 9 | — |
| Content-message header (audio message) | 3 | 3 | 3 |
| Audio, 6 × 48 × 4 (C-04) | 1 152 | 1 152 | 1 152 |
| Checksum (CRC-32C) | 4 | 4 | 4 |
| **Total** | 1 198 | 1 255 | 1 246 |
| Spare against 1 472 | 1 472 − 1 198 = 274 | 1 472 − 1 255 = 217 | 1 472 − 1 246 = 226 |

The arithmetic, column by column `[decision]`: default 16 + 22 + 1 + 3 + 1 152 + 4 = 1 198 bytes, 274 to spare; worst case with the extension 16 + 22 + 49 + 9 + 3 + 1 152 + 4 = 1 255 bytes, 217 to spare; worst case without it 16 + 22 + 49 + 3 + 1 152 + 4 = 1 246 bytes, 226 to spare. All four announcements and all 9 extension bytes fit beside the mandatory parts on a 1 472-byte path `[decision]`. The two worst-case columns are the format's maximum, the container's limit of 4 that a receiver accepts (Announcements, Validity rules); a v1 sender carries at most one announcement per packet (Announcement schedule and omission rule, below), so its largest unshared packet is 1 219 bytes, and it leaves nothing out on either path `[decision]`.

### Budget on a fallback path (1 280-byte MTU, 1 252 bytes of UDP payload)

Bytes per part at the C-06 fallback, where the headers and the checksum stay within the 100 bytes of C-137 `[decision]`:

| Part | Mandatory parts only | One announcement and the extension: a v1 sender's largest unshared packet |
|---|---|---|
| Packet header | 16 | 16 |
| Slice header | 22 | 22 |
| Announcement container (count byte, 12 bytes per announcement) | 1 | 1 + 1 × 12 = 13 |
| Measurement extension | — | 9 |
| Content-message header (audio message) | 3 | 3 |
| Audio, 6 × 48 × 4 (C-04) | 1 152 | 1 152 |
| Checksum (CRC-32C) | 4 | 4 |
| **Total** | 1 198 | 1 219 |
| Spare against 1 252 | 1 252 − 1 198 = 54 | 1 252 − 1 219 = 33 |

The arithmetic `[decision]`: the mandatory parts are 46 bytes of headers and checksum, 1 198 in all, 54 to spare; one announcement with the extension is 46 + 12 + 9 = 67 bytes, 1 219 in all, 33 to spare. Both stay within the 100 bytes of C-137, so 6 channels fit at the fallback with every mandatory header and the checksum `[decision]`. The format's maximum, four announcements with the extension, is 103 bytes, 1 255 in all, 3 bytes over the 1 252 that a packet sent with don't-fragment (rule 11) can cross; four announcements without the extension would fit (94 bytes, 1 246 in all), but no v1 sender builds either packet, because the announcement schedule below puts at most one announcement in a packet `[decision]`.

### Announcement schedule and omission rule

The announcement schedule, for every stream on every path `[decision]`: number a stream's slices by s = ⌊p / F⌋, p the slice position the slice header carries and F the stream's frames per slice (48 at the C-02 defaults), so every sender implementation derives the same s from the slice alone, and a receiver needs none of it. An announcement of type code t (Announcements, Type codes: 0 to 3 in v1) rides only in slices whose s mod 4 equals t, its kind's slot class. The sender first issues it in the first slice of that class at or after the slice in which the change becomes due, names a position at least 64 slices after that first issue (C-43), and re-issues it every 16 slices until the position passes (C-44); 16 is a multiple of 4, so every re-issue stays in the class, and the announcement rides in no slice its schedule does not name. Where s does not step by one from one slice to the next, the next issue goes in the latest slice of the class that keeps it within 16 slices sent of the previous one, counting every slice the stream sends `[decision]`. Two landed events break the numbering: a timeline epoch switch, after which positions are counted on the new epoch's line (C-10; Timeline epoch switch), and a re-alignment, if its new alignment shifts the stream's position grid, which is epic 005's state machine to say (C-186, C-208; Announcements, Re-alignment). Both are announced kinds with at most one pending and the next first announced only after the previous position passed, at least 64 slices later, so at most two breaks fall in any 16 slices sent. Any 4 consecutive slices between breaks hold every class, and 16 slices cut by at most two breaks keep a run of at least 6, so the slice the rule needs always exists `[decision]`.

Proof that the schedule keeps C-43 and C-44 exactly and puts at most one announcement in a packet `[decision]`. Each slice has one class and the four v1 kinds have four different classes, so no two kinds ever share a slice: at most one announcement rides per packet, and no kind waits for another. A kind's slot is held by construction from the moment it is scheduled, and kinds that become due in the same slice each take their own class, so no reservation step and no tie-break is needed. A first issue waits only for its own class, which recurs every 4 slices: at most 3 slices, 1.5 ms at the C-02 defaults, whether one kind or all four become due at once. The named position is picked after that wait, at least 64 slices after the first issue, so C-43 holds literally; the re-issues fall exactly 16 slices apart (at most 16 across a line break) until the position passes, in every bundle member, so C-44 holds literally in every non-gap slice — a scheduled slice that is a gap slice carries no announcement, because a gap slice is header-only (rule 10) and the packet header's validity rules reject the announcement flag on it (Media packet header, Validity rules), a case that predates this section and belongs to announcement loss (006-12) — and a change first announced exactly one lead ahead is carried 64, 48, 32 and 16 slices before its position, the four transmissions 006-12 counts on. In a bundle, C-44 asks for the re-issues in every member, and each member applies the schedule to its own s, its own slice positions and frames per slice: the landed text does not make the members of a bundle share p and F, since the regime stamp declares frames per slice per stream (Slice header, Multi-packet regime) and a bundle is a member list in the manifest (01-architecture.md § 7). The bundle names one position, the same value in every member, at least 64 slices after the latest of its members' first issues, each member's first issue waiting at most 3 of its own slices; C-43 then holds in every member, and each member re-issues every 16 of its own slices until the position passes, so C-44 holds per member.

Announcement types beyond v1 `[decision]`: the type space is open within a major version (Versioning, Minor and major changes, 004-04), but the four slot classes are used up by type codes 0 to 3, so a minor version that adds a type code must define that type's slot class or scheduling rule with it; "t mod 4" is not implied, because it would put two announcements in one packet and break the compiler's 13-byte count. A v1 receiver still accepts up to 4 announcements in a packet and skips a type it does not know by its length (Announcements, The container) `[decision]`.

Consequences of the schedule, recorded for the design owner `[decision]`: every first issue may wait up to 3 slices, so every announced position may lie up to 3 slices later than one lead after the change became due, and a deferred re-alignment up to 131 slices after its cause; a v1 sender carries at most one announcement per packet; and the four slot classes are used up, so any future announcement kind needs a scheduling rule of its own.

**Open item (frames-per-slice changes).** The line-break argument above counts the breaks of the two announced kinds; a change of a stream's frames per slice, declared by the regime stamp (C-18; Slice header, Multi-packet regime), also changes s, and the landed text does not say whether it can take effect while announcements are pending. Owner: ticket 004-25 (regime stamp sections), which lays out the multi-packet regime section. Falsifier: 004-25 lets a stream's frames per slice change mid-stream often enough that, with the two announced breaks, more than three breaks fall in 16 slices sent while an announcement is pending, so a window could hold no 4 consecutive slices and a kind's class could be missing `[decision]`.

The wait moves the position the sender names, which is harmless only where the sender picks that position `[decision]`. It does for three kinds: a source picks its own epoch-switch position (01-architecture.md § 2.2: each of its sources announces the future position at which it switches), a fan-in picks its re-alignment position one lead ahead of the slice that first carries it (C-208, C-215), and the end-of-stream position is the end of the content the sender itself will address (Announcements, End-of-stream).

**Open item (transport reference change position).** No landed rule says who picks the position of a transport reference change; the schedule assumes its sender does, as for the other three kinds, so that its first issue may wait up to 3 slices for its class. Owner: epic 005, which owns the transport-reference state machine (Announcements, Open items). Falsifier: epic 005 fixes that position by an event the sender cannot delay, so a first issue that waits for its class would leave less than 64 slices of lead (C-43); the schedule then needs another rule for that kind `[decision]`.

The omission rule that follows, in priority order — controller messages are moved first, announcements are deferred within the lead and never omitted, and the measurement extension is never omitted for size `[decision]`:

1. Controller messages first, and never by omission: controller messages beside audio count inside the cap, are never omitted from a slice and are never a reason to omit an announcement; a single-stream source whose controllers do not fit the shared-packet count moves them out to a second stream (C-181), and a bundle source's controllers are always in their own member stream (C-205), so no controller byte competes with an announcement on any path (Shared packets and the controller allowance) `[decision]`.
2. Announcements second, deferred and never omitted: the schedule defers a first issue by at most 3 slices and omits no announcement on any path, so every pending announcement rides in every slice its schedule names, in every bundle member (C-44), at least one lead after its first issue (C-43) `[decision]`.
3. The measurement extension never omitted for size: with at most one announcement per packet the largest unshared packet has 16 + 22 + 13 + 9 + 3 + 4 = 67 bytes of headers and checksum, so the extension fits beside it on every path and is omitted only when the stream does not measure (rule 4); C-137's permission to omit it on a fallback path is not used, and no consumer — nor any consumer downstream, to which a processor would forward the missing slice, since it never starts an extension (Measurement extension, Forwarding update rule) — loses a slice's measurement to the budget `[decision]`.
4. The checksum and the audio never: every default packet carries the 4-byte checksum (Checksum, Placement and the header cap) and the 1 152 audio bytes are the slice itself; nothing in this rule omits either `[decision]`.

The schedule is chosen over omitting the extension from a packet that carries four announcements on a fallback path `[decision]`. That alternative keeps C-43 and C-44 too, and with announcements carried only in the slices their schedules name it loses at most one slice in 16 and never two in a row — four 16-slice schedules coincide in at most one slice of each 16 — for as long as four kinds are pending, which has no upper bound since the lead is at least 64 slices; every consumer of that connection and every processor downstream of it, on any path, loses the same slices' measurement, and the packet depends on the path, so the sender must know each connection's path and follow every re-probe. The schedule removes the loss and the dependence together at the cost of a first-issue wait of at most 3 slices, and it must be the stream's on every path rather than the fallback connection's, because every consumer of a stream reads the same named positions.

Proof that every v1 default packet fits both paths `[decision]`: a packet without controller messages — a bundle source's audio member (C-205) or a single-stream source that shares nothing — carries at most one announcement, so its headers and checksum are at most 67 bytes and the packet at most 1 219; a packet with controller messages stays within the shared-packet count below, at most 100 bytes beside its audio and 1 252 in all. Every v1 default packet is therefore within the 100 bytes of C-137 and the 1 252 bytes of a fallback path whatever path it travels, with nothing omitted, and C-43 and C-44 hold as proved above. A bundle source's controller member carries no audio, so its headers and checksum, at most 16 + 22 + 13 + 9 + 4 = 64 bytes, leave its controller messages, their content-message headers included, at least the 1 252 − 100 = 1 152 bytes that Sparse keyed controller message states.

Knowing the path is therefore not load-bearing for the layout `[decision]`: a v1 sender emits the same packets on a 1 472-byte path as on a fallback path. The admission probe pair — a padded don't-fragment packet at the intended media packet size and its acknowledgement, falling back to the 1 280 bytes of C-06 on silence and refused with PathMtuTooSmall only if that probe fails too (01-architecture.md § 3.3; C-116, C-173) — decides whether a connection is admitted, not how its packets are laid out. A re-probe that moves a connection onto or off the fallback (01-architecture.md § 3.3: re-probe when large packets vanish while liveness holds) changes no packet: the layout is the same before and after it, from every slice.

**Open item (probe size).** The rule needs from the probe pair only that a connection admitted on the fallback carries a 1 252-byte UDP payload sent with don't-fragment, the largest v1 default packet; the probe pair's layout and sizes are ticket 004-27's and are not landed. Owner: ticket 004-27 (path-MTU probe pair). Falsifier: 004-27 admits a connection whose probes, the 1 280-byte fallback probe included, never carried 1 252 bytes of UDP payload with don't-fragment set — for example a fallback probe sized below the stream's largest default packet — so a connection that passed admission can still drop a default packet `[decision]`.

### Shared packets and the controller allowance

The shared-packet count, final `[decision]`: it applies only to a default-regime packet that carries controller messages beside its audio message, and it counts each byte once — the packet header (16), the slice header (22), the announcement container, the measurement extension, the audio message's 3-byte content-message header, every controller message whole with its own content-message header (so no header is counted twice), sparse keyed and transport state messages included, and the checksum (4); the total must be at most 100 (C-137, C-181). The compiler and the receiver count the two variable parts as follows `[decision]`:

- The compiler's count, from the manifest (C-181): the container as 1 + 12 = 13 bytes, one announcement's room, since the schedule puts at most one announcement in a packet; and the measurement extension as 9 bytes whenever the stream could carry it (defined below), otherwise 0 `[decision]`.
- The receiver's count, from the packet (Content messages, Validity rules, the shared-packet cap row): the container as 1 + 12 × max(1, k) bytes, k the announcements the packet carries, so one announcement's room is counted even when it carries none; and the extension as 9 bytes when the packet carries it, otherwise 0 `[decision]`.

The two counts agree on every conforming packet, and no conforming packet is rejected `[decision]`. A conforming packet carries k ≤ 1 announcements, so the receiver counts 13 container bytes, exactly the compiler's; it carries the extension only if its stream could carry it, so the receiver counts at most the compiler's 9; every other part is counted alike, so the receiver's count never exceeds the compiler's, which the compiler holds at or below 100. A shared packet with k ≥ 2 breaks the schedule; for it, as for any packet with k ≥ 1, the receiver's count equals the packet's real bytes beside its audio, so the row rejects it exactly when those bytes exceed 100. The count is a property of the stream, the same on every path, and a packet within it has at most 100 bytes beside its audio, so it fits a fallback path whatever path it travels `[decision]`.

A stream could carry the measurement extension when the compiler, from the manifests and the graph, finds that it may emit one `[decision]`: a source's stream could carry it when its source measures that stream (Measurement extension, Forwarding update rule: a source that measures a stream emits the extension with all three fields 0), and a processor's output stream could carry it when any of the processor's inputs could carry it, since any input may be its critical input (014-14) and a processor carries the extension on its output only when its critical input's slice carried it, never starting one; every other stream could not. A stream compiled as one that could not carry it does not carry it, even when a critical input arrives with one: the Forwarding update rule makes the input's extension a condition for carrying it, not a duty to carry it, and rule 4 lets a stream omit it, so such a stream's packets stay within the count it was compiled with `[decision]`.

**Open item (measuring declaration).** Whether a source measures a stream must be stated where the compiler reads it, in the manifest, whose encoding is ticket 004-26's and is not landed; the definition above assumes the manifest states it. Owner: ticket 004-26 (manifest encoding). Falsifier: 004-26's manifest gives the compiler no statement from which it can decide whether a source measures each of its streams, so the compiler cannot tell a stream that could carry the extension from one that could not and cannot apply the shared-packet count `[decision]`.

Proof for a packet with controller messages, in every reachable case `[decision]`: its bytes beside the audio never exceed the receiver's count, which never exceeds the compiler's, which is at most 100, so C-137 holds on every path with nothing omitted; C-43 and C-44 hold by the announcement schedule. The largest such packet is 1 252 bytes, within both paths.

The controller allowance for a single-stream source, at both path sizes: the 100-byte cap less the counted headers and the checksum `[decision]`:

| Path | Bound: the C-137 cap (bytes) | Audio beside the cap | Counted headers | Checksum | Allowance for controller messages |
|---|---|---|---|---|---|
| 1 472-byte path (C-100), a stream that could carry the measurement extension | 100 | 1 152 | 16 + 22 + 13 + 9 + 3 = 63 | 4 | 100 − 63 − 4 = 33 |
| 1 472-byte path, a stream that could not | 100 | 1 152 | 16 + 22 + 13 + 3 = 54 | 4 | 100 − 54 − 4 = 42 |
| Fallback path (1 280-byte MTU, 1 252 bytes), a stream that could carry the extension | 100 | 1 152 | 16 + 22 + 13 + 9 + 3 = 63 | 4 | 100 − 63 − 4 = 33 |
| Fallback path, a stream that could not | 100 | 1 152 | 16 + 22 + 13 + 3 = 54 | 4 | 100 − 54 − 4 = 42 |

The allowance is an upper bound, the most a source may share, not an amount it must share: a single-stream source may always put its controllers in a second stream instead, and must once they exceed the allowance (C-181) `[decision]`. It is the same at both path sizes because C-181 bounds headers plus controllers by the 100-byte cap itself and the count is checked per stream, not per path; on a 1 472-byte path a shared packet leaves 1 472 − 1 152 − 100 = 220 bytes of the path unused rather than lending them to controllers `[decision]`. A bundle source's audio member counts no controller messages (C-205): its controllers ride in their own member stream, so its budget is the unshared one above and its allowance is not used `[decision]`.

Examples with the landed sizes `[decision]`: in a stream that could not carry the extension the 42 bytes hold a transport state message (30 bytes) and a sparse keyed message of 1 entry of 8 bytes behind its 4-byte header (30 + 4 + 1 × 8 = 42 bytes), or four sparse entries alone (4 + 4 × 8 = 36 bytes), or one dense message of up to 9 controllers at one value per slice (6 + 9 × 4 = 42 bytes). In a stream that could carry the extension the 33 bytes hold a transport state message alone (30 bytes), or a sparse keyed message of three entries (4 + 3 × 8 = 28 bytes), or a dense message of up to 6 controllers at one value per slice (6 + 6 × 4 = 30 bytes), but not a transport state with a sparse entry beside it (42 > 33 bytes). A single controller at one value per frame (6 + 192 = 198 bytes) never shares a default packet `[decision]`. These are stated limits of sharing, not targets: a source that needs more uses a second stream (C-181). They replace the earlier figures of 45 and 54 bytes, which counted no announcement room and so could not hold on a fallback path with an announcement pending `[decision]`.

### Verdict

**Verdict.** The open sub-question asked whether all headers and the checksum fit the 100 bytes of C-137, and which announcements and measurement-extension bytes fit beside them on a 1 472-byte path. The verdict: C-137 holds. Its falsifier — headers and checksum above 100 bytes, which would cut C-04 to 5 channels at the fallback — does not occur: the mandatory headers and the checksum are 46 bytes, 54 below the cap, so 6 channels fit at the fallback (1 152 + 46 = 1 198 ≤ 1 252) and C-04 stays at 6 `[decision]`. The optional parts fit the cap in every combination but one, four announcements with the extension (103 bytes), which no v1 sender builds: the announcement schedule puts at most one announcement in a packet, 67 bytes at most without controllers, so every packet a v1 sender emits stays within 100 bytes on every path and nothing is omitted `[decision]`. On a 1 472-byte path all four announcements (49 bytes with the count byte) and all 9 extension bytes fit beside the mandatory parts: 1 255 bytes, 217 to spare `[decision]`.

### What this section amends

This section amends or scopes, each in its own section, the landed statements that the 3-byte content-message header made untrue, and closes the header-cap item of Content messages, Budget `[decision]`:

- Media packet header: the account of the 84 bytes after the header gives the 87 they would need at the format's maximum and the 51 a v1 sender's one announcement per packet keeps them to `[decision]`.
- Announcements, Maximums: all four announcements fit the cap beside the mandatory parts (94 bytes) but not beside the extension as well, and a v1 sender carries at most one per packet; the fallback clause that omitted an announcement and re-issued it later is replaced by the schedule; the cap-of-4 argument now says it holds without staggering first issues; the extension's and the checksum's shares of the 62 bytes are stated `[decision]`.
- Announcements, Sender obligations: the container never blocks the re-issue cadence because every kind rides in its own slot class, one announcement per packet, and none is omitted on any path `[decision]`.
- Announcements, Announced positions stand until they pass: the withdrawal argument no longer says that a withdrawal would be omitted with its announcement on a fallback path, since no announcement is `[decision]`.
- Measurement extension, Byte size and the header cap: the extension fits in every v1 packet again, now because of the schedule, so no v1 sender omits it on a fallback path `[decision]`.
- Content messages: Where controllers travel and the shared-packet cap row of Validity rules take the final counts above; Budget records the item as answered here; the 004-12 shared-packet figures follow the new allowance, and the sparse refresh falsifier treats them as stated limits `[decision]`.
- Checksum: Placement and the header cap gives the 103-byte format maximum and the schedule; Who computes and who verifies and the Worked example no longer name a path that omits an announcement, since every path carries the same covered layout `[decision]`.

### Machine-readable totals

wire.toml carries these totals in its `[budget]` table: the two payload bounds, the part sizes, the mandatory and maximal bytes beside the audio, the total and spare of every column above, and the two controller allowances, as integers equal to the figures shown here; checks/004-14.sh recomputes every part from the field tables of wire.toml and the constants table and compares `[decision]`.

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

A follower estimates its offset from the holder with request/reply exchanges on the sync port: the follower sends a sync request to the holder, and the holder answers each request with one sync reply. The holder's monotonic clock defines network time (01-architecture.md § 2.3), and a reply carries what a follower needs to place itself on the domain line: the holder's two timestamps of the exchange, the timeline epoch with its origin, and the holder's identity `[decision]`. This section fixes both layouts, the timestamps' unit and clock, the decisions this ticket owns, and every invalid form with its outcome `[decision]`.

### Exchange rules

The exchange is stateless: the reply echoes the request's own send timestamp verbatim, so the holder answers each request from that request alone and keeps no per-request state, and a follower pairs a reply with its request by the echoed value, read on its own clock `[decision]`. A stale or duplicate reply is therefore a valid exchange with an honestly large round trip — its echoed send timestamp is old, so the round trip the follower computes from it is long — and never an invalid form; the follower's exchange filtering, which this specification does not define, decides what a long round trip is worth `[decision]`. Nothing in either message names an exchange: the holder's per-module values in a reply — its clock, its module id, its current epoch and origin, its message counter — are the holder's own state, the same whichever request is being answered `[decision]`.

Sync messages travel by UDP unicast over IPv4 (C-100) at DSCP CS6 (48) (C-13): the follower sends its request to the holder's sync port, and the holder sends the reply to the source address and port the request came from, which is all the addressing a stateless reply needs `[decision]`. The class-selector codepoints are the 'xxx000' set (RFC 2474, § 4.2.2.1 The Class Selector Codepoints), CS6 being 110000 `[standard]`. Neither message carries a DSCP field: the holder records the DSCP it observes on each follower's requests on its clock thread (C-188) and reports it in its regime stamp (C-152), in the section 004-28 lays out `[decision]`.

The request rate is behaviour, never a field: a follower sends 50 requests per second (C-36), and the holder accepts 10 to 200 requests per second from each follower (C-112) `[decision]`. Requests beyond 200 per second from one follower are dropped before any work is done and counted per follower in the holder's regime stamp (C-204); that drop is not a rejection of a malformed message, it increments no counter of the closed enumeration, and its counter is laid out by 004-28, so neither message carries a rate field `[decision]`. How the per-follower entries of C-152 and C-204 identify a follower is 004-28's to fix; the request adds no identity field for it `[decision]`.

### Authentication reservation

Sync messages carry the authentication reservation of C-12 — scheme 8 bits, message counter 64 bits, tag 128 bits — in both the request and the reply, at the same offsets in both, with the v1 values of C-115: scheme 0 (none), tag 128 zero bits, and a live counter `[decision]`. This closes R11 for sync messages; the receiver outcomes are those of Reserved fields, Authentication reservation (C-12): a non-zero scheme or a non-zero tag is dropped and counted under ReservedNonZero, and the counter never produces ReservedNonZero `[decision]`. Falsifier: a later authentication scheme cannot be added to sync messages without a major change — for example a scheme that needs more than the 128-bit tag, or a field the reservation does not hold — in which case this decision is wrong and the reservation's shape is reopened by epic 004 `[decision]`.

**Why carry it.** Three arguments decide it, in order of weight `[decision]`:

1. Adding it later is a major change by construction. Both messages are fixed-size, with no length prefix and no optional part, and a receiver rejects every other datagram size under BadLength (Validity rules below); a field added after v1 changes that fixed size, so every v1 receiver rejects the new message, and by Versioning, Minor and major changes, a change an older receiver can neither parse nor skip is a major one. Leaving the reservation out would make the ticket's falsifier true on the day the first scheme arrives; carrying it, a scheme fills fields that already exist at fixed offsets with the message unchanged in shape, so no existing field moves or widens `[decision]`.
2. A forged reply is the most damaging forgery on the medianet. In v1 nothing authenticates a reply: one forged with the holder's module id and the domain passes every rule below. Its timestamps bias the follower's offset estimate, and its epoch, if it differs from the follower's current one and echoes a recent send timestamp, is taken as the holder's new epoch (Timeline epoch and holder identity below), after which every source of that module announces a switch and restarts its session (01-architecture.md § 2.2). The sync reply is therefore the first message a later scheme must cover, and the reservation is what lets it `[decision]`.
3. The cost is small. The reservation is 25 bytes per message (200 bits). At the holder, at most 64 followers (C-35) each sending 50 requests per second (C-36) make 3 200 requests in and 3 200 replies out per second; the reservation adds 25 × 3 200 = 80 000 bytes per second in each direction, 0.64 Mbit/s, and at the accepted ceiling of 200 per second per follower (C-112) 2.56 Mbit/s. Whole sync traffic at 50 per second, the 28 bytes of IPv4 and UDP headers included (Layer overview), is (74 + 28) × 3 200 × 8 = 2 611 200 bit/s per direction, about 2.6 Mbit/s — an eighth of one 6-channel stream's 21.1 Mbit/s (C-21). The holder's serving work grows by a 25-byte copy and two zero comparisons per message, with no cryptography in v1, so the holder-serving-lateness falsifier of C-35 is not brought nearer `[decision]`.

The counter is live in sync messages as in control messages (C-115): each module increments one sync-message counter by one for every sync message it sends, request or reply, and that counter is module state, not per-request state. In v1 no receiver rule depends on its value — liveness on the sync layer is judged on the replies themselves (C-37) — and a later scheme finds its replay counter already on the wire `[decision]`. The header keeps the field order ticket 004-16 names for the common control header — magic, version, domain, message type, authentication reservation — so one header parser can serve both layers; if 004-16 lays its header out differently, the sync layer keeps this layout, which no control parser reads `[decision]`.

### Request

Little-endian (Byte order), packed with no padding for alignment (Alignment and padding); offsets are byte distances from the first byte of the UDP payload. The request carries the common header and the follower's send timestamp, then explicit padding to the reply's size `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| 0 | 2 | u16 | Magic 0x4d4e, the ASCII letters M and N, fixed by 004-04 | 0x4d4e only |
| 2 | 2 | u16 | Protocol version, major in the high byte (bits 15..8), minor in the low byte; v1 packs 1.0 as 0x0100 (004-04) | major 1, any minor (Versioning, Receiver behaviour); senders write 0x0100 |
| 4 | 2 | u16 | Domain id (C-11); 0 is the unconfigured domain | 1 to 65535; 0 = unconfigured |
| 6 | 1 | u8 | Message type: 1 = sync request (Message type codes) | 1 only |
| 7 | 1 | u8 | Authentication scheme (C-12); 0 = none in v1 (C-115) | 0 only |
| 8 | 8 | u64 | Message counter (C-12): live (C-115), incremented by one for every sync message the follower sends | any 64-bit value |
| 16 | 16 | u128 | Authentication tag (C-12): 128 zero bits in v1 (C-115) | 0 only |
| 32 | 8 | u64 | Send timestamp: the follower's local monotonic clock in nanoseconds when it sends the request; opaque to the holder, which echoes it verbatim (Timestamps) | any 64-bit value |
| 40 | 34 | pad | Padding: brings the request to the reply's size, the room the reply's receive and send timestamps, holder module id, epoch origin and timeline epoch take; senders write zero (Reserved fields, general rule) | 0 only |

### Reply

Same conventions as the request; bytes 0 to 31 are the common header, the echoed timestamp sits at the request's send-timestamp offset, so a holder may build the reply over the request's own bytes, and the padding's room carries the holder's values `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| 0 | 2 | u16 | Magic 0x4d4e, the ASCII letters M and N, fixed by 004-04 | 0x4d4e only |
| 2 | 2 | u16 | Protocol version, major in the high byte (bits 15..8), minor in the low byte; v1 packs 1.0 as 0x0100 (004-04) | major 1, any minor (Versioning, Receiver behaviour); senders write 0x0100 |
| 4 | 2 | u16 | Domain id (C-11); 0 is the unconfigured domain | 1 to 65535; 0 = unconfigured |
| 6 | 1 | u8 | Message type: 2 = sync reply (Message type codes) | 2 only |
| 7 | 1 | u8 | Authentication scheme (C-12); 0 = none in v1 (C-115) | 0 only |
| 8 | 8 | u64 | Message counter (C-12): live (C-115), incremented by one for every sync message the holder sends | any 64-bit value |
| 16 | 16 | u128 | Authentication tag (C-12): 128 zero bits in v1 (C-115) | 0 only |
| 32 | 8 | u64 | Echoed send timestamp: the request's send timestamp, echoed verbatim; the follower's clock, never the holder's | any 64-bit value |
| 40 | 8 | u64 | Receive timestamp: the holder's monotonic clock in nanoseconds when the request was received (Timestamps) | any 64-bit value |
| 48 | 8 | u64 | Send timestamp: the holder's monotonic clock in nanoseconds when the reply is sent (Timestamps) | any 64-bit value |
| 56 | 8 | u64 | Holder module id (C-11): the module id of the holder that answers | any 64-bit value; a follower accepts only the holder it follows (Timeline epoch and holder identity) |
| 64 | 8 | i64 | Epoch origin: origin_ns of the current timeline epoch's line, nanoseconds of network time on the holder's scale (timeline.md, The epoch origin) | any 64-bit value |
| 72 | 2 | u16 | Timeline epoch (C-10): the holder's current timeline epoch | any 16-bit value; acceptance at a follower is in Timeline epoch and holder identity |

Request and reply are both 74 bytes long: the request's fields sum to 2 + 2 + 2 + 1 + 1 + 8 + 16 + 8 + 34 = 74 bytes and the reply's to 2 + 2 + 2 + 1 + 1 + 8 + 16 + 8 + 8 + 8 + 8 + 8 + 2 = 74 bytes, each tiling its message from byte 0 with no gap and no overlap `[decision]`. Equal sizes keep the exchange symmetric on the path and let a receiver judge a datagram's size before it reads the message type, since every sync message has the one fixed size `[decision]`. Both fit the 1 252-byte UDP-payload bound with room to spare, though that bound (C-17) is written for control messages `[decision]`.

**Message type codes.** The message type is scoped to the sync layer, which has its own socket (Rejection reasons, Counting model): 1 is the sync request and 2 the sync reply; 0 and 3 to 255 are unused in v1, so a zeroed buffer is never a valid message `[decision]`.

**The epoch origin.** The reply carries the origin of the current epoch's line as well as the epoch: a follower reconstructs the line `position = (network_time − epoch_origin) × 96 000 Hz` from a reply alone only if the reply names the origin, and epic 006 lists it among the fields a reply must carry (006-04, the epic's line 1b: epoch, origin, holder identity) `[decision]`. Its type is timeline.md's `origin_ns: i64` `[decision]`. It is carried now because a field added later would be the major change argued above; 006-04 confirms it, as the last subsection of this section records `[decision]`.

### Timestamps

All three timestamps are 64-bit unsigned counts of nanoseconds, one tick 1 ns, read from a local monotonic clock: the same unit, the same clock and the same width as the slice header's send timestamp (Slice header, Send timestamp), so a module reads one clock for media and sync and no new clock or unit enters the wire `[decision]`. The retired POC's exchange held the same four instants of a two-way exchange as 64-bit nanosecond counts (`clock.rs`, `Exchange`, fields `t1` to `t4`) `[code]`. The epoch origin is the one signed time on the sync layer, because timeline.md fixes it as i64 nanoseconds `[decision]`.

- **Request send timestamp.** The follower's local monotonic clock when it sends the request. It is opaque to the holder: the holder never interprets it, compares it or stores it, and copies it verbatim into the reply's echoed send timestamp `[decision]`.
- **Reply receive timestamp.** The holder's monotonic clock — network time (01-architecture.md § 2.3) — at the receipt of the request, taken from the holder's receive stamp of that datagram `[decision]`.
- **Reply send timestamp.** The holder's monotonic clock at the transmission of the reply, read as late before the send as the holder's backend allows `[decision]`. Its difference from the receive timestamp is the holder serving time C-167 measures (reply send time minus request receive time) `[decision]`.

The follower's fourth instant, its receive stamp of the reply, never travels: it is the follower's own reading, and the backend that builds each exchange from receive stamps is also where wall-clock step detection lives (C-121); the wire carries the timestamps that exchange needs, nothing more `[decision]`. With the 2^64 ns wrap period, about 584 years (Slice header, Send timestamp), no exchange ever sees a wrapped timestamp `[decision]`.

### Timeline epoch and holder identity

**Who the holder is.** A follower knows the holder it follows from the election and discovery messages of epic 005 (004-16, 004-17); the reply's holder module id is the wire's statement of who answered, and the follower compares it with the module id of the holder it follows `[decision]`. A reply whose holder module id differs — a former holder still answering after a takeover, a module that is not the holder, or a stray reply — is dropped and counted under UnadmittedSource `[decision]`. UnadmittedSource is the closest member of the closed enumeration: its meaning is a source whose module id has no admitted connection at this receiver, and on the sync layer the holder the follower follows is its one admitted source of time; its Layers cell now names sync, and its meaning names this case, which adds no reason to the enumeration (Rejection reasons, Closed enumeration) `[decision]`. The source address is not part of the rule: the module id is the identity the wire states, and how a follower learns its holder is election behaviour, epic 005's `[decision]`.

**Which epochs a follower accepts.** The reply is how a follower learns the holder's current epoch: the epoch changes on an election in which no candidate has usable time (C-113, C-213) and on a holder stall (C-136), and a takeover by a candidate with usable time keeps it (C-213), so an epoch different from the follower's is not by itself wrong `[decision]`. The follower holds its current epoch E together with A, the reading of its own local monotonic clock at which it adopted E — from a sync reply, an epoch-switch announcement or admission — and judges a reply from the holder it follows, carrying epoch e, by the echoed send timestamp, which is a reading of the same local clock `[decision]`. A is held on the follower's local monotonic clock, beside the network-time adoption instant that the wrap section keeps for the epoch's tenure (Media packet header, Timeline epoch width and wrap), not in its place: A is compared only with echoed send timestamps, which are on that same local clock, and never with a network time, so the two clocks are never compared with each other `[decision]`:

1. e equals E: accepted.
2. e differs from E and the echoed send timestamp is later than A: accepted. The request was sent after the follower adopted E, so the holder answered it with its epoch as it stood after that adoption, and the reply is the follower's evidence of a bump; what the follower then does with the new epoch and origin — line reconstruction, the switch its sources announce — belongs to epic 006 (006-04, 006-11) `[decision]`.
3. e differs from E and the echoed send timestamp is at or before A: dropped and counted under WrongEpoch. The reply answers a request sent before the follower adopted E, so its epoch predates what the follower already knows; a newer epoch carried by such a reply is not lost, because the next request, sent after A, fetches it under case 2 `[decision]`.
4. A follower with no current epoch — at cold start, after it starts following a different holder, or once its epoch's tenure has passed the wrap-ambiguity span W (Media packet header, Timeline epoch width and wrap) — accepts any epoch from the holder it follows; this is the reply that re-establishes the current epoch `[decision]`.

The rule needs no state at the holder and no serial-number comparison of epochs, which the wrap section declines: the follower's own clock orders the replies by the requests they answer, and the holder only ever moves its epoch forward `[decision]`. WrongEpoch's Layers cell now names sync and its meaning names case 3; on the sync layer the rejection is counted per socket like all sync rejections (Validity rules below), not on the per-connection other-epoch counter of media (C-135, C-159) `[decision]`. Whether 006-04's adoption rule matches cases 1 to 4 stays to be confirmed, as the last subsection of this section records `[decision]`.

### Receive timestamps

Neither message names a receive-timestamp source. The source the holder's receive timestamp comes from is one of the four values of C-131 — kernel, userspace, mac-hardware or interrupt — each reported with one of the capability states of C-129, and it is declared in the regime stamp (004-25's receive-timestamp section, C-18), not in sync messages `[decision]`.

**Per-OS notes (slot for epic 001).** Epic 001 supplies a non-blocking report on receive timestamps per OS; its remarks on how each OS stamps the receipt of a sync request or reply are recorded here when the report lands, and they affect only these notes on the timestamp fields, never a field's offset, width or meaning `[decision]`. No per-OS remark is recorded yet `[decision]`.

### Validity rules

Every malformed sync message is dropped — or ignored, for the unconfigured domain — and counted under one member of the closed Rejection reasons enumeration; nothing is dropped silently `[decision]`. A sync message matches no admitted input (Rejection reasons, Counting model), so every sync rejection is counted per (socket, reason) on the receiving sync socket `[decision]`. The rules run in the order of the table — the magic and the version first, as Versioning requires — and the first that applies decides the outcome; a message of the same major and another minor is accepted, as Versioning, Receiver behaviour fixes `[decision]`:

| Invalid form | Required outcome |
|---|---|
| datagram too short to hold the two leading fields every message starts with (under 4 bytes) | drop and count under TooShort: dropped before any field is read `[decision]` |
| bad magic (the leading two bytes are not 0x4d4e) | drop and count under BadMagic |
| version mismatch (the version's major, its high byte, is not 1: a different major) | drop and count under VersionMismatch, and raise the VersionMismatch event — never a silent drop (Versioning, Receiver behaviour) |
| datagram size not equal to the 74-byte fixed size, at 4 bytes or more | drop and count under BadLength: the body does not fill the fixed size `[decision]` |
| domain id 0, the unconfigured domain (C-11) | ignore and count under UnconfiguredDomain — never ForeignDomain (C-173) |
| foreign domain (a non-zero domain other than the local domain, C-11) | drop and count under ForeignDomain |
| message type not the one the receiving end expects: not 1 at a module answering requests, not 2 at a follower awaiting replies, or any value other than 1 and 2 | drop and count under BadLength `[decision]` — provisional, the message's declared kind disagrees with the receiving role, as the content messages' role rule uses BadLength (Content messages, Validity rules); confirmed by 004-21, see the end of this section |
| authentication scheme non-zero | drop and count under ReservedNonZero (Reserved fields, Authentication reservation) |
| authentication tag non-zero | drop and count under ReservedNonZero (Reserved fields, Authentication reservation) |
| request padding non-zero (any bit of bytes 40 to 73 of a request) | drop and count under ReservedNonZero (Reserved fields, general rule) |
| reply from a non-holder: the holder module id is not the module id of the holder the follower follows | drop and count under UnadmittedSource `[decision]` |
| reply with wrong epoch: the timeline epoch differs from the follower's current epoch and the echoed send timestamp is at or before the follower's adoption of that epoch (case 3 above) | drop and count under WrongEpoch `[decision]` |
| duplicate or stale reply carrying the follower's current epoch | none — a valid exchange with an honestly large round trip; never a rejection `[decision]` |
| a request beyond 200 per second from one follower (C-204) | none of the enumeration — dropped before any work is done and counted per follower in the holder's regime stamp (004-28) `[decision]` |

No value of the message counter or of any timestamp, origin or module id field is a malformed case: each accepts every value its type holds, and only the rules above reject `[decision]`.

### Open items

**Open item 1 — the reply's line fields.** The reply carries the epoch origin (i64) beside the epoch and the holder module id, ahead of 006-04's field list, because a field added later is a major change. Owner: epic 006, ticket 006-04 (sync reply contents and choice of epoch origin), which confirms the list. Falsifier: 006-04's list names a field the reply does not carry or a width other than the one laid out here (epoch 16 bits, origin 64 bits signed, module id 64 bits), or shows the origin is not needed in the reply; either reopens this layout `[decision]`.

**Open item 2 — epoch acceptance and message-type outcome.** Cases 1 to 4 of Timeline epoch and holder identity are the wire's minimal rule for a follower's epoch acceptance; 006-04's ticket asks for a serial comparison of epochs "from 006-03", while the landed wrap section (004-07, Media packet header, Timeline epoch width and wrap) declines serial-number arithmetic; the landed specification stands, and 006-04 resolves that tension under this item. Owner: epic 006, ticket 006-04 (and 006-11 for consumer-side adoption) for the epoch rule; ticket 004-21, the validity-rules completeness audit, for the BadLength outcome of an unexpected message type. Falsifier: 006-04's adoption rule adopts a reply this section counts under WrongEpoch, or two followers reconstruct different lines from the same replies under cases 1 to 4; or 004-21 finds a member of the closed enumeration that names an unexpected message type better than BadLength `[decision]`.

## Receiver report

The receiver report is a consumer's account of one bundle to the bundle's producer: the consumer's data thread sends one report per bundle every 100 ms (C-40), carrying per member the eleven counters of C-159 and the observed DSCP, per bundle the socket and interface drop counters, and the consumer's current A (C-198) `[decision]`. The v1 measurement layer has exactly two message kinds, this report and the one-way probe with its request and reply (One-way probe below); there is no third `[decision]`. The report is also the producer's only evidence that its consumer is alive: a producer times out after 2 s without receiver reports (C-41), which is behaviour, not a field `[decision]`.

### Transport

Reports travel by UDP unicast over IPv4 (C-100) at DSCP CS5 (40) (C-13), from the consumer to the reverse port its producer gave in the admission handshake; 004-18 lays out that port and its negotiation `[decision]`. The report is unicast only, never multicast, and there is no query message: a third party sees stream health in the regime stamps, where producer and consumer each publish per-connection counters (C-47, C-159) in the sections 004-25 lays out `[decision]`. The sender sets the IPv4 don't-fragment bit, as on every packet of the protocol (ADR 01) `[decision]`. Forward media packets carry no per-receiver state, and round-trip closure uses this measurement path (rule 8) `[decision]`.

No field carries a rate, an interval or a timeout: one report per bundle every 100 ms (C-40) and the producer's 2 s timeout (C-41) are behaviour of the state machines of epics 005 and 006, and a change of A travelling upstream one hop per report, with each fan-in that inherited the old value re-aligning (C-198, C-186), is behaviour too; this section fixes only what the fields mean `[decision]`.

### Size bound of the measurement layer

No landed rule bounds measurement messages: rule 5 of Field-table format and C-17 are written for control messages, and the Layer overview states no bound for the measurement layer `[decision]`. This section adopts the same bound for both measurement messages: every receiver report and every probe fits one unfragmented datagram on the fallback path, at most 1 252 bytes of UDP payload (C-06, C-100) `[decision]`. The reason is ADR 01: IP fragmentation is forbidden, and a connection may run on the 1 280-byte fallback path (C-06), so a report that did not fit there could not be sent on such a connection at all, and its producer would time out (C-41) on a healthy stream `[decision]`.

### Authentication reservation

The receiver report and the one-way probe do not carry the authentication reservation of C-12: neither message has a scheme, a message counter or a tag field, in v1 `[decision]`. Falsifier: a later authentication scheme is to cover the receiver report or the one-way probe, which requires ADR 22's exclusion of the data plane to be revised, and the scheme then cannot be added without a major change; in that case this decision is wrong and epic 004 reopens both layouts `[decision]`.

**Why not carry it.** Four arguments decide it, in order of weight `[decision]`:

1. ADR 22 settles the scope of the reserved scheme: "The data plane is unauthenticated, in v1 and in the reserved scheme". Both measurement messages are data-plane messages: C-198 places the report's carriage of A "on the data plane", the consumer's data thread sends it (01-architecture.md § 3.6), and the probe rides the media class EF (46) to measure the media path (C-13). Wire space in them would be space that no planned scheme fills `[decision]`.
2. A forged report could do harm, and that is stated plainly: one carrying the consumer's module id could keep a producer sending after its consumer died (C-41 never fires), carry a changed A that makes every fan-in that inherited A re-align (C-198, C-186), or falsify the counters the producer publishes in its regime stamp. v1 guarantees protection against accident, not against attack (ADR 22): a report from a foreign domain, for another source's module id, or from a reporter with no admitted connection is dropped and counted (Validity rules below), which is the protection the media layer has (only admitted peers are parsed, 01-architecture.md § 6.4) `[decision]`.
3. Cost is not the argument: the reservation is 25 bytes per message, at one report per bundle per 100 ms 250 bytes per second per bundle, and the report fits the bound with or without it (Maximum size below) `[decision]`.
4. Adding it later is a major change, because both messages have sizes their tables fix and a receiver rejects every other size under BadLength (Validity rules below); that is exactly the case the falsifier names, and the falsifier ties it to the revision of ADR 22 that would have to come first `[decision]`.

The Reserved fields section stays true as it stands: its C-12 rows name control and sync messages only, and the measurement messages carry no reserved field other than padding, which follows that section's general rule `[decision]`.

### Fixed part

Little-endian (Byte order), packed with no padding for alignment (Alignment and padding); offsets are byte distances from the first byte of the UDP payload. Bytes 0 to 6 are the common header the sync messages start with — magic, version, domain, message type — and byte 7 is the member count, so bytes 0 to 7 are the part every measurement message starts with; the fixed part is 38 bytes `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| 0 | 2 | u16 | Magic 0x4d4e, the ASCII letters M and N, fixed by 004-04 | 0x4d4e only |
| 2 | 2 | u16 | Protocol version, major in the high byte (bits 15..8), minor in the low byte; v1 packs 1.0 as 0x0100 (004-04) | major 1, any minor (Versioning, Receiver behaviour); senders write 0x0100 |
| 4 | 2 | u16 | Domain id (C-11); 0 is the unconfigured domain | 1 to 65535; 0 = unconfigured |
| 6 | 1 | u8 | Message type: 1 = receiver report (Message type codes below) | 1 only |
| 7 | 1 | u8 | Member count: how many member rows follow the fixed part, one per bundle member admitted to this consumer (C-08) | 1 to 32 |
| 8 | 8 | u64 | Bundle identity: the module id (C-11) of the bundle's source, the producer the report goes to; with the rows' stream numbers it names exactly one bundle (Bundle identity and member rows) | any 64-bit value; a producer accepts only its own module id |
| 16 | 8 | u64 | Reporter identity: the module id (C-11) of the consumer that sends the report (Bundle identity and member rows) | any 64-bit value; a producer accepts only a module with an admitted connection for the bundle |
| 24 | 2 | u16 | Report number: 0 for the connection's first report after admission, then incremented by one for every report the consumer sends for this bundle; it wraps modulo 2^16 (Report number) | any 16-bit value |
| 26 | 4 | u32 | Socket drop: datagrams the kernel dropped at the consumer's receiving socket of this bundle for lack of buffer room, the per-socket overflow counter (01-architecture.md § 6.4); cumulative, wraps modulo 2^32 | any 32-bit value |
| 30 | 4 | u32 | Interface drop: packets the consumer's receiving interface dropped, the low 32 bits of the interface's own drop count; cumulative, wraps modulo 2^32 | any 32-bit value |
| 34 | 4 | u32 | Current A: the consumer's current A (C-198), its own or the one it inherited (C-142), in nanoseconds; 0 = no A known to the reporter (Current A) | 0 to 4 294 967 295 ns: every value is valid; 0 = no A known |

### Per-member part

The member rows follow the fixed part back to back, one per member admitted to this consumer, in the order of the consumer's admission (Bundle identity and member rows). Each row is 37 bytes: six counters are 32 bits wide and five are 16 bits, by the unit each counts (Counters); offsets are byte distances from the row's first byte, and the sub-byte fields of byte 36 are numbered as Bit-field numbering fixes `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| 0 | 2 | u16 | Stream number (C-10) at the bundle's source of the member this row reports on | 0 to 255 (C-09); the member admitted to this consumer at this row |
| 2 | 4 | u32 | Received: slices of this member accepted with content, neither late, missing nor a marked gap | any 32-bit value (Counters) |
| 6 | 4 | u32 | Late: slices counted missing at their deadline that arrived later, while their position was within the ring span, and were reclassified late and discarded (C-108) | any 32-bit value (Counters) |
| 10 | 4 | u32 | Missing: slices absent at their deadline and not reclassified late since; decreases by one on every reclassification (C-108) | any 32-bit value (Counters) |
| 14 | 2 | u16 | Marked gap: positions the producer marked with a gap slice (rule 10), counted in slices | any 16-bit value (Counters) |
| 16 | 4 | u32 | Other-epoch: packets on another timeline epoch, each counted under WrongEpoch and never a miss (C-135, C-102) | any 32-bit value (Counters) |
| 20 | 4 | u32 | Too-early: packets that arrived too early for the consumer to place their slice and were discarded, as the gap classification of epic 009 decides | any 32-bit value (Counters) |
| 24 | 4 | u32 | Duplicates: packets discarded as duplicates by stream, position and packet index, the first valid copy winning (rule 9) | any 32-bit value (Counters) |
| 28 | 2 | u16 | Incomplete reassemblies: multi-packet slices not complete at their reassembly deadline (C-15, C-110), counted in slices | any 16-bit value (Counters) |
| 30 | 2 | u16 | Demoted rounds: rounds of the consumer, one per slice period (C-156), in which this member was in the health state Demoted and treated as gapped (C-109) | any 16-bit value (Counters) |
| 32 | 2 | u16 | Interrupted events: entries of this member into Interrupted, 8 consecutive missed slices (C-38, C-102); events, not slices | any 16-bit value (Counters) |
| 34 | 2 | u16 | Dead: entries of this member into Dead, 2 s without data counted from the last slice with content or a marked gap (C-39, C-111); events, not slices | any 16-bit value (Counters) |
| 36 | 1 bit | uint | Observed: bit 0 of byte 36; 1 = the observed DSCP holds a value read from a received packet of this member, 0 = no packet read yet, or the consumer cannot read the DSCP of received packets | 0 or 1 |
| 36 | 1 bit | pad | Padding, bit 1 of byte 36: senders write zero (Reserved fields, general rule) | 0 only |
| 36 | 6 bits | uint | Observed DSCP: bits 7..2 of byte 36, the DSCP of the IPv4 header of the last packet of this member the consumer received; EF (46) is what the producer sent (C-13) | 0 to 63; senders write 0 when Observed is 0, and a receiver then ignores the value |

**Message type codes.** The message type is scoped to the measurement layer, which has its own receiving socket (Rejection reasons, Counting model): 1 is the receiver report, 2 the one-way probe request and 3 the one-way probe reply; 0 and 4 to 255 are unused in v1, so a zeroed buffer is never a valid message `[decision]`.

**The observed DSCP.** The consumer verifies the producer's marking by reading the DSCP of each received packet and reporting it (01-architecture.md § 3.5); it reports it per member, because the regime stamps carry the observed DSCP per connection (C-47) and each member is marked on its own `[decision]`. The Observed bit exists because a stack that cannot read the received DSCP would otherwise report 0, which reads as a remarking to the default class; with the bit at 0 the producer records the value as not observed (C-129's absent or unverified state, as 004-25 stamps it) `[decision]`.

### Counters

Every counter of the report — the eleven per-member counters and the socket and interface drops — is cumulative and wraps modulo 2^w at its width w, 32 or 16 bits as the tables state; no counter saturates `[decision]`. The per-member counters start at 0 when the connection is admitted; the drop counters carry the low 32 bits of the counters the consumer reads from its socket and its interface `[decision]`.

A producer reads each counter's change between two reports as the difference of the two values modulo 2^w, taken as a signed value from −2^(w−1) to 2^(w−1) − 1 `[decision]`. Nothing is lost with a lost report: the next report's cumulative values carry every count the lost one held `[decision]`. The producer builds the per-connection counters of its regime stamp (C-47, C-159) from these changes, at whatever width 004-25 lays them out; the report's widths bound only the change between two reports a producer uses `[decision]`.

**The width rule.** A width must keep a change unambiguous across every report a producer may miss: the change between two reports the producer uses must stay within −2^(w−1) to 2^(w−1) − 1, that is 32 767 for 16 bits `[decision]`. Two consecutive reports a producer uses are never more than 2 s apart, by the report number's rule (Report number below): after 2 s without a newer report the producer has timed out (C-41), and its next report after re-admission starts a new baseline `[decision]`. Each counter's worst-case change in those 2 s is derived below in the unit it counts, under every regime v1 admits: the frames field admits slices of 1 to 65 535 frames and the packet count 1 to 65 535 packets per slice (Slice header), so the slice rate S of a member is at most 96 000 per second, 1-frame slices at the domain rate (C-01), and its packet rate is bounded only by its admission `[decision]`:

| Counter | Unit | Worst-case change in 2 s | Width | Unambiguous |
|---|---|---|---|---|
| received | slices | 2S: at most 192 000; 4 000 at defaults (C-02) | 32 bits | under every admitted regime |
| late | slices | S × (2 s + ring span): the slices whose deadline passed in the window or within one ring span before it, the span being sized to A (C-96), at most 4.29 s (Current A); at most about 604 000 | 32 bits | under every admitted regime |
| missing | slices, signed | +2S new misses and −S × (2 s + ring span) reclassifications: within ±604 000 | 32 bits | under every admitted regime |
| marked gap | slices | 2S: at most 192 000; 4 000 at defaults | 16 bits | while S ≤ 16 383 per second, slices of 6 frames or more (Open item 2) |
| other-epoch | packets | the packets the member can deliver in 2 s, bounded by the receiving interface's line rate: under 300 million even at 100 Gbit/s | 32 bits | under every admitted regime |
| too-early | packets | as other-epoch: under 300 million | 32 bits | under every admitted regime |
| duplicates | packets | as other-epoch: under 300 million; the drain bound (C-16, C-156) gives 8 000 × k in 2 s at defaults with k packets per slice, above 32 767 from k = 5 | 32 bits | under every admitted regime |
| incomplete reassemblies | slices | 2S: at most 192 000 | 16 bits | while S ≤ 16 383 per second (Open item 2) |
| demoted rounds | rounds | 2S, one round per slice period: at most 192 000 | 16 bits | while S ≤ 16 383 per second (Open item 2) |
| interrupted events | events | 2S / 9: an entry needs 8 consecutive misses (C-38) and a re-entry a slice that ends the run first; about 21 333 | 16 bits | under every admitted regime |
| dead | events | 2: an entry needs 2 s without data (C-39, C-111) | 16 bits | under every admitted regime |

The packet bound is the line rate: an Ethernet frame occupies at least 84 bytes of wire time with its preamble and inter-frame gap, so 100 Gbit/s carries no more than 148.8 million frames per second, 297.6 million in 2 s, far inside the 2 147 483 647 that a 32-bit signed difference resolves `[memory]`. Widening all of them does not fit: a row with all nine slice and packet counters wide would take 43 bytes, and the report then exceeds the 1 252-byte bound adopted above `[decision]`. The six counters that the slice rate alone does not bound, or that carry the ring-span term, are therefore 32 bits — received, late, missing and the three packet counters — and the five others stay 16 bits: the two event counters are unambiguous under every admitted regime, and marked gap, incomplete reassemblies and demoted rounds are unambiguous up to 16 383 slices per second per member, eight times the default rate, which Open item 2 hands to the admission check that owns the slice rate `[decision]`.

**The C-108 reclassification.** The report carries the reclassification as it happens: the missing counter is the C-159 missing class, so it is not monotonic — it decreases by one on every reclassification while the late counter increases by one — and the signed difference keeps the pair unambiguous: between two reports missing changes by the slices newly missing less the slices reclassified, and late by the slices reclassified, so a producer sees a negative change of missing exactly when reclassifications outnumber new misses `[decision]`. Every other counter only grows, and its change reads non-negative `[decision]`.

**Interrupted, demoted, dead.** The interrupted counter counts interrupted events, not slices: one per entry into Interrupted after 8 consecutive missed slices (C-38), where only missing or late slices count as misses and a marked gap or a slice on another timeline epoch resets the count (C-102) `[decision]`. Demoted is carried by the demoted-rounds counter (C-109, C-159), in the report as in the regime stamp: a third party tells a demoted input from a gapped one by that counter, which grows only while the input is Demoted, whereas a gapped input grows marked gap or missing `[decision]`. The dead counter counts entries into Dead, events like the interrupted counter `[decision]`.

### Report number

The report number of a connection's first report after admission is 0, and the consumer increments it by one for every later report of that connection, modulo 2^16; a re-admission starts a new connection, whose first report is 0 again `[decision]`. A producer compares report numbers in serial order over half the 16-bit range: a number r is newer than the last used number u exactly when (r − u) modulo 2^16 lies in 1 to 32 767, and within 2 s at most 20 reports pass (C-40), far inside that range `[decision]`. A producer always uses the first report of a connection, whatever its number, because admission and re-admission reset its baseline `[decision]`.

Only a report newer than the last one used is a further receiver report in the sense of C-41 ("2 s without receiver reports"): it refreshes the producer's timeout and updates its counters and its A `[decision]`. A report that is not newer — reordered or duplicated — repeats information the producer already holds: it neither refreshes the timeout nor updates the counters or the A; it is a valid message, never a rejection `[decision]`. This is the field's meaning on the wire, not a state machine, which epics 005 and 006 own; it makes the width rule's premise true by construction, because two reports a producer uses are at most 2 s apart or the producer has already timed out `[decision]`.

### Bundle identity and member rows

A report names its bundle by the module id of the bundle's source (C-11) and its members by stream number `[decision]`. No bundle identifier has landed: the manifest's bundle group id (01-architecture.md § 7) has no wire width yet (004-26), and the module id with any row's stream number names exactly one bundle, because a source numbers its streams once (C-09, C-10; streams are identified by source and stream number, rule 7) and a stream belongs to one bundle `[decision]`. Falsifier: a stream that belongs to two bundles of one source, or a landed bundle identifier that a producer needs to tell two of its bundles apart; either reopens this field `[decision]`.

The report names its sender by the reporter identity, the consumer's module id (C-11), never by its source address: two consumer modules on one host (01-architecture.md § 2.6) share an address and can be admitted to the same bundle, and their counters, their A and their C-41 liveness must stay apart `[decision]`. A producer keys a report to a connection by the pair (bundle identity, reporter identity) `[decision]`.

The rows follow the order of the member list in the consumer's admission handshake (004-18): one row per member admitted to this consumer, which for a narrow sink is a subset of the bundle (01-architecture.md § 7), each row starting with that member's stream number, so a producer matches rows by position and checks each by its number `[decision]`. A report whose member count is not the number of members admitted to the reporting consumer is dropped and counted under BadLength: for one admitted connection the report has exactly one size, 38 + 37 × n bytes for its n admitted members, and a report of another size does not fill it `[decision]`. A row whose stream number is not the member admitted at that row is dropped and counted under UnknownStream, whose meaning names this case on the measurement layer (Rejection reasons) `[decision]`.

A report whose bundle identity is not the receiving producer's own module id, or whose reporter identity names no module with an admitted connection for the bundle its rows name, reports on no connection of this producer: it is dropped and counted under UnadmittedSource `[decision]`. UnadmittedSource is the closest member of the closed enumeration — a source with no admitted connection at the receiver — read in the reverse direction; its Layers cell now names the measurement layer and its meaning names this case, which adds no reason to the enumeration (Rejection reasons, Closed enumeration) `[decision]`.

### Current A

The current A field carries the consumer's current A (C-198): its own presentation group's A (C-55), the A it inherited from the downstream group it feeds (C-142), or the explicit budget it is configured with when it feeds no group (C-142); it is the A the consumer was given, never a clamped value (C-216) `[decision]`. The unit is nanoseconds, a duration on network time: A is added to a position's network time in the emission formula emit(p) = T(p) + A (01-architecture.md § 9.1; C-226), and timeline.md counts network time in nanoseconds `[decision]`. The width is 32 bits unsigned, 0 to 4 294 967 295 ns, about 4.29 s: A is the mouth-to-ear latency of its group (01-architecture.md § 9.1), and no constant states a bound on it, so the width is a decision whose falsifier is a commissioned A above that value (Open items) `[decision]`. The value 0 means no A known to the reporter: it has no group's A, no inherited A and no configured budget to report; a producer that takes the largest A of the groups it feeds (C-142) counts 0 as no value `[decision]`. Whether a module that aligns nothing relays the A of its downstream group upstream in its own reports is not settled by C-142 or C-198, and this section does not decide it (Open item 4) `[decision]`. Every 32-bit value is therefore valid, and no value of this field is a malformed case `[decision]`.

The field has the same width and unit as the downstream A field of 004-18 (C-142), and this section binds 004-18 to them: 32 bits of nanoseconds, with 0 for no A known (Open items) `[decision]`. A change of A propagates upstream one hop per report, on the data plane, and each fan-in that inherited the old value re-aligns (C-198, C-186); this field is the only carrier of a changed A after admission `[decision]`.

### Maximum size

A report with n members is 38 + 37 × n bytes: the fixed part and one row per member `[decision]`. With the maximum of 32 members (C-08) a report is 38 + 32 × 37 = 1 222 bytes `[decision]`. That fits the bound adopted above, at most 1 252 bytes of UDP payload on the fallback path, with 30 bytes to spare `[decision]`.

### Validity rules

Every malformed measurement message is dropped — or ignored, for the unconfigured domain — and counted under one member of the closed Rejection reasons enumeration; nothing is dropped silently `[decision]`. A measurement message matches no admitted input (Rejection reasons, Counting model), so every rejection on this layer is counted per (socket, reason) on the receiving measurement socket `[decision]`.

The rules below apply to both measurement messages, the report and the probe, in this order, and the first that applies decides the outcome `[decision]`: a datagram under 4 bytes; the magic; the version's major; a datagram of 4 to 7 bytes, too small for bytes 0 to 7 (the common header with the report's member-count byte or the probe's padding byte); the domain, 0 before foreign; the message type; the size its type fixes (for a report the member count first, then 38 + 37 × member count, which a report under the 38-byte fixed part never meets; for a probe 48 bytes); any padding or reserved bit; then the identity rules (for a report the bundle identity and reporter identity, the admitted member count and the rows' stream numbers; for a probe those of One-way probe, Validity rules). A message of the same major and another minor is accepted, as Versioning, Receiver behaviour fixes `[decision]`:

| Invalid form | Required outcome |
|---|---|
| too short: a datagram under 4 bytes, too small for the two leading fields every message starts with | drop and count under TooShort: dropped before any field is read `[decision]` |
| bad magic (the leading two bytes are not 0x4d4e) | drop and count under BadMagic |
| version mismatch (the version's major, its high byte, is not 1: a different major) | drop and count under VersionMismatch, and raise the VersionMismatch event — never a silent drop (Versioning, Receiver behaviour) |
| datagram size of 4 to 7 bytes, too small for bytes 0 to 7 (the common header and the member-count or padding byte) | drop and count under BadLength: the body does not fill the fixed size `[decision]` |
| domain id 0, the unconfigured domain (C-11) | ignore and count under UnconfiguredDomain — never ForeignDomain (C-173) |
| foreign domain (a non-zero domain other than the local domain, C-11) | drop and count under ForeignDomain |
| message type not one the receiving module expects: any value outside 1 to 3, or 1 (a receiver report) at a module that sources no bundle with an admitted consumer | drop and count under BadLength `[decision]` — provisional, the message's declared kind disagrees with the receiving role, as the sync section rules; confirmed by 004-21 (Open items) |
| member count of 0 or above 32 (C-08) | drop and count under BadLength: a count outside its fixed range `[decision]` |
| datagram size inconsistent with the member count: not 38 + 37 × member count bytes | drop and count under BadLength: the body does not fill the size the count declares `[decision]` |
| a padding or reserved bit non-zero: bit 1 of byte 36 of a member row, any padding byte of a probe request or reply, or bits 7..1 of the probe reply's clock state | drop and count under ReservedNonZero (Reserved fields, general rule) |
| unknown bundle: the bundle identity is not the receiving producer's own module id, or the reporter identity names no module with an admitted connection for the bundle its rows name | drop and count under UnadmittedSource `[decision]` |
| member count of the bundle's report not the number of members admitted to the reporting consumer | drop and count under BadLength: one admitted connection fixes one report size `[decision]` |
| a row of the bundle whose stream number is not the member admitted to the reporting consumer at that row | drop and count under UnknownStream `[decision]` |
| observed DSCP above 63 | cannot occur: the field is 6 bits and holds 0 to 63 only; a non-zero padding bit of its byte is dropped and counted under ReservedNonZero `[decision]` |
| current A out of its valid range | cannot occur: every 32-bit value is valid, 0 meaning no A known (Current A); never a rejection `[decision]` |

No value of the report number, a counter, a drop counter or the current A is a malformed case: each accepts every value its type holds, and only the rules above reject `[decision]`.

### Open items

**Open item 1 — the A field and the member order at admission.** The current A is laid out here, 32 bits of nanoseconds with 0 for no A known, ahead of 004-18's downstream A field (C-142), which must have the same width and unit; and the rows follow the member list of the consumer's admission handshake, which 004-18 lays out. Owner: ticket 004-18 (admission handshake messages). Falsifier: 004-18 lays out the downstream A field with a width or a unit other than 32 bits of nanoseconds, or with no value for no A known, or its handshake carries no ordered member list for the consumer's admitted members; or a commissioned A exceeds 4 294 967 295 ns; any of these reopens this layout `[decision]`.

**Open item 2 — slice rate of the three 16-bit slice counters.** Marked gap, incomplete reassemblies and demoted rounds are 16 bits and stay unambiguous while a member's slice rate is at most 16 383 per second, slices of 6 frames or more; the slice header admits slices down to 1 frame (96 000 per second), and ADR 01 leaves shorter slices than the default to configuration. Whether a stream is admitted at a given slice rate is the receiver's SliceRate check at admission, which ticket 005-14 (admission handshake transition table) encodes, with the refusal reason SliceRate (C-185). Owner: ticket 005-14. Falsifier: 005-14's SliceRate check admits a member stream above 16 383 slices per second; then these three counters can wrap ambiguously within the producer's 2 s timeout (C-41), and the layout is reopened to widen them `[decision]`.

**Open item 3 — message-type outcome.** An unexpected message type is counted under BadLength provisionally, as in the sync section. Owner: ticket 004-21, the validity-rules completeness audit. Falsifier: 004-21 finds a member of the closed enumeration that names an unexpected message type better than BadLength `[decision]`.

**Open item 4 — A through a module that aligns nothing.** C-142 and C-198 do not say whether a module that aligns none of its inputs relays the A of its downstream group upstream in its own receiver reports, or reports 0 (no A known); the field carries either. Owner: ticket 005-28 (changed A propagation and fan-in re-alignment), with 005-27 (fan-in A inheritance and alignment opt-out). Falsifier: a fan-in upstream of a module that aligns nothing never receives the A, or a changed A, of the presentation group downstream of that module, although C-142 says it inherits it `[decision]`.

## One-way probe

The one-way probe is a request and its reply between two modules that share an admitted connection, in either direction: either end, producer or consumer, sends a request on demand, and the other end answers each request with one reply `[decision]`. Four instants close the exchange: the requester's send time, the responder's receive and send times, and the requester's own receive time of the reply, which never travels, as in the sync exchange `[decision]`. It is not the path-MTU probe pair of the admission handshake (C-116), which is a control message that 004-18 lays out `[decision]`.

### Exchange rules

Probes travel by UDP unicast over IPv4 (C-100) at DSCP EF (46) (C-13), the media class, so they cross the network in the queue the media packets use and measure the media path `[decision]`. A consumer sends its requests to the reverse port its producer gave in the admission handshake (004-18); a producer sends its requests to the source address and port the consumer's reports come from; a reply goes to the source address and port of its request, which is all the addressing a stateless reply needs `[decision]`. The sender sets the IPv4 don't-fragment bit (ADR 01) `[decision]`. A probe is sent on demand: no field carries a rate, and any limit on how often a module probes or answers is behaviour, which this section does not fix `[decision]`.

The responder is stateless: the reply echoes the request's requester identifier and send timestamp verbatim, so the responder answers each request from that request alone, and the requester pairs a reply with its request by the echoed values, as the sync exchange does (Sync request and reply, Exchange rules) `[decision]`. A stale or duplicate reply is therefore a valid exchange with an honestly large round trip, never an invalid form `[decision]`.

**Matching fields.** A reply matches when its echoed requester identifier is the receiving module's own module id and its responder module id is the module id of a module that shares an admitted connection with the receiving module `[decision]`. A reply that does not match was meant for another module, or comes from a module that is no peer: it is dropped and counted under UnadmittedSource, as is a request whose requester identifier names a module that shares no admitted connection with the responder; UnadmittedSource's Layers cell names the measurement layer and its meaning names these cases (Rejection reasons) `[decision]`.

### Request

Little-endian (Byte order), packed with no padding for alignment (Alignment and padding); offsets are byte distances from the first byte of the UDP payload. The request carries the common header, the requester's identity and its send timestamp, then explicit padding to the reply's size `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| 0 | 2 | u16 | Magic 0x4d4e, the ASCII letters M and N, fixed by 004-04 | 0x4d4e only |
| 2 | 2 | u16 | Protocol version, major in the high byte (bits 15..8), minor in the low byte; v1 packs 1.0 as 0x0100 (004-04) | major 1, any minor (Versioning, Receiver behaviour); senders write 0x0100 |
| 4 | 2 | u16 | Domain id (C-11); 0 is the unconfigured domain | 1 to 65535; 0 = unconfigured |
| 6 | 1 | u8 | Message type: 2 = one-way probe request (Receiver report, Message type codes) | 2 only |
| 7 | 1 | pad | Padding: senders write zero (Reserved fields, general rule) | 0 only |
| 8 | 8 | u64 | Requester identifier: the requester's module id (C-11), echoed by the reply | any 64-bit value; a responder answers only a module that shares an admitted connection with it |
| 16 | 8 | i64 | Send timestamp: the requester's network time in nanoseconds when it sends the request (What the probe measures) | any 64-bit value |
| 24 | 24 | pad | Padding: brings the request to the reply's size, the room the reply's responder module id and its two timestamps take; senders write zero (Reserved fields, general rule) | 0 only |

### Reply

Same conventions as the request; bytes 0 to 6 are the common header and byte 7, padding in the request, carries the responder's clock state; the echoed fields sit at the request's offsets, so a responder may build the reply over the request's own bytes, and the padding's room carries the responder's values `[decision]`:

| Offset (bytes) | Width (bytes) | Type | Meaning | Valid range |
|---|---|---|---|---|
| 0 | 2 | u16 | Magic 0x4d4e, the ASCII letters M and N, fixed by 004-04 | 0x4d4e only |
| 2 | 2 | u16 | Protocol version, major in the high byte (bits 15..8), minor in the low byte; v1 packs 1.0 as 0x0100 (004-04) | major 1, any minor (Versioning, Receiver behaviour); senders write 0x0100 |
| 4 | 2 | u16 | Domain id (C-11); 0 is the unconfigured domain | 1 to 65535; 0 = unconfigured |
| 6 | 1 | u8 | Message type: 3 = one-way probe reply (Receiver report, Message type codes) | 3 only |
| 7 | 1 bit | uint | Usable time: bit 0 of byte 7, the responder's clock state when it stamped the reply; 1 = its time was usable, Locked or in holdover with its alignment claim not lapsed (C-213, C-65), 0 = acquiring, or Unlocked with time unusable (C-162) (Clock state) | 0 or 1 |
| 7 | 7 bits | pad | Reserved, bits 7..1 of byte 7: senders write zero (Reserved fields, general rule) | 0 only |
| 8 | 8 | u64 | Echoed requester identifier: the request's requester identifier, echoed verbatim | any 64-bit value; it matches only the receiving module's own module id (Matching fields) |
| 16 | 8 | i64 | Echoed send timestamp: the request's send timestamp, echoed verbatim; the requester's reading, never the responder's | any 64-bit value |
| 24 | 8 | u64 | Responder module id (C-11): the module id of the module that answers | any 64-bit value; it matches only a module that shares an admitted connection with the requester (Matching fields) |
| 32 | 8 | i64 | Receive timestamp: the responder's network time in nanoseconds when the request was received | any 64-bit value |
| 40 | 8 | i64 | Send timestamp: the responder's network time in nanoseconds when the reply is sent | any 64-bit value |

Request and reply are both 48 bytes long: the request's fields sum to 2 + 2 + 2 + 1 + 1 + 8 + 8 + 24 = 48 bytes and the reply's to 2 + 2 + 2 + 1 + 1 + 8 + 8 + 8 + 8 + 8 = 48 bytes, byte 7 holding the clock state, each tiling its message from byte 0 with no gap and no overlap `[decision]`. Equal sizes give both directions the same serialisation time, so the forward and reverse one-way delays differ only by the path and its queues, never by the message's size, and a receiver judges a probe's size before it reads which of the two it is `[decision]`. Both fit the 1 252-byte bound adopted for the measurement layer (Receiver report, Size bound of the measurement layer) `[decision]`.

### What the probe measures

All three timestamps on the wire are readings of network time — the holder's monotonic clock (01-architecture.md § 2.3), as each end's servo publishes it, the holder reading its own clock — in signed 64-bit nanoseconds, the representation timeline.md fixes for network time; with t1 the request's send timestamp, t2 and t3 the reply's receive and send timestamps, and t4 the requester's network time at the receipt of the reply, the exchange yields both the round-trip delay and a one-way delay in each direction `[decision]`:
- Round trip = (t4 − t1) − (t3 − t2), free of clock agreement: t1 and t4 are readings of the requester's network time and t2 and t3 of the responder's, so each end's offset from the holder cancels. It is exact up to the difference of the two ends' published rates over the exchange: each end stays within 10 ppm of the holder's rate while locked (C-60), so the two differ by at most 20 ppm, 20 ns per millisecond of exchange; an exchange across a phase step at either end (acquisition is exempt from the slew limits, 01-architecture.md § 2.4) is spoiled `[decision]`.
- One-way delay forward = t2 − t1 and reverse = t4 − t3, each only as good as the two ends' agreement on network time: its error is the sum of the two ends' offset errors, which each end's published error bound states (01-architecture.md § 2.4, lock is a number), and no in-band method observes the path asymmetry hidden inside that error (§ 2.4, asymmetry). A one-way figure is therefore usable only when both ends' time was usable as they stamped: the requester knows its own clock state, and the reply's usable-time bit states the responder's (Clock state below); it carries both ends' bounds (01-architecture.md § 11.1: one-way figures are blank until lock and carry the bound thereafter) `[decision]`.

Falsifier: a measurement consumer needs a one-way delay the probe's timestamps cannot yield — one with an error below the sum of the two ends' published bounds, or one between modules whose time is not both usable — in which case this decision is wrong and epic 004 reopens the probe's contents `[decision]`.

The timestamps have the same width, 64 bits, and the same unit, one tick 1 ns, as the slice-header send timestamp (Slice header, Send timestamp) and the sync timestamps (Sync request and reply, Timestamps), but a different clock: those read the sender's local monotonic clock, opaque to the receiver, while the probe's read network time, because a one-way delay needs both ends on one clock; the type is therefore i64, as timeline.md fixes network time, where the slice header and the sync messages use u64 `[decision]`. A module whose servo publishes no network time yet sends no probe and answers none; the holder always has network time, its own clock `[decision]`.

**Clock state.** The reply's byte 7 carries the responder's clock state in the landed vocabulary of the servo: time is usable when the clock is Locked or in holdover with its alignment claim not lapsed (C-213, C-65), and unusable after the claim lapses, when one-way figures stop (C-162); the servo core publishes that state (C-187) `[decision]`. Holdover before the lapse counts as usable because C-162 stops one-way figures only at the lapse; the bound itself does not travel, and the requester reads it from the responder's regime stamp, Clock group (01-architecture.md § 11.1) `[decision]`. With the bit clear, the requester may still use the round trip, which needs no clock agreement, but it uses neither one-way figure of that exchange `[decision]`. The request's byte 7 stays padding, because the responder needs no state of the requester's, and both messages stay 48 bytes `[decision]`. With the 2^63 ns span of an i64, about 292 years, no exchange sees a wrapped timestamp `[decision]`.

### Validity rules

The probe follows the rule order of Receiver report, Validity rules, whose header, padding and reserved-bit rows apply to it as well, and every rejection is counted per (socket, reason) on the receiving measurement socket; its own forms and their required outcomes `[decision]`:

| Invalid form | Required outcome |
|---|---|
| probe datagram size not 48 bytes, at 8 bytes or more | drop and count under BadLength: the body does not fill the fixed size `[decision]` |
| a request whose requester identifier matches no module that shares an admitted connection with the responder | drop and count under UnadmittedSource `[decision]` |
| reply without matching fields: the echoed requester identifier is not the receiving module's own module id, or the responder module id names no module that shares an admitted connection with the receiving module | drop and count under UnadmittedSource `[decision]` |
| stale or duplicate reply with matching fields (its echoed send timestamp is old) | none — a valid exchange with an honestly large round trip; never a rejection `[decision]` |

No value of a timestamp or a module id field is a malformed case beyond the matching rules above: each accepts every value its type holds `[decision]`.

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
