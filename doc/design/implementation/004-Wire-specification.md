# 004 — Wire specification

**Phase:** P1 · **Scope:** L (29 tickets, about 30.5 days) · **Depends on:** 001 (report, non-blocking) · **Blocks:** 005, 006, 007, 009, 010, 011, 012, 013, 014, 008a, 008b, 015a, 018, 022a, 023, 024b, 025a, 027

## Goal

Write the field-level wire specification of the Hadronyche Medianets protocol: media packets, control messages (including the regime stamp and the manifest carried at admission), sync exchanges and measurement messages, with exact offsets, widths, byte order, versioning and the required handling of every malformed case. The design fixes the rules; this task fixes the bytes.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules (sources, processors, sinks) discover each other, agree on one clock holder and exchange *slices*: the content of one stream for one stretch of media time, addressed by *position* on the domain-wide timeline (position = network time counted in samples at the domain rate from a domain epoch). There is one timeline per domain, because the wire carries one sample rate only (C-01); an edge whose hardware runs at another rate converts at the edge (C-80), and a resampler node may appear only inside an edge module, where its conversion delay is part of the edge's plumbing delay D and never counted in L (C-134, C-170, C-207). Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers. Two implementations will speak this protocol from one `no_std` core crate, so the specification must be implementable without allocation and with bounded parsing time.

**Evidence tags.** `[code]` read in the POC · `[measured]` observed · `[kernel-doc]` documented kernel behaviour · `[standard]` cited from a named, published standard with the clause; replaces `[memory]` once the citation is checked (C-132) · `[memory]` recalled, unverified · `[decision]` design choice.

**Rulings that answer former questions `[decision]`.** Each is a row of the constants table and is restated where it applies below.

- **IP version (C-100).** IPv4 only in v1; multicast with TTL 1. A 1 500-byte path leaves 1 472 bytes of UDP payload (20 IPv4 + 8 UDP header bytes); the 1 280-byte fallback (C-06) leaves 1 252 bytes (C-17).
- **Path-MTU probe pair (C-116).** Exempt from the control message bound C-17; the probe is sent at the intended media packet size.
- **Authentication reservation in v1 (C-115).** Scheme 0 and tag zero; the 64-bit counter is live, because it is the liveness counter. "Zeroed in v1" applies to the scheme and the tag only.
- **Regime stamp transport (C-18).** The regime stamp is a dedicated control message split into sections that each fit C-17. It is multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch. The hello carries only a stamp version number.
- **Known regime stamp contents.** The multi-packet regime of a stream (C-05); per-connection counters published by producer and consumer: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159, completing C-47; other-epoch per C-135), plus the observed DSCP (C-47); the target's calibration factor, time per abstract operation, measured by the DSL conformance suite of epic 025 (C-106, C-176); the declared commissioning window while it is open, with its expiry (C-107, C-153); active injections, including the test-only hooks of C-120 and the panic injection of C-145; capability states, each one of four values: present, absent, unverified, not requested (C-129), including the best-effort restriction of CPU idle states under strict mode (C-127), real-time scheduling and memory lock stamped present by construction on a device (C-147), and the control-isolation capability (C-161); the receive-timestamp source, one of kernel, userspace, mac-hardware or interrupt, each reported with a C-129 capability state (C-131); the DSCP observed on each follower's sync requests (holder) and on each peer's hellos (every module) (C-152); in the alignment group, lateness percentiles 50th, 99th, 99.9th and the maximum (C-160) and the reclaimable latency (C-157), all over a rolling 60 s window, with the maximum since the connection went live reported beside them (C-182); in the holder's stamp, per follower, the count of sync requests dropped above 200 per second (C-204).
- **Session number (C-19).** Assigned by the source, one per stream; bumped on restart and on timeline epoch change; every consumer of a stream sees the same value. It is therefore not a per-receiver field.
- **Timeline epoch bumps.** The epoch changes through the in-band epoch switch announcement, and an election in which no candidate has usable time bumps it: the holder-eligible candidate with the lowest module id wins (C-113); and a holder whose monotonic clock diverges from its suspend-aware clock by more than the 2 ms step threshold bumps it (holder stall, C-136, C-67).
- **Who may take over as holder (C-213).** A holder-eligible module whose time is usable: Locked, or in holdover with its alignment claim not lapsed (C-65, C-169). Such a candidate takes over with frequency continuity and the timeline epoch is kept; the epoch is bumped (C-113) only when no candidate has usable time. Election messages must therefore make usable time visible, not lock state alone.
- **Degraded flag (C-122).** Follows the content: an output at position p is degraded if its input at p − L was gapped.
- **Gap classes (C-108, C-102).** At the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span it is reclassified as late (missing −1, late +1). Only missing or late slices count toward the interrupted threshold; a marked gap or a slice on another timeline epoch resets it. A slice on another timeline epoch also restarts the dead clock and has its own per-connection counter (C-135).
- **Observability (C-47).** Receiver reports are not multicast and there is no query message for stream health; third parties read the regime stamps. The receiver report therefore carries the counts the producer needs for its own stamp: the per-connection counters of C-159. The Demoted state (C-109) is visible to third parties through the demoted-rounds counter of C-159 in the regime stamp.
- **Manifest content.** Channel map (C-04: there is no per-channel id byte in media packets); structural latency L as a whole number of domain-rate samples (C-105); the whole-sample delay the compiler inserted at joins of paths of different latency, and a flag on each join whose path has frequency-dependent delay (C-104); the initial value of each dense controller, default 0 (C-158); for an edge module, the resampler's conversion delay, reported separately on the device-facing port and never counted in L (C-207, C-134); cost bound in counted abstract operations per slice (C-106); required E per presentation group, or none; the sink's declared E, or unknown (C-103); only a commissioned sink declares a numeric E, uncertified and self-monitored sinks declare unknown (C-140).
- **Refusal for E (C-103).** A sink with no certified E declares it as unknown and is refused with reason `RequiredEExceeded` from any presentation group that sets a required E; it is admitted to groups that set none. E is defined by C-73. Only the commissioned certification state gives a numeric E (C-140).
- **Pending (C-45).** Pending has no timeout, so no admission message carries one.
- **Admission details (C-173, C-185).** The refusal reasons are the closed set of 18 named in C-185: `PortCeiling`, `PacketRate`, `SliceRate`, `InputCount`, `BufferMemory`, `CostBound`, `ContentMismatch`, `NotAnInput`, `VersionMismatch`, `ForeignDomain`, `PathMtuTooSmall`, `RequiredEExceeded`, `FollowerCap`, `ModuleCap`, `NameTaken`, `BadMemberList`, `ControlDegraded` and `NoSkewBudget`. `ControlDegraded` means the target's housekeeping has stopped restarting (C-139) and is answered by the housekeeping supervisor's minimal refusal path; `NoSkewBudget` is a fan-in with no downstream group and no configured budget (C-142). `VersionMismatch`, `NameTaken` and `ModuleCap` are raised at discovery, where they stop a module joining, not at admission, so no admission refusal carries them. Among the others are `RequiredEExceeded` (C-103), `PathMtuTooSmall`, which applies when the probe at the 1 280-byte fallback also fails, `ForeignDomain`, for a different non-zero domain, and `BadMemberList`, for an unknown or duplicate bundle member or more than 32 members, the controller member included (C-209). A hello with domain 0 is ignored and counted, not refused. A connection alarm is raised on refusal, interrupted, dead, demoted, cannot-meet-A (C-174) and control-degraded (C-139); these are the alarm values the hello's alarm field must express.
- **Fan-in A (C-142).** A fan-in outside any presentation group inherits the A of the downstream presentation group it feeds (the largest if several), learned at admission; the admission handshake therefore carries that A. A later change of A travels upstream in the receiver report, which carries the consumer's current A, its own or the one it inherited; it propagates one hop per report and each fan-in that inherited the old value re-aligns (C-198, C-186).
- **Controllers of a bundle source (C-205).** Always in their own member stream, never in an audio member's packets. That member counts towards the 32 of C-08, so a source with controllers has at most 31 audio members (C-209).
- **Where controllers travel (C-181, revised: permissive).** A single-stream source may always put its controllers in a second stream, and must do so when headers plus controllers exceed the 100-byte cap of C-137; it may share the audio packet while they fit. The compiler checks this from the manifest; the size budget counts shared controllers inside the cap, and a receiver accepts a controller second stream even when its controllers would have fitted.
- **Wire cost of a controller stream (C-219).** Computed like an audio stream: one packet per slice of (headers up to the 100-byte cap + controller bytes per slice + 66 bytes of Ethernet, IP and UDP overhead), at 2 000 packets/s. Admission and the commissioning tool use this same figure, so a controller stream's declared rates follow it.
- **Default tempo (C-199).** 120 beats per minute. Before any transport state has arrived a receiver assumes stopped, playhead at zero, tempo 120 beats per minute (C-184), so the transport state message needs no "unset" value.
- **Transport state on a gap (C-141, C-200).** The receiver holds the last known state and advances the playhead as that state dictates: a stopped transport stays where it is, a playing one moves on at its tempo; the output is marked degraded. Receiver behaviour, no wire value.
- **Re-alignment after a changed A (C-186, C-208).** Each fan-in that learns of the change picks its own position, one lead (C-43) ahead, and announces it in its own output with the in-band re-alignment announcement, exactly as for a ratchet-triggered re-alignment; the announcement payload must serve both causes.
- **Sync requests above the accepted rate (C-204).** Requests beyond 200 per second from one follower (C-112) are dropped before any work is done and counted per follower in the holder's regime stamp; the request carries no field for this.
- **Header cap (C-137). A default slice must fit 1 252 bytes of UDP payload: 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum. In-band announcements and the measurement extension fit inside that cap or are omitted on a fallback path.
- **Commissioning window (C-153).** Announced by the commissioning tool on the control group with an expiry time; modules stamp it while open; it closes at expiry or on explicit close. The announcement and the close are control messages. While a window is open, ordinary production measurements in stamps (latency, loss, alignment figures) are unqualified; the commissioning tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results, each recorded with the window it was taken in (C-178), so the window identifier is what they reference.
- **Bound on processing one bad input (C-164, proposal pending L1).** Linear in packet length and at most 10 µs per packet; every validity rule must allow it.
- **Duties of epic 015 (C-144).** The hosted backend emits the gap slice when a module misses its own deadline and sends the regime stamp message; this epic only lays out their bytes.
- **Identity collisions (C-46).** Third parties quarantine a colliding module id until only one instance nonce has been seen for 3 s (C-31). A colliding hello is a state-machine matter (epic 005), not a parse rejection.
- **Where outputs live (C-130, C-133, C-171).** `wire.md` and the machine-readable tables go in `doc/design/spec/`; any report goes in `doc/design/reports/`; the table checker is tool code and lives in the `hadronyche-tools` crate. Nothing is written into the frozen POC tree. The architecture specification the review uses is `doc/design/01-architecture.md` (C-171).
- **Definition-of-done applicability (C-125). A checklist line with no subject in this epic is marked not applicable with the reason (see after the checklist).

### The four layers

| Layer | Carries | Transport | DSCP (C-13) |
|---|---|---|---|
| Discovery and control | hello, election, admission handshake with the manifest and the path-MTU probe pair, regime stamp sections and the stamp request | UDP over IPv4 (C-100): multicast with TTL 1, and unicast | CS5 (40) |
| Clock sync | request / reply exchanges with the holder | UDP unicast | CS6 (48) |
| Media | slices and in-band announcements | UDP unicast, one packet per slice by default | EF (46) |
| Measurement | receiver reports; one-way probes | UDP unicast, on the reverse port given in the handshake | reports CS5 (40); probes EF (46) |

### Settled sizes and widths `[decision]`

| Id | Constant | Value |
|---|---|---|
| C-01 | Domain sample rate | 96 000 Hz, the only rate on the wire; every stream on a medianet is at the domain rate |
| C-02 | Slice length (default) | 48 frames = 0.5 ms = 2 000 slices/s |
| C-03 | Sample format | 4 bytes per sample, f32 or Q1.31; no 3-byte variant |
| C-04 | Channels per stream at defaults | 6 (192 bytes per channel per slice = 48 × 4; 1 152 bytes of audio; no per-channel id byte, the channel map is in the manifest) |
| C-05 | Default slice | fits one packet at the path MTU; multi-packet slices are an explicit, stamped regime |
| C-06 | Path-MTU fallback | 1 280 bytes |
| C-07 | Copy index | 2 bits in the packet header, reserved for a deferred feature (duplication) |
| C-08 | Members per bundle | at most 32, the controller member included (C-209) |
| C-09 | Streams per source | at most 256 |
| C-10 | Stream number / session / timeline epoch | 16 / 32 / 16 bits |
| C-11 | Module id / instance nonce / domain id | 64 / 64 / 16 bits; domain 0 = unconfigured |
| C-12 | Authentication reservation in control messages | scheme 8 bits (0 = none) + message counter 64 bits + tag 128 bits; in v1 scheme 0 and tag zero, counter live (C-115) |
| C-13 | DSCP classes | sync CS6 (48); media and one-way probes EF (46); control CS5 (40) |
| C-17 | Control message size bound | at most 1 252 bytes of UDP payload (one packet at the fallback MTU); the path-MTU probe pair is exempt (C-116) |
| C-18 | Regime stamp transport | dedicated control message in sections that each fit C-17; multicast on change and every 10 s; unicast request for an immediate fetch; hello carries only the stamp version number |
| C-19 | Session number | one per stream, assigned by the source; bumped on restart and on timeline epoch change; same for every consumer |
| C-20 | Wire size of one default packet (derived) | about 1 320 bytes including all Ethernet overhead |
| C-21 | Wire rate per stream (derived) | about 21.1 Mbit/s (about 3.5 Mbit/s per channel) |
| C-22 | Wire rate per source (derived) | 24 ch: 4 streams, 84 Mbit/s; 48 ch: 8 streams, 169 Mbit/s; 96 ch: 16 streams, 338 Mbit/s |
| C-34 | Active modules per medianet | at most 64 |
| C-40 | Receiver report | one per bundle every 100 ms |
| C-43 | In-band announcement lead | 64 slices, flat |
| C-44 | Announcement re-issue | every 16 slices until the named position passes, in every bundle member |
| C-100 | IP version | IPv4 only in v1; multicast with TTL 1 |
| C-137 | Header cap | a default slice fits 1 252 bytes of UDP payload: 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum; announcements and the measurement extension fit inside the cap or are omitted on a fallback path |

A 1 500-byte path leaves 1 472 bytes of UDP payload. Six channels use 1 152 bytes, leaving about 320 for all headers, announcements, the measurement extension and the checksum; the binding limit is the header cap of C-137: at most 100 bytes for all headers and the checksum, so that the default slice also fits the 1 252-byte fallback payload. The derived wire size of one default packet is about 1 320 bytes including all Ethernet overhead (C-20), about 21.1 Mbit/s per stream (C-21). If the final layout changes the packet size by more than 5 %, the capacity constants derived from it (C-20, C-21, C-22 and the constants computed from them) must be recomputed.

### Rules the media layout must satisfy `[decision]`

1. **Packet header:** magic, version, domain (16), stream number (16), session (32, C-19), timeline epoch (16), copy index (2), flags including *gap* and *degraded*.
2. **Slice header:** position (64-bit, content position on the domain line in samples at the domain rate), frames, packet index and packet count within the slice, send timestamp of this packet.
3. Optional **in-band announcements**: end-of-stream, re-alignment, timeline epoch switch, transport reference change. Each names a future position.
4. Optional **measurement extension**: hop count, accumulated residence time, accumulated declared delay. Production streams may omit it.
5. **Content messages:** audio (no per-channel id byte; channel order follows the manifest's channel map, C-04), dense controllers, sparse keyed controllers (one field of one voice moving linearly to a value over the slice), read-only transport state.
6. **Slice checksum** over the reassembled slice.
7. Streams are identified by **source and stream number**, never by destination. The receiver demultiplexes on the source address pinned at admission plus the stream number. The 64-bit module id is not in media packets.
8. **Forward packets carry no per-receiver state.** Round-trip closure uses the measurement path.
9. **Dedup rule** for the copy index: by stream, position and packet index; the first valid copy wins. No separate sequence number.
10. A **gap slice** is header-only and marks positions the sender did not produce or receive. The **degraded flag** marks a slice computed from a gapped input (C-122).
11. Every packet is sent with don't-fragment set; IP fragmentation is forbidden.
12. A **version mismatch is a named event**, never a silent drop. The version is also carried in the hello so it is detected before a connection exists.
13. Foreign-domain packets are dropped and counted.

### Rules for the control layer `[decision]`

14. **Binary format**, not text. Fixed maximum sizes so a 64-entry peer map and every parser fit a small device.
15. Hello carries: module id, instance nonce, domain, protocol version, name, capability bits (including *holder-eligible*), ports with content types, bundles (group id, member list, channel map), election epoch, believed holder, timeline epoch, transport role, alarm field, data-plane liveness counter, regime stamp version number (C-18), the authentication reservation.
16. The 64-bit message counter of the authentication reservation doubles as the liveness counter; it is live in v1 while scheme and tag are zero (C-115).
17. Admission handshake carries: declared rates (bit/s on the wire, packets/s, slice rate), bundle member list, the path-MTU probe pair (sent at the intended media packet size, exempt from C-17, C-116), the reverse measurement port, pinned IPv4 addresses (C-100), the manifest (ports, content types, channel map, L in whole samples (C-105), inserted whole-sample join delays with the frequency-dependent-delay flag per join (C-104), dense controller initial values (C-158), an edge's conversion delay on the device-facing port (C-207), cost bound, required E per presentation group or none, declared E or unknown (C-140)), the A of the downstream presentation group for a fan-in outside any group (C-142), and a refusal reason when refused, from the 18 named in C-185; the three raised at discovery (`VersionMismatch`, `NameTaken`, `ModuleCap`) are never carried by an admission refusal.
18. The **regime stamp** is a dedicated control message in sections, each at most 1 252 bytes, multicast to the control group on any change and every 10 s, fetched immediately by a unicast request (C-18). It carries at least the contents listed under "Known regime stamp contents" above.

### Rules for sync exchanges `[decision]`

19. Stateless request/reply: the reply echoes the request's own send timestamp, so a stale or duplicate reply is a valid exchange with an honestly large round trip.
20. Every reply carries the **timeline epoch** and the holder's identity.
21. Requests and replies are the same size.

### Rules for the measurement layer `[decision]`

22. The v1 message set is exactly two; receiver reports are unicast to the producer, never multicast, and there is no query message (C-47):

| Message | Sender | Rate | Content |
|---|---|---|---|
| Receiver report | consumer's data thread | one per bundle per 100 ms (C-40) | per-member counters of C-159 (C-47, C-102, C-108, C-135); socket and interface drop counters; observed DSCP; the consumer's current A (C-198) |
| One-way probe request / reply | either end | on demand | timestamps closing a round trip |

### What the POC does, for contrast `[code]`

`proto.rs`: an 88-byte little-endian header (`HDR = 88`, `MAGIC = 0x4844`, `VERSION = 11`) with fields overloaded by direction (`ack_seq` is the forwarding module id in echoes, `ack_t4` a starved-samples count, `path_delay` the sample rate master-to-follower), per-receiver acknowledgements in forward packets (`F_ACK`), an 8-byte message header (`Msg`: kind in the high 4 bits, channels, count, length, signed offset) and 8-byte musical-event parts (`Part`). A version mismatch makes `Header::decode` return `None` silently. Gossip is text (`"HADRO/1"`). All of that is retired; the `Msg` and `Part` shapes are reasonable starting points for content messages.

## Deliverables

1. `doc/design/spec/wire.md` (beside the design documents), containing:
   - byte order and alignment rules;
   - a field table (offset, width, type, meaning, valid range) for: media packet header, slice header, each announcement type, measurement extension, each content message type, checksum; hello; each election message; each admission message, the manifest and the path-MTU probe pair; the regime stamp section layout, each known section type and the stamp request; sync request and reply; receiver report; one-way probe;
   - the size budget table for the default packet showing that 6 channels plus all mandatory headers fit 1 472 bytes, with the remaining bytes stated;
   - the same budget at the 1 280-byte fallback (1 252 bytes of UDP payload over IPv4, C-100), showing that 6 channels fit with at most 100 bytes of headers and checksum (C-137) and which optional parts are omitted there;
   - the validity rules per message and, for each violation, the required outcome (drop and count under a named reason; named event);
   - versioning rules: what a minor and a major change are, and what a receiver does with each; adding a regime stamp section type is a minor change;
   - reserved fields and their required value (copy index, authentication reservation per C-115);
   - the closed enumeration of refusal reasons: the 18 of C-185 (`PortCeiling`, `PacketRate`, `SliceRate`, `InputCount`, `BufferMemory`, `CostBound`, `ContentMismatch`, `NotAnInput`, `VersionMismatch`, `ForeignDomain`, `PathMtuTooSmall`, `RequiredEExceeded`, `FollowerCap`, `ModuleCap`, `NameTaken`, `BadMemberList`, `ControlDegraded` and `NoSkewBudget`), with `VersionMismatch`, `NameTaken` and `ModuleCap` marked as raised at discovery;
   - a traceability table from rules 1 to 22 and the constants above to fields and validity rules.
2. A machine-readable form of the field tables (`doc/design/spec/wire.toml` or similar) from which task 007 generates vectors. Its schema and the checker that proves it parses and round-trips are chosen and documented by this epic (ticket 004-02); the checker lives in the `hadronyche-tools` crate (C-133) and runs in CI without privileges.
3. A list of rejection reasons as a closed enumeration, specified in `wire.md` and the machine-readable file. Its home in code (crate and module) is chosen by the implementing tasks 009 to 014.

```rust
// shape, names may be refined
#[repr(u8)]
pub enum Reject {
    TooShort, BadMagic, VersionMismatch, ForeignDomain, UnknownStream,
    UnadmittedSource, BadLength, BadChecksum, BadPacketIndex, WrongEpoch,
    OversizeControl, ReservedNonZero,
}
```

## Dependencies

- Needs from 001: the report on receive timestamps and don't-fragment per OS (non-blocking; affects only notes on timestamp fields).
- Provides to 005 and 006: message definitions the state machines and the epoch rules refer to. Epic 005 confirms the proposed election and admission message sets, the announcement payloads, and the transport role and alarm encodings.
- Provides to 007: the tables from which conformance vectors are generated, with the bounds and reason lists for fuzzing under the fuzz budget C-95; per C-193 this epic supplies the wire layouts and reject reasons, epics 005 and 006 the state tables and scenarios.
- Provides to 009, 010, 011, 012, 013, 014: the layouts they implement, including the `Reject` enumeration whose code location they choose.
- Provides to 026: the checksum algorithm whose cost on the reference embedded target it measures (falsifier of the checksum sub-question).

## Acceptance criteria

1. Every rule numbered 1 to 22 in Background maps to at least one field or validity rule in `wire.md`; a traceability table shows the mapping.
2. The default-packet budget shows 6 channels fit in 1 472 bytes with mandatory headers, and states the spare bytes; all headers and the checksum take at most 100 bytes, so the default slice fits 1 252 bytes (C-137).
3. Every control message, every regime stamp section and every admission message including the manifest has a stated maximum size and none exceeds 1 252 bytes (C-17), including a hello for a source with a 32-member bundle (C-08). The path-MTU probe pair is the only exception and is sent at the intended media packet size (C-116). If a message cannot fit, the task reports it as a falsified constant.
4. Every message type has an exhaustive list of invalid forms with a rejection reason.
5. Media packets contain no destination-specific and no per-receiver field.
6. The capacity figures (about 1 320 bytes per default wire packet, C-20; C-21; C-22) are recomputed from the final layout, with the IPv4 and Ethernet overhead bytes stated, and the difference stated; the wire cost of a controller stream is stated by the formula of C-219.
7. Contributes to system criteria AC-1 (100 % of conformance vectors pass; zero parser panics or hangs under fuzzing) and AC-23 (malformed, wrong-version and foreign-domain traffic: no crash, no state change, each rejection counted per input and per reason, version mismatch produces a named event).
8. Every value the constants table places on the wire (C-04 channel map, C-18 stamp version, C-47 and C-159 counters, C-103 and C-140 declared and required E, C-104 inserted delay and frequency-dependent-delay flag, C-105 L in whole samples, C-106 cost bound and calibration factor, C-107 and C-153 commissioning window, C-120 and C-145 injection stamp, C-129 capability states, C-131 receive-timestamp source, C-142 downstream A, C-152 observed DSCP, C-198 consumer's current A in the receiver report, C-204 dropped sync requests per follower, C-157 reclaimable latency, C-158 dense controller initial value, C-160 lateness percentiles with the C-182 maximum since live, C-185 refusal reasons (all 18 named) and C-173 alarms, C-207 conversion delay on the device-facing port) has a field with a width and a unit.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] `wire.md` reviewed against the architecture specification section on protocol and packetisation, with the traceability table attached
- [ ] The machine-readable tables parse and round-trip

**Not applicable in this epic (C-125).** "Every parser touched is fuzzed": this epic writes no wire parser; the wire parsers are written and fuzzed by tasks 009 to 014 and 007 under C-95, from the bounds and reason lists this epic delivers. The table checker reads its file with a standard TOML library; if ticket 004-02 writes its own file parser instead, the line applies to that parser and it is fuzzed under C-95.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Does a 16-bit timeline epoch wrap within a plausible uptime? Bump events: the epoch switch announcement, an election in which no candidate has usable time (C-113, C-213; a takeover by a candidate with usable time keeps the epoch) and a holder stall (C-136); the rate is estimated from the election timers C-32, C-33, C-37 | This task; epic 005 confirms the bump rate | An epoch-bump rate that wraps it in under a year of operation |
| Do all headers and the checksum fit the 100 bytes of C-137, and which announcements and measurement-extension bytes fit beside them on a 1 472-byte path? | This task | Headers and checksum exceed 100 bytes (C-137 falsified; C-04 falls to 5 channels at the fallback) |
| Does a hello for a 32-member bundle fit 1 252 bytes, and what are its maxima (name length and encoding, ports, content types per port, capability bits)? | This task | It does not; hello must be split or bundle descriptions moved to the handshake |
| Values of the hello's transport role, and encoding of the alarm field (alarm set fixed by C-173 and C-139) | This task proposes; epic 005 confirms the transport role | Epic 005 needs a role the encoding cannot express |
| Checksum algorithm and width | This task | Chosen algorithm too slow on the reference embedded target (task 026) |
| Send-timestamp width and unit in the slice header | This task | The width wraps within one reassembly or alignment span, or the unit cannot resolve one sample period at 96 000 Hz |
| Magic value, version field width and scheme, v1 version value | This task | A v1 receiver cannot tell a POC packet or a future major version from a v1 packet before reading any other field |
| Election message set and fields, given C-113 and C-213 (usable time, that is Locked or holdover with the alignment claim not lapsed, and holder eligibility must be visible; a candidate with usable time takes over and keeps the epoch; lowest module id wins and bumps the epoch when none has usable time) | This task proposes; epic 005 confirms | Epic 005's election state machine needs a message or field the set lacks |
| Admission message set (the refusal reasons are settled: the 18 named in C-185) | This task proposes the message set; epic 005 confirms | Epic 005's admission state machine needs a message or field the set lacks |
| Payload of the re-alignment and transport reference change announcements (the re-alignment announcement serves both a ratchet step and a changed A, each fan-in announcing its own position one lead ahead, C-186, C-208) | This task proposes the minimum; epic 005 confirms | A receiver cannot act on the announcement from its payload alone |
| Field sets of dense controllers and read-only transport state (the transport state carries a tempo, default 120 beats per minute before any has arrived, C-199, C-184; a bundle source's controllers ride in their own member stream, C-205; a single-stream source may always put its controllers in a second stream and must when they exceed the 100-byte cap, sharing the audio packet only while they fit, C-181; a controller stream's wire cost follows C-219) | This task | A producer cannot express a value it must send without a major change |
| Unit of accumulated residence time, and width of accumulated declared delay (a whole-sample sum, since L is a whole number of samples, C-105) | This task | The declared-delay field saturates over the maximum hop count at the largest L a manifest admits |
| Maximum count of inserted join delays (C-104) in the wire manifest | This task | A compiled module has more padded joins than the bound, or the manifest then exceeds 1 252 bytes |
| Regime stamp section types beyond those the constants table names | Each epic that stamps state, as a minor change; this task defines the container | A stamped item fits no section type without a major change |
| Meaning of "input" in the AC-23 counting key for traffic that matches no admitted input (C-16 and C-109 use "input" for an admitted stream at a consumer) | This task | A rejected packet maps to no counter key, or the counter set exceeds a fixed bound |
| Accepted values of the copy index in v1 (C-07 is reserved for a deferred feature) | This task | The deferred duplication feature needs a value v1 receivers reject, forcing a major version |
| Whether sync and measurement messages carry the C-12 reservation (C-12 names control messages only) | This task | A later authentication scheme cannot be added to them without a major change |
| What the one-way probe measures, given that its reply closes a round trip | This task | A measurement consumer needs a one-way delay the probe's timestamps cannot yield |
