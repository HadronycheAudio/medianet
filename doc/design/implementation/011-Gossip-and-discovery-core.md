# 011 — Gossip and discovery core

**Phase:** P3 · **Scope:** L (26 tickets, about 29 days) · **Depends on:** 004, 005, 007, 008, 013; 002 for one rule · **Blocks:** 012, 015, 017, 026

## Goal

Implement discovery and clock election in `hadronyche-core` as pure `no_std` state machines with no I/O and no allocation: the binary hello, the fixed 63-entry peer map (C-217), identity and collision handling, the election machine, and holder eligibility. Both the hosted and the embedded profile run this one implementation; a platform supplies only a transport and a clock.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated IPv4 LANs whose only traffic is this runtime's. Modules discover each other by multicast gossip, agree on one clock holder and exchange *slices* addressed by *position* on a domain-wide timeline. Every stream on a medianet runs at the single domain rate, 96 kHz (C-01). Linux hosts are the backbone and the only clock holders in v1; macOS and Windows machines are edges; embedded devices are followers. A medianet needs at least one holder-capable node, or there is no audio.

**Evidence tags.** `[code]` read in the POC · `[measured]` observed · `[decision]` design choice.

### Constants

Values are those of the constants table (`02-constants.md`), which wins over this file.

| Id | Constant | Value | Status |
|---|---|---|---|
| C-30 | Hello period | 1 s | inherited from the POC |
| C-31 | Hello expiry | 3 s, for non-live peers only | inherited |
| C-32 | Listen-before-claim | 1.5 s | inherited |
| C-33 | Election window | 400 ms | inherited |
| C-34 | Active modules per medianet | at most 64; commissioning refuses more | decided |
| C-35 | Followers per holder | at most 64; excess refused with a named event | decided; L1 (task 019) confirms |
| C-36 | Sync requests per follower | 50 per second | inherited |
| C-37 | Election on reply silence | 1.5 s | decided |
| C-42 | Stalled data plane visible to peers | within 2 hello periods | decided |
| C-01 | Domain sample rate | 96 000 Hz, the only rate on the wire | decided |
| C-10 | Timeline epoch width | 16 bits | decided |
| C-11 | Module id / instance nonce / domain id | 64 / 64 / 16 bits; domain 0 = unconfigured | decided |
| C-12 | Authentication reservation | scheme 8 bits (0 = none) + message counter 64 bits + tag 128 bits | reserved |
| C-115 | Authentication reservation in v1 | scheme 0 and tag zero; the 64-bit counter is live because it is the liveness counter | decided |
| C-17 | Control message size bound | at most 1 252 bytes of UDP payload | decided |
| C-18 | Regime stamp transport | a dedicated, sectioned control message, multicast on change and every 10 s, fetchable by unicast request; the hello carries only a stamp version number | decided |
| C-46 | Quarantine of a colliding id by third parties | until only one instance nonce has been seen for one hello-expiry period (3 s) | decided |
| C-100 | IP version | IPv4 only in v1; multicast with TTL 1; C-06's 1 280-byte fallback gives the 1 252 bytes of C-17 | decided |
| C-112 | Accepted sync request rate | 10 to 200 per second | decided |
| C-113 | Election when no candidate has usable time | the holder-eligible candidate with the lowest module id wins and bumps the timeline epoch; "usable time" is defined in C-213 | decided |
| C-114 | Follower-cap test | at most 63 real followers; the 64-follower load and the refusal of a 65th are tested with synthetic followers | decided |
| C-120 | Test-only injection hooks | includes forcing a module id and nonce; stamped like any injection | decided |
| C-95 | Fuzz budget | each parser 60 s on every change and 1 hour nightly; the corpus is kept | decided |
| C-13 | DSCP for control | CS5 (40) | decided |
| C-125 | Definition-of-done applicability | a line with no subject is marked not applicable with the reason | decided |
| C-126 | Epic scope labels | recomputed from the ticket tally | decided |
| C-08 | Members per bundle | at most 32, the controller member included (C-209; a bundle source's controllers ride in their own member stream, C-205) | decided |
| C-133 | Where code lives | all protocol types, control messages included, in `hadronyche-core` | decided |
| C-144 | Owner of the regime stamp message | defined by epic 013; sent by the hosted backend (epic 015) | decided |
| C-147 | Strict mode | hosted-only; any run whose numbers enter the constants table runs in strict mode | decided |
| C-152 | Peer verification of control DSCP | every module reports in its stamp the DSCP observed on each peer's hellos | decided |
| C-162 | Clock state after the holdover claim lapses | Unlocked, with time unusable | decided |
| C-164 | Bound on processing one bad input | linear in packet length and at most 10 µs per packet | proposal, pending L1 |
| C-165 | Operator policy in the commissioning configuration | sets strict mode (among others) | decided |
| C-167 | Holder serving lateness (L1) | reply send time minus request receive time at the 99.9th percentile; "no growth" means within 20 % from 1 to 64 followers | proposal, pending first runs |
| C-173 | Admission details (hello part) | a hello with domain 0 is ignored and counted, not refused; ForeignDomain is for a different non-zero domain | decided |
| C-185 | Refusal reasons raised at discovery | VersionMismatch, NameTaken and ModuleCap are raised at discovery, where they stop a module joining, not at admission | decided |
| C-187 | Publisher of the clock lock state | the servo core (epic 008a), through the published time view | decided |
| C-188 | Where the holder answers sync requests | on the clock thread, never on housekeeping; it also records there the DSCP observed on each follower's requests | decided |
| C-213 | Who may take over as holder | a holder-eligible module whose time is usable: Locked, or in holdover with its alignment claim not lapsed (C-65, C-169); it takes over with frequency continuity and the timeline epoch is kept; the epoch is bumped (C-113) only when no candidate has usable time | decided |
| C-221 | Several candidates with usable time (C-213) | a Locked candidate is preferred over one in holdover; among equals the lowest module id wins | decided |
| C-217 | Peer map capacity | 63 peers: a medianet has at most 64 modules (C-34) and a module does not hold an entry for itself | decided |
| C-204 | Sync requests above the accepted rate | requests beyond 200 per second from one follower are dropped before any work is done and counted per follower in the holder's stamp | decided |

### Invariants this task must respect `[decision]`

- **L-3:** gossip is discovery, admission bootstrap and election input only. A follower that receives no sync replies for 1.5 s opens an election regardless of hellos.
- **L-6:** the holder is judged by sync replies only. Hello expiry never opens an election while replies flow.
- **L-1:** gossip silence alone never tears down a live stream; a peer with a live connection is not removed from the map by hello expiry.
- **L-5:** a hello with a new address never redirects a live stream; destinations are pinned at admission.
- **L-8:** no lock is shared between housekeeping and the data or clock threads. The state machines here are owned by housekeeping; everything they export to other threads goes through messages or a lock-free publication.

### Hello `[decision]`

Binary, bounded size (at most 1 252 bytes, C-17). Carries: module id, instance nonce, domain, protocol version, name, capability bits (including **holder-eligible**), ports with content types, bundles (group id, member list, channel map), election epoch, believed holder, timeline epoch (16 bits, C-10), transport role, an alarm field, the data-plane liveness counter, and the regime stamp version number. The hello carries no per-stream sample rate: every stream is at the domain rate (C-01). The authentication reservation is present with scheme 0 and a zero tag; its 64-bit message counter is live and is the liveness counter (C-115). The regime stamp itself is not in the hello: it travels in its own sectioned control message, defined by epic 013 and sent by the hosted backend, epic 015 (C-144), and a peer whose advertised stamp version changes is fetched by unicast request (C-18). Sent at join (the end of listen-before-claim, see below) and every 1 s on an IPv4 multicast group with TTL 1 (C-100). The transport (socket, multicast TTL, DSCP CS5 per C-13) belongs to the platform (epics 015, 026); core only fills and reads buffers.

- A foreign-domain hello (a different non-zero domain) is dropped and counted.
- A hello carrying domain 0 is ignored and counted under its own reason, whatever the receiver's domain; it is not a foreign-domain refusal (C-173).
- A malformed hello is dropped and counted. A bundle with more than 32 members (C-08), the controller member included (C-209), is an out-of-range count and therefore malformed; refusing a connection for a bad member list is admission's BadMemberList (C-173, epic 012).
- Parsing a bad hello costs time linear in its length (C-164); the 10 µs per packet bound is confirmed at L1.
- For every peer, discovery records the DSCP observed on its hellos, which the platform reads from the received packet and passes to `on_hello`; the value is published in the module's regime stamp (C-152).
- A version mismatch is counted and is a named event (the version is in the hello precisely so it is detected before any connection). VersionMismatch is raised here, at discovery, not at admission (C-185): a peer of another version is never inserted in the map, so it never joins.
- Counts are kept per reason and exposed read-only by `Discovery::reject_counters()`.
- Domain 0 means unconfigured; strict mode refuses to start with it. Core provides the check `check_start(domain, strict)`, which the platform calls before `Discovery::new`. Strict mode is set by the operator policy in the commissioning configuration (C-165) and is hosted-only (C-147): the hosted platform (epic 015) reads it, while a device (epic 026) has no strict mode and passes `strict = false`. What else strict mode requires (real-time scheduling, memory lock, C-127, C-128) is checked by epic 015.

### Identity and collisions `[decision]`

- Id: 64 bits from the OS random source, plus a 64-bit instance nonce.
- Devices without a true random source mix a hardware unique identifier with a persistent boot counter; otherwise forty identical units would boot with the same id and nonce. The `IdSource` implementations (OS random; hardware id plus boot counter) belong to the platform epics 015 and 026; core defines only the trait and a deterministic test source.
- **Collision:** the same id seen with a different nonce. The pair detects it within one hello period; the **later joiner redraws** its id and rejoins (if both joined at once, both redraw); **third parties quarantine** the id (ignore both) until only one instance nonce has been seen for the id for one hello-expiry period, 3 s (C-46). It is logged on both modules: in this I/O-free core, "logged" means `MapEvent::Collision` is raised on both modules and the platform writes the log line. A collision is never silent.
- Tests force a collision through the test-only hook that sets a module id and nonce (C-120); its use is stamped like any injection.
- A newcomer whose **name** is already present refuses to join, using the 1.5 s listen-before-claim: from start it listens for 1.5 s, sends no hello, and either raises `NameTaken` and stays silent, or joins by sending its first hello at t = 1.5 s. NameTaken is a discovery refusal, not an admission one (C-185).
- **Module cap at discovery (C-185, C-34):** a newcomer that has heard 64 distinct active module ids by the end of listen-before-claim would be the 65th; it raises `ModuleCap` and stays silent exactly as for `NameTaken`. The map holds 63 peers, never an entry for the module itself (C-217), so the newcomer sees this as a full map plus a further distinct id refused with `MapFull` during listening. An established module whose map is already full (63 peers, so 64 modules counting itself) does not insert a further id and raises `MapFull`. ModuleCap is raised here, not at admission (epic 012).

`[code]` Retired POC behaviour: id = XOR of wall time, pid and monotonic time, 31 effective bits (`module.rs:1113`); a hello carrying one's own id is taken as one's own echo and silently swallowed (`module.rs:404-409`), so a collision merges two modules; duplicate names only warned (`module.rs:537-539`); text messages prefixed `"HADRO/1"` (`module.rs:47`) parsed with string splitting.

### Election `[decision]`

- **Candidates** must set the holder-eligible bit, have **usable time** (C-213): Locked, or in holdover with its alignment claim not lapsed (C-65, C-169) (the clock state is published by the servo core, epic 008a, through the published time view, C-187; a module whose holdover claim has lapsed is Unlocked with time unusable, C-162, and is not a candidate), be on a host not under I/O pressure, and be able to serve up to 64 followers. Embedded, macOS and Windows nodes do not set the bit in v1. The reasons devices are excluded are serving load, availability (devices get power-cycled) and oscillator quality, not timestamp quality.
- **Trigger:** 1.5 s without sync replies at a follower; the holder coming under pressure; an operator force. **Never** hello expiry.
- **No rotation.** The holder changes only on loss or pressure.
- Window 400 ms. Every module computes the same winner from the same candidate set, deterministically: among candidates with usable time a Locked candidate is preferred over one in holdover, and among equals the lowest module id wins (C-221).
- **Merge:** higher election epoch wins; rank is a stale snapshot and is not compared across epochs.
- **Handover** by a candidate with usable time, Locked or in holdover, is frequency-continuous (the servo's takeover entry point from task 008) and keeps the timeline epoch (C-213). If no candidate has usable time, the holder-eligible candidate with the lowest module id wins and bumps the timeline epoch, an announced discontinuity (C-113). A module without the holder-eligible bit is never elected.
- A holder serving 64 followers refuses the 65th with a named event. Since a medianet has at most 64 modules (C-34), a holder has at most 63 real followers; the 64-follower load and the 65th refusal are tested with synthetic followers (C-114). The holder accepts sync requests at 10 to 200 per second per follower (C-112); serving them is the clock thread's work (epic 008), not this core's: the holder answers sync requests on the clock thread, never on housekeeping (C-188), and requests beyond 200 per second from one follower are dropped there before any work is done and counted per follower in the holder's stamp (C-204). Whether serving lateness stays flat up to 64 followers is measured at L1 by task 019 (C-35, C-167).
- Partition heal: one holder within election window plus hello period, 1.4 s.
- **Events.** An input can produce several events (several expiries in one tick; `HolderChanged` together with `TimelineEpochBump`). `on_hello`, `on_tick` and `step` return the first event; the others wait in a fixed-capacity queue that the caller drains with `next_event()` until it returns `None`.

`[code]` POC behaviour to keep: epoch adoption on hello (`module.rs:547-557`), eligibility by I/O pressure read from the kernel's pressure-stall information (`module.rs:193-213`). POC behaviour to retire: the deterministic draw by hashing id with epoch (`module.rs:641`), replaced by the ordering of C-221; rotation after 2 mandates (`MANDATES = 2`, `module.rs:62`); `leave()` opening an election when the holder's hello expires (`module.rs:575-585`); election re-requests every 10 s (`REQUEST_EVERY_NS`). The I/O-pressure rule rests on experiment data whose integrity task 002 checks.

## Deliverables

All in `hadronyche-core`, `no_std`, no allocation, capacities fixed.

```rust
// shape, names may be refined
pub struct ModuleId(pub u64);
pub struct Nonce(pub u64);
pub struct Domain(pub u16);
pub struct Addr { pub ip: [u8; 4], pub port: u16 }               // IPv4 only (C-100)
pub struct Outbox { /* fixed-capacity buffer of outgoing datagrams, drained by the platform */ }

pub struct LocalInfo { /* every hello field, fixed-capacity arrays */ }
pub enum Reject { ForeignDomain, UnconfiguredDomain, Malformed, VersionMismatch { peer: ModuleId, theirs: u8 } }   // C-173
pub struct RejectCounters { pub foreign_domain: u64, pub unconfigured_domain: u64, pub malformed: u64, pub version_mismatch: u64 }

pub struct Hello<'a> { /* borrowed view over a received buffer */ }
impl<'a> Hello<'a> {
    pub fn parse(bytes: &'a [u8], my_domain: Domain) -> Result<Self, Reject>;
    pub fn encode(info: &LocalInfo, out: &mut [u8]) -> usize;   // <= 1252 bytes
}

pub enum StartError { UnconfiguredDomain }
pub fn check_start(domain: Domain, strict: bool) -> Result<(), StartError>;

pub const MAX_PEERS: usize = 63;                                 // C-217: 64 modules, no entry for self
pub struct Peer { /* owned copy of the hello fields + address + bookkeeping */ }
pub struct PeerMap { /* [Option<Peer>; MAX_PEERS] */ }

pub enum MapEvent {
    Joined(ModuleId), Left(ModuleId), Collision(ModuleId), Quarantined(ModuleId),
    NameTaken, ModuleCap, VersionMismatch { peer: ModuleId, theirs: u8 }, ForeignDomain,   // NameTaken, ModuleCap, VersionMismatch: C-185
    MapFull, Stalled(ModuleId), StampVersionChanged { peer: ModuleId, version: u32 },
}

pub struct Discovery { /* map + own identity + timers + event queue */ }
impl Discovery {
    pub fn new(me: LocalInfo, rng: &mut dyn IdSource) -> Self;
    pub fn on_hello(&mut self, now_ns: i64, from: Addr, dscp: u8, bytes: &[u8], out: &mut Outbox) -> Option<MapEvent>;  // dscp as received (C-152)
    pub fn on_tick(&mut self, now_ns: i64, out: &mut Outbox) -> Option<MapEvent>;   // listen 1.5 s, then hello every 1 s; expiry at 3 s (non-live only)
    pub fn next_event(&mut self) -> Option<MapEvent>;                                // drains further events of the last input
    pub fn mark_live(&mut self, peer: ModuleId, live: bool);                         // from the connection machine
    pub fn redraw_id(&mut self, rng: &mut dyn IdSource, out: &mut Outbox);
    pub fn reject_counters(&self) -> RejectCounters;
    pub fn observed_dscp(&self, peer: ModuleId) -> Option<u8>;                         // for the regime stamp (C-152)
}

pub struct Election { /* epoch, candidates, deadline, event queue */ }
pub enum ElectionInput<'a> {
    ReplySilence,                       // 1.5 s without sync replies, from the clock thread
    Pressure, Forced(ModuleId),
    Message(&'a [u8]), Tick,
}
pub enum ElectionEvent {
    Opened { epoch: u32 }, HolderChanged { holder: ModuleId, epoch: u32, continuous: bool },
    TimelineEpochBump, FollowerRefused(ModuleId),
}
impl Election {
    pub fn step(&mut self, now_ns: i64, input: ElectionInput<'_>, map: &PeerMap,
                me: &Candidacy, out: &mut Outbox) -> Option<ElectionEvent>;
    pub fn next_event(&mut self) -> Option<ElectionEvent>;
}
pub struct Candidacy { pub eligible_bit: bool, pub time_usable: bool, pub under_pressure: bool, pub followers: u8 }   // time_usable: Locked or holdover claim standing (C-213)

pub trait IdSource { fn draw(&mut self) -> (ModuleId, Nonce); }   // platform: OS random, or hardware id + boot counter
```

Plus: scenario-vector tests from task 007 (read from the vectors crate at the workspace root, C-192), a fuzz target for `Hello::parse` and for election messages (run per C-95; the fuzzing tool is the implementing team's choice, C-192), a `const` that reports the map's memory footprint, a test-only forced-identity `IdSource` (C-120), the holder-eligible bit added to the regime stamp, and the per-peer observed hello DSCP exported for the stamp (C-152). The width of the stamp version number in `StampVersionChanged` follows the epic 004 layout; `u32` above is a placeholder.

## Dependencies

- Needs from 004: the hello and election message layouts and the width of the regime stamp version number. From 013: the regime stamp message (C-144), including the section that holds the holder-eligible bit and the field for the per-peer observed hello DSCP (C-152). From 005: the normative transition tables and the discovery and election scenario scripts; from 004 also the reject reasons (C-193). From 007: vectors and their format. From 008a: the servo's clock state (Locked, holdover with the alignment claim standing, or Unlocked), through the published time view (C-187, C-213). From 008: the frequency-continuous takeover. From 002: whether the I/O-pressure eligibility rule is inherited.
- Provides to 012: the peer map (addresses, ports, bundles, capability bits) that the admission handshake resolves against. To 015: the machines the hosted housekeeping thread drives, and the start check. To 017: discovery for the reference modules. To 026: the same code on the device.
- Platform epics 015 and 026 supply the `IdSource` implementations, the transport (IPv4 multicast, TTL 1, DSCP CS5), the DSCP of each received hello, and the pressure-stall reading; epic 015 supplies the strict-mode flag from the commissioning configuration (C-165, C-147) and sends the regime stamp message (C-144).

## Acceptance criteria

1. All discovery and election scenario vectors pass (system criterion AC-1).
2. AC-22: a forced id collision (forced through the C-120 hook) is detected within one hello period (1 s); the later joiner redraws; third parties quarantine the id until only one nonce has been seen for 3 s (C-46); `Collision` is raised on both modules; no merged peer.
3. AC-20 (b): with the holder's hellos suppressed while sync replies flow, no election is opened. AC-20 (a): with all hellos suppressed for 30 s, no peer with a live connection is removed from the map.
4. AC-36: after a healed partition, one holder within 400 ms + 1 s = 1.4 s; every module computes the same winner.
5. AC-25 (core part): with 64 synthetic followers, the 65th is refused with a named event (C-114).
6. No rotation: in a 24-hour simulated run without loss or pressure, the holder never changes.
7. Only candidates with the eligible bit set and usable time (Locked, or in holdover with the alignment claim not lapsed, C-213) are elected while one exists, a Locked candidate before one in holdover and the lowest module id among equals (C-221), and such a takeover is frequency-continuous with the timeline epoch kept; with no candidate with usable time, the holder-eligible candidate with the lowest module id wins and the result carries a timeline epoch bump (C-113); a module without the eligible bit is never elected.
8. A hello for a source with a 32-member bundle (the controller member included, C-209) encodes in at most 1 252 bytes, or the task reports the constant as falsified.
9. A 64th peer (a 65th module counting the receiver) is not inserted; `MapFull` is raised (C-217).
10. AC-23: foreign-domain, domain-0, malformed and wrong-version hellos cause no state change and are counted per reason in `reject_counters()`; a version mismatch produces a named event; a domain-0 hello is ignored, not refused (C-173).
11. Compiles for a `no_std` target without an allocator.
12. Each parser (hello, election messages) is fuzzed 60 s on every change and 1 hour nightly, with the corpus kept (C-95).
13. Every hello carries the sender's regime stamp version; a change of a peer's version raises `StampVersionChanged` once (C-18).
14. In every encoded hello the authentication scheme is 0, the tag is zero and the counter equals the liveness counter (C-115).
15. A newcomer whose name is present raises `NameTaken` within 1.5 s and sends no hello; otherwise its first hello is at 1.5 s.
16. For every peer in the map, `observed_dscp()` returns the DSCP of its last hello as passed in by the platform (C-152).
17. A newcomer that hears 64 distinct active modules during listen-before-claim raises `ModuleCap` and sends no hello (C-185, C-34).

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No election path is reachable from hello expiry (shown by test and by inspection of the transition table)
- [ ] The holder-eligible capability bit is in the regime stamp

Not applicable in this epic (C-125): none. Every line has a subject here: the epic writes code and tests, two parsers (hello, election messages), stamp content (holder-eligible bit, per-peer observed hello DSCP) plus the stamp version in the hello, constants and transition-table annotations, and it cites POC code it must not modify.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Is the I/O-pressure eligibility rule inherited? | Task 002 | The POC experiment data behind it is corrupt or multi-writer |
| How a module decides it is the "later joiner" of a collision, and what "joined at once" means (the hello carries no join time) | Task 005 | A collision vector in which both or neither module redraws when the rule says otherwise, or a merged peer |
| Exact stall rule for C-42: what "no advance of the liveness counter" is measured over, and whether a module with no data plane is exempt | Task 005 | A module with no streams is reported stalled, or a stalled data plane is not reported within 2 hello periods |
| Equal election epochs with different holders after a partition heal: how it is resolved within 1.4 s | Task 005 | A heal vector in which two holders persist at the same epoch past 1.4 s |
| How the holder learns of a follower attach (the input that leads to `FollowerRefused`) | Task 005, with 008 for the clock-thread side, where sync requests are answered (C-188) | A follower beyond the cap is served without `FollowerRefused` |
| Does a 63-entry map (C-217) with full hellos fit the reference device's memory? | Task 026 | It does not; C-34 or the hello content must shrink |
| Can a holder really serve 64 followers at 50 requests/s each (C-36), and at up to 200 requests/s (C-112)? | L1 (task 019), per C-35 | Holder serving lateness (99.9th percentile of reply send minus request receive) grows by more than 20 % from 1 to 64 followers (C-167) |
| Does processing one bad hello or election message stay within 10 µs per packet (C-164)? | L1 measurement | A measured per-packet cost above 10 µs, or a cost that grows faster than linearly with packet length |
