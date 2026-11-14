# 020 — L2 certification harness

**Phase:** P4 · **Scope:** L (28 tickets, about 37 days) · **Depends on:** 008, 016, 017, 018 · **Blocks:** 021, 022, 027, 028

## Goal

Build the harness that certifies a **physical edge** (a sink or capture module on a real audio device) and run it for Linux. The same harness, with a per-platform audio backend, certifies macOS (task 021) and Windows (task 022). L2 is the first place where the clock servo's smoothness requirements meet a hardware crystal, and where the numbers E and D stop being proposals.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules exchange *slices* addressed by *position* on a domain-wide timeline: `position = (network_time − epoch_origin) × rate`; write `T(p)` for the network time of position p. Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers.

**What is being certified `[decision]`.**

- A sink emits sample p at `emit(p) = T(p) + A` and hands it to hardware at `submit(p) = emit(p) − D`. A is the presentation group's audition offset (operator configuration; it has no default, C-55); for ladder runs, this one included, the harness computes A with the commissioning rule (largest over sinks of maximum lateness + D + headroom) and records it with the results (C-148). D is the sink's declared plumbing delay: its total delay from hand-off to emission, driver buffer + resampler + converter (C-170).
- **E** = 99.9th percentile of |actual emission − (T(p) + A)| over a **one-hour** run, with the maximum reported beside it (C-73). On hosted platforms E is a percentile claim, not a bound.
- E decomposes into clock bound + rate-tracking error + uncertainty of D. A sink self-monitors the first two; the analogue part of D needs this external measurement. Certification is two-tier: *self-monitored* and *commissioned*. Only the commissioned state gives a numeric E; uncertified and self-monitored sinks declare E as unknown (C-140). A commissioned sink whose clock becomes unlocked stays commissioned: its alignment claim is suspended and does not count towards a group's required E until the clock is locked again, then resumes without re-commissioning (C-180).
- A sink with no certified E declares E as unknown; it is refused with reason RequiredEExceeded from any presentation group that sets a required E, and admitted to groups that set none (C-103). The E this harness certifies is therefore what lets a sink join a group with a required E.
- **D must be repeatable across restarts.**
- Each edge has two loops: loop 1 maps the host clock to the domain line (the servo, via sync exchanges with the clock holder); loop 2 maps the device sample counter to the host clock (regression over sample-count and host-time pairs from the audio API). The steered ratio, the quotient of the two slopes, is that of the edge's rate actuator: a steerable resampler, separate from the fixed-ratio 2:1 / 1:2 node, whose ratio varies by parts per million around its nominal value and whose delay is part of D (C-189). The arithmetic kit (epic 024a) builds it and defines its trait, with a ratio range of ±1 000 ppm around the nominal ratio (C-197, C-210), which covers the ±100 ppm drift ramp and the 50 ppm handover; the edge builds none of its own. The rate loop's bandwidth is well below 1 Hz (C-68).

**Rates.** The domain rate is 96 000 Hz and it is the only rate on the wire (C-01). An edge whose hardware runs at another rate converts inside the edge module; at 48 kHz the edge converts 2:1 through the kit's resampler node (C-80); only 2:1 and 1:2 are offered, so structural latency stays a whole number of domain-rate samples (C-105). A resampler node exists only inside an edge module, and the edge's conversion delay is part of its D and never counted in L (C-134, C-207). The device rate of the sink and of the capture device is therefore a property of the rig and is recorded, but every position, marker and tolerance in samples is in domain-rate samples.

**Targets per platform (all pending measurement).**

| Platform | E target | Restart spread of D, 20 restarts | Typical D |
|---|---|---|---|
| Linux, ALSA reference sink | 50 µs (C-70) = 20 clock + 20 tracking + 10 D | at most 25 µs (C-74) | about 1 ms (C-81) |
| macOS, built-in or bus-attached device | 100 µs (C-71) = 40 clock + 40 tracking + 20 D (C-150, proposal) | at most 50 µs (C-74) | a few ms (C-81) |
| Windows, WASAPI exclusive | 1 ms (C-72) = 100 clock + 100 tracking + 800 D (C-150, proposal) | at most 0.5 ms (C-74) | 6 to 20 ms (C-81) |

**Servo requirements whose effect is measured here.** Published time continuous while locked (zero phase step); slew at most 10 ppm from the holder's rate (C-60); rate change at most 1 ppm/s (C-61); lock bound at most 25 µs (C-62); acquisition to Locked within 10 s of the first exchange (C-63); frequency-continuous handover between holders; a step larger than 2 ms (C-67) is never slewed but announced as a timeline-epoch change. The slew limits C-60 and C-61 bound the servo's **correction term**, not the drift estimate it tracks (C-119): a holder drifting by 100 ppm is followed through the drift estimate while the correction term stays within 10 ppm and 1 ppm/s. After a restart an edge goes through normal admission, Locking then Aligning (4 s, C-50).

**Two traps the procedure must avoid `[decision]`.**

1. **The idle servo.** A lone host is its own clock holder; its servo does nothing. To exercise the servo on one machine, the sink must *follow another in-process module* whose clock has **injected drift and steps** (the runtime's ground-truth injection API). The injected values are the ground truth. For the same reason, the 50 ppm crystal difference between the two holders of the handover is produced by injected drift; because the handover is a holder loss, the second holder runs on a second holder-eligible Linux node of the rig (C-212), so the injected drift is set so that it and the measured natural difference between the two nodes' crystals add up to 50 ppm (C-78), and both parts are recorded in the run report (C-223).
2. **The shared crystal.** A loopback from a sound card's output to its own input shares one crystal; if both directions resample, the two resamplers cancel each other's error. Capture must therefore be read **raw against the hardware sample counter**, with no capture resampler.

**Injection.** Beyond the public injection API of task 017, the test-only hooks are: force a module id and nonce; force a stream onto a given timeline epoch; send a raw packet on a media or control socket. Every injection is stamped (C-120).

**Strict mode.** Any run whose numbers enter the constants table runs in strict mode (C-147): real-time scheduling granted on the data and clock threads and memory locked; restricting CPU idle states stays best-effort and is stamped (C-127). A capability reported as unverified counts as absent, so the run is refused (C-128). Capability states are read from the regime stamp (present, absent, unverified, not requested; C-129). CI runs on recorded files do not use strict mode.

**Procedure (per platform).**

1. Setup: one machine, the edge under test, a reference module in the same process acting as clock holder with injection available, a physical loop from the device output to a capture input (same device, raw) or to an independent capture device. For the handover the rig adds a second holder-eligible Linux node running the reference holder that takes over (C-212).
2. The sink plays a stream containing **position markers** (a known impulse or code at known positions; the waveform is chosen by this task and recorded with its reason in `marker.rs`).
3. The harness reads raw capture frames with their hardware frame counts, locates each marker, and converts its capture frame to a network time using the capture side's device line.
4. **Emission error** for a marker = (network time at which it was captured − capture-side delay) − (T(p) + A). Capture-side delay is the capture mirror of D: from acoustic instant to the sample being available (C-170); it is calibrated once per rig (method: see Open sub-questions). A is computed by the harness with the commissioning rule (C-148) and is written into the record.
5. Run for one hour per condition. Restart the edge 20 times for the repeatability figure; after each restart D is measured over a 60 s window, and the restart spread of D is max − min (C-166).
6. Disturbances, each followed by a check that the marker offset returns to its prior value, with the offsets before and after compared over 10 s windows (C-166), the window after starting when the injected event ends (C-191): injected packet loss, 1 % random plus bursts of 8 slices (C-166) spaced 1 s apart, on the media path only (C-191; L2 has no separate sync-loss scenario, C-202); a clock handover between two holders whose crystals differ by **50 ppm** (C-78), triggered 60 s into the run (C-191) as a holder loss: the old holder stops answering, the sink passes through about 2 s of holdover (C-37 plus C-33) and the new holder, on the rig's second holder-eligible Linux node, takes over (C-212); the new holder has usable time when the old one stops answering (Locked, or in holdover with its alignment claim not lapsed), so it takes over with frequency continuity and the timeline epoch is kept (C-213); emission error is checked while the sink's clock is in holdover between holders (C-202) and frequency continuity at takeover, a rate change within C-61 and no phase step (C-212), and a run in which holdover was not observed, or in which the epoch was bumped, is failed, not passed (C-212, C-213); an announced timeline step of 5 ms (C-191).
7. Record the steered resampler ratio, the servo's drift estimate and its correction term throughout, and compute wander.
8. `drift-ramp`: the injected crystal drift sweeps ±100 ppm at no more than 0.5 ppm/s; the pass is E held throughout the sweep (C-119).

**Pass numbers (proposals until measured).**

- E within the platform target. Every pass criterion that names E means the platform's target E (C-70, C-71, C-72), never the E measured in the same run (C-149).
- Restart spread of D (max − min, C-166) within the platform figure.
- **Round-trip marker offset constant** over the run (C-75): on Linux within ±2 samples; on macOS and Windows within that platform's restart spread (C-74: 50 µs and 0.5 ms). Sample tolerances are measured against the median of the first 60 s (C-166).
- **Zero accumulated slip** (C-76): after each disturbance the offset returns to its prior value, on Linux within ±1 sample; on macOS and Windows within that platform's restart spread (C-74). A one-sample slip passes on Linux and is reported as a slip event (C-166).
- **No step at the handover** (020-A2): within ±1 sample on Linux, within the platform restart spread elsewhere (C-166). An embedded device declares its own restart spread, measured at L3 (C-191), not by this harness.
- **Holdover during the handover** (020-A2): emission error stays within the platform target E while the sink's clock is in holdover between holders (C-202, C-149). This is how L2 covers sync loss; there is no separate sync-loss scenario. The handover is a holder loss, so holdover must occur: a run in which holdover was not observed is failed, not passed (C-212). Frequency continuity at takeover means a rate change within C-61 and no phase step (C-212); the new holder has usable time, so the timeline epoch is kept, and a run whose epoch is bumped fails (C-213). After the loss every candidate is in holdover, so the lowest module id among them wins (C-221); the rig gives holder B the lowest id among its holder-eligible modules (C-120 hook) so that B is the one that takes over. Per-sink emission error in holdover is asserted here, at L2, only: the L3 rig cannot observe it and asserts the skew between sinks instead (C-212, C-220).
- **Wander:** MTIE at most 10 µs for intervals up to 1 s, at most the platform target E for any interval (C-77, C-149). The harness evaluates MTIE on a logarithmic interval grid from one marker period to the run length that always includes 1 s.
- **Drift ramp:** E held throughout the ±100 ppm sweep (C-119); C-60 and C-61 are checked on the correction term.
- **Cross-sink skew against the Linux reference sink** (for macOS and Windows, tasks 021 and 022): at most the sum of the two E values — 150 µs for macOS, 1.05 ms for Windows (C-82), with skew taken as at L3: the 99.9th percentile of absolute skew, maximum reported beside it (C-79). Where a pair of different sinks is judged on a no-step or recovery tolerance, the tolerance is the larger of the two sinks' own, and both sinks' figures are reported beside each verdict (C-203).

MTIE (maximum time interval error) for an interval τ is the largest peak-to-peak variation of the time error within any window of length τ.

**Position units.** At the domain rate of 96 kHz (C-01) one sample is 10.4 µs, so ±2 samples is about ±21 µs and ±1 sample about ±10.4 µs, whatever the device rate.

**Slip events and the C-76 verdict.** The analysis reports two things after each disturbance: a *slip event* whenever the settled offset changes by an amount that rounds to a non-zero whole number of samples, and the *C-76 verdict* against the platform tolerance. The self-check (020-A5) tests the first; certification uses the second. A 1-sample slip on Linux passes certification and is reported as a slip event (C-166).

**Reference hardware.** This task creates the reference list of OS, driver and device combinations, starting with the Linux reference rig it builds; tasks 021 and 022 add their entries. Reference hardware is proposed by the implementing team, approved by the product owner and recorded in the constants table (C-172); the harness's list mirrors that record. A run on any combination outside the list is stamped *unverified*.

**Uncertifiable, to be stamped not tested.** Any bound on Windows scheduling latency; a device whose driver misreports latency without this measurement; macOS aggregate devices; Bluetooth and network audio outputs; any OS, driver and device combination outside the reference list (stamped *unverified*).

**Packaging by the commissioning tool.** When task 028 packages the group-E procedure, it drives test traffic only inside a declared commissioning window that every module stamps (C-107); the tool announces the window on the control group with an expiry time, and it closes at expiry or on explicit close (C-153). Measurements the tool takes in that window (D, restart spread, group certification) are valid commissioning results, each recorded with the window it was taken in, while ordinary production figures in stamps are unqualified for the window's duration (C-178). The harness itself runs on an isolated rig, with no window open.

## Deliverables

Crate `hadronyche-tools` in the design set's repository (C-133; created by this task if it does not already exist), binary `hadronyche-l2`. Timeline, slice and stamp types come from `hadronyche-core` (C-133):

- `marker.rs` — marker generation (reference source module) and detection in raw capture (sub-sample interpolation); the waveform choice and its reason.
- `rig.rs` — rig description (output device, capture device, device rates, loop type, calibrated capture delay, the second holder-eligible Linux node for the handover, C-212) and calibration procedure.
- `run.rs` — scenario runner, including the computation of A (C-148) and the strict-mode check (C-147): `steady` (1 h), `restarts` (20), `loss`, `handover-50ppm`, `step`, `drift-ramp` (injected drift swept ±100 ppm at no more than 0.5 ppm/s, C-119); a `holder` mode of `hadronyche-l2` that runs the takeover holder on the rig's second Linux node under the runner's control (C-212).
- `analysis.rs` — emission-error percentiles and maximum, marker-offset series, slip detection, ratio series, correction-term checks against C-60 and C-61, MTIE.
- `report.rs` — a certification record: platform, OS build, audio device, driver, device rate, buffer sizes, A and its inputs (C-148), D and its method, E (99.9th percentile and maximum), restart spread, pass/fail per criterion, both stamps (the regime stamps of the sink and of the holder it followed, C-191), the strict-mode capability states (C-129, C-147), the commissioning window the measurement was taken in, if any (C-178), and for a handover run the two parts of the 50 ppm difference, measured natural and injected (C-223). The record's JSON and Markdown layout and its file names are decided here and documented in `report.rs` (C-206).
- A versioned, self-sufficient recording format for raw capture and its side channels (marker schedule, injected ground truth, steered ratio, drift estimate, correction term, A and its inputs), with a file-backed capture backend, so the analysis runs in CI without audio hardware. The recording and rig-file parsers are fuzzed 60 s on every change and 1 hour nightly, with the corpus kept (C-95).

Trait the per-platform backends implement (shape, names may be refined):

```rust
/// Raw capture for measurement: frames plus the device's own frame counter and
/// the host time of that counter. No resampling.
pub trait RawCapture {
    fn open(device: &str, rate: u32, channels: u16) -> std::io::Result<Self> where Self: Sized;
    fn read(&mut self, out: &mut [f32]) -> std::io::Result<RawBlock>;
    /// True if any rate conversion is in the capture path; an unknown answer is true.
    fn resampler_in_use(&self) -> std::io::Result<bool>;
}
pub struct RawBlock { pub frames: usize, pub first_frame_count: u64, pub host_ns: i64 }
```

Document `l2-results-linux.md` in the reports directory (`doc/design/reports/`, C-130): the Linux certification record, and the measured values written back to the constants table `doc/design/02-constants.md` (C-68, C-70, C-74, C-75, C-76, C-77, C-81).

## Dependencies

- **Needs:** the servo simulation results used to judge the MTIE mask (008); real-time setup and wake-latency measurement (016); reference source and injection API, including the test-only hooks of C-120 (017); the Linux ALSA edge and the shared two-loop core (018).
- **Needs these interfaces, not yet described:** from 017, injected drift and steps on a reference holder and a way to make one in-process holder stop answering, so that the handover happens as a holder loss (C-212; the election rules themselves belong to the control state machines, task 005), and the reference holder with injected drift run on the rig's second holder-eligible Linux node (C-212); from 018, read access to the sink's steered resampler ratio, its servo drift estimate and correction term (separately, for C-119), its declared D, its per-connection counters (C-47, completed by C-159), its lateness percentiles and maximum (C-160, for A under C-148; the percentiles cover a rolling 60 s window with the maximum since the connection went live beside them, C-182), its capability states (C-129, for strict mode) and its clock state and bound, so the harness knows when the sink is in holdover (C-202, C-212). The clock lock state and bound are published by the servo core (epic 008a) through the published time view (C-187); the harness reads them through the sink.
- **Provides:** the harness reused by the macOS (021) and Windows (022) tasks; the certified Linux reference sink that L3 (027) compares against; the group-E certification procedure that the commissioning tool (028) packages.

## Acceptance criteria

1. **020-A1 = AC-26 (Linux).** E at most 50 µs over one hour (99.9th percentile; maximum reported); restart spread at most 25 µs over 20 restarts; marker offset within ±2 samples; zero slip after loss, the 50 ppm handover and an announced step; MTIE at most 10 µs up to 1 s and at most E (the 50 µs target, C-149) overall.
2. **020-A2 = AC-5 (hardware part).** During the 50 ppm handover the steered ratio changes by at most 1 ppm/s and the marker offset shows no step (within ±1 sample, C-166); the handover is triggered as a holder loss to a holder on a second holder-eligible Linux node, and emission error stays within the target E while the sink's clock is in holdover between holders (C-202, C-212); the takeover keeps the timeline epoch (C-213); the 50 ppm is the measured natural difference between the two holders' crystals plus an injected drift, both recorded in the certification record (C-223); a run in which holdover was not observed, or the epoch was bumped, fails (C-212, C-213).
3. **020-A3 — Trap 1 demonstrably avoided.** A control run with no injection shows a servo that is idle (ratio tracks only the device crystal); the certified run shows the servo active.
4. **020-A4 — Trap 2 demonstrably avoided.** The harness refuses to run if the capture path reports a resampler in use (`resampler_in_use` returns true or cannot answer).
5. **020-A5 — Harness self-check.** With a synthetic capture stream containing a known 30 µs emission error and a known 1-sample slip, the analysis reports both within 2 µs and raises a slip event for the slip.
6. **020-A6 — Rig calibration** is repeatable: capture-side delay within ±1 sample of the mean of 10 calibrations (C-191).
7. **020-A7 — Stamps.** The certification record carries complete stamps and names the reference hardware; a run on other hardware is marked *unverified*.
8. **020-A8 — Write-back.** Results are written back to the constants table with the `[measured]` tag, or the target is revised with a stated reason.
9. **020-A9 — Drift ramp.** Over the ±100 ppm sweep at no more than 0.5 ppm/s, E stays within the platform target and the correction term stays within C-60 and C-61 (C-119).

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The analysis code runs in CI on recorded capture files (no audio hardware needed)
- [ ] The rig (devices, cables, loop type, calibration value) is documented well enough to rebuild
- [ ] The Linux certification record is committed

**Not applicable in this epic (C-125).** "Regime stamp fields touched are complete and documented": this task adds or changes no regime stamp field. It reads the sink's stamp (declared D, E or unknown per C-103 and C-140, the C-47 and C-159 counters, lateness percentiles per C-160, capability states per C-129, injection stamps per C-120) and copies it into the certification record, whose own fields are documented in `report.rs`. The regime stamp's fields belong to the wire specification (task 004) and the sink (task 018).

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| How is the capture side's own delay calibrated without circularity (independent capture device with a datasheet figure, an oscilloscope against a network-time pulse, or symmetric-loop halving)? | This task | No method resolves below 10 µs, the D share of the Linux target |
| Is 50 µs reachable on a stock kernel, or does the Linux reference need a real-time kernel? | This task | Target met on a stock kernel over one hour |
| MTIE mask: are the two proposed points sufficient, or is a full mask needed? | This task, with the servo simulation results of task 008 | An audible or measurable artefact passes the two-point mask |
| A spectral criterion (sidebands on a pure tone): is one needed, and with which tone and threshold? | This task | A pitch artefact is audible while all criteria pass |
| Threshold separating an idle servo from an active one (020-A3) | This task, by measurement of the control run | Control and injected runs overlap on the servo-activity metric |
