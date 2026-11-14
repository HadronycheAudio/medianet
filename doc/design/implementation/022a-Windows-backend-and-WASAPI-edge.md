# 022a — Windows backend and WASAPI edge

**Phase:** P4 (second half; does not block P5) · **Scope:** L (21 tickets, about 30.75 days) · **Depends on:** 001, 004, 005, 008a, 015, 016, 018, 020, 024a · **Blocks:** 022b

Epic 022 is split under C-143 into two parts: **022a** (this file) builds the Windows backend and the WASAPI edge; **022b** certifies it at L2 and publishes the certification record. 022b depends on 022a. A reference elsewhere to "epic 022" means both parts. Acceptance criteria keep the original `022-AC-n` numbering in both parts.

## Goal

Make a Windows machine a working **edge** member of a medianet, ready for certification: a sink and a physical-input capture module on WASAPI in exclusive mode, with a Windows implementation of the hosted backend traits (sockets, host clock, real-time threads) and a complete `windows-std` regime stamp. The Windows edge never holds the clock and never stands for election. Its certified numbers are deliberately looser than Linux's, and the regime stamp says exactly how. The L2 certification runs and the certification record are epic 022b.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules exchange *slices* addressed by *position* on a domain-wide timeline: `position = (network_time − epoch_origin) × rate`; write `T(p)` for the network time of position p. Linux hosts are the backbone (clock holder, elections, conformance, commissioning). macOS and Windows machines join as **edges** — typically a musician's laptop. Embedded devices are followers.

**Evidence warning.** Every Windows platform fact below is `[memory]` — recalled, not verified — until task 001 (receive-stamp probes) and the L2 run of epic 022b confirm it. A figure that comes from a named, published standard may instead be cited with its clause and tagged `[standard]` once the citation is checked (C-132); vendor platform behaviour is not a standard and must be measured. Supported OS versions are set after task 001 reports; its probes test the current release and the oldest obtainable one (C-138).

**Scope rulings `[decision]`.**

- **Edge only.** The holder-eligible capability bit is never set; the build uses the edge-only feature.
- **WASAPI exclusive mode is mandatory for certification.** Shared mode passes through the audio engine with opaque latency and possible resampling.
- **Capture is physical device inputs only.** Application-output capture (a virtual audio device, which on Windows means a signed kernel driver) is out of v1.
- If the musician's own software holds the interface (exclusive mode cannot be obtained because another process has the device open, C-175), the edge **reports it and certifies nothing**.
- **No spinning ticker on Windows in v1.** Sink edges do not pace: they consume. **Capture edges pace emission off the audio callback, batching several slices per engine period; this is stamped.**
- The media deadline clock exists in core only as an abstract trait; this backend provides it from the audio callback plus a QPC regression.
- **One rate on the wire.** The domain rate is 96 kHz and it is the only rate on the wire (C-01). Devices often run at 48 kHz; such an edge converts 2:1 through the kit's resampler node and the wire stays at the domain rate (C-80). The resampler node may appear only inside an edge module (C-134), and it is offered only at 2:1 and 1:2, whose delay is a whole number of samples; other ratios are out of v1 (C-105). A Windows edge device must therefore run at 96 kHz or 48 kHz. The conversion delay is part of D (C-134, C-170).
- **The edge builds no resampler** (C-197). Both the fixed 2:1 / 1:2 node and the steerable rate actuator that tracks the device crystal (C-189) come from the arithmetic kit, epic 024a. This epic only wires them in. Their delay counts only in D and never in L: L is the module's structural delay in the domain-rate timeline, and the module's manifest reports the conversion delay separately, on the device-facing port (C-207). The rate actuator is driven through the kit's rate-actuator trait, defined in epic 024a with a ratio range of ±1 000 ppm around the nominal ratio (C-210); this epic consumes it and defines no trait of its own.
- **IPv4 only** in v1; multicast is sent with TTL 1; the 1 280-byte path-MTU fallback (C-06) leaves 1 252 bytes of UDP payload (C-100).

**The edge contract (same on every platform) `[decision]`.**

- `emit(p) = T(p) + A`; `submit(p) = emit(p) − D`. A is the presentation group's audition offset (C-55); D is the sink's total plumbing delay from hand-off to emission: driver buffer, resampler and converter (C-170). Capture stamps a sample with `T⁻¹(capture_time − D_capture)`, where D_capture is the mirror: from acoustic instant to the sample being available (C-170).
- **E** = 99.9th percentile of |actual emission − (T(p) + A)| over a one-hour run, maximum reported beside it (C-73). A percentile claim, not a bound.
- A presentation group carries a required E; a sink whose E exceeds it is refused at admission with the named reason RequiredEExceeded. With a 1 ms E, a Windows sink will be refused by tight groups; that is the intended behaviour. Only the certification state *commissioned* gives a numeric E; an uncertified or self-monitored sink (not yet measured, or an OS build / device / driver combination outside the reference list) declares E as **unknown** (C-140): it is refused with RequiredEExceeded by any group that sets a required E and admitted to groups that set none (C-103). A commissioned sink whose clock is unlocked stays commissioned: its alignment claim is suspended and does not count towards a group's required E until the clock is locked again, then resumes without re-commissioning (C-180). The lock state and bound come from the servo core through the published time view (epic 008a, C-187).
- **Two-loop timebase, shared core code** (task 018): loop 1 maps host clock H (QPC) to the domain line through sync exchanges; loop 2 maps the device sample counter S to H by regression over (device position, QPC time) pairs; the ratio of the kit's steerable rate actuator (C-189, C-197) is the quotient of the slopes; rate-loop bandwidth well below 1 Hz (C-68).
- **Wall-clock step detection lives in this backend** (C-121): the backend builds each sync exchange from its receive stamps and passes to the servo only exchanges that pass the step check. On Windows both the receive stamp and H are QPC, so the check rejects a stamp that is not in QPC units, that is later than the userspace QPC read taken just after the receive returned, or that runs backwards against the previous stamp on the same socket.

**Windows targets (pending measurement in 022b).** E = **1 ms** (C-72), with a proposed budget of 100 µs clock + 100 µs tracking + 800 µs uncertainty of D (C-150); restart spread of D at most **0.5 ms** over 20 restarts (C-74); typical D **6 to 20 ms** (C-81); cross-sink skew against the Linux reference sink at most **1.05 ms** (C-82). E is dominated by D uncertainty and scheduling spikes, not by the clock.

**Consequences of the engine period.** Exclusive-mode hand-off is quantised to the device period, about 3 to 10 ms. A capture edge therefore receives 6 to 20 slices (48 frames, 0.5 ms each, C-02) per callback and emits them as a batch. Each slice is one packet (C-05) carrying 6 channels of 192 bytes, 1 152 bytes of audio with no per-channel id byte; the channel map is in the manifest (C-04); all headers and the checksum take at most 100 bytes, so the slice fits 1 252 bytes of UDP payload (C-137); about 1 320 bytes on the wire (C-20), 2 000 packets/s and about 21.1 Mbit/s per stream (C-21, C-22). Positions stay exact (each sample is stamped by its capture time), but lateness at consumers includes the period, so the group's A must cover it. A consumer measures arrival lateness for 4 s (C-50) and sets its offset to the observed maximum plus headroom (C-51, C-52), so the batch pattern is absorbed if it is stable. If it is not, the consumer sees late slices: an input with at least one late slice in each of 10 consecutive 1 s windows is persistently late and ratchets its offset by one slice (C-101, C-53); persistently late at the ceiling A, it is Demoted (C-109). As a consumer, the Windows sink drains at most 2 × the admitted packets per slice per input per slice period (C-16, C-156). Its receiver reports carry its current A (C-198); when the group's A changes while it is live, the sink re-enters aligning with its ring and ratchet ceiling resized (C-186), switching at its own position one lead (C-43) ahead, as a fan-in does (C-208), with nobody to announce it to (C-214).

**Platform mapping `[memory]`.**

| Need | Windows mechanism | Check |
|---|---|---|
| Host clock H | `QueryPerformanceCounter` with its published frequency | Unit test |
| Receive timestamp | A Winsock timestamping control for UDP receive, stamped in QPC units, on recent Windows 10 and 11 builds. Gate by a capability bit | Task 001 probe. If absent: userspace read immediately after receive, stamped *userspace*. Of the C-131 source values only *kernel* and *userspace* exist on Windows; each is stamped with a C-129 state |
| Real-time threads | MMCSS "Pro Audio" task. Raises priority; not hard real-time. A granted MMCSS registration is what this epic counts as "real-time scheduling granted" for strict mode (C-127) | Wake-latency measurement in the self-test |
| Memory lock | `VirtualLock` after raising the process working-set minimum | Lock call result read back; stamped present or absent |
| Device clock S | The device clock interface returns a device position with a QPC timestamp | Regression residual |
| D | Reported stream latency is unreliable across drivers: **measurement only** | L2 (022b) |
| Don't-fragment | An IPv4 socket option | Capture on the wire |
| DSCP | Plain `IP_TOS` is typically ignored; marking goes through the QoS flow API and arbitrary values may need administrator rights or policy | Peers read the byte and publish it: media per connection (C-47), sync by the holder and control by every module (C-152). Otherwise stamped *unverified* |
| Per-socket drop counter | None | Stamp *drop accounting: absent* |
| Batch receive | Loop of non-blocking receives with control data | Cost measured |

**The self-test.** The wake-latency self-test is `OsRealtime::measure_wake_latency`, run at startup on the data and clock threads after MMCSS registration. It reports percentiles (50th, 99th, 99.9th, maximum) as a diagnostic in the stamp and has no pass threshold: bounded real-time is *absent* on Windows and deadline overruns are diagnostics only.

**Porting traps (the ones that bite) `[memory]`.**

- **Timer granularity.** The default is 15.6 ms; the best waitable timer is about 0.5 ms. A 0.5 ms slice clock cannot be paced by sleeping. Hence the no-ticker ruling. Sync requests at 50/s (20 ms period, C-36, inside the accepted range of 10 to 200 per second, C-112) are fine with a 1 ms timer. `rt.rs` owns the timer-resolution request.
- **Connection-reset on UDP.** A socket that sent to a dead peer starts failing its receives with a connection-reset error unless that behaviour is switched off with a socket control. Switch it off on every socket.
- **Firewall.** Inbound UDP is blocked until a first-run prompt is accepted; if dismissed, traffic is silently dropped. Detect and report.
- **DSCP** as in the platform mapping above.
- **No per-socket drop counter**: overflow cannot be told from network loss.
- **Default receive buffer is 64 KB**; size it explicitly to the declared rate times the skew budget. The skew budget is capped at the presentation group's A (C-96), so the buffer is the sum of the declared stream rates (about 21.1 Mbit/s per stream, C-21) times A, never below the 64 KB default.
- **Driver interrupt work** from third-party drivers produces millisecond scheduling spikes that user space cannot bound.
- Multicast must be joined and sent on the medianet interface by index; a laptop has Wi-Fi up beside it.
- Power: sleep, modern standby and background power throttling end or degrade a live edge; request to stay awake while live and mark the process as audio work. `power.rs` owns the stay-awake request, the power-throttling opt-out and the suspend/resume handling.

**Sleep is a departure** (C-124). When a laptop edge sleeps, both timebase loops are reset and on resume the edge returns through normal admission (Locking, then Aligning), in the same process. The time to return is therefore the acquisition time to Locked (C-63) plus the aligning duration (4 s, C-50); no separate target applies. A capture edge bumps the session number of each of its streams on that return, as on any restart or timeline epoch change (C-19).

**Strict mode** (C-127 to C-129, C-147, C-165). Strict mode is set by the operator policy in the commissioning configuration (C-165). It requires real-time scheduling granted on the data and clock threads (MMCSS registration on Windows) and memory locked; restricting CPU idle states stays best-effort and is stamped. A capability that is unverified counts as absent, so strict mode with any required capability absent or unverified is refused (C-128). Every capability is stamped in one of four states: present, absent, unverified, not requested (C-129). Strict mode is hosted-only, and every run whose numbers enter the constants table runs in strict mode (C-147), so the 022b certification runs depend on this gate. One failure is fatal outside strict mode too: if the thread layout makes seqlock safety depend on a CPU pin and that pin fails or cannot be verified, the module refuses to start in every mode (C-177, C-201). The default priorities (clock thread above data thread) never depend on pinning (C-177).

**Report-only on Windows.** Overflow accounting; DSCP marking; bounded real-time (deadline overruns are diagnostics only); sub-millisecond pacing (stamped *pacing: batch per engine period*).

**Uncertifiable in v1 regardless of engineering.** Any bound on scheduling latency; a device whose driver misreports latency, without the commissioning measurement; any OS build, driver and device combination outside the short reference list (stamped *unverified*, E unknown); isolation of the laptop (dual-homed, emits discovery chatter); a device that runs at neither 96 kHz nor 48 kHz (C-105).

**Stamp transport and observability.** The regime stamp is a dedicated control message split into sections that each fit the 1 252-byte control bound (C-17); it is multicast to the control group on any change and refreshed every 10 s, a unicast request fetches it immediately, and the hello carries only the stamp version number (C-18). The layout is epic 004's; sending it belongs to the hosted backend (epic 015, C-144); this epic supplies the Windows fields. Each producer and consumer publishes, per connection, the counters received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events and dead (C-159, completing C-47), the DSCP it observed (C-47), and, as a consumer, lateness percentiles 50th, 99th, 99.9th and maximum (C-160) over a rolling 60 s window, with the maximum since the connection went live beside them (C-182). Receiver reports are not multicast and there is no query message (C-47). Peer verification of DSCP: media is verified when this edge is a producer and the consumer's stamp shows the observed value (C-47); sync is verified from the DSCP the holder reports for this follower's sync requests, and control from the DSCP every module reports for this edge's hellos (C-152). This edge, as a module, reports the DSCP it observes on each peer's hellos (C-152). The stamp also carries the commissioning-window state — commissioning may drive test traffic only inside a declared window that every module stamps (C-107), announced by the tool with an expiry time and closed at expiry or on explicit close (C-153); while a window is open the stamp's ordinary production measurements (latency, loss and alignment figures) are unqualified, whereas commissioning results such as D are valid, recorded by the commissioning tool with the window they were taken in (C-178) — the target's cost calibration factor (C-106, recorded by epic 025 per C-176 and loaded in 022b), and control isolation *present*, which holds on a hosted platform by construction of the thread model (C-161).

**Duties that are not this epic's** (C-144). The hosted backend (epic 015) emits the gap slice when a module misses its own deadline, raises the processing-overrun event, owns the housekeeping restart policy (C-139) and its criterion AC-20 (c), and sends the regime stamp message. The panic-injection test hook (C-145) is core. This epic supplies the Windows facts they use (late-callback diagnostic, stamp fields).

**Reference list.** The reference list of certified OS build / device / driver combinations is a table in the certification record and a data file in the crate read by the stamp builder. The stamp builder and its lookup are this epic's; publishing the list is epic 022b.

**Idle live sink.** For criterion 022-AC-6, an idle live sink is a sink admitted to a group and receiving one 6-channel stream whose content is silence, with no other module on the machine; its process CPU is averaged over 10 minutes.

**Trust statement.** "Trusted segment per the recommended configuration; Windows and macOS edges are best-effort members of a weaker segment." Commissioning baseline: dedicated medianet NIC, discovery chatter suppressed, firewall scoped to medianet ports (checklist delivered by task 028).

**Rules that apply unchanged.** Interrupted after 8 consecutive missed slices (C-38), where only slices absent at the deadline (missing or late) count and a marked gap or a slice on another timeline epoch resets the count (C-102); dead after 2 s (C-39) counted from the last slice received with content, a marked gap (C-111) or a slice on another timeline epoch, which also has its own counter (C-135); an absent slice is counted missing at the deadline and reclassified as late if it arrives while still within the ring span (C-108); receiver report per bundle every 100 ms (C-40); producer stops after 2 s without reports (C-41); a sink whose source is absent stays Pending with no timeout (C-45); DSCP sync 48, media and one-way probes 46, control 40 (C-13), and hellos and regime stamps are control messages (C-17, C-18), so discovery sockets use 40; don't-fragment on every packet; path-MTU fallback 1 280 bytes (C-06, C-100); the path-MTU probe pair is sent at the media packet size (C-116); 6 channels per stream (C-04); test-only injection hooks are stamped like any injection (C-120); tests that need a privilege run in a separate privileged nightly job, and "unprivileged" applies to per-change CI (C-146).

## Deliverables

Crate `hadronyche-os-windows`:

- `clock.rs` — `HostClock` on QPC.
- `net.rs` — `OsNet` / `DatagramSocket`: IPv4 only, interface-pinned multicast with TTL 1, connection-reset disabled, receive timestamps (kernel if task 001 confirms and the build supports it, else userspace), don't-fragment, explicit buffers sized to rate × A, QoS-flow DSCP, receive of the TOS byte; per-peer hello DSCP report and sync/control DSCP verification (C-152); sync-exchange building with the wall-clock step check (C-121).
- `rt.rs` — `OsRealtime`: MMCSS registration, timer resolution, clock-above-data priorities, the seqlock pin check (C-177, C-201), memory lock, the wake-latency self-test; capability report with *bounded real-time: absent* and the C-129 states; strict-mode refusal (C-127, C-128, C-165).
- `firewall.rs` — detects a blocking firewall state and reports a named startup failure.
- `power.rs` — stay-awake request and power-throttling opt-out while live; suspend is a departure, resume returns through normal admission (C-124).
- `wasapi/sink.rs` — exclusive-mode event-driven render; position ring → kit resamplers (epic 024a, C-197) → device buffer.
- `wasapi/capture.rs` — exclusive-mode capture through the kit resamplers (024a, C-197); per-callback batch of slices, each stamped from the device line; batch size recorded; session number per stream (C-19).
- `wasapi/device.rs` — `DeviceClock` from (device position, QPC) pairs; exclusive-mode acquisition at 96 kHz or 48 kHz only (C-105) with "interface held, nothing certified" reporting (C-175).
- `deadline.rs` — the core deadline-clock trait implemented from the audio callback plus QPC regression.
- The `windows-std` regime stamp builder and its reference-list lookup.

```rust
impl HostClock for QpcClock { /* now_ns, declared_jitter_ns */ }
impl OsNet for WinNet { /* open_media, open_discovery, iface_stats */ }
impl OsRealtime for WinRealtime { /* setup_process, setup_thread, measure_wake_latency */ }
impl DeviceClock for WasapiDevice { /* nominal_rate, declared_jitter_ns */ }
impl RawCapture for WasapiRawCapture { /* open, read */ }   // for the L2 harness
```

CI: a per-change unprivileged Windows job and a privileged nightly Windows job (C-146).

Outside the crate (C-133): the Windows build of the L2 harness binary lives in the `hadronyche-tools` crate; throwaway probes (for example the firewall-detection probe) live in `probes/`.

## Dependencies

- **Needs:** the stamp-probe result for Windows (001); the regime-stamp message layout and the peer stamp fields of C-47, C-152, C-159 and C-160 (004); the departure behaviour of a sleeping edge (005); the hosted trait surface and the stamp sender (015, C-144; 016); the clock lock state and bound through the published time view (008a, C-187); the shared two-loop core and edge structure (018); the fixed 2:1 / 1:2 resampler node, the steerable rate actuator and its trait (024a, C-197, C-210); the L2 harness trait `RawCapture` (020), whose binaries live in `hadronyche-tools` (C-133).
- **Provides:** to epic 022b, a complete Windows edge (sink, capture, stamp, strict mode) and `WasapiRawCapture` for the L2 harness.

## Acceptance criteria

1. **022-AC-3. Exclusive mode.** If exclusive mode cannot be obtained because another process has the device open (C-175), the edge reports "interface held" and its stamp says *uncertified*; it never silently falls back to shared mode. A device that supports neither 96 kHz nor 48 kHz is refused (C-105).
2. **022-AC-4. Connection-reset.** After the peer process is killed, the edge's sockets keep receiving from other peers with no error.
3. **022-AC-5. Firewall.** With inbound UDP blocked, startup fails with a named reason within 5 s (not a silent hang).
4. **022-AC-6. No ticker.** No thread spins to pace sends; CPU use of an idle live sink (as defined in Background) stays below one tenth of a core.
5. **022-AC-7. Edge only.** The hello never carries the holder-eligible bit.
6. **022-AC-8. Stamp (AC-31).** Profile `windows-std`, OS build, audio device, driver, receive-timestamp source (*kernel* or *userspace*, C-131, each with its C-129 state: for example *kernel: unverified* until task 001 confirms the control on that build), drop accounting *absent*, DSCP per class in the C-129 states (*present* when peer-verified per C-47 and C-152, *unverified*, *absent* when marking was refused), pacing *batch per engine period* with the batch size, bounded real-time *absent*, real-time scheduling, memory lock and CPU-idle restriction in the C-129 states, strict mode, wake-latency self-test result, D (*measured*) and E (numeric only when *commissioned*, otherwise *unknown*, C-140, C-103), certification state (uncertified, self-monitored or commissioned, C-140), cost calibration factor (C-106), commissioning-window state, with production figures marked unqualified while a window is open (C-107, C-153, C-178), the per-connection counters of C-159 with the observed DSCP (C-47), lateness percentiles over a rolling 60 s window with the maximum since live beside them (C-160, C-182), the DSCP observed on each peer's hellos (C-152), and control isolation *present* (C-161). The stamp is published per C-18 and its version changes whenever a field changes.
7. **022-AC-11. Sleep (C-124).** After a forced suspend and resume, both timebase loops have been reset, the edge returns through Locking and Aligning without a process restart, and each capture stream carries a new session number (C-19).
8. **022-AC-12. Unknown E (C-103, C-140).** A Windows sink whose certification state is not *commissioned* declares E unknown; a group that sets a required E refuses it with RequiredEExceeded, and a group that sets none admits it. A *commissioned* sink whose clock becomes unlocked stays *commissioned*, with its alignment claim suspended until relock (C-180).
9. **022-AC-13. Strict mode (C-127, C-128, C-165).** With strict mode requested by the operator policy, the edge starts only if MMCSS registration is granted on the data and clock threads and memory is locked; otherwise startup is refused with a named reason; an unverified capability is treated as absent. Independently of strict mode, a failed or unverified pin on which seqlock safety depends refuses startup (C-177, C-201).
10. **022-AC-14. Step check (C-121).** An exchange whose receive stamp fails the step check never reaches the servo and is counted.
11. **022-AC-16. Peer DSCP verification (C-152).** The edge's stamp reports the DSCP observed on each peer's hellos; its sync class is *present* exactly when the holder's stamp shows 48 for this edge, and its control class exactly when peers' stamps show 40 for its hellos.

022-AC-1, 022-AC-2, 022-AC-9, 022-AC-10 and 022-AC-15 are delivered by epic 022b.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Every `[memory]` fact in this file is either confirmed (`[measured]`) or corrected
- [ ] A short reference list of certified OS build / device / driver combinations is published; everything else stamps *unverified*
- [ ] CI builds the crate on Windows; hardware tests are a documented manual job

**Not applicable in this part** (C-125): "The frozen POC is not modified" — this part writes only the new crate and has no subject in the POC tree. Every other line has a subject here: the parser is the receive control-message parser in `net.rs` (timestamp and TOS byte), fuzzed to the C-95 budget (60 s on every change, 1 hour nightly, corpus kept); "unprivileged" applies to per-change CI, privileged tests run in the nightly privileged job (C-146); the `[memory]` facts are measured in this part's tickets and the text is rewritten in 022b-08; the reference-list line is met here by the stamp lookup that makes every non-listed combination stamp *unverified*, and the list itself is published in 022b-07; the manual hardware job document is 022b-01, the Windows CI build is 022a-01.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Is the UDP receive-timestamping control present, and on which builds? | Task 001 (current and oldest obtainable release, C-138) | Absent: clock bound rises; revisit the 1 ms target only if the measured clock share exceeds the 100 µs allotment of C-150 |
| Can DSCP be set without administrator rights on a standalone laptop? | Epic 022b (022b-06), feeding task 028 | No: marking stays *unverified* and the checklist documents the policy route |
| Does the firewall query alone detect a dismissed first-run prompt, or is the active probe needed? | This part (measurement in ticket 022a-08) | The query reports "allowed" while the probe datagram is dropped: the probe becomes mandatory |
| Exact trait signatures and the name of the core deadline-clock trait (the rate-actuator trait is settled as epic 024a's, C-210) | Tasks 015, 016, 018, 020 | Signatures cannot express a Windows fact (for example QPC-unit receive stamps): the trait surface is changed there first |
| How the medianet interface index is configured and chosen | Task 016 (hosted configuration surface) | An edge picks the Wi-Fi interface on a dual-homed laptop |
| Who measures D and D_capture in the field and how; this epic reads the commissioned D from an edge configuration key | Task 028 (inside a commissioning window, C-107, C-153; the result is a valid commissioning result recorded with its window, C-178) | A commissioned D differs from the L2-measured D by more than the restart spread |
| Format of the batch-size stamp field (single number or minimum / typical / maximum) | Task 004 (stamp message layout, C-18) | A consumer's commissioning of A cannot be explained from the stamped field |
| Does a sleeping edge announce its departure, so that peers see an end rather than dead after 2 s? | Task 005 (control state machines) | Peers of a suspended edge wait out the dead threshold (C-39) |
| A driver-level low-latency backend (ASIO-class) as a later option | Deferred past v1 | Product requires a Windows E well below 1 ms |
