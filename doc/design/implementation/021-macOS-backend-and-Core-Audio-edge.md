# 021 — macOS backend and Core Audio edge

**Phase:** P4 (second half; does not block P5) · **Scope:** L (31 tickets, about 37 days) · **Depends on:** 001, 015, 016, 018, 020, 024a, 025b (cost calibration factor only, the last ticket; see the roadmap) · **Blocks:** 028, 027

## Goal

Make a macOS machine a certified **edge** member of a medianet: a sink and a physical-input capture module on Core Audio, with a macOS implementation of the hosted backend traits (sockets, host clock, real-time threads). The macOS edge never holds the clock and never stands for election.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules exchange *slices* addressed by *position* on a domain-wide timeline: `position = (network_time − epoch_origin) × rate`; write `T(p)` for the network time of position p. Linux hosts are the backbone (clock holder, elections, conformance, commissioning). macOS and Windows machines join as **edges** — typically a musician's laptop. Embedded devices are followers.

**Evidence warning.** Every macOS platform fact below is `[memory]` — recalled, not verified — until task 001 (receive-stamp probes) and this task's L2 run confirm it. Treat each as a hypothesis with a check attached.

**Scope rulings `[decision]`.**

- **Edge only.** The holder-eligible capability bit is never set. The build uses the hosted crates' edge-only feature (no holder candidacy). In a medianet with no holder-eligible candidate, no holder can be chosen (takeover under C-213 and election under C-113 both consider only holder-eligible modules), so a medianet of only macOS edges reports "no holder-capable node".
- **Capture is physical device inputs only.** Capturing an application's output (a virtual audio device) is out of v1.
- **Direct devices only.** Aggregate devices are excluded: their drift correction makes D drift, so they cannot be certified.
- If the musician's own software holds the interface so that the edge cannot use it as required, the edge **reports it and certifies nothing**. "The interface is held" means the audio device cannot be opened in the required mode because another process has it open (C-175).
- The media deadline clock exists in core only as an abstract trait; this backend provides it from mach host time.
- **One rate on the wire.** The domain sample rate is 96 kHz and it is the only rate on the wire (C-01). Laptop devices often run at 48 kHz; such an edge converts 2:1 through the kit's resampler node, and the wire stays at 96 kHz (C-80). A resampler node may appear only inside an edge module; the compiler rejects it anywhere else, and an edge's conversion delay is part of its plumbing delay D (C-134). That delay is counted only in D and never in L: L is the module's structural delay in the domain-rate timeline, and the manifest reports the conversion delay separately, on the device-facing port (C-207). The resamplers are the arithmetic kit's (epic 024a), which builds both the fixed 2:1 / 1:2 node and the steerable rate actuator; this epic uses them and builds none of its own (C-197). The two are distinct (C-189): the fixed node is instantiated only when the device is not at the domain rate, while the steerable rate actuator, whose ratio varies by parts per million around its nominal value to track the device crystal, runs on every edge whatever the device rate, and its delay is also part of D. The rate-actuator trait is defined by the kit together with its implementation, with a ratio range of ±1 000 ppm around the nominal ratio (C-210). The node's delay is always a whole number of domain-rate samples; the node is offered only for the 2:1 and 1:2 ratios, and other ratios are out of v1 (C-105). A device that cannot run at the domain rate or at a 2:1 or 1:2 ratio to it is therefore refused.
- **Stream format.** Slices of 48 frames, 2 000 per second (C-02); 4-byte samples, f32 or Q1.31 (C-03); at most 6 channels per stream at defaults, 192 bytes per channel per slice and 1 152 bytes of audio, with no per-channel id byte: the channel map is in the manifest (C-04). A default packet is about 1 320 bytes on the wire (C-20); a 6-channel stream is about 21.1 Mbit/s (C-21).
- **IPv4 only.** IP is IPv4 only in v1 and multicast is sent with TTL 1 (C-100).

**The edge contract (same on every platform) `[decision]`.**

- `emit(p) = T(p) + A`; `submit(p) = emit(p) − D`. A is the presentation group's audition offset (C-55); D is the declared plumbing delay from hand-off to emission (driver buffer + resampler + converter); for capture, the mirror, from acoustic instant to the sample being available (C-170). Capture stamps a sample with `T⁻¹(capture_time − D_capture)`.
- **E** = 99.9th percentile of |actual emission − (T(p) + A)| over a one-hour run, maximum reported beside it (C-73). A percentile claim, not a bound.
- A presentation group carries a required E; a sink whose E exceeds it is refused at admission with a named reason. Only the certification state *commissioned* gives a numeric E; an *uncertified* or *self-monitored* sink declares E as **unknown** (C-140); it is refused with reason `RequiredEExceeded` from any presentation group that sets a required E and admitted to groups that set none (C-103). The refusal is core admission; this epic only declares the sink's E.
- The sink's alignment ring is sized to the presentation group's A (C-96). When A changes while the sink is live, it re-aligns (C-186) and switches at its own position one lead (C-43) ahead, as a fan-in does (C-208), announcing nothing since it has nobody to announce it to (C-214).
- **Two-loop timebase, shared core code** (built in task 018): loop 1 maps the host clock H to the domain line through sync exchanges; loop 2 maps the device sample counter S to H by regression over (sample count, host time) pairs; the ratio of the kit's steerable rate actuator (C-197) is the quotient of the slopes; rate-loop bandwidth well below 1 Hz (C-68). A platform supplies only two traits — a host clock and a device clock — each with a declared jitter.
- **Step detection lives in the backend.** The platform backend that builds each sync exchange from receive stamps is the home of wall-clock step detection; the servo only ever receives exchanges that passed it (C-121). This backend therefore builds exchanges and filters them.

**macOS targets (pending measurement).** E = **100 µs** for built-in or bus-attached devices (C-71); restart spread of D at most **50 µs** over 20 restarts (C-74); typical D a few ms (C-81); L2 cross-sink skew against the Linux reference sink at most **150 µs**, the sum of the two E targets (C-82). The proposed split of the macOS E budget is 40 µs clock + 40 µs tracking + 20 µs uncertainty of D, pending L2 (C-150). L2 pass criteria use the platform's target E, never the E measured in the same run (C-149). For every ladder run the harness computes A with the commissioning rule (largest over sinks of maximum lateness + D + headroom) and records it with the results (C-148); the L2 test parameters of C-166 apply. Any run whose numbers enter the constants table runs in strict mode (C-147). The reference hardware is proposed by the implementing team after P0, approved by the product owner and recorded in the constants table (C-172); the supported OS versions are set after task 001 reports, which tests the current and the oldest obtainable release (C-138). The short reference list is the approved reference hardware on the supported OS versions that passes L2.

**Platform mapping `[memory]`.**

| Need | macOS mechanism | Check |
|---|---|---|
| Host clock H | mach absolute time (convert with the timebase ratio; ticks are not nanoseconds on Apple Silicon). Stops during sleep | Unit test against a second clock |
| Receive timestamp | Kernel stamps on a monotonic basis via a socket option (`SO_TIMESTAMP_MONOTONIC`); would be cleaner than Linux's wall-clock basis | Task 001 probe. If absent: userspace read immediately after receive, stamped *userspace* |
| Real-time threads | Time-constraint thread policy, available unprivileged; join the audio device's workgroup so the scheduler coordinates with the I/O thread | Wake-latency measurement in the self-test |
| Device clock S | The device I/O callback supplies sample time and host time for input and output on every cycle, plus a rate scalar | Regression residual |
| D | Device latency + stream latency + safety offset + buffer size properties, plus the resampler node's delay (whole domain-rate samples, C-105) when the device is not at 96 kHz, plus the rate actuator's delay (C-189), both counted only in D and never in L (C-134, C-207), plus a commissioning measurement | L2 |
| Don't-fragment | A dedicated IPv4 socket option, not Linux's path-MTU-discovery option | Capture on the wire |
| DSCP | `IP_TOS` on the socket; verified by the receiving peer | Receiver report |
| Per-socket drop counter | None | Stamp *drop accounting: absent* |

**Porting traps (the ones that bite) `[memory]`.**

- A laptop has Wi-Fi up beside the medianet port. **Multicast follows the default route unless every socket is pinned to the medianet interface.** Pin every socket (join and send) by interface.
- Recent macOS releases gate local-network access behind a **per-app permission prompt**; until granted, traffic is silently blocked.
- The machine **sleeps on lid close**. A live edge must hold a power assertion. Mach absolute time stops during sleep.
- **Background throttling** applies unless the process is marked as audio work.
- There is **no per-socket drop counter**: socket overflow cannot be told from network loss. Loss classification is weaker and the stamp says so.
- Other applications can change the device's sample rate under the edge.
- Bluetooth and network (AirPlay-class) outputs have large variable latency and are excluded.

**Sleep is a departure `[decision]`.** Sleep is handled as a departure: both timebase loops are reset and the edge returns through normal admission, Locking then Aligning (C-124). No loop state is carried across sleep, so the stopped mach clock needs no special handling beyond the reset. Peers see the edge interrupted after 8 consecutive missed slices (C-38, counting only slices absent at the deadline, C-102) and dead 2 s after the last slice received with content or a marked gap (C-39, C-111). The time back to Locked is bounded by the acquisition time C-63 (at most 10 s from the first exchange, pending simulation), followed by the Aligning duration C-50 (4 s). A capture edge is a source: on return it starts a new session number for each stream (C-19). A *commissioned* sink stays commissioned while its clock is unlocked: its alignment claim is suspended and does not count towards a group's required E until the clock is locked again, then resumes without re-commissioning (C-180).

**Regime stamp transport `[decision]`.** The stamp is a dedicated control message split into sections that each fit the control message bound of 1 252 bytes (C-17); it is multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch; the hello carries only a stamp version number (C-18). The hosted backend (epic 015) sends the stamp message that epic 013 defines (C-144); this backend supplies the macOS fields. Producer and consumer each publish per-connection counters in their own stamp — received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-47, C-159) — and the observed DSCP; receiver reports are not multicast. Lateness is published as the 50th, 99th and 99.9th percentiles and the maximum (C-160), over a rolling 60 s window with the maximum since the connection went live beside them (C-182). Every module reports the DSCP observed on each peer's hellos, and the holder reports the DSCP observed on each follower's sync requests (C-152). The stamp also records a declared commissioning window while it is open (C-107); the tool announces the window on the control group with an expiry time, it closes at expiry or on explicit close (C-153). While a window is open, ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified; the commissioning tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results, each recorded with the window it was taken in (C-178). It carries the target's cost calibration factor (C-106) and the certification state: *uncertified*, *self-monitored* or *commissioned*, with a numeric E only when commissioned (C-140).

**Cost calibration `[decision]`.** The compiler's cost bound counts abstract operations per slice; each target has a calibration factor (time per operation), measured once and recorded in the regime stamp, and admission converts with it (C-106). The macOS edge runs the kit's resampler node, so a macOS target needs its factor. The procedure is fixed: the DSL conformance suite (epic 025) runs a reference workload once per listed target and records time per operation; the value is carried in the regime stamp (C-176). macOS must be a listed target.

**Strict mode and capability states `[decision]`.** Strict mode requires real-time scheduling granted on the data and clock threads, and memory locked; restricting CPU idle states stays best-effort and is stamped (C-127). In strict mode an *unverified* capability is refused: unverified is treated as absent (C-128). Every capability in the regime stamp has one of four states: present, absent, unverified, not requested, where "not requested" marks a deliberate configuration choice as distinct from a denial (C-129). The receive-timestamp source takes one of the values kernel, userspace, mac-hardware or interrupt, each with one of those states (C-131); a macOS edge can only report *kernel* or *userspace*. The control-isolation capability (housekeeping cannot delay the data path) is *present* on hosted platforms by construction of the thread model (C-161). Strict mode is set by the operator policy in the commissioning configuration (C-165); it is hosted-only, and every run whose numbers enter the constants table as design constants runs in it (C-147). That macOS can lock memory only per region and gives no user control of CPU idle states is `[memory]`.

**Where outputs live `[decision]`.** Reports and results, including `l2-results-macos.md` and every `[measured]` note this epic produces, go in `doc/design/reports/`; specifications in `doc/design/spec/`; the constants table is `doc/design/02-constants.md` (C-130). Throwaway probes go in `probes/`; harness and tool binaries in the `hadronyche-tools` crate (C-133). A fact cited from a named, published standard, with its clause, is tagged `[standard]` and replaces `[memory]` once the citation is checked (C-132).

**Report-only on macOS.** Overflow accounting. (Real-time scheduling, DSCP via `IP_TOS`, fault injection and self-test are expected to be available.)

**Uncertifiable in v1 regardless of engineering.** A device whose driver misreports latency, without a commissioning measurement; aggregate devices; Bluetooth and network outputs; any OS, driver and device combination outside the short reference list (stamped *unverified*); **isolation of the laptop** — it is dual-homed to the internet and emits discovery chatter on the medianet port.

**Trust statement.** "Trusted segment per the recommended configuration; Windows and macOS edges are best-effort members of a weaker segment." Commissioning baseline: a dedicated medianet NIC, OS discovery chatter suppressed, firewall scoped to medianet ports (checklist delivered by task 028). The commissioning tool drives test traffic only inside a declared commissioning window that every module stamps (C-107).

**Liveness and control rules that apply unchanged.** Interrupted after 8 consecutive missed slices (C-38, misses as defined by C-102); dead after 2 s counted from the last slice with content or a marked gap (C-39, C-111); receiver report per bundle every 100 ms (C-40); producer stops after 2 s without reports (C-41); 50 sync requests/s (C-36), inside the accepted range of 10 to 200 per second (C-112); DSCP sync 48, media and one-way probes 46, control 40 (C-13); don't-fragment on every packet; path-MTU fallback 1 280 bytes, giving 1 252 bytes of UDP payload (C-06, C-100); the path-MTU probe pair is sent at the intended media packet size (C-116). Fuzz budget: each parser 60 s on every change and 1 hour nightly, corpus kept (C-95).

## Deliverables

Crate `hadronyche-os-macos`:

- `clock.rs` — `HostClock` on mach absolute time.
- `net.rs` — `OsNet` / `DatagramSocket` implementation: IPv4 only, interface-pinned multicast with TTL 1, receive timestamps (kernel if task 001 confirms, else userspace), don't-fragment, `IP_TOS`, receive of the TOS byte; building of sync exchanges from receive stamps with step detection (C-121).
- `rt.rs` — `OsRealtime` implementation: time-constraint policy, workgroup join, memory locking; capability report in the four states of C-129; strict-mode refusal (C-127, C-128); refusal to start, in every mode, of a thread layout whose seqlock safety depends on a CPU pin that failed or is unverified (C-177, C-201), default priorities (clock above data) never depending on pinning.
- `power.rs` — power assertion held while any connection is live; sleep/wake notification → departure, loop reset and normal admission (C-124).
- `coreaudio/sink.rs`, `coreaudio/capture.rs` — the edge modules on the device I/O callback, converting through the kit's resampler node when the device is not at 96 kHz and always through its steerable rate actuator (epic 024a, C-189, C-197, C-210); the module manifest reports the conversion delay on the device-facing port, outside L (C-207); `DeviceClock` from callback timestamps; D from device properties.
- `coreaudio/guard.rs` — refuses aggregate, Bluetooth and network devices, and devices that cannot run at the domain rate or a 2:1 or 1:2 ratio to it (C-105); detects sample-rate changes and loss of the device; reports "interface held" (C-175) with certification state *uncertified*, so nothing is certified (C-140).
- `permissions.rs` — detects a missing local-network permission and reports it as a named startup failure.
- The macOS fields of the regime stamp `macos-std`, including the cost calibration factor (C-106).

The traits implemented are those defined by tasks 015, 016, 018 and 020:

```rust
impl HostClock for MachClock { /* now_ns, declared_jitter_ns */ }
impl OsNet for MacNet { /* open_media, open_discovery, iface_stats */ }
impl OsRealtime for MacRealtime { /* setup_process, setup_thread, measure_wake_latency */ }
impl DeviceClock for CoreAudioDevice { /* nominal_rate, declared_jitter_ns */ }
impl RawCapture for CoreAudioRawCapture { /* open, read */ }   // for the L2 harness
```

Document `doc/design/reports/l2-results-macos.md`: the certification record (C-130).

## Dependencies

- **Needs:** the stamp-probe result for macOS (001); the hosted trait surface and the sending of the regime stamp message (015, 016, C-144); the shared two-loop core and edge module structure (018); the kit's resampler node and steerable rate actuator (024a, C-197); the L2 harness (020); the calibration run of the DSL conformance suite (025, C-176).
- **Provides:** a certified platform profile and its per-OS checklist inputs to the commissioning tool (028).

## Acceptance criteria

1. **021-AC1 (AC-27).** E at most 100 µs (C-71; 99.9th percentile over one hour, maximum reported, C-73); restart spread of D at most 50 µs over 20 restarts (C-74); cross-sink skew against the Linux reference sink at most 150 µs (C-82).
2. **021-AC2 (L2 common criteria).** On the reference device, in strict mode (C-147): marker offset constant within the macOS restart spread of 50 µs (C-75); zero accumulated slip, offset returns to its prior value within 50 µs (C-76), after injected loss (1 % random plus bursts of 8 slices, C-166, the bursts spaced 1 s apart on the media path only, C-191), a 50 ppm handover (C-78; the measured natural difference between the two holders' crystals plus an injected drift, both recorded in the run report, C-223) triggered as a holder loss 60 s into the run, with emission error within the 100 µs target E during the holdover between holders (an L2 assertion only, C-220) and frequency continuity at takeover (rate change within C-61, no phase step), a run without observed holdover failing, on a rig with a second holder-eligible Linux node (C-191, C-202, C-212), the timeline epoch kept across the takeover (C-213), and an announced step of 5 ms (C-191), each judged over a 10 s comparison window starting when the injected event ends (C-191); MTIE at most 10 µs up to 1 s and at most the 100 µs target E overall (C-77, C-149); the 100 µs target E held throughout the drift ramp of ±100 ppm at no more than 0.5 ppm/s (C-119, C-149).
3. **021-AC3 (interface pinning).** With Wi-Fi up and default-routed, every multicast and unicast packet leaves by the medianet interface (verified by capture on both interfaces); every packet is IPv4, multicast with TTL 1 (C-100).
4. **021-AC4 (sleep).** Lid close with a live stream: peers classify the edge interrupted then dead (C-38, C-39, C-102, C-111); on wake the edge resets both loops and re-admits through Locking and Aligning without process restart (C-124). With the power assertion held, idle sleep does not occur.
5. **021-AC5 (guards).** Selecting an aggregate, Bluetooth or network device is refused with a named reason. A sample-rate change by another application ends the stream cleanly with a named event.
6. **021-AC6 (edge only).** The hello never carries the holder-eligible bit; a medianet of only macOS edges reports "no holder-capable node".
7. **021-AC7 (stamp, AC-31).** Profile `macos-std`, OS build, audio device, driver, receive-timestamp source (*kernel* or *userspace*, C-131), every capability in one of the four states of C-129 (real-time scheduling, memory lock, CPU idle-state restriction among them), drop accounting *absent*, DSCP *peer-verified* or *unverified*, D and its method, certification state (*uncertified*, *self-monitored* or *commissioned*) with E numeric only when *commissioned* and *unknown* otherwise (C-103, C-140), cost calibration factor (C-106, C-176), commissioning window when open (C-107, C-153), per-connection counters and observed DSCP (C-47, C-159), DSCP observed on each peer's hellos (C-152); every section fits 1 252 bytes (C-17, C-18).
8. **021-AC8 (capture scope).** Only physical inputs are offered; no virtual device is installed.
9. **021-AC9 (evidence).** Task 021-25 replaces the receive-timestamp row of the platform mapping in this epic with a `[measured]` statement citing task 001.
10. **021-AC10 (strict mode).** In strict mode the edge starts only with real-time scheduling on its data and clock threads and memory locked both *present*; an *absent* or *unverified* one refuses startup with a named failure (C-127, C-128).

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Every `[memory]` fact in this file is either confirmed (`[measured]`) or corrected
- [ ] A short reference list of certified OS build / device combinations is published; everything else stamps *unverified*
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job

**Not applicable in this epic (C-125).** "The frozen POC is not modified (task 003 is the only exception)": this epic creates a new crate and has no work on the POC, so there is nothing to modify or check beyond the crate living outside the POC tree.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Are kernel receive stamps on a monotonic basis available for UDP? | Task 001 | Absent: clock bound and the 100 µs target rise |
| Is E = 100 µs reachable, and for which device classes (built-in, bus-attached, USB class)? | This task (L2 macOS) | Measured E above target on the reference device |
| Do device-reported latency properties match the commissioning measurement? | This task (L2 macOS) | Mismatch larger than 20 µs, the D share of the macOS E budget (C-150): D must be measurement-only, as on Windows |
| Does the permission prompt appear for a command-line binary, how is it pre-granted in commissioning, and does a self-addressed multicast probe detect the block? | This task, feeding task 028 | No supported way to pre-grant; or a blocked edge passes the probe |
| What statistic does the L2 cross-sink skew limit (C-82) apply to: the 99.9th percentile with maximum beside it, as C-79 defines for L3, or the maximum? | Task 020 (L2 harness) | — (a definition) |
| What wake-latency value, if any, fails the self-test? | This task (L2 macOS), entered in the constants table by the design owner | E misses target and the excess correlates with wake latency |
