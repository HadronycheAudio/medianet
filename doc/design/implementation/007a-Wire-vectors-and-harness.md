# 007a — Wire vectors and harness

**Phase:** P1 · **Scope:** L (15 tickets, about 22.5 days) · **Depends on:** 004, 005, 006 · **Blocks:** 007b, 009, 010, 011, 012, 013, 014, 025, 026

## Goal

Produce the first half of the implementation-agnostic conformance data for the protocol: the vector crate with its file formats and `no_std` reader, the runner trait any implementation implements, the generator that derives wire vectors from the specification tables, byte-level vectors for every message in both directions including malformed inputs with their required outcome, and the parser fuzz targets. Any implementation, hosted or embedded, is judged against these. Epic 007 was split under C-143; this part is 007a, and 007b (scenario and reference vectors) builds on it. A reference to "epic 007" means both parts.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules discover each other, agree on one clock holder and exchange *slices* addressed by *position* on a domain-wide timeline. Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers. Two profiles (hosted and embedded) are built from one `no_std` core crate; conformance vectors are how the specification is shown to be complete and how the two profiles are shown to agree.

**Evidence tags.** The design set defines six tags: `[code]` read in the POC · `[measured]` observed · `[kernel-doc]` · `[memory]` · `[standard]` a figure cited from a named, published standard with the clause, which replaces `[memory]` once the citation is checked (C-132) · `[decision]` design choice (definitions in `00-context.md`). This part uses `[decision]`.

**Constants.** Every number below comes from the constants table `doc/design/02-constants.md` and is cited by its C-id; if a value here differs from the table, the table wins. Epic criteria keep the ids of the original epic 007 (007-AC1 to 007-AC8); this part carries 007-AC1, 007-AC2, 007-AC4 and 007-AC8, and 007b carries the others. System criteria keep their system ids (AC-1, AC-23).

**Where things live.** Code lives in the design set's repository: harness and tool binaries (the generator `gen` and the fuzz harness) in the `hadronyche-tools` crate, protocol types in `hadronyche-core` (C-133). `conformance.md` goes to `doc/design/spec/`, results to `doc/design/reports/` (C-130).

### The verification tiers `[decision]`

| Tier | What it proves | Mechanism |
|---|---|---|
| Transport integrity | Bytes delivered equal bytes sent, per slice | Slice checksum |
| **Protocol conformance** | Any implementation speaks the protocol | **Epic 007:** byte-level vectors both directions including malformed inputs (this part); state-machine scenario vectors (007b); fuzzing (parsers here, state machines and CI budget in 007b) |
| Reference modules | Kept permanently | Reference sender, echo and mixer; bit-exact by construction |
| Exact tier | exact-class modules | Record input slices and output hashes; replay the same code offline; compare |
| Tolerance tier | bounded-class modules | Analyzer windows and per-voice projections with stated epsilons |

### Rules the wire vectors must cover `[decision]`

*Media packets.* Packet header (magic, version, domain 16 bits, stream number 16, session 32, timeline epoch 16, copy index 2 bits, flags including gap and degraded; widths C-10, C-11); slice header (64-bit position, frames, packet index and count, send timestamp); optional in-band announcements (end-of-stream, re-alignment, timeline epoch switch, transport reference change; lead 64 slices, C-43, re-issued every 16 slices, C-44); optional measurement extension (hop count, accumulated residence, accumulated declared delay); content messages (audio at 4 bytes per sample f32 or Q1.31 with no 3-byte variant (C-03), dense controllers, sparse keyed controllers, read-only transport state); slice checksum. A bundle source's controllers always ride in their own member stream; a single-stream source's controllers share the audio packet while headers plus controllers stay within the 100-byte cap of C-137, and beyond that the source emits a second stream for them (C-181, C-205). Every stream is at the domain rate of 96 000 Hz, the only rate on the wire; an edge whose hardware runs at another rate converts at the edge (C-01, C-80). Audio carries no per-channel id byte: the channel map is in the manifest, so a channel is 192 bytes per slice at defaults (C-04). The session number is assigned by the source, one per stream, bumped on restart and on timeline epoch change, and identical for every consumer of the stream (C-19).

*Control messages.* Every control message fits one packet at the fallback MTU: at most 1 252 bytes of UDP payload (C-17). The one exception is the path-MTU probe pair, which is sent at the intended media packet size (C-116); PathMtuTooSmall applies only when the probe at the 1 280-byte fallback also fails (C-173). The commissioning tool announces a commissioning window on the control group with an expiry time and may close it explicitly (C-153). The regime stamp travels in a dedicated control message split into sections that each fit C-17; it is multicast to the control group on any regime change and refreshed every 10 s, and a unicast request gives an immediate fetch; the hello carries only a stamp version number (C-18). The stamp message is defined by epic 013 and sent by the hosted backend, epic 015 (C-144). Stamp content encoded by the vectors: per-connection counters published by producer and consumer, received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead, plus the observed DSCP (C-47, C-135, C-159); lateness percentiles 50th, 99th, 99.9th and maximum (C-160); reclaimable latency in the alignment group (C-157), the percentiles and reclaimable latency taken over a rolling 60 s window with the maximum since the connection went live reported beside them (C-182); the DSCP the holder observes on each follower's sync requests and every module observes on each peer's hellos (C-152); the holder's per-follower count of sync requests dropped above 200 per second (C-204); the calibration factor (C-106); the open commissioning window (C-107, C-153); capability states, each present, absent, unverified or not requested (C-129; strict mode rules C-127, C-128; control isolation C-161); and the receive-timestamp source (kernel, userspace, mac-hardware or interrupt) with one of those states (C-131). Receiver reports are unicast, never multicast, and there is no query message (C-47); each receiver report carries the consumer's current A, its own or the one it inherited (C-198). The refusal reasons number 18 in all (C-185): the 16 of C-173 plus ControlDegraded (the target's housekeeping has stopped restarting, C-139) and NoSkewBudget (a fan-in with no downstream group and no configured budget, C-142); VersionMismatch, NameTaken and ModuleCap are raised at discovery, where they stop a module joining, not at admission.

*Required outcomes for bad input.* Never crash, never change state, bounded time; drop and count per input and per reason. The time bound is linear in packet length and at most 10 µs per packet (C-164, a proposal pending L1). A version mismatch is a named event. A foreign-domain packet is dropped and counted (domain 0 means unconfigured, C-11); a hello with domain 0 is ignored and counted, not refused, and ForeignDomain is the refusal for a different non-zero domain (C-173). A packet from a source not pinned at admission is not parsed beyond the filter. Reserved fields must be zero in v1: the copy index (C-07) and, in the authentication reservation of control messages (scheme 8 bits + counter 64 bits + tag 128 bits, C-12), the scheme (0 = none) and the tag. The 64-bit counter is not reserved: it is live in v1 because it is the liveness counter (C-115).

*Dedup.* By stream, position and packet index; the first valid copy wins. Dropped copies are counted in the duplicates counter (C-159).

*Sizes.* IPv4 only in v1, multicast with TTL 1 (C-100). Default slice: 6 channels, 48 frames (C-02, C-04), 1 152 bytes of audio, one packet (C-05). Header cap: a default slice must fit 1 252 bytes of UDP payload, 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum; in-band announcements and the measurement extension fit inside that cap or are omitted on a fallback path (C-137). At most 1 472 bytes of UDP payload on a 1 500-byte path, about 1 320 bytes on the wire including all Ethernet overhead (C-20). Fallback path MTU 1 280 bytes, giving 1 252 bytes of UDP payload (C-06, C-100). A multi-packet slice is an explicit, stamped regime (C-05). At most 32 members per bundle, the controller member included, so a source with controllers has at most 31 audio members (C-08, C-209), 256 streams per source (C-09), 64 modules in the map (C-34).

*Test hooks.* On a real implementation the driver may reach the test-only injection hooks: force a module id and nonce, force a stream onto a timeline epoch, send a raw packet on a media or control socket (C-120). The panic-injection hook (C-145) serves the housekeeping-restart criterion of epic 015 and is not used here.

*Fuzzing.* Each parser is fuzzed 60 s on every change and 1 hour nightly; the corpus is kept (C-95). This part builds the parser targets; the CI job that applies the budget is in 007b.

## Deliverables

1. Crate `hadronyche-vectors` (data plus a tiny `no_std` reader), at the workspace root of the design set's repository (C-192), with:
   - the vector file formats for all six vector kinds (one schema per kind, each with a mandatory specification version and a unique id) and the reader API, both defined by this part and used by 007b;
   - `vectors/wire/valid/*.bin` + `*.json`: for every message type, media and control (including the sectioned regime stamp message, its unicast request, the commissioning window messages and the path-MTU probe pair), at least one minimal, one typical and one maximal valid encoding, with the decoded structure;
   - `vectors/wire/invalid/*.bin` + `*.json`: for every validity rule of the wire specification, at least one violating input with the required rejection reason, plus the bad-input outcome vectors (AC-23) and the dedup sequence vectors;
   - the six data directories, including the empty `vectors/control/`, `vectors/timeline/`, `vectors/transport/` and `vectors/reference/` that 007b fills.
2. A generator (`gen`, hosted only, a tool binary in the `hadronyche-tools` crate of the design set's repository, C-133) that derives the wire vectors from the machine-readable tables of the wire specification, so vectors and specification cannot drift.
3. A runner trait any implementation implements to be tested:

```rust
// shape, names may be refined
pub trait UnderTest {
    fn decode(&mut self, layer: Layer, bytes: &[u8]) -> Result<Decoded, Reject>;
    fn encode(&mut self, msg: &Decoded, out: &mut [u8]) -> usize;
    fn control_step(&mut self, machine: Machine, now_ns: i64, input: ControlInput) -> ControlOutput;
}
```

   The trait as shown cannot check 007-AC8, so it is extended, still `no_std`, with: reading drop counters per input and per reason, reading the named events emitted (alarms included, C-173), reading a state snapshot or digest, and setting the local domain and the sources pinned at admission. The types `Layer`, `Decoded`, `Reject`, `Machine`, `ControlInput` and `ControlOutput` are defined by this part to match the vector schemas; `ControlInput` can carry a reading of a clock that keeps running through suspend (C-136). On a real implementation the driver may reach the test-only injection hooks of C-120.
4. The fuzz harness skeleton in the `hadronyche-tools` crate (C-133) with one target per parser and one for the vector reader, seeded from the wire vectors. The fuzzing tool is the implementing team's choice (C-192); it must run unprivileged.

## Dependencies

- Needs from 004, 005, 006 (C-193): from epic 004 the machine-readable wire tables (message types and layouts, validity rules, rejection reason enumeration, control message list, specification version stamp); from epic 005 the state tables and scenario scripts for discovery, election, admission and connection (the `Machine` set); from epic 006 the timeline and epoch scenarios (used by 007b).
- Provides to 007b: the formats, reader, runner trait, generator codec, wire vectors and parser fuzz targets that the scenario and reference vectors, the CI budget job, the full run and `conformance.md` build on.
- Provides to 009 to 014 (core implementation): the wire tests they must pass.
- Provides to 025 (DSL conformance) and 026 (embedded): the same vectors, run on generated code and on the device.

## Acceptance criteria

1. **007-AC1.** Every message type of the wire specification has valid vectors in both encode and decode direction.
2. **007-AC2.** Every rejection reason has at least one invalid vector; coverage is checked mechanically against the reason enumeration.
3. **007-AC4.** The generator reproduces the committed wire vectors byte for byte from the specification tables.
4. **007-AC8.** System criterion AC-23: malformed, wrong-version and foreign-domain inputs produce no crash and no state change; each is counted per input and per reason; a version mismatch produces a named event. Each input is processed within the C-164 bound recorded in its vector file.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No vector is extracted from the POC before task 002 has cleared the relevant instrument
- [ ] The fuzz budget (time or executions per target per CI run) is stated in `conformance.md`
- [ ] Vector files carry the specification version they were generated from

**Not applicable in this part (C-125).** "Regime stamp fields touched are complete and documented": this part adds and changes no regime stamp field. It encodes the regime stamp message as vectors, with the fields defined by the wire specification (epic 004), and their completeness is checked by the message-type coverage of 007-AC1. "No vector is extracted from the POC before task 002 has cleared the relevant instrument": this part extracts nothing from the POC (that is 007b). "The fuzz budget ... is stated in `conformance.md`": the budget job and the document are in 007b (007b-21, 007b-23). "Unprivileged" applies to per-change CI (C-146); no test of this part needs a privilege.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| What are the control message types and their layouts, the validity rules and the rejection reason enumeration? | Epic 004 | A message type or reason appears in the specification with no vector |
| What does a receiver do with a liveness counter (C-115) that does not advance? | Epic 004 | A replayed control message is accepted or rejected inconsistently across implementations |
| What is the format of the specification version stamp carried by vector files? | Epic 004 | Two vector files from the same specification carry different stamps |
| Does the bad-input bound of C-164 (linear in packet length, at most 10 µs per packet) hold? | L1 measurement | A conforming implementation exceeds the bound on a valid target, or a hang goes undetected under it |

Settled since the split: the control state machines are discovery, election (clock holder), admission and connection, with their state tables from epic 005 (C-193); epic 004 supplies wire layouts and reject reasons, epic 005 state tables and scenario scripts, epic 006 timeline and epoch scenarios (C-193); `hadronyche-vectors` sits at the workspace root and the fuzzing tool is the implementing team's choice (C-192); the refusal reasons are 18 (C-185); controllers travel in their own member stream for a bundle source and inside the 100-byte cap or a second stream for a single-stream source (C-181, C-205); the controller member counts towards the 32 members of a bundle (C-08, C-209); the receiver report carries the consumer's current A (C-198); the stamp's lateness figures use a rolling 60 s window with the maximum since live beside them (C-182); the holder's stamp counts dropped excess sync requests per follower (C-204); vector file names and the generator's output layout within the stated directories are decided here and recorded in the crate documentation (C-206).
