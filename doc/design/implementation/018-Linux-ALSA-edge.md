# 018 — Linux ALSA edge

**Phase:** P4 · **Scope:** L (25 tickets, about 29 days) · **Depends on:** 004, 008, 009, 012, 013, 015, 016, 024a · **Blocks:** 020, 021, 022, 026, 027

## Goal

Build the first physical edge: a Linux sink and a Linux capture module on ALSA. In doing so, build the **shared two-loop timebase** in `hadronyche-core` (device-clock regression, resampler-ratio control) that every other edge (macOS, Windows, embedded) reuses. It builds no resampler and defines no rate-actuator trait: both the fixed 2:1 / 1:2 node and the steerable rate actuator come from the arithmetic kit (epic 024a, C-197), and the rate-actuator trait is defined there with its implementation (C-210). This is the reference sink against which every other platform is measured: at L2 a macOS sink must stay within 150 µs and a Windows sink within 1.05 ms of it (C-82), and at L3 the skew between two sinks is held to the sum of their E values (C-79).

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules exchange *slices* addressed by *position* on a domain-wide timeline: `position = (network_time − epoch_origin) × rate`. The wire carries one sample rate only, the domain rate of 96 000 Hz (C-01), so there is one line for the whole domain; write `T(p)` for the network time of position p. Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers.

**The edge contract `[decision]`.** For a sink in a presentation group with audition offset A:

- **Emission:** `emit(p) = T(p) + A` — the instant sample p leaves the converter.
- **Hand-off to hardware:** `submit(p) = emit(p) − D`.

D is the sink's declared total plumbing delay from hand-off to emission: driver buffer + resampler group delay + converter (C-170). Capture is the mirror (D_capture runs from acoustic instant to the sample being available, C-170): a captured sample is stamped with the position of its acoustic instant, `T⁻¹(capture_time − D_capture)`. With both honest, A is exactly the mouth-to-ear latency of the medianet.

- **A** is operator configuration, one per presentation group (default group: the whole domain). It has no default value (C-55); the commissioning tool proposes it. A sink with no A configured cannot compute `submit(p)`: it does not start emitting and reports a named reason until the operator sets A. The runtime verifies continuously; a sink that cannot meet A raises the connection alarm "cannot meet A": maximum lateness + D + headroom exceeds A in each of 10 consecutive 1 s windows (C-174, headroom per C-52), the maximum lateness being taken over a rolling 60 s window (C-182).
- A group carries a **required E**. A sink whose E exceeds it is refused at admission with a named reason. A sink with no certified E declares its E as unknown: it is refused with reason `RequiredEExceeded` from any group that sets a required E, and admitted to groups that set none (C-103). Only the *commissioned* certification state gives a numeric E; *uncertified* and *self-monitored* sinks declare E as unknown (C-140). A refusal raises a connection alarm (C-173).
- **A changed while live (C-186, C-198, C-214).** When the group's A changes while the sink's input is live, the sink re-enters aligning with its ring resized to the new A, as fan-ins that inherited the old A do (C-186). It switches at its own position, one lead (C-43) ahead, as a fan-in does (C-208); it has nobody to announce it to (C-214). Its receiver reports carry its current A, so the change propagates upstream one hop per report (C-198). A commissioned change of the sink's own A reaches it through the commissioning tool, inside a declared window (C-153): the tool validates buffer memory first (C-211) and then announces the new A to the group's members (C-224). If a change that does not fit reaches the sink anyway, the clamp of C-211 applies to the sink as to a fan-in (C-222): it keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm and keeps emitting; the alarm stays raised until a later change of A that fits, and its receiver reports carry the A it was given, not the clamped value (C-216).
- **Clock unlocked (C-180). A *commissioned* sink whose clock is unlocked stays commissioned: its alignment claim is suspended and does not count towards a group's required E until the clock is locked again, then resumes without re-commissioning.

**E and D honesty `[decision]`.**

- **E** = 99.9th percentile of |actual emission − (T(p) + A)| over a one-hour certification run, with the maximum reported beside it (C-73). On hosted platforms E is a percentile claim, not a bound.
- **Linux reference sink target: E = 50 µs** (C-70, pending measurement), decomposed as **20 µs clock bound + 20 µs rate-tracking error + 10 µs uncertainty of D**. This requires the servo's lock bound to be at most E/2 = 25 µs (C-62, pending simulation).
- A sink **self-monitors** the first two components continuously. The analogue part of D needs an **external commissioning measurement**. Certification is therefore two-tier.
- **D must be repeatable across restarts:** spread at most 25 µs over 20 restarts on Linux (C-74, pending measurement). Restart-to-restart variation of driver delay is the classic failure of sound-card pipelines.
- Typical D on Linux: about 1 ms (C-81, pending measurement).
- A sink that cannot bound E cannot claim presentation alignment. Certification failures go into the regime stamp.
- Pass criteria at L2 mean the platform's target E (50 µs on Linux, C-70), never the E measured in the same run (C-149).
- Edge modules resample, so they are **never exact-tier**. As hand-written modules that do not claim exactness they are in the bounded class (C-123) and are verified on the tolerance tier.
- The commissioning-measured part of D is persisted by this epic per (device, driver, device rate) in a versioned file in the module's state directory. It is written only inside a declared commissioning window that the module stamps; outside a window the module is read-only to the commissioning tool (C-107). The tool announces the window on the control group with an expiry time; it closes at expiry or on explicit close (C-153). While a window is open, the module's ordinary production measurements in the stamp (self-monitored E, input lateness figures, loss counters) are unqualified; the commissioning tool's own measurements (D, restart spread) are valid commissioning results, each recorded with the window it was taken in (C-178).

**Two-loop timebase `[decision]`.** Every edge has a host clock H (here `CLOCK_MONOTONIC`) and a device sample counter S (the sound card's crystal).

- **Loop 1** (core servo, task 008): H → domain line, via sync exchanges with the clock holder.
- **Loop 2** (this task, in core): S → H, by regression over (sample count, host time) pairs supplied by the audio API. On ALSA these come from the PCM status: hardware pointer, delay, and its timestamp.
- **Resampler ratio** = quotient of the two slopes (domain rate against device rate, both expressed in host time). The rate loop holds filter state, so `EdgeTimebase` has an explicit `update` step that advances it; `ratio` returns the filtered value.
- The rate loop is a second servo cascaded on the first. Its bandwidth is **well below 1 Hz** (C-68, pending measurement at L2), so it filters servo jitter instead of passing it into pitch.
- Published quality number: clock bound + regression residual.
- **v1 steers a software resampler on every profile.** A **rate-actuator trait**, defined by the kit epic 024a together with its implementation and consumed by every edge (C-210), has that one implementation; hardware clock trimming can be added later with nothing on the wire. The rate actuator is a steerable resampler whose ratio varies by parts per million around its nominal value to track the hardware crystal; it is separate from the fixed-ratio DSL resampler node (C-189). Both are built once by the arithmetic kit (epic 024a, phase P4, ahead of this epic); this edge uses the kit's and builds none of its own (C-197). The kit's actuator accepts ratios within ±1 000 ppm of its nominal ratio (C-210).
- **Sleep is a departure (C-124).** When the host sleeps, both loops are reset and the edge returns through normal admission (Locking, Aligning). `EdgeTimebase` therefore has a `reset`.

**What this edge needs from the published time view (task 008).** `TimeView` must expose: the domain rate expressed in host time, the clock bound, the lock and holdover state, and `T(p)` for a position. `emission_error_ns` takes the view as an argument for `T(p)`.

**Why the servo's smoothness matters here.** Sources pace position from network time, so at a hardware edge servo behaviour becomes resampler ratio. Servo requirements this edge consumes: published time continuous while locked (zero phase step); slew at most 10 ppm (C-60, pending simulation); rate change at most 1 ppm/s (C-61, pending simulation); frequency-continuous handover between clock holders; steps only as announced timeline-epoch changes. The slew limits bound the servo's correction term, not the drift it tracks (C-119): the resampler ratio follows the real drift between device and domain line and is not itself capped at 1 ppm/s. In holdover the clock bound grows by 1 µs/s (C-64); the alignment claim lapses after about 25 s (C-65) and the sink must stop claiming presentation alignment. A sink that declares E as unknown (*uncertified* or *self-monitored*, C-140) takes E as twice the lock bound, 50 µs, for this rule and stops claiming when the clock bound it currently publishes exceeds 50 µs (C-169). After the lapse the clock is Unlocked with time unusable: one-way figures and alignment claims stop (C-162), so self-monitored E is blank. The certification state does not change: a *commissioned* sink stays commissioned with its claim suspended until relock (C-180).

**Sink behaviour.** A sink is a consumer with a deadline: it holds its input in a ring indexed by position, sized to the group's A (C-96), and takes position p at the fixed time, with whatever arrived. Absent slices are gaps (default concealment for audio: zero-fill).

- A slice absent at the deadline is counted missing; if it arrives later while its position is still within the ring span it is reclassified as late (missing −1, late +1) and discarded (C-108).
- Interrupted after 8 consecutive missed slices (C-38). Only slices absent at the deadline count as misses (missing, or arriving late); a marked gap or a slice on another timeline epoch proves the upstream alive and resets the count (C-102).
- Dead after 2 s (C-39), counted from the last slice received with content or a marked gap (C-111).
- A sink ratchets its offset as any consumer does (C-53, C-163, epic 009), up to a ceiling of A − D (C-226): an input that is persistently late (at least one late slice in each of 10 consecutive 1 s windows, C-101) steps the offset by one slice. A sink has no output to announce in, so it takes the step at its own position one lead (C-43) ahead, as it does for a changed A (C-225, C-214). An input persistently late with the offset already at the ceiling enters the health state Demoted: treated as gapped, counted separately, cleared only by re-admission (C-109).
- A slice on another timeline epoch restarts the dead clock, does not count as a miss, and has its own counter (C-135).
- The sink raises a connection alarm on interrupted, dead and Demoted, through the hosted runtime's alarm path (C-173).
- On a timeline-epoch change the sink restarts alignment at the announced position.

**Capture behaviour.** The capture module is a source. It assigns one session number per stream, bumped on restart and on timeline-epoch change; every consumer sees the same value (C-19). Each stream carries at most 6 channels (C-04); a capture wider than 6 channels emits a bundle of such streams, at most 32 members, a controller member included (C-08, C-209); the capture carries no controller member, so all 32 can be audio and it carries at most 192 channels. The channel map is carried in the manifest, not per channel on the wire (C-04). The bundle's wire format belongs to the wire specification (task 004). When the capture misses its own deadline, the hosted backend (task 015) emits the gap slice and raises the processing-overrun event (C-144).

**Observability.** Sink (as consumer) and capture (as producer) each publish per-connection summary counters in their own regime stamp (C-47), completed by C-159: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead, plus the observed DSCP. The sink also publishes input lateness percentiles: 50th, 99th, 99.9th and the maximum (C-160), over a rolling 60 s window, with the maximum since the connection went live reported beside them (C-182). The regime stamp travels in a dedicated control message split into sections that each fit 1 252 bytes of UDP payload (C-17, C-18); it is multicast on any regime change, refreshed every 10 s, and fetchable by unicast request; the hello carries only a stamp version number. The hosted backend (task 015) sends the stamp message (C-144); this epic fills its fields.

**Rates.** Domain rate 96 kHz, the only rate on the wire (C-01); slices of 48 frames = 0.5 ms (C-02); 4-byte samples f32 or Q1.31 (C-03); 192 bytes per channel per slice, 1 152 bytes of audio for a 6-channel stream, no per-channel id byte (C-04). A device running at 48 kHz is converted 2:1 at the edge through the kit's resampler node (C-80); that node is fixed-ratio with whole-sample delay, and the ppm correction is applied by the separate, steerable rate actuator (C-189). Both delays add to D (C-134, C-189) and never to L; the manifest reports the conversion delay separately, on the device-facing port (C-207). A resampler node may appear only inside an edge module; the compiler rejects it anywhere else (C-134). Structural latency L is always a whole number of domain-rate samples, and the resampler node is offered only for 2:1 and 1:2 conversion; other ratios are out of v1 (C-105).

**Resampler.** Two kinds, both from `hadronyche-kit` (no libm; shipped coefficient tables) (C-189), built by the arithmetic kit, epic 024a, not by this epic (C-197): the fixed-ratio resampler node (2:1 or 1:2, whole-sample delay, C-105) and the steerable rate actuator (`SoftwareResampler`, ratio within parts per million of its nominal value). Their delays are known constants and are part of D, not of the structural latency L (C-134, C-170, C-189, C-207). Their filter design, default tables and cost in counted abstract operations per slice (C-106) are epic 024a's (C-197).

**Strict mode.** The operator policy in the commissioning configuration sets strict mode (C-165). Strict mode requires real-time scheduling granted on the data and clock threads and memory locked; restricting CPU idle states stays best-effort and is stamped (C-127). In strict mode an unverified capability is treated as absent and the module is refused (C-128). In every mode, strict or not, a module whose thread layout's seqlock safety depends on a CPU pin refuses to start when that pin fails or is unverified (C-177, C-201); the default priorities (clock above data) never depend on pinning. Each capability is stamped as present, absent, unverified or not requested (C-129). Any run whose numbers enter the constants table, including every L2 run, is in strict mode (C-147).

**Interface held.** If the audio device cannot be opened in the required mode because another process has it open, the module reports "the interface is held" (C-175).

**Testing.** Fault injection for verification uses the public injection API plus the test-only hooks of C-120 (force a stream onto a given timeline epoch, send a raw packet), stamped like any injection. Each parser is fuzzed 60 s on every change and 1 hour nightly, with the corpus kept (C-95). Tests that need a privilege (strict mode, real-time grant) run in the separate privileged nightly job; "unprivileged" applies to per-change CI (C-146). L2 scenario parameters are those of C-166; A for ladder runs is computed by the harness (C-148).

**POC note `[code]`.** The POC has no sound-card path at all ("The payload is synthetic sines. No sound card is involved", README limits). Nothing is ported here.

## Deliverables

Crate `hadronyche-core`, module `edge/` (no_std; shapes, names may be refined):

```rust
/// One observation from the audio API: device frame count at a host time.
pub struct DevicePair { pub frames: u64, pub host_ns: i64 }

pub trait DeviceClock {
    fn nominal_rate(&self) -> u32;
    fn declared_jitter_ns(&self) -> u32;
}

/// Loop 2: S -> H. Fixed-memory regression with a bandwidth well below 1 Hz.
pub struct DeviceLine { /* slope, intercept, residual */ }
impl DeviceLine {
    pub fn update(&mut self, pair: DevicePair);
    pub fn frames_at(&self, host_ns: i64) -> f64;
    pub fn residual_ns(&self) -> u32;
}

// The rate-actuator trait (what steers the device toward the domain line) is not defined here:
// it is defined by the kit epic 024a with its implementation, ratio range ±1 000 ppm around the
// nominal ratio (C-210), and consumed by `alsa/` and every other edge.

/// Combines the published time view (loop 1) and the device line (loop 2).
pub struct EdgeTimebase { /* rate-loop filter state, … */ }
impl EdgeTimebase {
    /// Advances the rate-loop filter; called once per period.
    pub fn update(&mut self, view: &TimeView, line: &DeviceLine);
    /// Filtered ratio from the last update.
    pub fn ratio(&self) -> f64;
    /// Self-monitored emission error for the frame about to be submitted; None while the clock is not locked.
    pub fn emission_error_ns(&self, view: &TimeView, position: u64, a_ns: i64, d_ns: i64, submit_host_ns: i64) -> Option<i64>;
    pub fn quality_ns(&self, view: &TimeView, line: &DeviceLine) -> u32; // clock bound + residual
    /// Departure (sleep, C-124): forget both loops' state.
    pub fn reset(&mut self);
}
```

Also in `edge/`: `EmissionStats` (fixed-memory 99.9th percentile and maximum of |emission error|), `AlignmentClaim` and `CertificationState` (*uncertified*, *self-monitored*, *commissioned*).

Not built here: the rate-actuator trait (`RateActuator`, C-210), the kit's steerable rate actuator (`SoftwareResampler`, implementing it) and the fixed-ratio resampler node for 2:1 and 1:2 conversion (C-80, C-105, C-189) all come from epic 024a (C-197, C-210).

Crate `hadronyche-os-linux`, module `alsa/`:

- `sink.rs` — `AlsaSink` module: position ring → fixed 2:1 node (48 kHz device) and rate actuator → PCM write; `submit(p)` scheduling; input health (missing, late, interrupted, dead, Demoted, other-epoch); input lateness statistics; self-monitoring of E; alarms (C-173, C-174); strict mode (C-127, C-128); sleep as departure; re-alignment on a changed A at its own position one lead ahead (C-186, C-198, C-214).
- `capture.rs` — `AlsaCapture` module: PCM read → rate actuator and fixed 1:2 node (48 kHz device) → slices stamped with `T⁻¹(capture_time − D_capture)`; session numbers; emits a bundle when wider than 6 channels.
- `status.rs` — reading (frames, host time) pairs from PCM status timestamps; `DeviceClock` for ALSA.
- `declared.rs` — D from driver delay + resampler delays (fixed node and rate actuator, C-189) + configured converter delay; persistence of the commissioning-measured part, written only inside a commissioning window (C-107).
- Manifests of `AlsaSink` and `AlsaCapture`: the conversion delay reported separately on the device-facing port and kept out of L (C-207).

Tooling: an optional per-period emission trace, and an offline tool in the `hadronyche-tools` crate (C-133) computing E (99.9th percentile and maximum), MTIE and the restart spread of D (max − min, C-166) from it, for task 020, with the L2 scenario parameters of C-166 and C-191 and the holder-loss handover of C-212 (which needs a second holder-eligible Linux node on the rig); the 50 ppm difference of that handover is the measured natural difference between the two holders' crystals plus an injected drift totalling 50 ppm, both parts recorded in the run report (C-223). The trace format, file names and tool output are this epic's and are recorded in its documentation (C-206).

Stamp fields (format owned by task 013): audio device, driver, device rate, buffer and period sizes, D and its components and method, E self-monitored (99.9th percentile and maximum), regression residual, ratio statistics, certification state (*uncertified*, *self-monitored*, *commissioned*), the unqualified flag on production figures while a commissioning window is open (C-178), the capability states of strict mode (C-127, C-129), input lateness percentiles over a rolling 60 s window with the maximum since the connection went live (C-160, C-182), and the per-connection counters of C-47 and C-159.

## Dependencies

- **Needs:** wire specification, for the bundle format and session numbers (004); published time view and servo (008); alignment ring and gap classes (009); admission with the manifest carrying required E (012); stamp format (013); hosted runtime and its alarm path (015); real-time setup (016); the kit's fixed 2:1 / 1:2 resampler node, steerable rate actuator and rate-actuator trait (024a, C-197, C-210).
- **Provides:** the reference sink and the shared `edge/` core to L2 certification (020), the macOS (021) and Windows (022) edges, the embedded device (026) and the L3 rig (027).

## Acceptance criteria

This epic delivers mechanisms, tests on fake backends and verification tooling. Criteria marked *checked at L2* are judged on hardware by task 020; this epic supplies the procedure and the tools.

- **018-A1 Core unit tests (CI).** With synthetic pairs from a device crystal offset by ±100 ppm and pair jitter of 50 µs, the device line converges and the ratio error stays below 1 ppm after 30 s; a step in servo jitter of 20 µs changes the ratio by less than 0.1 ppm (bandwidth check).
- **018-A2 Self-monitored E** is computed for every submitted period and published; the value is blank until the clock is locked.
- **018-A3 Holdover lapse.** After 25 s of holdover (C-65) the sink reports that it no longer claims presentation alignment; a sink declaring E unknown stops claiming once its published clock bound exceeds 50 µs (C-169).
- **018-A4 Refusal (AC-30).** A sink configured with a self-declared E above the group's required E is refused at admission with a named reason; a sink with no certified E declares E unknown and is refused with `RequiredEExceeded` by a group that sets a required E, and admitted by one that sets none (C-103).
- **018-A5 Capture stamping (checked at L2).** A marker played by an external source at a known network time is captured with a position within E of `T⁻¹` of that time minus D_capture.
- **018-A6 Contribution to AC-26 (checked at L2 by task 020, in strict mode, C-147, with the parameters of C-166):** E at most 50 µs, the target E (C-149), (99.9th percentile over one hour, C-70, C-73); restart spread of D at most 25 µs over 20 restarts (C-74); round-trip marker offset constant within ±2 samples (C-75); zero slip (offset returns within ±1 sample) after injected loss (bursts spaced 1 s apart, media path only, C-191), a clock handover with 50 ppm crystal difference (C-78), triggered as a holder loss with about 2 s of holdover, during which emission error stays within the target E, and with frequency continuity at takeover (rate change within C-61, no phase step); a run in which holdover was not observed fails (C-202, C-212), and an announced step of 5 ms (C-76, C-191); MTIE at most 10 µs for intervals up to 1 s and at most E for any interval (C-77); E held throughout a drift ramp of ±100 ppm at no more than 0.5 ppm/s (C-119).
- **018-A7 No libm** call in the regression or in the kit's resamplers as linked into the edge (checked by symbol inspection of the `no_std` build; the resamplers themselves are epic 024a's, C-197).
- **018-A8 Best effort.** The sink runs unprivileged in best-effort mode (strict mode not set) and stamps what it lacks, each capability as present, absent, unverified or not requested (C-129).
- **018-A9 Input health.** On fake inputs the sink counts missing, late (with reclassification, C-108), marked gaps, interrupted (C-38, C-102), dead (C-39, C-111), other-epoch slices (C-135) and Demoted (C-101, C-109) as specified, raise the connection alarms of C-173, and both modules publish the C-47 and C-159 counters in their stamps.
- **018-A10 Sleep.** A simulated host sleep resets both loops and the edge returns through Locking and Aligning (C-124).
- **018-A11 Strict mode.** With strict mode set and real-time scheduling or memory lock absent or unverified, the sink and the capture refuse to start with a named reason (C-127, C-128); the stamp shows each capability in one of the four states of C-129. In any mode, a failed or unverified CPU pin on which the thread layout's seqlock safety depends refuses start (C-177, C-201).
- **018-A12 Cannot meet A.** On fake inputs, the alarm is raised when maximum lateness + D + headroom exceeds A in each of 10 consecutive 1 s windows, and not after 9 (C-174), with maximum lateness taken over the rolling 60 s window (C-182).
- **018-A13 Changed A.** On fake inputs, a change of the group's A while the input is live makes the sink re-enter aligning at its own position one lead (C-43) ahead, unannounced, with its ring resized to the new A (C-186, C-214); its receiver reports carry the new A (C-198).

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The `edge/` module builds `no_std` with no allocation
- [ ] A named reference hardware configuration (machine, sound device, driver, kernel) is recorded for certification
- [ ] D measurement method documented, including which part is self-reported and which is commissioned

**Not applicable in this epic (C-125).** "The frozen POC is not modified": the POC has no sound-card path and nothing is ported from it, so this epic has no POC subject; the line holds trivially. Every other line applies.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Is E = 50 µs reachable on the reference hardware? (C-70) | Task 020 (L2 Linux) | Tracking error plus D uncertainty exceed 30 µs |
| Is D repeatable across restarts on ALSA with the chosen device? (C-74) | Task 020 (L2 Linux) | Spread above 25 µs over 20 restarts |
| Rate-loop bandwidth value (C-68) | Task 020 (L2 Linux) | The edge passes servo jitter into the resampler ratio, or tracks too slowly after a handover |
| Resampler quality versus cost (filter length): settled as no longer this epic's, since both resamplers and their filter design are the kit's (C-197) | Epic 024a, then task 026 for the device | Audible artefacts at L2, or the device cannot afford it |
| Does the ratio need a bounded phase correction beyond the slope quotient? | This task (ticket 018-03) | With a slope-only ratio, emission error in a 10-minute fake run drifts beyond the 20 µs rate-tracking budget |
| Which statistic is the regression residual (RMS, percentile, maximum)? | This task (ticket 018-02) | The published quality number is below the emission error observed at L2 |
| Which reference hardware (machine, sound device, driver, kernel) is chosen | This task proposes it after P0 (ticket 018-20); the product owner approves; recorded in the constants table (C-172) | The chosen device fails the D repeatability test (C-74) at L2 |
| `TimeView` exposes domain rate in host time, clock bound, lock and holdover state, and `T(p)` (the lock state and bound are published by the servo core through the time view, C-187) | Task 008a | Any of the four is missing from the published view |
| Wire format of a capture bundle (shared position per slice, member order) | Task 004 | Spec 004 cannot carry a 32-member bundle with one shared position per slice |
| Which edge stamp fields are regime changes (immediate multicast) and which are values refreshed every 10 s (C-18); definition of "ratio statistics" | Task 013 | Stamp traffic exceeds the control budget, or a configuration change is seen only after the refresh |
| How does a sink ratchet, and when is a persistently late sink input Demoted (C-101, C-109)? Settled by C-225: it ratchets like any consumer and takes each step at its own position one lead (C-43) ahead, unannounced, as for a changed A (C-214); Demoted only with the offset already at the ceiling | Settled (C-225); implemented by ticket 018-22 with epic 009's ratchet | — |
| At which position does a sink re-enter aligning after a changed A (C-186)? Settled by C-214: at its own position, one lead (C-43) ahead, as a fan-in does (C-208), with nobody to announce it to | Settled (C-214); implemented by ticket 018-28 | — |
