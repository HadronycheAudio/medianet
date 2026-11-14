# 026a — Embedded HAL and host integration

**Phase:** P6 · **Scope:** L (13 tickets, about 18 days) · **Depends on:** 005, 008, 009, 010, 011, 012, 013, 014, 018, 024a · **Blocks:** 026b, 027

## Goal

Bring the runtime to a `no_std` microcontroller, first part: define the hardware abstraction (`hadronyche-embedded`) and integrate it with `hadronyche-core` so that the device side of a **reference device** — a narrow sink that joins a medianet as a follower — is complete and verified on a host, with mock hardware. The device runs the same `hadronyche-core` state machines and codecs as a hosted machine; it implements only transport, time and audio hardware. This part delivers the traits and their host mocks, identity derivation, the embedded profile declaration, the cooperative poll loop with follower discovery, the follower servo and holdover, admission against declared memory, the media receive path with receiver reports, the software-ratio rate actuator, the audio edge's two-loop timebase, sectioned stamp publication, the hosted stamp collector, and the commissioning-window state in the stamp. Everything here is testable on a host.

Epic 026 was split under C-143 (an epic above 40 days by ticket tally is split into lettered parts; a reference to epic 026 means both parts). Part 026b builds the reference device firmware on a named board, runs the on-device measurements and acceptance tests, adds the remaining stamp fields, and writes the porting guide. A narrow capture source on the same hardware is possible later behind the same traits, but it is not a deliverable of either part.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. IP is **IPv4 only** in v1, multicast with TTL 1 (C-100). Modules exchange *slices* — all content of one stream for one stretch of media time — addressed by *position* on a domain-wide timeline: `position = (network_time − epoch_origin) × rate`; write `T(p)` for the network time of position p. The rate is the domain rate, 96 kHz, and it is the only rate on the wire (C-01). Linux hosts are the backbone (clock holder, elections, conformance, commissioning). macOS and Windows machines are edges. **Embedded devices are followers**: narrow sinks that never hold the clock in v1.

**The one implementation rule `[decision]`.** Control logic lives in `hadronyche-core` as pure state machines — discovery map, election, admission handshake, connection states: message and time in, messages and events out. A device **does not reimplement** hello, election or sync logic. It implements a **transport trait** (send and receive datagrams, join the discovery group, read time) and drives core. This is what makes one conformance suite (task 007) cover both profiles; the device runs it over real Ethernet in 026b.

Consequences already built into core:

- **Fixed-capacity peer map: 63 entries** (C-217): a medianet has at most 64 active modules, and commissioning refuses more (C-34); a module holds no entry for itself.
- **Control messages are binary and bounded:** at most 1 252 bytes of UDP payload (C-17), which is what the 1 280-byte IPv4 fallback path MTU leaves (C-06, C-100). The path-MTU probe pair is exempt: it is sent at the intended media packet size (C-116).
- Control messages carry the authentication reservation with scheme 0 and a zero tag; the 64-bit message counter is live because it is the liveness counter (C-115).
- The regime stamp is a **wire object**: a device cannot write files, so it **publishes its stamp over the network** and a hosted collector records it.

**Regime stamp transport (C-18).** The stamp travels in a dedicated control message split into sections that each fit C-17. It is multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch. The hello carries only a stamp version number. Capability fields take one of four states: present, absent, unverified, not requested (C-129). This part builds the publication and the collector, the embedded profile fields, and the commissioning-window state; 026b adds the per-connection counters (C-47, C-159), the lateness percentiles and reclaimable latency (C-160, C-157, over the rolling 60 s window of C-182), the DSCP observed on each peer's hellos (C-152), the calibration factor (C-106, C-176) and the declared restart spread (C-191). The device's stamp carries, among others:

- the receive-timestamp source, with one of the C-131 values (on a device: mac-hardware, interrupt, or userspace for a clock read after receive in the poll loop), in one of the four capability states;
- real-time scheduling and memory lock stamped present by construction (C-147), and control isolation absent or unverified unless bounded control handling is shown (C-161);
- the commissioning-window state: commissioning tools may drive test traffic only inside a declared window that every module stamps (C-107). The tool announces the window on the control group with an expiry time; modules stamp it while open; it closes at expiry or on explicit close (C-153). While a window is open, ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified; the commissioning tool's own measurements (D, restart spread, certification) are valid commissioning results, each recorded with the window it was taken in (C-178).

**Role `[decision]`.**

- **Follower only.** Holder eligibility is a capability bit in the hello; no v1 device sets it. Reasons: serving load (a holder answers 50 requests/s from each of up to 64 followers, C-35, C-36), availability (devices get power-cycled), oscillator quality. A device with MAC-level hardware timestamping can stamp *better* than a Linux host; "no kernel timestamps" is not the reason.
- After a holder loss, a holder-eligible module whose time is usable (Locked, or in holdover with its alignment claim not lapsed, C-65, C-169) takes over with frequency continuity and the timeline epoch is kept (C-213); a Locked candidate is preferred over one in holdover, and among equals the lowest module id wins (C-221). Only when no candidate has usable time does the holder-eligible candidate with the lowest module id win and bump the timeline epoch (C-113). A device is never a candidate, whatever its id. Seen from the device, a takeover with usable time is a change of holder with the epoch kept: its servo follows the new holder and its received streams are not restarted.
- Consequence to document for operators: a medianet needs at least one holder-capable (Linux) node, or there is no audio.

**Identity.** Module id 64 bits, instance nonce 64 bits, domain id 16 bits with 0 = unconfigured (C-11). A hello with domain 0 is ignored and counted by peers, not refused (C-173), so a device joins only once a non-zero domain id is stored on it. **Devices without a true random source must not draw the id from a pseudo-random generator** (forty devices would boot with the same id and the same nonce): mix a **hardware unique identifier with a persistent boot counter**. The module id is stable across reboots and the instance nonce changes on every boot: the nonce is what distinguishes successive instances of one module id, and third parties quarantine an id seen with two nonces until only one nonce has been seen for one hello-expiry period, 3 s (C-11, C-46, C-31). The id therefore derives from the hardware unique identifier alone; the nonce from the unique identifier and the boot counter (or a true random source when present, still advancing the counter). The exact mixing function is an implementation detail documented in the crate.

**Clock.** The device runs the core follower servo against the holder at 50 sync requests/s (C-36), inside the accepted range of 10 to 200 per second (C-112). Requirements it inherits: published time continuous while locked; correction term at most 10 ppm from the holder's rate (C-60) and its rate of change at most 1 ppm/s (C-61) — these bound the servo's correction term, not the drift estimate it tracks (C-119); lock bound at most 25 µs (C-62, pending simulation); acquisition to Locked within 10 s (C-63, pending simulation); holdover bound grows 1 µs/s (C-64). Receive timestamps come from MAC hardware timestamping if the part has it, else from the receive interrupt; the source is a capability in the stamp (C-131). Wall-clock step detection lives in the platform backend that builds each exchange from receive stamps; on a device that backend is in `hadronyche-embedded`, and the servo only receives exchanges that passed it (C-121). The device enters holdover after the holdover entry delay, which is pending simulation (epic 008b proposes it) and expected at or below the 1.5 s election silence threshold, so the device is already in holdover with a growing bound when the election on 1.5 s of reply silence starts (C-195, C-37). The clock lock state and bound are published by the servo core (epic 008a) through the published time view (C-187); the device reads them and computes none of its own.

**Holdover margin.** The alignment claim lapses when the holdover bound exceeds the module's E; from the lock bound this takes (E − C-62) / C-64, about 25 s for the Linux reference sink's 50 µs (C-65, C-70). For the device the margin is its own declared E (C-73). A device that declares E unknown takes as its fixed reference twice the lock bound (2 × C-62, that is 50 µs) and stops claiming alignment when the bound it currently publishes exceeds that value, about 25 s into holdover at the growth of C-64 (C-169). When the claim lapses the clock state is Unlocked with time unusable: one-way figures and alignment claims stop (C-162). A commissioned device stays commissioned while its clock is unlocked: its declared E is kept, its alignment claim is suspended and does not count towards a group's required E until the clock is locked again, then resumes without re-commissioning (C-180).

**Audio edge.** Same contract as every edge: `emit(p) = T(p) + A`, `submit(p) = emit(p) − D`, where A is the presentation group's audition offset (C-55) and D the device's declared plumbing delay. Two-loop timebase from core (built in task 018): loop 1 device clock → domain line; loop 2 device sample counter → device clock, by regression over (sample count, timer-captured time) pairs — on a device, a timer capture of the audio DMA position.

- **v1 device clock is 48 kHz.** The wire stays at 96 kHz; the edge converts 2:1 through the kit's resampler node (C-80, C-01); 2:1 is one of the two ratios for which the node is offered, because its delay is a whole number of samples (C-105). A resampler node may appear only inside an edge module, and the edge's conversion delay is part of its plumbing delay D (C-134), never of L; the manifest reports the conversion delay separately, on the device-facing port (C-207): D is the total delay from hand-off to emission, driver buffer, resampler and converter included (C-170). The **rate actuator** is a separate resampler (C-189): the kit's DSL resampler node is fixed-ratio (2:1 or 1:2, whole-sample delay, C-105), while the rate actuator is steerable, its ratio varying by parts per million around its nominal value to track the device's crystal; its delay is also part of D. **Both resamplers come from the arithmetic kit (epic 024a)**, which builds them once; every edge, the device included, uses the kit's and builds none of its own (C-197).
- **Rate-actuator trait with one v1 implementation: software ratio**, an adapter over the kit's steerable rate actuator (C-197). The `RateActuator` trait is defined by the kit epic (024a) together with its implementation, with a ratio range of ±1 000 ppm around the nominal ratio; every edge consumes it (C-210). `hadronyche-embedded` defines four traits and re-exports the kit's `RateActuator` as the fifth of a device's hardware surface; `Device` holds the software-ratio actuator internally, which is why it is generic over four traits. Hardware clock trimming can be added later behind the same trait with nothing on the wire.
- **E** for a device is declared per device (no fixed platform target) with the common definition: the 99.9th percentile of absolute emission error over a one-hour certification run, the maximum reported beside it (C-73). Only the commissioned certification state gives a numeric E; until a commissioned run is done the device declares E as unknown (C-140); it is then refused with reason RequiredEExceeded by any presentation group that sets a required E and admitted to groups that set none (C-103). This part carries D (with its components) and E (`Certified` or `Unknown`) in the profile and stamp; the values are measured and certified on the board in 026b.

**Capacity.** A stream is one packet per slice by default: 48 frames = 0.5 ms (C-02), 6 channels (C-04), 4-byte samples (C-03): 192 bytes per channel per slice, 1 152 bytes of audio, no per-channel id byte — the channel map is in the manifest (C-04). About 1 320 bytes on the wire (C-20), 2 000 packets/s, about 21.1 Mbit/s (C-21). A 100 Mbit device port holds about **19 channels** at the 70 % admission ceiling (C-26, C-23). A device is a narrow sink: it subscribes only to the bundle members (streams) it needs. The unit of subscription is the stream, so the device receives and buffers all 6 channels of a subscribed stream and selects the channels it plays through the manifest's channel map; a 2-output codec plays 2 of them. Alignment rings are sized to the presentation group's A (C-96) and hold wire slices at the domain rate, so buffer memory is channels × 4 bytes × 96 000 × A: 23 040 bytes for one 6-channel stream at A = 10 ms. Admission checks this **buffer memory** and refuses beyond the device's declared limit. Since the port holds about 19 channels, a 7th channel (a second stream) can only be refused on the declared memory limit, not on the port. The commissioning tool checks this declared memory before applying a change of A, sinks included (C-211, C-222). A change of A reaches the device through the tool's announcement inside a commissioning window (C-224); the device switches at its own position one lead ahead (C-214), and if the A does not fit it keeps the deepest ring its memory holds, with the cannot-meet-A alarm raised until a later change that fits and the given A, not the clamped one, in its receiver reports (C-211, C-216, C-222). This is handled in 026b (026b-13). The reference device subscribes to audio members only; were a device to subscribe to a controller member, admission (epic 012) would cost it by C-219.

**Liveness (unchanged).** Interrupted after 8 consecutive missed slices, 4 ms (C-38); only slices absent at the deadline count as misses — a marked gap or a slice on another timeline epoch proves the upstream is alive and resets the count (C-102). Dead after 2 s (C-39), counted from the last slice received with content or a marked gap (C-111); a slice on another timeline epoch also restarts the dead clock and has its own counter (C-135). A slice absent at its deadline counts missing and is reclassified late if it arrives later while its position is still within the ring span (C-108). These describe streams the device receives: a sink-only device's own liveness, as seen by its producers, is judged by the producer timeout (C-41, C-168). Receiver report per bundle every 100 ms (C-40), unicast to the producer, never multicast (C-47); the producer stops after 2 s without reports (C-41). In-band announcements: lead 64 slices (C-43), re-issued every 16 (C-44). A Pending connection whose target is absent persists until the target appears or the operator cancels (C-45). Persistent lateness (a late slice in each of 10 consecutive 1 s windows, C-101) triggers a ratchet step of one slice period (C-53); at the ceiling A the input is Demoted (C-109). A ratchet step is a re-alignment like any other: an upstream fan-in takes it at a position it announces one lead (C-43) ahead in its own output (C-215), which the device sees as an in-band announcement on the input it consumes; the device, a sink with no output, takes its own step at its own position one lead ahead (C-225), as for a changed A (C-214). An aligned multi-packet input uses the consumer's own deadline for reassembly (C-110). The drain per input is bounded to 2 × its admitted packets per slice in each slice period (C-16, C-156). The session number of each stream is assigned by its source (C-19).

**What weakens on `no_std` `[decision]`.**

| Guarantee on hosted | On the device |
|---|---|
| Housekeeping isolated from the data plane; no shared lock | Not enforced by an OS. The implementer must show **bounded control handling**; otherwise stamped *report-only*. The control-isolation capability is absent or unverified unless that bound is shown (C-161) |
| Supervised, restartable housekeeping | **Unsatisfiable.** Any fault is a whole-device reset (watchdog) |
| Privilege report (real-time scheduling, memory lock) | Replaced by a profile declaration: *embedded, by construction*. Strict mode is hosted-only; a device stamps real-time scheduling and memory lock as present by construction (C-147) |
| Fault injection, interface drop counters, DSCP marking | Each may be **absent**; stamped with the C-129 states. A test-only fault hook, including the panic-injection hook (C-145), is stamped like any injection (C-120) |
| Record and replay of exact-class modules | Done on a host: the same generated `no_std` code is rerun against captured input |
| Multi-packet slices, skew buffers | Bounded by declared memory; admission refuses beyond it |
| Files | None; the stamp is published on the network (C-18) |

This part declares every row in the profile (026a-04) and publishes it (026a-11); the measurements that decide the bounded-or-report-only row and the fault-reset row are made on the board in 026b.

**Where outputs live.** Measurement results and bench logs go to `doc/design/reports/`, specifications to `doc/design/spec/` (C-130). Host-side harnesses and tool binaries (traffic generators, the stamp-collector command line, bench controllers) go in the `hadronyche-tools` crate; throwaway probes in `probes/` (C-133). Datasheet figures carry `[memory]` until checked; a figure cited from a published standard carries `[standard]` with its clause (C-132). Fuzzing follows the C-95 budget: each parser 60 s on every change and 1 hour nightly, corpus kept.

## Deliverables

Crate `hadronyche-embedded` (no_std, no alloc; shapes, names may be refined):

```rust
/// The only network surface a device implements. IPv4 only (C-100).
pub trait Transport {
    type Error;
    /// `to` is an IPv4 address and UDP port; multicast is sent with TTL 1.
    /// `class` is one of the C-13 DSCP classes: sync CS6, media and probes EF, control CS5.
    fn send(&mut self, to: Endpoint, class: DscpClass, packet: &[u8]) -> Result<(), Self::Error>;
    /// Non-blocking. Returns length, source, and a receive time on the device clock.
    fn recv(&mut self, buf: &mut [u8]) -> Option<Rx>;
    fn join_discovery(&mut self, group: Endpoint) -> Result<(), Self::Error>;
    fn mtu(&self) -> u16;
}
pub struct Rx { pub len: usize, pub from: Endpoint, pub rx_time_ns: i64, pub source: StampSource }
/// C-131 values available on a device.
pub enum StampSource { MacHardware, Interrupt, Userspace }

pub trait DeviceTime {
    fn now_ns(&self) -> i64;                 // monotonic, 64-bit
    fn declared_jitter_ns(&self) -> u32;
}

/// Audio hardware: frames out, plus (frame count, time) pairs for loop 2.
pub trait AudioOut {
    fn rate(&self) -> u32;                   // 48_000 in v1
    fn write(&mut self, frames: &[i32]) -> usize;
    fn position(&self) -> DevicePair;        // (frame count, timer-captured device time)
    fn fixed_delay_ns(&self) -> u32;         // converter and interface part of D
}

/// Defined by the kit (epic 024a, C-210), ratio range ±1 000 ppm around nominal; re-exported here, not defined.
/// Held inside `Device`; v1 implementation: software ratio, over the kit's steerable rate actuator (C-197).
pub use kit::RateActuator; // shape as the kit defines it, e.g. set_ratio(f64), group_delay_frames() -> u32

pub trait Identity {
    fn hardware_unique(&self) -> [u8; 16];
    fn next_boot_counter(&mut self) -> u64;  // persistent
    fn true_random(&mut self) -> Option<[u8; 16]>;
    fn domain_id(&self) -> u16;              // persistent; 0 = unconfigured, ignored by peers (C-11, C-173)
}

/// The cooperative main loop: bounded work per call.
pub struct Device<T, C, A, I> { /* core state machines, servo, ring, fixed 2:1 resampler node, steerable resampler (RateActuator, C-189) */ }
impl<T: Transport, C: DeviceTime, A: AudioOut, I: Identity> Device<T, C, A, I> {
    pub fn poll(&mut self) -> PollStats;     // work done this call, its elapsed time, and the running worst case
}
```

- Host mocks of the four board traits in a `mock` module behind a `std`/`test` feature, the CI jobs (host build and tests, a device-target build, the C-95 fuzz job) and the no-alloc / no-libm symbol-check script that 026b applies to the firmware image.
- Identity derivation, the follower servo with holdover and the alignment claim, admission against declared buffer memory, the media receive path with receiver reports, the software-ratio `RateActuator` adapter over the kit's actuator, and the audio edge's two-loop timebase, all inside `Device` and verified by host tests.
- `profile.rs` — the embedded capability declaration for the stamp.
- `stamp_publish.rs` — sectioned publication of the stamp: multicast on change and every 10 s, unicast answer to a request (C-18), including the commissioning-window state (C-107, C-153).
- A hosted **stamp collector** addition in `hadronyche-std` that records device stamps; its record format and file names are decided here and recorded in its documentation (C-206).

## Dependencies

- **Needs:** control state machines, for the commissioning-window state every module stamps (005); servo (008), with the servo core's published lock state and bound (008a, C-187); alignment ring and gap classes (009); packetiser and bundle demux (010); discovery and election state machines with the 63-entry peer map (011, C-217); admission (012); stamp wire object and its sectioned transport (013); receiver reports (014); the shared two-loop edge core (018); the arithmetic kit's fixed 2:1 resampler node, its steerable rate actuator and the `RateActuator` trait, which the device uses and does not build or define (024a, C-197, C-210).
- **Provides:** the crate, the host-verified device side and the stamp collector on which 026b builds the reference device firmware. 026b depends on this part; no ticket of this part depends on 026b. Ticket 026a-01 uses the target triple of the board chosen in 026b-01 when it is available and a generic Cortex-M target otherwise, so it does not wait for it.

## Acceptance criteria

The epic criteria 026-AC-1 to 026-AC-12 are completed on the reference device in 026b, which keeps their ids. This part meets the host-verifiable share of them:

1. **026a-AC-1 (host share of 026-AC-1, AC-35).** On mock transport the device joins a simulated medianet as a follower with a **63-entry peer map** (C-217); `poll` does bounded work per call and reports its elapsed time and running worst case; its stamp is **published over the network** in sections of at most 1 252 bytes (C-17, C-18) and recorded by the hosted collector.
2. **026a-AC-2 (host share of 026-AC-3).** The hello never carries the holder-eligible bit, even when the device has the lowest module id (C-113). With the simulated holder removed, the device goes to holdover after the holdover entry delay (C-195), its bound grows at 1 µs/s, and it stops claiming presentation alignment once the bound exceeds its declared E (C-65), or 2 × C-62 with E unknown (C-169); the clock reads Unlocked (C-162), and the claim resumes after relock without re-commissioning (C-180). With a second simulated holder-eligible module whose time is usable, that module takes over with the epoch kept and the device's received stream is neither restarted nor re-admitted (C-213, C-221).
3. **026a-AC-3 (host share of 026-AC-4).** With the true-random source disabled, mock identities with different hardware unique ids yield different ids and nonces; rebooting one keeps its id and changes its nonce. A device with domain id 0 is ignored and counted by peers (C-173).
4. **026a-AC-4 (host share of 026-AC-7).** The liveness rules hold in host tests: interrupted after 8 slices absent at the deadline (C-38, C-102), dead 2 s after the last slice with content or a marked gap (C-111), resumption at the frozen offset within 2 s, re-admission after.
5. **026a-AC-5 (host share of 026-AC-9).** A request for a 7th channel (a second stream, beyond the device's declared buffer memory) is refused with a named reason from epic 012's enum.
6. **026a-AC-6 (host share of 026-AC-10).** D is computed with its components (C-170, C-189) and is in the stamp with its method, together with E (`Certified` per C-73 or unknown per C-103 and C-140); both forms round-trip through the stamp.
7. **026a-AC-7 (host share of 026-AC-11).** The crate builds for a device target with no allocator and no libm symbol, checked by the CI symbol-check script.
8. **026a-AC-8 (host share of 026-AC-12).** The stamp recorded by the collector carries the capability fields in the C-129 states (real-time scheduling and memory lock present, C-147; control isolation per C-161), the receive-timestamp source (C-131) and the commissioning-window state (C-107, C-153).
9. **026a-AC-9 Audio edge on the host.** With a mock `AudioOut`, the two-loop timebase schedules emission at `T(p) + A` within 1 frame at 48 kHz, through the kit's fixed 2:1 node and the software-ratio adapter over the kit's rate actuator (C-189, C-197, C-210).

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The crate builds for at least one device target and for the host (for tests) in CI
- [ ] The reference board, its timestamp source and its oscillator are named in the docs
- [ ] The table "what weakens on no_std" is reflected field by field in the device's stamp

Not applicable in this part (C-125): "The reference board, its timestamp source and its oscillator are named in the docs" — this part has no board; the board is chosen and named in 026b (026b-01, 026b-02, 026b-03, 026b-07, 026b-21). Every other line has a subject here: the part writes code and parsers (stamp request, sectioned stamp, collector decoder, control, admission and media paths), touches stamp fields, and works next to the frozen POC without modifying it. Fuzzing follows the C-95 budget.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Allowed methods for "D with method" (what D contains is settled by C-170) | Task 018 (shared edge core) | Two edges declare D by methods that cannot be compared |
| Which of the 18 refusal reasons (C-173, completed by C-185; no further reason may be added) the exceeded-buffer-memory refusal carries; the table does not name it | Task 012 (admission) | The device's refusal is not distinguishable from another reason |
| Message encoding of the commissioning-window announcement (its life cycle is settled by C-153), and of the commissioning tool's announcement of a changed A that the device receives (who sends it and when is settled by C-224) | Task 005 (control state machines) with the wire specification (004) | A module cannot tell whether test traffic is inside a window |
| How a non-zero domain id reaches a device's persistent storage (C-173 ignores domain-0 hellos) | Commissioning tooling with task 005 | A device cannot be brought into a domain without rebuilding its firmware |
| Collector record format for device stamps | This part (ticket 026a-12): presentation details of the tool it builds, recorded in the collector's documentation (C-206) | Device and hosted stamps cannot be compared from the records |

The measurement sub-questions of epic 026 (resampler CPU fit and `f64` cost, bounded control handling, timestamp source and jitter, DSCP and don't-fragment support, device E, reference board, how device alarms reach an operator) are owned by 026b.
