# 010 — Packetiser and bundles core

**Phase:** P3 · **Scope:** L (24 tickets, about 23 days) · **Depends on:** 004, 006, 007, 009 · **Blocks:** 012, 014, 015, 017, 024, 026

## Goal

Implement the send side of the data plane in `hadronyche-core`, as pure `no_std` code with no sockets and no allocation: slice and packet encoding, position stamping, channel bundles, the multi-packet regime, the intra-slice pacing schedule with per-source phase, gap slices, and in-band announcements with their lead and re-issue.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules (sources, processors, sinks) exchange *slices*: the content of one stream for one stretch of media time, addressed by *position* on a domain-wide timeline (position = network time counted in samples from a domain epoch). There is one sample rate on the wire, the domain rate (C-01), and therefore one timeline per domain; an edge whose hardware runs at another rate converts at the edge (C-80). Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers.

**Evidence tags.** `[code]` read in the POC · `[decision]` design choice · `[measured]` measured on a named machine or in CI · `[kernel-doc]` taken from kernel documentation · `[standard]` cited from a named, published standard with the clause, replacing `[memory]` once the citation is checked (C-132) · `[memory]` recalled, unverified.

**Where outputs go (C-130).** Design-document updates made by this epic go to the design set in `doc/design/` (the constants table is `doc/design/02-constants.md`, specifications are in `doc/design/spec/`); measured figures such as the phase-offset evaluation go in a report under `doc/design/reports/` and are cited from the design note. The architecture specification is `doc/design/01-architecture.md` (C-171). Nothing is written into the frozen POC tree.

### Constants `[decision]` unless marked

The constants table of the design set is canonical; the values below repeat it.

| Id | Constant | Value |
|---|---|---|
| C-01 | Domain sample rate | 96 000 Hz, the only rate on the wire; every stream is at the domain rate |
| C-02 | Slice length (default) | 48 frames = 0.5 ms = 2 000 slices/s (inherited) |
| C-03 | Sample format | 4 bytes per sample, f32 or Q1.31; no 3-byte variant |
| C-04 | Channels per stream at defaults | 6: 192 bytes per channel per slice (48 × 4, no per-channel id byte; the channel map is in the manifest), 1 152 bytes of audio, about 320 bytes left in a 1 472-byte payload. Falsifier: headers plus in-band content exceed those ~320 bytes |
| C-05 | Default slice | fits one packet at the path MTU; multi-packet is an explicit, stamped regime |
| C-06 | Path-MTU fallback | 1 280 bytes |
| C-07 | Copy index | 2 bits, reserved; v1 always sends copy 0 |
| C-08 | Members per bundle | at most 32, the controller member included (C-209) |
| C-09 | Streams per source | at most 256 |
| C-10 | Stream number / session / timeline epoch | 16 / 32 / 16 bits |
| C-14 | Intra-slice egress spread | at most 25 % of the slice period (125 µs), plus a per-source phase offset derived from the module id |
| C-18 | Regime stamp transport | a dedicated control message split into sections that each fit one fallback-MTU packet; multicast on any regime change and refreshed every 10 s; the hello carries only a stamp version number |
| C-19 | Session number | assigned by the source, one per stream; bumped on restart and on timeline epoch change; every consumer of a stream sees the same value |
| C-20 | Wire size of one default packet | about 1 320 bytes with all Ethernet overhead (derived) |
| C-21 | Wire rate per stream | about 21.1 Mbit/s, about 3.5 Mbit/s per channel (derived) |
| C-22 | Wire rate per source | 24 ch: 4 streams, 84 Mbit/s, 8 000 packets/s. 48 ch: 8 streams, 169 Mbit/s, 16 000 packets/s. 96 ch: 16 streams, 338 Mbit/s, 32 000 packets/s (derived) |
| C-43 | Announcement lead | 64 slices (32 ms at defaults), flat |
| C-44 | Announcement re-issue | every 16 slices until the named position passes, in every bundle member |
| C-47 | Stream health observability | producer and consumer each publish per-connection summary counters in their own regime stamp: one counter per gap class (received, late, missing, marked gap), plus interrupted events, dead, and the observed DSCP |
| C-95 | Fuzz budget | each parser 60 s on every change and 1 hour nightly; the corpus is kept |
| C-100 | IP version | IPv4 only in v1. IPv4 + UDP headers are 28 bytes: 1 472 bytes of UDP payload at MTU 1 500, 1 252 bytes at the 1 280-byte fallback |
| C-102 | Miss for the interrupted threshold | only slices absent at the deadline; a marked gap proves the upstream is alive and resets the count |
| C-105 | Structural latency L | always a whole number of domain-rate samples; the earlier rational-L rule is withdrawn (a resampler node exists only inside an edge module, C-134; its conversion delay is part of D, never of L, C-207) |
| C-111 | Start of the dead threshold | 2 s counted from the last slice received with content or a marked gap |
| C-116 | Path-MTU probe size | the probe pair is sent at the intended media packet size |
| C-120 | Test-only injection hooks | include forcing a stream onto a given timeline epoch; stamped like any injection |
| C-122 | Degraded flag on an output with latency L | follows the content. Stamps denote content position: while a module processes input position p it emits the output stamped p − L, whose content came from input position p − L; that output is degraded if the input at p − L was gapped |
| C-137 | Header cap | a default slice must fit 1 252 bytes of UDP payload: 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum. In-band announcements and the measurement extension fit inside that cap or are omitted on a fallback path |
| C-144 | Owner of orphan duties | the hosted backend (epic 015) emits the gap slice when a module misses its own deadline, and sends the regime stamp message that epic 013 defines |
| C-155 | Pacing fit | phase offset plus spread stay inside the slice period: the phase offset is in [0, 75 %) of the period |
| C-159 | Per-connection counters in the stamp (completes C-47) | received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead |
| C-165 | Operator policy | the commissioning configuration sets strict mode, per-input alignment opt-out and pacing override |
| C-181 | Where controllers travel | a bundle source's controllers always ride in their own member stream (C-205). A single-stream source may always put its controllers in a second stream, and must do so when headers plus controllers exceed the 100-byte cap (C-137); it may share the audio packet while they fit. The compiler checks this from the manifest |
| C-205 | Controllers of a bundle source | always in their own member stream |
| C-207 | Resampler delay and L | an edge's resampler delay is counted only in D and never in L; L is the module's structural delay in the domain-rate timeline; the manifest reports the conversion delay separately, on the device-facing port |
| C-208 | Re-alignment position after a changed A | each fan-in that learns of the change picks its own position, one lead (C-43) ahead, and announces it in its own output, exactly as for a ratchet-triggered re-alignment; fan-ins at different depths switch at different, announced positions |
| C-209 | Controller member and the bundle cap | the controller member counts towards the 32 of C-08: a source with controllers has at most 31 audio members (186 channels); a wider source with controllers is rejected at compile time |
| C-215 | A ratchet step | is a re-alignment like any other: it takes effect at a position the fan-in announces one lead (C-43) ahead in its own output, never silently on the next round |
| C-219 | Wire cost of a controller stream | computed like an audio stream: one packet per slice of (headers up to the 100-byte cap + controller bytes per slice + 66 bytes of Ethernet, IP and UDP overhead), at 2 000 packets/s. Admission and the commissioning tool use this same figure |

### Slices and packets `[decision]`

- The handler on the receiving side gets a slice whole or not at all.
- By default a slice is one packet. Every packet is sent with don't-fragment set; IP fragmentation is forbidden. IP is IPv4 only (C-100), so the UDP payload is the path MTU minus 28 bytes.
- Sizing: the audio of one channel for one slice is `frames × 4` bytes (192 at defaults, C-04). The number of channels that fit one packet is ⌊(path MTU − 28 − h) / (frames × 4)⌋, where h is the header budget (packet header, slice header and the reserve for in-band content) fixed by the wire specification (epic 004). At defaults a stream carries 6 channels (C-04); at the 1 280-byte fallback the payload is 1 252 bytes, and the header cap (C-137) limits all headers and the checksum to 100 bytes, so a default 6-channel slice always fits one packet, also at the fallback. In-band announcements and the measurement extension that do not fit inside the cap on a fallback path are omitted from that slice (C-137).
- A slice larger than the path MTU is the **multi-packet regime**: the runtime splits it into ordered packets, each carrying the slice identity, its index, the packet count and **its own send timestamp**, and the regime is recorded in the regime stamp. Transport state, controllers and announcements go in packet 0; audio fills the rest. Controllers appear there only for a single-stream source in the shared case (C-181); a bundle source's audio members never carry controllers (C-205).
- Packet layout: packet header (magic, version, domain, stream number, session, timeline epoch, copy index, flags incl. gap and degraded), slice header (64-bit position, frames, packet index and count, send timestamp), optional announcements, optional measurement extension (hop count, accumulated residence, accumulated declared delay), content messages, slice checksum. Exact offsets come from the wire specification (epic 004).
- Streams are identified by source and stream number, never by destination. **Forward packets carry no per-receiver state**, so the same bytes can go to several consumers.
- The intended media packet size computed here is also the size of the path-MTU probe pair (C-116); this epic exposes it, the backend sends the probe.
- Encoders never panic on a short output buffer: `finish` and `gap_slice` return an error and write nothing usable.

### Position stamping `[decision]`

Sources pace position from network time, not from a local crystal. The runtime stamps output positions; module code cannot change them. A module with structural delay L has its output stamped p − L while it processes input p. Stamps are never restamped per hop. L is always a whole number of domain-rate samples (C-105), so p − L is an exact integer position and no rounding exists. Stamps denote content position (C-122): the output emitted while processing input p is stamped p − L and its content came from input p − L, which is also the input whose gap makes it degraded. Positions count from the domain epoch, so no position exists before it: when p < L there is no output position and `output_position` returns none.

### Bundles `[decision]`

Streams stay narrow; a wide source emits a **bundle** of aligned streams.

- A bundle is a group id, a member list and a channel map, declared in the manifest. At defaults each member carries 6 channels; a 96-channel source is 16 members.
- Members are independent on the wire: per-member gaps and per-member interrupted/dead.
- Each member stream has its own session number, assigned by the source (C-19). All members of a bundle bump their sessions together at a timeline epoch switch. One admission handshake lists the members a consumer wants; a narrow sink subscribes to only the members it needs.
- Bundle-level state (the member packetisers, announcements shared by every member, the epoch switch applied to every member at once) lives in a bundle packetiser that holds one per-stream packetiser per member in a fixed array.
- A bundle source's controllers always ride in their own member stream (C-205), whatever their size; no audio member carries controller content. The controller member counts towards the 32 of C-08 (C-209): a source with controllers has at most 31 audio members (186 channels), and the bundle layout refuses to add a controller member beyond that; the compile-time rejection of a wider source is the compiler's.
- A source with a single stream and controllers (C-181): the source may always put its controllers in a second stream, and must do so when headers plus controllers exceed the 100-byte cap (C-137); it may share the audio packet while they fit. The second stream is laid out as a 2-member bundle (audio member plus controller member). The choice is declared in the manifest and the compiler checks it; this epic implements both layouts and refuses a shared layout whose controllers do not fit the cap.
- The wire cost of a controller stream (C-219) is computed like an audio stream: one packet per slice of header bytes (at most the 100-byte cap) plus controller bytes per slice plus 66 bytes of Ethernet, IP and UDP overhead, at 2 000 packets/s. `wire_bits_per_second` and `packets_per_second` count the controller member this way, so admission and the commissioning tool use the same figure.
- Announcements are re-issued in every member, because subscribers may take a subset.
- Stated cost: "all content for a time stretch arrives together or not at all" holds **per member**, not per source.

### Pacing `[decision]`

- The packets of one slice period (for a 96-channel source: 16 packets) are spread over at most 25 % of the period. A spread above 250 ‰ violates C-14 and is refused.
- Each source adds a phase offset derived from its module id. Without it, every source sharing the timebase fires on the same slice boundary and bursts arrive coherently at a switch port. The offset is a deterministic function of the id; since offsets take finitely many values, two different ids can share one, and the requirement is an even spread over the at most 64 active modules of a medianet (C-34), not distinctness for every pair.
- The phase offset lies in [0, 75 %) of the slice period, so phase offset plus the at most 25 % spread stays inside the period (C-155).
- Pacing runs in userspace on the deadline clock so per-packet send timestamps stay true. Per-output override: 0 disables spreading; the override is set by operator policy in the commissioning configuration (C-165) and reaches this code as a parameter.
- This task computes the schedule (a list of send instants); the backend executes it. The schedule covers all packets of one source's slice period: at defaults one packet per stream, so up to 256 (C-09), hence the packet count is 16 bits. The capacity P is a const parameter chosen by the caller; a request for more than P packets is refused with an error.

### Gap slices and announcements `[decision]`

- A **gap slice** is header-only and marks positions the sender did not produce or receive. Being header-only, it carries no announcements and no content. If a module misses its own deadline the hosted backend (epic 015) emits a gap slice for it through `gap_slice` (C-144). An arrival-driven module that sees a jump in input positions emits gap slices for the missed positions belatedly; they serve classification, not timing. They are kept: consumers count marked gaps as their own class (C-47), a marked gap resets the interrupted count (C-102) and restarts the dead threshold (C-111).
- `gap_slice` takes a send timestamp like any other slice.
- The **degraded flag** marks a slice computed from a gapped input (C-122); the caller decides it and passes it in the slice flags.
- **Announcements** (end-of-stream, re-alignment, timeline epoch switch, transport reference change) name a future position: current position + 64 slices, converted to samples with the stream's slice length (3 072 samples at defaults). The lead is flat (C-43). They are re-issued every 16 slices until the position passes. Consumers process positions in order, so an announcement is always processed before the position it names; the lead only has to survive loss.
- A module that relays a received announcement passes its target position on **unchanged**: no recomputation and no shift by the module's L (system criterion AC-17). The relay entry point is separate from `announce`, which always computes a fresh target. Repeated receipts of the same re-issued announcement give one pending entry.
- A re-alignment after a changed A is not relayed (C-208, C-186): each fan-in that learns of the change picks its own position one lead ahead and announces it in its own output through `announce`, as for a ratchet-triggered re-alignment. A ratchet step is the same (C-215): a re-alignment the fan-in announces one lead ahead in its own output through `announce`, never applied silently on the next round.
- Epoch switch on the source side: learn the new line from sync, then call `switch_epoch`, which issues the timeline-epoch-switch announcement itself with target T = current position + 64 slices (C-43, flat) and returns T. At T every member of the bundle ends its old session and starts a new one: session bumped (C-19), new epoch.

### Send-side observability

- Packets per slice and pacing mode (spread and whether a phase offset is applied) are exposed for the regime stamp. The regime stamp itself is a sectioned control message (C-18) defined by epic 013 and sent by the hosted backend, epic 015 (C-144); this epic only exposes the field values, and a change of any of them is a regime change.
- For the producer's C-47 summary (counter list completed by C-159), the packetiser keeps per-stream counters of slices encoded with content and of gap slices encoded (marked gap), allocation-free and monotonic, plus a counter of in-band items omitted under the header cap on a fallback path (C-137).
- For tests only (C-120), behind a test-only feature, a stream can be forced onto a given timeline epoch; the forced state is reported so it can be stamped like any injection.

`[code]` POC behaviour retired: one 9.6 KB datagram per slice for a 48-channel feed (7 IP fragments at MTU 1500, about 150 Mbit/s); send timestamp taken once before a per-destination `sendto` loop (`sender.rs:290`, `module.rs:979-986`); the ticker on local monotonic time (`module.rs:1630-1634`); events and audio already sent as two datagrams with the same header.

## Deliverables

All in `hadronyche-core`, `no_std`, no allocation.

```rust
// shape, names may be refined
pub struct StreamId { pub number: u16 }
pub struct BundleLayout<const M: usize> {       // M <= 32, controller member included (C-209)
    pub group: u16, pub members: [MemberLayout; M], pub len: u8,
}
pub struct MemberLayout { pub stream: StreamId, pub content: ContentType, pub channels: u8, pub first_channel: u16 }

pub const fn channels_per_packet(path_mtu: u16, frames: u16, header_bytes: u16) -> u8;
pub const fn packets_per_slice(channels: u8, frames: u16, path_mtu: u16, header_bytes: u16) -> u8;
pub const fn media_packet_bytes(channels: u8, frames: u16, path_mtu: u16, header_bytes: u16) -> u16; // UDP payload of packet 0; also the C-116 probe size
pub const fn wire_bits_per_second(layout: &BundleLayout<..>, frames: u16) -> u64; // admission unit, at the domain rate C-01; controller member per C-219
pub const fn packets_per_second(layout: &BundleLayout<..>, frames: u16) -> u32;

pub struct Packetiser { /* per stream: session (C-19), epoch, announcements pending, counters */ }
impl Packetiser {
    pub fn new(stream: StreamId, session: u32, epoch: u16 /* , domain, ... */) -> Self; // session supplied by the source
    pub fn begin_slice(&mut self, position: u64, frames: u16, flags: SliceFlags) -> SliceWriter<'_>;
    pub fn gap_slice(&mut self, position: u64, frames: u16, send_ns: i64, out: &mut [u8]) -> Result<usize, OutTooSmall>;
    pub fn announce(&mut self, kind: Announcement, now_position: u64);      // target = now + 64 slices
    pub fn relay_announcement(&mut self, kind: Announcement, target: u64, now_position: u64); // target unchanged
    pub fn switch_epoch(&mut self, now_position: u64, new: &Line) -> u64;  // issues the announcement, returns T
    pub fn counters(&self) -> SendCounters;                                 // C-47 producer side
    pub fn send_regime(&self) -> SendRegime;                                // packets per slice, pacing mode
}
impl SliceWriter<'_> {
    pub fn content(&mut self, msg: ContentMessage<'_>) -> Result<(), Full>;
    pub fn finish<'b>(self, send_ns: impl FnMut(u8) -> i64, out: &'b mut [u8]) -> Result<Packets<'b>, OutTooSmall>; // one stamp per packet
}
pub struct BundlePacketiser<const M: usize> { /* one Packetiser per member */ } // announce, relay, switch_epoch on every member

pub struct PaceSchedule<const P: usize> { pub at_ns: [i64; P], pub len: u16 }
pub fn pace<const P: usize>(slice_start_ns: i64, slice_ns: i64, packets: u16, spread_permille: u16, module_id: u64) -> Result<PaceSchedule<P>, PaceError>;

pub fn output_position(input_position: u64, structural_delay_samples: u64) -> Option<u64>; // p - L, L whole samples (C-105); None when p < L
```

Plus unit tests, conformance-vector tests, a fuzz target that round-trips arbitrary slices through the packetiser and the reassembler of task 009, and a test-only hook forcing a stream's timeline epoch (C-120).

## Dependencies

- Needs from 004: layouts and the header budget h. From 006: `Line`, the switch procedure. From 007: vectors. From 009: the reassembler for the round-trip fuzz target.
- Provides to 012: `wire_bits_per_second`, `packets_per_second` (controller member costed per C-219, the figure the commissioning tool also uses) and member lists for the handshake. To 014: where the measurement extension is written. To 015: packets, the pacing schedule to send, the media packet size for the path-MTU probe, and `gap_slice` for the missed-deadline gap slice it emits (C-144). To 017: the reference sender. To 024: the output-slice shape for generated code. To 026: the same code on the device. To epic 013, which defines the regime stamp message that epic 015 sends (C-144): `SendRegime` and `SendCounters`.

## Acceptance criteria

1. Conformance vectors for media packets encode byte for byte (system criterion AC-1).
2. `channels_per_packet(1500, 48, h)` returns at least 6 for the header budget h fixed by the wire specification (fewer means the C-04 falsifier has fired), and `channels_per_packet(1280, 48, h)` returns ⌊(1 252 − h) / 192⌋ (C-100), which is at least 6 because h ≤ 100 (C-137); the test shows the arithmetic.
3. For 24, 48 and 96 channels the layout yields 4, 8 and 16 streams and `wire_bits_per_second` yields about 84, 169 and 338 Mbit/s with 8 000, 16 000 and 32 000 packets/s (within 5 %, or the constants table is updated).
4. Every packet of a multi-packet slice carries its own send timestamp and no packet exceeds the path MTU (UDP payload at most path MTU − 28 bytes).
5. No encoded media packet contains any destination-specific byte: encoding once and sending to two consumers is valid.
6. Pacing: all packets of a slice period fall within 25 % of the period after the source's phase offset; the phase offset is a deterministic function of the module id, lies in [0, 75 %) of the period (C-155), and a fixed list of distinct test ids gets distinct offsets; spread 0 yields a single instant; a spread above 250 ‰ or more packets than the schedule capacity is refused without panic.
7. Announcements: target position equals issue position + 64 slices; present in every bundle member; re-issued every 16 slices; absent once the position has passed.
8. System criterion AC-17 (send part): descendants receive the announced position unchanged through an intermediate arrival-driven module.
9. Epoch switch: `switch_epoch` issues the announcement naming T = issue position + 64 slices; every member switches at T to a bumped session (C-19) and the new epoch in the header (contributes to AC-18).
10. Compiles for a `no_std` target without an allocator.
11. `output_position` returns p − L for L in whole samples (C-105) and none when p < L, without panic.
12. Per-stream counters of content slices and gap slices encoded are exposed and match the slices produced (C-47).
13. The fuzz target runs 60 s on every change and 1 hour nightly, and its corpus is kept (C-95).
14. A default 6-channel slice encodes as one packet of at most 1 252 bytes of UDP payload at the 1 280-byte fallback; announcements and the measurement extension that would exceed the cap there are omitted from that slice (C-137).
15. A source with controllers builds at most 31 audio members plus its controller member; a layout that would need a 33rd member for the controllers is refused without panic (C-209).
16. A controller member costs one packet per slice of header bytes plus controller bytes per slice plus 66 bytes of overhead, at 2 000 packets/s, in `wire_bits_per_second` and `packets_per_second` (C-219).

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No allocation and no lock anywhere on the send path
- [ ] Packets per slice and pacing mode are exposed for the regime stamp

**Not applicable in this epic (C-125):** none. Every line has a subject here. This epic writes encoders rather than parsers; the line "Every parser touched is fuzzed" applies through the round-trip fuzz target, which drives the epic 009 reassembler with this epic's output. The POC line applies as a constraint: no ticket touches the POC.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Does the 25 % spread plus phase offset avoid queue drops without breaking consumer margins? | L1 (task 019) | Queue drops with the spread on, or consumers gap because of it |
| How is the phase offset derived from the id so that 64 modules spread evenly, and how many shared slots among 64 ids are acceptable? | This task proposes (ticket 010-10); L1 (task 019) sets the threshold | Two of 64 ids share a phase slot and L1 shows queue drops attributable to it |
| Value of the header budget h within the 100-byte cap (C-137), and hence how much room is left for announcements and the measurement extension at the 1 280-byte fallback | Task 004 | Headers and checksum alone exceed 100 bytes (C-137, C-04 falsifier) |
| Which Ethernet terms C-20 counts (preamble, header, FCS, inter-frame gap; VLAN tag or not). The size of each term is cited from IEEE 802.3 with the clause and tagged `[standard]` (C-132) in ticket 010-05 | Task 004 | The computed per-packet wire size differs from C-20 by more than 5 % |
| Placement of the slice checksum and the measurement extension in a multi-packet slice, and the content of "transport state" in packet 0 | Task 004 | A reassembled multi-packet slice cannot be verified or measured end to end |
| Announcement edge rules: does the slice at the target still carry it; what happens when a re-issue falls on a gap slice; what happens if the slice length changes while an announcement is pending | Task 004 | A descendant misses an announcement despite four re-issues (C-43 falsifier) |
| Reading of the epoch-switch boundary: is T the first position of the new session (boundary), or do both sessions emit a slice at T? | Task 006 (switch procedure) | Consumers see a duplicate or missing position across the switch |
| Which `no_std` target the CI build uses | Task 026 (device) | The device target fails to build code that passes the CI target |
