# 015a — Hosted runtime core

**Phase:** P4 · **Scope:** L (24 tickets, about 35 days) · **Depends on:** 004, 008, 009, 010, 011, 012, 013, 014, 016 · **Blocks:** 015b, 017, 018, 019, 021, 022, 028, 029

## Goal

Build the hosted (operating-system) half of the runtime for Linux: the process and thread model, the sockets, the receive and send paths, pacing, housekeeping and its supervision, regime stamp publication with its counters, and file output. It drives the pure `no_std` state machines and codecs of `hadronyche-core` against real sockets and real threads. It contains no protocol logic of its own.

This is part a of epic 015, split under C-143. Part b (015b) verifies this runtime end to end (isolation, fail-stop, sync rate, allocation, wall-clock step, don't-fragment), adds strict mode and the test-only hooks, takes the strict-mode measurements and updates the design documents. Part b depends on this part; this part does not depend on part b. A reference elsewhere to "epic 015" means both parts.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules (sources, processors, sinks) exchange *slices* — all content of one stream for one stretch of media time — addressed by *position* on a domain-wide timeline: `position = (network_time − epoch_origin) × 96 000`. The wire carries one sample rate only, the domain rate of 96 000 Hz; an edge whose hardware runs at another rate converts at the edge (C-01, C-80). Linux hosts are the backbone (clock holder, elections, conformance, commissioning); macOS and Windows machines are edges; embedded devices are followers. The runtime owns *when and whether* content moves; module code owns *what* it is.

**Where this task sits.** `hadronyche-core` (tasks 008 to 014) provides, as pure code: the servo and its seqlock, the packetiser and reassembler, the alignment ring and gap classification, the binary control state machines (discovery map, election, admission handshake, connection states), the regime stamp format, and the measurement messages. Core talks to the world through abstract time and transport traits. This task implements those traits on Linux and supplies the threads. Real-time thread setup is task 016; this task calls it through a hook whose default is a no-op, so the stub backend and the macOS and Windows backends compile without it.

**Duties assigned to the hosted backend (C-144).** Emitting the gap slice when a module misses its own deadline; raising the processing-overrun event; the housekeeping-restart criterion AC-20 (c); the single-writer file service; sending the regime stamp message that epic 013 defines. All five are built in this part; the AC-20 (c) scenario test is in 015b.

**Wire figures used here.** At defaults a stream carries 6 channels; each channel is 192 bytes per slice (48 frames × 4 bytes) and there is no per-channel id byte, so a slice is 1 152 bytes of audio; the channel map is in the manifest (C-03, C-04). A default slice fits 1 252 bytes of UDP payload: 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum (C-137). A default packet is about 1 320 bytes on the wire including all Ethernet overhead (C-20), about 21.1 Mbit/s per stream (C-21). A 96-channel source is 16 streams, 338 Mbit/s and 32 000 packets/s (C-22). Two 96-channel bundles are 32 streams: 32 packets per 0.5 ms slice, 64 000 packets/s and 676 Mbit/s (C-92). IP is IPv4 only, multicast with TTL 1; the 1 280-byte fallback path MTU (C-06) gives 1 252 bytes of UDP payload (C-100), which is also the bound on every control message (C-17).

**Thread model `[decision]`.**

- One **housekeeping** thread and one **clock** thread *per process*; one **data** thread *per module*. The POC ran three threads per module; that is retired.
- **L-1** (as applied here): liveness — interrupted, dead, receiver reports and the producer timeout — is executed on the data threads and never passes through housekeeping.
- **L-7:** a panic in a data or clock thread terminates the process (fail-stop). A panic in housekeeping does **not**: housekeeping is supervised and restarted, live streams continue, and the module is marked *control-degraded*.
- **Housekeeping restart policy (C-139):** exponential backoff from 100 ms to 10 s; after 5 panics within 60 s restarts stop: the module stays control-degraded with an alarm, its live streams continue, and it accepts no new connection until an operator restarts it. A connection alarm is raised on control-degraded (C-173). Once restarts have stopped, the housekeeping supervisor keeps a minimal refusal path that answers an admission request with the refusal reason ControlDegraded (C-185, one of 18 reasons).
- **L-8:** no lock is shared between housekeeping and the data or clock threads. They communicate by messages or lock-free publication only.
- **Seqlock safety and CPU pinning (C-177):** if a thread layout's seqlock safety depends on a CPU pin and that pin fails, the runtime refuses to start, in every mode, strict or not. A pin that cannot be verified is treated as failed (C-201). Default priorities (clock above data) never depend on pinning.
- **L-4:** housekeeping admits unknown sources only; it never touches a live data path.
- **L-5:** destinations are pinned at admission and change only by re-admission. A hello carrying a new address never redirects a live stream.
- The data thread's ticker runs on **network time** (from the servo's published view), not on local monotonic time.

**What the POC did, and why it is retired `[code]`.**

- `src/module.rs:1518`: the clock thread takes `inner.target.lock()` — a mutex shared with the non-real-time housekeeping thread (`:1484`) — on every iteration. Violates L-8.
- `src/module.rs:1669`: the data thread drains every input socket completely (`while let Some(..) = net::recv_now(..)`) before it looks at its own deadline. One flooding input starves the ticker.
- `src/module.rs:1126`: clock requests are paced by a 5 ms receive timeout. Measured result: 37 completed exchanges/s at `--sync-hz 50` and 77/s at `--sync-hz 1000` `[measured]`, on a kernel with `CONFIG_HZ=250` `[measured]` (P-02, P-03). Timeouts are not deadlines.
- `src/module.rs:1557` and `:1671`: receive time is derived as `real_ns() − kernel_stamp`. `SO_TIMESTAMPNS` delivers `CLOCK_REALTIME` stamps `[kernel-doc]`, so a wall-clock step between stamp and read corrupts the sample, and `.max(0)` hides it as zero delay.
- `src/module.rs:1630`: the ticker epoch is `mono_ns()`, so the media clock follows the local crystal.
- `src/module.rs:1639`: `routes = r.clone()` allocates on the data thread.
- `src/module.rs:1354`: each module calls `File::create` on the same metrics path; in loopback four handles overwrite each other and the file is corrupt `[measured]`.

**Receive path rules `[decision]`.**

- **Bounded drain (C-16):** per input per slice period (the cycle of C-16 is one slice period, C-156), at most 2 × the admitted packets per slice for that input. The drain bound is also the `recvmmsg` batch size: one call per input per slice period, with the message vector preallocated at admission to the drain bound (64 for a 32-member bundle at one packet per slice, C-05, C-08).
- **Source filter:** only packets from the source address pinned at admission are parsed; others are counted and dropped.
- **One socket per bundle**, demultiplexed by the 16-bit stream number. A bundle is a group of aligned narrow streams from one source (6 channels each at defaults, C-04; at most 32 members, the controller member included, C-08, C-209: a bundle source's controllers always ride in their own member stream, C-205). **Batch receive is required** (several datagrams per system call): two 96-channel bundles are 32 packets per 0.5 ms slice.
- **Socket buffer** sized to the declared rate times the skew budget. The skew budget is capped by the presentation group's audition offset A (C-96), which has no default and is commissioned per group (C-55); the buffer is therefore declared rate × A, computed at admission. A fan-in outside any presentation group uses the A of the downstream group it feeds or its configured budget (C-142); with neither, admission refuses its aligned inputs with NoSkewBudget (C-185) and no socket is opened. The inherited A reaches the fan-in in the receiver reports of its consumers, which carry each consumer's current A, its own or the one it inherited (C-198); a change propagates one hop per report, and a fan-in that inherited the old value re-aligns (C-186) at a position it picks itself, one lead (64 slices, C-43) ahead, announced in its own output, so fan-ins at different depths switch at different announced positions (C-208). If a new A exceeds what a fan-in's buffer memory holds, the fan-in keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm (C-174) and keeps its streams running (C-211); the alarm stays raised until a later change of A that fits, and the fan-in's receiver report carries the A it was given, not the clamped value, so upstream modules size themselves to what the sinks need (C-216). A sink whose A changes switches at its own position, one lead ahead, and announces it to nobody (C-214). A ratchet step is a re-alignment like any other, announced one lead ahead in the fan-in's own output, never applied silently on the next round (C-215). The commissioning tool's check of declared buffer memory before applying a change of A is epic 028's. The kernel may clamp the request; the effective value is reported.
- **Per-socket overflow counter** read with every receive: `SO_RXQ_OVFL` returns a cumulative drop count in a control message `[kernel-doc]`. It goes into the regime stamp and the receiver report.
- **Kernel receive timestamps** with wall-clock step detection: compare the realtime-minus-monotonic difference before and after; if it moved, discard the sample for clock purposes. Detection lives in this backend, which builds each exchange from receive stamps; the servo only ever receives exchanges that passed it (C-121). The detector tolerance and `declared_jitter_ns()` are configurable here and measured in strict mode by 015b (C-147).
- **Supported kernels.** Supported OS versions are set after the receive-stamp probe task (001) reports; its probes test the current release of each OS and the oldest obtainable one (C-138). The `recvmmsg` spike (015a-02) runs on the same two Linux kernels.
- Interface drop counters (`rx_dropped`, `tx_dropped` in `/sys/class/net/<if>/statistics` `[kernel-doc]`) are sampled each time a regime stamp is built — on every regime change and at every 10 s refresh (C-18) — and stamped.

**Send path rules `[decision]`.**

- **Don't-fragment on every packet.** IP fragmentation is forbidden. By default a slice is one packet (C-05) at the path MTU; fallback path MTU is 1 280 bytes, 1 252 bytes of UDP payload (C-06, C-100).
- **Path MTU.** The path is verified by the probe pair, which is sent at the intended media packet size and is exempt from the control message bound (C-116). This backend supplies the interface MTU (an upper bound), sends the probe with don't-fragment set and surfaces a local "message too big" as a distinct send error; when to probe and how a failed probe selects the fallback belong to core (tasks 012, 014).
- **DSCP classes (C-13):** sync CS6 (48); media and one-way probes EF (46); control (hello, election, admission, receiver reports, regime stamp) CS5 (40). Marking is verified by the receiving peer, which reads the received TOS byte (`IP_RECVTOS`). The receiver report of a bundle carries the media DSCP observed on that bundle; every observed class is published per connection in the observer's own regime stamp, or marked "unverified" (C-47). For sync and control (C-152): the holder reports in its stamp the DSCP observed on each follower's sync requests, and every module reports the DSCP observed on each peer's hellos. The holder answers sync requests on the clock thread, never on housekeeping, and that is where it records the DSCP observed on each follower's requests (C-188).
- **Userspace pacing on the deadline clock:** packets of one slice period are spread over at most 25 % of the period (125 µs at defaults, C-14), plus a per-source phase offset derived from the module id. Phase offset plus spread stay inside the slice period: the phase offset is in [0, 75 %) of the period (C-155). The schedule is computed by core (task 010); this task executes it and refuses a schedule that does not fit. Per-output override 0 disables spreading; the override is operator policy set in the commissioning configuration (C-165).
- **Own deadline missed (C-144).** When a module misses its own deadline, its data thread emits a marked-gap slice for that position on every output stream and raises the processing-overrun event. Downstream, the marked gap resets the interrupted count (C-102) and restarts the dead clock (C-111).
- Sync requests are paced by deadline (`ppoll` with a computed timeout or equivalent); the achieved rate must equal the configured rate (50/s by default, C-36) and be reported. The accepted configured range is 10 to 200 requests per second, and AC-7 is checked over this range (C-112); the check is 015b's. On the holder, requests beyond 200 per second from one follower are dropped before any work is done and counted per follower in the holder's stamp (C-204).

**Liveness, as executed here (L-1).** The consumer's data thread declares an input *interrupted* after 8 consecutive missed slices (C-38). A miss is a slice absent at its deadline — missing, or arriving late; a marked gap or a slice on another timeline epoch proves the upstream is alive and resets the count (C-102). An absent slice is counted missing at the deadline and reclassified as late if it arrives while still within the ring span (C-108). The input is *dead* 2 s after the last slice received with content or a marked gap (C-39, C-111); a slice on another timeline epoch also restarts the dead clock and has its own per-connection counter (C-135). The consumer sends one receiver report per bundle every 100 ms (C-40), unicast to the producer, carrying the consumer's current audition offset A (C-198); receiver reports are never multicast (C-47). The producer's data thread stops sending after 2 s without reports (C-41). A sink-only device is judged by the producer timeout alone (C-168). None of this passes through housekeeping.

**Elections, hellos and holder stall.** An election starts only on 1.5 s of sync-reply silence (C-37); hello expiry (3 s, C-31) applies to non-live peers only. The clock thread publishes the time of the last valid sync reply lock-free, and housekeeping feeds it to the election state machine, so a holder whose hellos stop while its replies flow is never replaced. A holder compares its monotonic clock with a clock that keeps running through suspend; a divergence above the step threshold (2 ms, C-67) bumps the timeline epoch (C-136). Reading the two clocks is this backend's (on Linux `CLOCK_MONOTONIC` and `CLOCK_BOOTTIME` `[kernel-doc]`); the epoch bump is core's.

**Regime stamp and stream health.** The regime stamp is a dedicated control message split into sections that each fit 1 252 bytes (C-17). Housekeeping multicasts it to the control group on any regime change and refreshes it every 10 s; a unicast request gets an immediate reply; the hello carries only the stamp version number (C-18). Data threads, the clock thread, the sockets and the file service publish their counters lock-free; housekeeping reads a snapshot when it builds the stamp (L-8). Each producer and consumer publishes per-connection counters in its own stamp (C-47, completed by C-159): received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead; plus the observed DSCP, the lateness 50th, 99th and 99.9th percentiles and maximum (C-160), and the reclaimable latency, offset − (current maximum lateness + headroom) when positive, in the alignment group (C-157). The percentiles and maximum, the maximum lateness used for reclaimable latency, and the cannot-meet-A alarm use a rolling 60 s window; the maximum since the connection went live is reported beside them (C-182). While a commissioning window is open (announced on the control group with an expiry time, closed at expiry or on explicit close, C-153) the stamp carries it, and ordinary production measurements in the stamp (latency, loss, alignment figures) are unqualified (C-178); the window state is core's, housekeeping feeds it the announcements and stamps it. There is no query message beyond the stamp request (C-47).

**Capability states.** Every capability is stamped in one of four states: present, absent, unverified, not requested (C-129). The receive-timestamp source is one of kernel, userspace, mac-hardware or interrupt, each reported with a C-129 state (C-131); on Linux only kernel and userspace occur. This part counts which source each received message used and keeps the task 016 hook's result; mapping to states, strict mode (C-127, C-128) and the stamp fields are 015b.

**Where outputs live.** Reports and results go to `doc/design/reports/`, values to the constants table `doc/design/02-constants.md`, evidence to `doc/design/06-evidence-appendix.md` (C-130); the architecture specification, which holds the thread model, is `doc/design/01-architecture.md` (C-171). Throwaway probes go in `probes/`; harness and tool binaries in the `hadronyche-tools` crate (C-133). Nothing is written into the frozen POC tree.

**Files.** A file has **one writer, owned by the process**, never by a module (C-144). Writes go through a dedicated writer thread, because under I/O load the kernel can hold a writer for seconds `[code]` (POC comment at `src/module.rs:1351`). Records are framed one per line.

## Deliverables

Crate `hadronyche-std` (hosted common):

- `process.rs` — `Runtime`: owns the housekeeping thread, the clock thread, the file writer and the counter board; registers modules; calls the task 016 real-time hook and keeps its per-thread result; refuses to start when a pin on which a thread layout's seqlock safety depends fails or cannot be verified (C-177, C-201).
- `supervisor.rs` — restarts housekeeping after a panic under the C-139 policy; sets the control-degraded mark and raises its alarm; once restarts stop, keeps a minimal refusal path that answers admission requests with ControlDegraded (C-185).
- `data_thread.rs` — per-module loop: bounded drain → core alignment ring → module process callback → paced send → receiver reports carrying the current A, and re-alignment of a fan-in when the A it inherited changes, at a position it picks and announces (C-198, C-186, C-208), keeping its streams running if the new A exceeds its buffer memory, with the alarm held until an A that fits and the given A still reported (C-211, C-216); a sink's switch to a changed A at its own position (C-214); the announcement of a ratchet step one lead ahead (C-215); emits the gap slice and the processing-overrun event on a missed own deadline (C-144); publishes per-connection health counters.
- `clock_thread.rs` — deadline-paced sync exchanges driving the core servo; on the holder, answers sync requests (never housekeeping, C-188), records their DSCP (C-152) and drops, before any work, requests beyond 200 per second from one follower, counting them per follower (C-204); per-follower tables are preallocated for 63 peers (C-217); publishes through the core seqlock the time view, which carries the clock lock state and bound computed by the servo core (epic 008a, C-187); publishes last-reply time for the election; runs the holder stall check (C-136).
- `housekeeping.rs` — drives the core discovery, election and admission state machines; builds and sends the regime stamp, including an open commissioning window and the unqualified mark on production figures (C-153, C-178); talks to other threads by bounded lock-free queues and reads lock-free counters.
- `pacing.rs` — executes a core pacing schedule against the host clock.
- `files.rs` — single-writer file service.

Crate `hadronyche-os-linux`:

- `net.rs` — sockets (IPv4 only): `recvmmsg` batch receive with `SO_TIMESTAMPNS`, `SO_RXQ_OVFL`, `IP_RECVTOS`; `IP_MTU_DISCOVER` set to forbid fragmentation; `IP_TOS`; multicast join pinned to the medianet interface, TTL 1.
- `ifstats.rs` — interface MTU, speed and drop counters.
- `clock.rs` — `HostClock` on `CLOCK_MONOTONIC`; suspend-inclusive read on `CLOCK_BOOTTIME`; wall-clock step detector.

Trait shapes (names may be refined):

```rust
/// Host clock H, in nanoseconds, monotonic.
pub trait HostClock: Send + Sync {
    fn now_ns(&self) -> i64;
    fn declared_jitter_ns(&self) -> u32;
    /// A clock that keeps running through suspend (C-136); None where unavailable.
    fn suspend_inclusive_ns(&self) -> Option<i64>;
}

/// Receive-timestamp source (C-131); Linux produces Kernel or Userspace only.
pub enum StampSource { Kernel, Userspace, MacHardware, Interrupt }

pub struct RxMeta {
    pub len: usize,
    pub from: core::net::SocketAddrV4,
    /// Receive time on the host clock; None if the sample was discarded (wall-clock step).
    pub rx_host_ns: Option<i64>,
    pub source: StampSource,
    /// Cumulative drops on this socket, when the platform provides it.
    pub dropped_total: Option<u32>,
    /// Received TOS byte, for peer-verified DSCP.
    pub tos: Option<u8>,
}

pub trait DatagramSocket {
    /// Receive up to `max` datagrams without blocking. Returns the count.
    fn recv_batch(&mut self, bufs: &mut [RxBuf], meta: &mut [RxMeta], max: usize) -> usize;
    /// Never blocks; a full queue is a counted drop at the sender.
    fn send_to(&mut self, packet: &[u8], to: core::net::SocketAddrV4) -> Result<(), SendError>;
}

pub trait OsNet {
    type Socket: DatagramSocket;
    fn open_media(&self, iface: IfaceId, class: DscpClass, rcvbuf: usize) -> io::Result<Self::Socket>;
    fn open_discovery(&self, iface: IfaceId, group: core::net::SocketAddrV4) -> io::Result<Self::Socket>;
    fn iface_stats(&self, iface: IfaceId) -> IfaceStats;
}
```

Supporting types (minimal; refined in the crate docs):

- `RxBuf` — one preallocated receive buffer, large enough for the largest datagram the socket admits (the interface MTU minus IPv4 and UDP headers).
- `SendError` — `QueueFull` (counted drop at the sender), `MessageTooBig` (would need fragmentation; never fragmented), `Other(errno)`.
- `IfaceId` — the medianet interface, by kernel interface index.
- `DscpClass` — `Sync` (48), `Media` (46; also one-way probes), `Control` (40).
- `IfaceStats` — interface MTU, link speed (or unknown), `rx_dropped`, `tx_dropped`.
- `CapabilityState` — `Present`, `Absent`, `Unverified`, `NotRequested` (C-129).

## Dependencies

- **Needs:** wire layouts (004); servo, seqlock, time view (008; the clock lock state and bound in the time view are published by the servo core, epic 008a, C-187); alignment ring, gap classes, reassembly, lateness statistics (009); packetiser, bundle demux, pacing schedule, announcements, gap slice encoding (010); discovery and election state machines (011); admission and connection state machines with pinned destinations (012); stamp format (013); receiver report and probe messages (014); real-time thread setup (016), called through a hook from `process.rs`. The timeline epoch bump on a holder stall is core's (006, 008, 011). The kernel support list comes from task 001's report (C-138).
- **Provides:** a running Linux runtime to part 015b, the reference modules and L0 harness (017), the ALSA edge (018), the L1 harness (019), commissioning (028) and the monitor (029); the trait surface the macOS (021) and Windows (022) backends implement.

## Acceptance criteria

1. **015a-A1 · Drop accounting (AC-37).** An induced socket overflow appears in the stamp's drop counters and in the receiver report.
2. **015a-A2 · Peer-verified DSCP (AC-34).** The receiver report of a bundle shows media 46; the observer's regime stamp shows, per connection, media and probes 46 and control 40 on the peer's hellos; the holder's stamp shows sync 48 on each follower's sync requests (C-152) and, per follower, the count of sync requests dropped above 200 per second (C-204); a class not yet observed reads "unverified" (C-47).
3. **015a-A3 · Regime stamp transport (C-18, C-144, C-178).** The stamp is multicast on every regime change and refreshed every 10 s, each section at most 1 252 bytes (C-17); a unicast request is answered immediately; the hello carries the current stamp version. While a commissioning window is open the stamp carries it and marks production latency, loss and alignment figures unqualified; after expiry or explicit close it does not (C-153, C-178).
4. **015a-A4 · Stream health in the stamp (C-47, C-159, C-160, C-157, C-182).** For each connection, producer and consumer stamps carry every C-159 counter, interrupted events, dead and the observed DSCP; the consumer's stamp carries the lateness 50th, 99th, 99.9th percentiles and maximum over a rolling 60 s window with the maximum since the connection went live beside them (C-182), and the reclaimable latency when positive, computed from the windowed maximum; a slice reclassified from missing to late moves one count between the two (C-108); a slice on another timeline epoch increments the other-epoch counter and restarts the dead clock (C-135).
5. **015a-A5 · Own deadline missed (C-144).** A module that overruns its deadline on a slice emits one marked-gap slice per output stream for that position, no content for it, and one processing-overrun event; its consumers raise no interrupted event over 8 consecutive overruns.
6. **015a-A6 · Holder stall (C-136).** A simulated divergence of more than 2 ms between the monotonic and the suspend-inclusive clock on a holder hands exactly one stall to core; 2 ms or less hands none.
7. **015a-A7 · Housekeeping restart policy (C-139).** Restarts follow a backoff from 100 ms doubling to 10 s; the 5th panic within 60 s stops restarts, the module stays control-degraded with an alarm, live streams continue and no new connection is admitted: an admission request is answered with the refusal reason ControlDegraded (C-185).

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The trait surface compiles with a stub backend (proves macOS and Windows can implement it)
- [ ] Per-packet receive cost measured on the development machine and recorded for task 019

**Not applicable in this part (C-125):** "Per-packet receive cost measured on the development machine and recorded for task 019" — the receive path is built here, but the measurement runs in strict mode (C-147) and is ticket 015b-10. Every other line has a subject: the parsers are the control-message parser (015a-06) and the sysfs parser (015a-07), fuzzed to C-95 (60 s on every change, 1 hour nightly, corpus kept); the stamp fields are the drop, DSCP, health, alignment-figure (with the C-182 window), commissioning-window, overrun, stall and per-follower sync-drop fields; the trait surface is defined in 015a-01 and extended in 015a-21. "Unprivileged" applies to per-change CI (C-146); no test of this part needs a privilege. The consolidated design-document update for both parts is 015b-11.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Does `recvmmsg` return per-message `SO_RXQ_OVFL` and timestamps as expected on the target kernels? | This part (015a-02), on the current and oldest obtainable kernels; the support list comes from task 001's report (C-138) | A kernel in the support list returns neither |
| Does interrupt coalescing bias kernel receive stamps enough to matter? | Task 019 (L1) | Servo offset shifts when coalescing is toggled |
| Value of `declared_jitter_ns()` for `CLOCK_MONOTONIC` | Measurement in strict mode, 015b-15 (C-147), recorded in the constants table; confirmed at L2 | Measured read jitter on the development machine exceeds the declared value |
| Tolerance of the wall-clock step detector (how far realtime-minus-monotonic may move without a step) | Measurement in strict mode, 015b-15 (C-147), under NTP slewing, recorded in the constants table | A slewed but unstepped clock discards samples, or a 1 ms step passes undetected |
| Stamp fields for host counters not named by C-47 or C-159: discarded clock samples, sender-side queue drops, the control-degraded mark, the processing-overrun event, holder stalls, per-follower sync requests dropped above 200 per second (placed in the holder's stamp by C-204) | Wire specification (004), implemented in 013 | A counter this part produces has no field to land in |
| Capacities of the bounded queues between housekeeping and the data and clock threads | This part (015a-15), recorded in the constants table | A full queue is counted during a nominal admission burst of 64 modules (C-34) |
| Does a producer resume sending when receiver reports return after the C-41 timeout, or only after re-admission? | Control state machines (005) | Behaviour differs between the producer's data thread and the connection state machine |
| Content schema of the metrics file (framing is one record per line) | L1 harness (019), its consumer | The harness cannot parse a record this part writes |
| Does the CI host allow unprivileged user namespaces (needed for the forced-MTU send test of 015a-05)? | A check on the CI host | The namespace cannot be created and the test cannot run unprivileged |
