# 013 — Regime stamp wire object

**Phase:** P3 · **Scope:** L (27 tickets, about 28.5 days) · **Depends on:** 001, 004, 005, 007 · **Blocks:** 014, 015, 017, 019, 020, 026, 028, 029, 011, 016, 018, 023, 024b, 025b

## Goal

Define and implement the **regime stamp** in `hadronyche-core`: a compact, versioned wire object that states under which conditions every measurement was taken, with capabilities in four states (C-129), plus the rules that bind records to stamps. Every node publishes its stamp over the network (C-18); a node that cannot write files (an embedded device) relies on that publication and a hosted collector records it.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated IPv4 LANs (C-100) whose only traffic is this runtime's. Modules exchange *slices* addressed by *position* on a domain-wide timeline. Every stream on the wire is at the single domain rate of 96 kHz (C-01); an edge whose hardware runs at 48 kHz converts at the edge (C-80). Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers. The runtime reports latency, loss and alignment figures; the stamp is what makes those figures citable.

**Evidence tags.** `[code]` read in the POC · `[measured]` observed · `[kernel-doc]` documented · `[standard]` cited from a named, published standard with the clause (replaces `[memory]` once checked, C-132) · `[memory]` recalled, unverified · `[decision]` design choice.

### Why this exists `[measured]`

The proof of concept's first measured run (25 s loopback, unprivileged, kernel 6.12) produced: a corrupt metrics file (about 6 parseable lines of about 80, because four modules each opened the same path); a garbage "lost" counter of about 21.7 million on a run with zero loss; two check families reporting "0 of 0 windows" on a run that looked green; and an achieved sync rate of 37/s where 50 was configured (read from that corrupt file; superseded by the clean 36.1/s, P-02, REPORT-0b section (e)). A run where real-time scheduling silently failed looked identical to a good one. Numbers governance is therefore first-class surface, not a logging detail.

### Stamp contents `[decision]`

The stamp has eight **groups** (the unit of completeness; the Capabilities group was called "Capability bits" before C-129 gave it four states), plus repeated per-connection entries, per-connection summary counters and a self-test result.

| Group | Fields |
|---|---|
| Identity | protocol version, build identity, full configuration, domain, profile (linux-std, macos-std, windows-std, embedded) |
| Capabilities (present / absent / unverified / not requested, C-129) | receive timestamp source (kernel, userspace, mac-hardware or interrupt, each with its state, C-131), real-time scheduling granted (on the data and clock threads, C-127), memory locked, CPU idle states restricted (best-effort, C-127), drop accounting, DSCP marking, fault injection, holder-eligible, control isolation (housekeeping cannot delay the data path, C-161) |
| Network | interface (IPv4 only, C-100), interface MTU, path MTU per connection, packets per slice, pacing mode, DSCP classes and peer-observed values, interrupt-coalescing state; per-peer observed DSCP: on each follower's sync requests (holder only) and on each peer's hellos (C-152); per follower, the sync requests dropped above 200 per second (holder only, C-204) |
| Clock | discipline mode, lock state and bound at each one-way figure, timeline epoch |
| Alignment | offset, the lateness percentiles (50th, 99th, 99.9th and maximum, C-160) and reclaimable latency (C-157), both over a rolling 60 s window, with the maximum since the connection went live beside them (C-182), audition offset A (the current A, own or inherited, C-198; a fan-in with no downstream group stamps its configured budget, C-142, or none, its aligned inputs then refused with NoSkewBudget, C-185), required E (or none), sink certification state (commissioned with a numeric E, failed, or E unknown for an uncertified or self-monitored sink; C-103, C-140; a commissioned sink stays commissioned while its clock is unlocked, C-180) |
| Platform | OS build, audio device, driver, compiler cost calibration factor (time per abstract operation, C-106, measured by epic 025, C-176) |
| Drops | per-socket overflow, interface drop counters, measurement-buffer overflow |
| Injection | any active injection, always stamped, including the test-only hooks of C-120 and C-145; the commissioning window state and expiry while open (C-107, C-153) |

| Other content | Fields |
|---|---|
| Per-connection entry (part of Network) | connection identity, path MTU, packets per slice, pacing mode |
| Per-connection summary (C-47, C-159) | per connection, published by producer and consumer each in its own stamp: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead, observed DSCP |
| Self-test | the five items of Rule 9, the verdict and the failed items |

**Regime fields and counters.** Fields fall in two kinds. *Regime fields* describe the conditions of measurement (everything in Identity, Capabilities, Network, Clock, Alignment, Platform, Injection, the per-connection entries and the self-test, except the counters listed next). *Counters* are reported values that move during a run: the Drops group, the per-connection summaries, the per-peer observed DSCP entries of C-152, the per-follower dropped sync request counts of C-204, and the lateness percentiles, the since-live maximum and reclaimable latency of the Alignment group (C-157, C-160, C-182), which are written in a counter section of their own. The `StampId` is computed over regime fields only; a counter change does not change it and does not start a record set. This follows from C-18 and C-47: counters travel in the stamp and are refreshed every 10 s, which is not a regime change.

### Rules `[decision]`

1. A **change of regime mid-run starts a new record set**; figures are never averaged across regimes. A timeline epoch change is a regime change. A counter change is not (see Regime fields and counters).
2. A measurement involving two endpoints carries **both profiles**.
3. One-way figures are blank until the clock is locked and carry the bound thereafter. The stamp carries the lock state and bound; the producers of one-way figures (epics 015, 017) do the blanking.
4. **A check family with zero samples is a failed run.**
5. **A file has one writer, owned by the process**, never by a module.
6. In the multi-packet regime loss is reported at both levels: packet loss (the link) and slice loss (the application). The stamp carries packets per slice per connection and the slice-level summary of C-47; the producers of loss figures (epics 015, 017) report both levels.
7. Any active **injection** (clock offset or drift; packet loss, delay, reordering; per-hop delay; and the test-only hooks of C-120: forced module id and nonce, forced timeline epoch on a stream, raw packet sent on a media or control socket; and the panic injected into a data, clock or housekeeping thread of C-145) is written into the stamp, so an injected run can never pass as a real one. Injection may be absent on embedded, which is itself stamped.
8. **DSCP marking is verified by the receiving peer**, which reads the received byte and reports it; the stamp records the classes requested and the values the peer observed. No platform is trusted to self-report. Requested classes: sync CS6 (48), media and one-way probes EF (46), control CS5 (40) (constant C-13). Each consumer also publishes the DSCP it observed per connection in its own stamp's summary (C-47). For sync and control (C-152): the holder reports in its stamp the DSCP observed on each follower's sync requests, and every module reports the DSCP observed on each peer's hellos.
9. A **self-test** runs before any number is reported: receive timestamps present and consistent with the monotonic clock; real-time scheduling actually granted; measured wake latency; packets per slice equal to the stamped regime's value (one at defaults, C-05); clock locked within its lock bound (C-62, pending simulation). Numbers produced before it passes are labelled **unqualified**, and so are ordinary production measurements (latency, loss, alignment figures) taken while a commissioning window is open (C-153, C-178). The commissioning tool's own measurements in a window (D, restart spread, group certification, soak) are valid commissioning results, recorded by epic 028 with the window they were taken in (C-178).
10. **Strict mode** turns a failed self-test item, domain 0 (unconfigured), or a required capability not present into a refusal to start, naming the item. ("Failed check" at start-up means a failed self-test item; no other check has run yet.) Strict mode requires real-time scheduling granted on the data and clock threads and memory locked (C-127); unverified is treated as absent (C-128), and not requested is refused as well, since the requirement is that both are granted. Restricting CPU idle states stays best-effort: it is stamped and never refuses (C-127). Strict mode is hosted-only; a device stamps real-time scheduling and memory lock as present by construction (C-147). It is set by the operator policy in the commissioning configuration (C-165), and any run whose numbers enter the constants table runs in strict mode (C-147). The refusal when seqlock safety depends on a CPU pin that failed or is unverified (C-177, C-201) applies in every mode; it is not a strict-mode check and not this epic's.
11. Measurement-buffer overflow is counted and stamped, because the samples lost under load are the worst ones and silent loss flatters the tail.
12. Certification failures of a sink (it cannot bound its emission error E) are in the stamp, in the Alignment group. A sink with no certified E declares E as unknown (C-103).
13. Missing privileges are the capabilities `rt_scheduling` and `memory_locked` when Absent or Unverified. Not requested is a deliberate configuration choice, not a denial (C-129); it is labelled as such, not as a missing privilege. The outputs that carry labels in this epic are the record-set files, the collector's store and the end-of-run summary; the commissioning tool and monitor (epics 028, 029) read the stamp themselves.

**Settled since the ticket split.** These answers come from the constants table (`doc/design/02-constants.md`, which wins over this file):

- **Transport (C-18).** The stamp travels in a dedicated control message, split into sections that each fit the control message bound of 1 252 bytes of UDP payload (C-17; 1 280-byte fallback MTU on IPv4, C-06, C-100). It is multicast (TTL 1, C-100) to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch. The hello carries only a stamp version number. A full stamp therefore never needs to fit one message; a repeated section larger than one message is split by the rule of ticket 013-13.
- **Third-party observability (C-47).** Producer and consumer each publish, in their own stamp, per-connection summary counters: one per gap class (received, late, missing, marked gap), plus interrupted events, dead, and the observed DSCP. Receiver reports are not multicast and there is no query message. Counting rules: late versus missing per C-108 (a slice counted missing at the deadline and arriving later within the ring span is reclassified: missing −1, late +1, so the missing counter can decrease); interrupted per C-38 and C-102; dead per C-39 and C-111. A connection is named by its stream (source module id, stream number, session; C-10, C-11, C-19, the session being the same for every consumer) and its peer module id.
- **Fuzz budget (C-95).** Each parser is fuzzed 60 s on every change and 1 hour nightly; the corpus is kept.
- **Calibration factor (C-106).** Each target's time per abstract operation is measured once and recorded in the stamp (Platform group).
- **Commissioning window (C-107).** Every module stamps whether a declared commissioning window is active (Injection group).
- **Sink with no certified E (C-103).** Declares E unknown; the Alignment group distinguishes certified, failed and unknown.
- **Test-only hooks (C-120).** Stamped like any injection.
- **Who sends and who owns files (C-144).** The hosted backend (epic 015) sends the regime stamp message this epic defines and provides the single-writer file service. This epic decides when the message is sent and what it contains (013-24), and writes record sets and the collector's store through that service (013-15, 013-17).
- **Control isolation (C-161).** Housekeeping cannot delay the data path. Present on hosted platforms by construction of the thread model; absent or unverified on a device unless the implementer shows a bound.
- **Lateness percentiles (C-160).** 50th, 99th, 99.9th and the maximum. Reclaimable latency (C-157) is published beside them in the Alignment group.
- **Commissioning window (C-153, C-178).** Announced by the tool with an expiry time; modules stamp it while open; ordinary production measurements taken in a window are unqualified, so `qualified()` is false while it is open. The commissioning tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results recorded with their window (epic 028); this epic does not label them unqualified (tickets 013-10, 013-19, 013-25).
- **Per-connection counters (C-159, C-135).** The summary list of C-47 is completed by other-epoch, too-early, duplicates, incomplete reassemblies and demoted rounds; Demoted inputs (C-109) are therefore counted as demoted rounds. A slice on another timeline epoch has its own counter, restarts the dead clock and is not a miss (C-135).
- **Sync and control DSCP (C-152).** Per-peer observed values, see Rule 8.
- **Sync requests above the accepted rate (C-204).** Requests beyond 200 per second from one follower (C-112) are dropped before any work is done and counted per follower in the holder's stamp. The count is a counter (excluded from `StampId`), carried in the holder's per-follower entry (ticket 013-27).
- **Which A is stamped (C-198).** The current A, its own or the one it inherited (C-142). The receiver report (C-40) carries it upstream one hop per report and each fan-in that inherited the old value re-aligns (C-186); since A is a regime field, the change gives a new `StampId` and a new record set (ticket 013-08).
- **Certification state (C-140).** Only a commissioned sink gives a numeric E; uncertified and self-monitored sinks declare E unknown.
- **Clock after holdover lapse (C-162).** Unlocked with time unusable: the stamp shows the clock unlocked with no bound. For a sink with E unknown the lapse is judged against 2 × C-62 (50 µs, C-169); the stamp shows the same result.
- **Publisher of the lock state and bound (C-187).** The servo core (epic 008a), through the published time view; the Clock section carries what that view publishes (ticket 013-08).
- **Certification while the clock is unlocked (C-180).** The sink stays commissioned; its alignment claim is suspended until the clock is locked again and resumes without re-commissioning. The certification field does not change; the suspension is read from the Clock group's lock state (ticket 013-08).
- **Window of the lateness figures (C-182).** Percentiles and reclaimable latency use a rolling 60 s window; the maximum since the connection went live is a separate counter field beside them (ticket 013-08).
- **A fan-in with no skew budget (C-142, C-185).** It stamps its configured budget as A, or A = none when it has neither and its aligned inputs are refused with NoSkewBudget (ticket 013-08).
- **Where the holder records sync DSCP (C-188).** On the clock thread, where it answers sync requests (ticket 013-26).
- **Both stamps of a certification record (C-191).** The sink's and the holder's it followed: `stamp` and `peer_stamp` of the record header (ticket 013-14).
- **Vectors and reject reasons (C-192, C-193).** The vectors crate sits at the workspace root; epic 004 supplies the wire layouts and the reject reasons, so the stamp's `Reject` variants follow epic 004 (tickets 013-02, 013-12). The fuzzing tool is the implementing team's choice (ticket 013-03).
- **Additional test hook (C-145).** A panic injected into a data, clock or housekeeping thread is stamped like any injection.
- **Calibration factor procedure (C-176).** Measured by the DSL conformance suite (epic 025) once per listed target; this epic carries the value.
- **Strict mode scope (C-147, C-165).** See Rule 10.
- **Privileged tests (C-146).** "Unprivileged" in the definition of done applies to per-change CI; no test of this epic needs a privilege.
- **Collected stamps location (C-171).** The collector's store is in the deployment's data directory, where certificates are stored beside the collected stamps.
- **Peer stamp change latency.** A local stamp change starts a new record set before any record carrying the new id; a peer stamp change is seen through the regime-change multicast or, if that is lost, the next 10 s refresh (C-18), so the bound is 10 s.
- **Record-set and collector formats.** Chosen by tickets 013-15 and 013-17 and written into `doc/design/spec/stamp.md`; no other epic constrains them. File names and label text are presentation details decided here and recorded in this epic's documentation; the spec carries only what another epic or an operator's script depends on (C-206).
- **Split with epic 014.** This epic defines the stamp fields and fills the local Linux socket and measurement-ring counters (AC-37, stamp part); epic 014 reads the received DSCP byte and delivers peer-observed values and peer drop counters into the builder.
- **Capability states (C-129).** Four: present, absent, unverified, not requested. Value 0 is not valid and is rejected by the parser.
- **Receive-timestamp source (C-131).** Values: kernel (hosted OS kernel stamp), userspace (clock read after receive), mac-hardware (Ethernet MAC timestamp register, embedded), interrupt (stamped in the receive interrupt, embedded). Each is reported with one of the four C-129 states, so an unconfirmed kernel stamp on macOS or Windows is "kernel, unverified".
- **Strict mode (C-127, C-128).** See Rule 10.
- **Where outputs live (C-130, C-133).** The specification goes to `doc/design/spec/stamp.md`; stamp probe reports go to `doc/design/reports/`; the constants table is `doc/design/02-constants.md`; nothing is written into the frozen POC tree. The stamp types live in `hadronyche-core`.

### Platform facts feeding the capability bits

- Linux: kernel receive timestamps on a wall-clock basis via `SO_TIMESTAMPNS`; per-socket drop counter via `SO_RXQ_OVFL` `[kernel-doc]`.
- macOS and Windows: kernel receive stamps claimed but unverified; no per-socket drop counter `[memory]`. Task 001 measures these and supplies the capability-bit table.
- Hosted platforms: NIC interrupt coalescing biases software receive stamps; commissioning disables it on medianet interfaces and the stamp records its state.
- Embedded: MAC hardware timestamps where available, else interrupt-time; control isolation is absent or unverified unless the implementer shows a bound (C-161); real-time scheduling and memory lock are present by construction (C-147).

### Related constants

| Id | Constant | Value |
|---|---|---|
| C-01 | Domain sample rate | 96 000 Hz, the only rate on the wire |
| C-05 | Default slice size | one packet at the path MTU; multi-packet slices are an explicit, stamped regime |
| C-06 | Path-MTU fallback | 1 280 bytes |
| C-10 / C-11 | Stream number / session / epoch; module id / nonce / domain id | 16 / 32 / 16 bits; 64 / 64 / 16 bits, domain 0 = unconfigured |
| C-13 | DSCP classes | sync 48; media and probes 46; control 40 |
| C-17 | Control message size bound | at most 1 252 bytes of UDP payload |
| C-18 | Regime stamp transport | dedicated sectioned control message; multicast on regime change, refreshed every 10 s; unicast request; hello carries a stamp version number |
| C-19 | Session number | assigned by the source, one per stream, same for every consumer |
| C-34 | Active modules per medianet | at most 64 |
| C-47 | Third-party stream health | per-connection summary counters in each side's own stamp |
| C-62 | Lock bound | at most E/2, 25 µs against C-70 (pending simulation) |
| C-70 / C-71 / C-72 | E targets | Linux 50 µs / macOS 100 µs / Windows 1 ms (pending measurement) |
| C-73 | Definition of E | 99.9th percentile of absolute emission error over a one-hour run; maximum reported beside it |
| C-95 | Fuzz budget | 60 s per parser on every change, 1 hour nightly, corpus kept |
| C-100 | IP version | IPv4 only; multicast TTL 1 |
| C-102 / C-108 / C-109 / C-111 | Miss, late, demoted and dead counting | see Settled since the ticket split |
| C-103 | Sink with no certified E | declares E unknown |
| C-106 | Cost calibration factor | recorded in the stamp |
| C-107 | Commissioning window | stamped by every module |
| C-112 | Accepted sync request rate | 10 to 200 per second |
| C-120 | Test-only injection hooks | stamped like any injection |
| C-125 / C-126 | Definition-of-done applicability; scope labels | not-applicable lines named with a reason; scope from the ticket tally |
| C-127 / C-128 | Strict mode requirements | real-time scheduling on data and clock threads and memory locked; CPU idle restriction best-effort and stamped; unverified treated as absent |
| C-129 | Capability states | present, absent, unverified, not requested |
| C-130 | Where task outputs live | specs in `doc/design/spec/`, reports in `doc/design/reports/`, constants table `doc/design/02-constants.md` |
| C-131 | Receive-timestamp source values | kernel, userspace, mac-hardware, interrupt, each with a C-129 state |
| C-132 | Evidence tag `[standard]` | published standard cited with its clause |
| C-133 | Where code lives | protocol types, stamps included, in `hadronyche-core` |
| C-135 | Slice on another timeline epoch | own per-connection counter; restarts the dead clock; not a miss |
| C-140 | Certification state giving a numeric E | commissioned only; uncertified and self-monitored declare E unknown |
| C-142 | Fan-in outside any presentation group | inherits the A of the downstream presentation group it feeds; with none, an explicit configured budget or its aligned inputs are refused |
| C-144 | Orphan duties of epic 015 | sends the stamp message this epic defines; single-writer file service |
| C-145 | Additional test-only hook | panic injected into a data, clock or housekeeping thread |
| C-146 | Privileged tests | separate privileged nightly job; "unprivileged" applies to per-change CI |
| C-147 | Strict mode and measurements | runs feeding the constants table are strict; hosted-only; devices stamp RT and memory lock present |
| C-152 | Peer verification of sync and control DSCP | holder reports per follower (sync); every module reports per peer (hellos) |
| C-153 | Commissioning window life cycle | announced with an expiry; stamped while open; measurements in it unqualified |
| C-157 | Reclaimable latency | offset − (current maximum lateness + headroom), in the Alignment group |
| C-159 | Per-connection counters | received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead |
| C-160 | Lateness percentiles in the stamp | 50th, 99th, 99.9th and maximum |
| C-161 | Control-isolation capability | housekeeping cannot delay the data path; hosted present by construction |
| C-162 | Clock after holdover lapse | Unlocked, time unusable |
| C-165 | Operator policy | the commissioning configuration sets strict mode |
| C-169 | Alignment claim in holdover when E is unknown | unknown E taken as 2 × C-62 (50 µs) for the lapse rule |
| C-171 | More locations | certificates beside the collected stamps in the deployment's data directory |
| C-176 | Calibration factor procedure | measured by epic 025's conformance suite, carried in the stamp |
| C-177 / C-201 | Seqlock safety and a failed or unverified pin | refusal to start in every mode (not strict mode; not this epic) |
| C-178 | What an open commissioning window disqualifies | ordinary production measurements; the tool's own measurements are valid, recorded with the window |
| C-180 | Certification of a sink whose clock is unlocked | stays commissioned; claim suspended until relocked |
| C-182 | Window of the stamp's lateness figures | rolling 60 s; maximum since the connection went live beside them |
| C-185 | Refusal reasons, completed | 18, including NoSkewBudget (fan-in with no downstream group and no configured budget) |
| C-187 | Publisher of the clock lock state and bound | the servo core (epic 008a), through the published time view |
| C-188 | Where the holder answers sync requests | on the clock thread, where it records the DSCP observed per follower |
| C-191 | "Both stamps" in a certification record | the regime stamps of the sink and of the holder it followed |
| C-192 / C-193 | Vectors crate; supplier of layouts and reject reasons | workspace root, fuzzing tool the team's choice; epic 004 |
| C-198 | Carrier of a changed A upstream | the receiver report (C-40) carries the consumer's current A, own or inherited; one hop per report; inheriting fan-ins re-align (C-186) |
| C-204 | Sync requests above the accepted rate | beyond 200 per second from one follower: dropped before any work, counted per follower in the holder's stamp |
| C-206 | Presentation details of tools | decided by the building epic, recorded in its documentation; only what others depend on goes into specs |

`[code]` Retired POC behaviour: warnings about missing privileges are collected in a process-global list and shown in the UI (`rt.rs:7-19`) but never written next to the numbers; per-module metrics writers on one path (`module.rs:1354`); measurement rings that drop silently.

## Deliverables

In `hadronyche-core`, `no_std`, no allocation:

```rust
// shape, names may be refined
#[repr(u8)] pub enum CapState { Present = 1, Absent = 2, Unverified = 3, NotRequested = 4 } // C-129; 0 and 5..=255 rejected
#[repr(u8)] pub enum Profile { LinuxStd, MacosStd, WindowsStd, Embedded }       // wire values per epic 004
#[repr(u8)] pub enum RxStampSource { Kernel, Userspace, MacHardware, Interrupt } // C-131

pub struct Capabilities {
    pub rx_stamp: RxStampSource, pub rx_stamp_state: CapState,
    pub rt_scheduling: CapState, pub memory_locked: CapState, pub cpu_idle_restricted: CapState,
    pub drop_accounting: CapState, pub dscp_marking: CapState, pub fault_injection: CapState,
    pub holder_eligible: CapState, pub control_isolation: CapState,
}

pub struct StampId(pub u32);              // content-derived over regime fields; counters excluded
pub struct Stamp<'a> { /* borrowed view; sections are length-prefixed and versioned */ }
impl<'a> Stamp<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Reject>;
    pub fn id(&self) -> StampId;
    pub fn qualified(&self) -> bool;      // self-test passed, no injection active, no commissioning window open (C-153)
    pub fn is_complete(&self) -> bool;    // all eight groups present
}
pub struct StampBuilder<'b> { /* writes into a caller buffer */ }
impl<'b> StampBuilder<'b> {
    pub fn identity(&mut self, ..); pub fn capabilities(&mut self, c: &Capabilities);
    pub fn network(&mut self, ..); pub fn connection(&mut self, conn: ConnId, path_mtu: u16, packets_per_slice: u8, pacing: PacingMode);
    pub fn connection_summary(&mut self, conn: ConnId, s: &ConnSummary);   // C-47, C-159
    pub fn peer_dscp(&mut self, peer: ModuleId, hello: Option<u8>, sync: Option<u8>); // C-152; sync in a holder's stamp only
    // holder only: per-follower count of sync requests dropped above 200/s (C-204); setter shape chosen in 013-27
    pub fn clock(&mut self, ..); pub fn alignment(&mut self, ..); pub fn platform(&mut self, ..);
    pub fn drops(&mut self, ..); pub fn injection(&mut self, ..); pub fn self_test(&mut self, r: &SelfTest);
    pub fn finish(self) -> &'b [u8];
}
pub struct SelfTest { pub rx_stamps_consistent: bool, pub rt_granted: bool, pub wake_latency_ns: u32,
                      pub packets_per_slice: u8, pub locked_within_bound: bool }
pub struct ConnSummary { pub received: u64, pub late: u64, pub missing: u64, pub marked_gap: u64,   // C-47, C-159
                         pub other_epoch: u64, pub too_early: u64, pub duplicates: u64,
                         pub incomplete_reassemblies: u64, pub demoted_rounds: u32,
                         pub interrupted_events: u32, pub dead: bool, pub observed_dscp: Option<u8> }

pub struct RecordHeader { pub stamp: StampId, pub peer_stamp: Option<StampId>, pub family: CheckFamily }
```

The `..` parameters follow the field tables written by the section tickets. `Reject` is defined here, with variants following the reject reasons epic 004 specifies for the stamp layout (C-193); `ConnId`, `PacingMode` and `CheckFamily` come from the epics that own them (see Open sub-questions). Counter widths in `ConnSummary` are a shape, fixed with the byte layout from epic 004.

Also in `hadronyche-core`: the rule and reader for splitting a stamp section that exceeds one control message (C-18, C-17).

In `hadronyche-std`:

- a **stamp publication schedule** for hosted nodes: decides when the stamp message goes out (to the control group on any regime change, every 10 s, and in answer to a unicast request; C-18), produces the split messages, and exposes the stamp version number for the hello; the hosted backend (epic 015) sends them (C-144);
- a **stamp collector** that receives stamps published by any node (including devices), requests unknown ones by unicast, and records them in the deployment's data directory (C-171) through epic 015's single-writer file service (C-144);
- a **record-set writer**: writes record sets through epic 015's single-writer file service (C-144), so a file has one writer owned by the process; starts a new record set whenever the local or the peer stamp id changes;
- an end-of-run check that fails the run and names any check family with zero samples;
- **strict mode** (hosted only, set by the operator policy in the commissioning configuration; C-147, C-165): a start-up switch that refuses to start on domain 0, any failed self-test item, or real-time scheduling or memory locking not present (C-127, C-128), naming it;
- labels for unqualified, injected, commissioning-window and missing-privilege figures in the outputs of Rule 13.

Documentation: `doc/design/spec/stamp.md` with the field tables, the encoding, how a stamp is published and requested, and the capability-bit table from task 001.

## Dependencies

- Needs from 001: the capability-bit definitions and measured platform facts. From 004: the byte layout of the stamp message, its envelope, section headers and enum wire values; the transport itself is decided (C-18, see Background). From 005: the hello that carries the stamp version number and the control-plane handling of the unicast stamp request. From 007: vectors for the stamp encoding, in the vectors crate at the workspace root (C-192), with epic 004's reject reasons (C-193). Reads from 008a (C-187): the lock state and bound, through the published time view; this epic only carries them, so it is not a dependency. From 015 (C-144): the single-writer file service and the sending of the stamp message; tickets 013-15, 013-17 and 013-24 use them, so those parts of 015 land first although 015 as a whole depends on this epic. From 025 (C-176): the measured calibration factor per target.
- Provides to 014: the stamp fields for peer-observed DSCP and drop counters, which 014 fills. To 015: the stamp message and its publication schedule, which 015 sends (C-144). To 015 and 017: the collector, the record-set writer, the zero-sample failure, and the lock state, bound and packets-per-slice inputs they need for Rules 3 and 6. To 019 and 020: the provenance attached to every ladder result. To 026: the publish-over-network path and splitting rule for devices. To 028 and 029: the object the commissioning tool and the monitor read, including the per-connection summaries (C-47).

## Acceptance criteria

1. **013-AC-1** Stamp vectors encode and decode byte for byte (system criterion AC-1); every parser of this epic survives fuzzing with zero panics at the C-95 budget (60 s per change, 1 hour nightly, corpus kept).
2. **013-AC-2** AC-31: every record set carries a complete stamp (all eight groups present); injection and missing privileges are labelled; a regime change starts a new record set; one writer per file.
3. **013-AC-3** AC-9: a run in which a check family analysed zero samples exits non-zero and names the family.
4. **013-AC-4** Changing any regime field (for example toggling an injection, losing real-time scheduling, a timeline epoch change, a path-MTU fallback) changes the `StampId` and starts a new record set: before any record carrying the new local id, and within 10 s (C-18) of a peer's change. A change of counters only leaves the `StampId` unchanged and starts no record set.
5. **013-AC-5** A two-endpoint record carries both stamp ids, and the collector can resolve both.
6. **013-AC-6** `qualified()` is false while the self-test has not passed, any injection is active, or a commissioning window is open (C-153, C-178); unqualified numbers are labelled as such in every output of Rule 13. The commissioning tool's own results are not labelled unqualified by this epic (C-178).
7. **013-AC-7** Strict mode refuses to start with domain 0, with any failed self-test item, or with `rt_scheduling` or `memory_locked` in any state other than present (unverified treated as absent, C-128), naming the item; a CPU idle restriction that is not present never refuses (C-127); a device profile never runs strict mode (C-147).
8. **013-AC-8** AC-34 (stamp part): the stamp shows requested DSCP classes and the peer-observed values, or "unverified"; a consumer's stamp shows the observed DSCP per connection (C-47); the holder's stamp shows the DSCP observed on each follower's sync requests and every module's stamp the DSCP observed on each peer's hellos (C-152).
9. **013-AC-9** AC-37 (stamp part): an induced socket overflow appears in the stamp's drop counters.
10. **013-AC-10** A device-profile stamp builds without an allocator, and every message carrying it is at most 1 252 bytes of UDP payload (C-17, C-18); the rule for splitting a section larger than one message is specified and tested.
11. **013-AC-11** Producer and consumer of a connection each publish its summary counters (received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead, observed DSCP; C-47, C-159); a third party resolves both through the collector (C-47).
12. **013-AC-12** A hosted node's publication schedule emits its stamp on a regime change, every 10 s, and in answer to a unicast request (C-18); epic 015 sends what it emits (C-144).
13. **013-AC-13** The calibration factor (C-106, C-176), the commissioning window state and expiry (C-107, C-153) and each test-only hook (C-120, C-145) appear in the stamp, and toggling the window or a hook changes the `StampId`.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Every stamp field has a stated source on each of the four profiles, or is marked absent there
- [ ] The stamp format is versioned independently of the media format, with the rule for unknown sections (skip and count)

**Not applicable in this epic (C-125):** none. Every line has a subject here: the epic adds code and tests, parsers, stamp fields, design documents (the constants table is `doc/design/02-constants.md`, C-130), open items, new factual claims and a versioned wire format; it does not touch the frozen POC, so that line applies as a prohibition. "Unprivileged" applies to per-change CI (C-146); no test of this epic needs a privilege.

## Open sub-questions

Decided and moved to Background: whether a full stamp fits one control message (C-18: it need not; sectioned transport); how an unverified receive-stamp source is expressed (C-129, C-131: a source value plus a capability state); the meaning of `control_isolation` (C-161: kept); which lateness percentiles are carried (C-160); whether an open commissioning window makes `qualified()` false (C-153: yes); whether Demoted inputs are in the summary (C-159: demoted rounds); what an open window disqualifies (C-178: production figures only); the window of the lateness figures (C-182: rolling 60 s plus the since-live maximum); the certification state of an unlocked sink (C-180: stays commissioned).

| Question | Owner | Falsifier |
|---|---|---|
| How is "full configuration" encoded compactly and comparably? | This epic (ticket 013-04) | Two identical configurations yield different stamp ids |
| Which interrupt-coalescing states can be read unprivileged on each OS? | Tasks 015, 021, 022 | Not readable; the field becomes "unverified" there and a commissioning checklist item |
| Which receive-stamp source does each OS really provide? | Task 001 (C-138: oldest and current release of each OS) | Kernel stamps absent; capability bits and E budgets change |
| How does the hello's stamp version number (C-18) relate to the `StampId`, and does it change on a counter refresh? | Epic 004, with epic 005 | Peers fetch on every refresh, or miss a regime change |
| Byte layout: envelope, section headers, enum wire values, counter widths | Epic 004 | Vectors from epic 007 cannot be reproduced from the layout |
| Wake-latency pass limit and receive-stamp consistency tolerance for the self-test | A measurement (task 001, then L2), recorded in the constants table | A healthy host fails the self-test, or a degraded one passes |
| `ConnId`, `PacingMode` and `CheckFamily` definitions | Epic 004 (`ConnId`, `PacingMode`); epics 015 and 017 (`CheckFamily`) | This epic's placeholders differ from the owners' types |
