# 027 — L3 cross-sink rig

**Phase:** P6 · **Scope:** L (25 tickets, about 28.75 days) · **Depends on:** 004, 005, 017, 018, 020, 021, 022, 026 · **Blocks:** 028

## Goal

Build the rig and run the measurement that **proves cross-sink simultaneity**: two different sinks — the certified Linux reference sink and the embedded reference device — play the same stream, and their physical outputs are compared on one capture clock. This is the top rung of the certification ladder and the only direct evidence that "one audition offset for the whole group" works in the physical world.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. IP is IPv4 only, and multicast is sent with TTL 1 (C-100), so every rig host sits on one switch and one subnet. Modules exchange *slices* — all content of one stream for one stretch of media time — addressed by *position* on a domain-wide timeline: `position = (network_time − epoch_origin) × rate`; write `T(p)` for the network time of position p. Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers.

**One rate on the wire.** The domain sample rate is 96 000 Hz and it is the only rate on the wire (C-01). An edge whose hardware runs at another rate converts at the edge: the embedded reference device's hardware runs at 48 kHz and converts 2:1 through the kit's resampler node (C-80). A resampler node may appear only inside an edge module, and its conversion delay is part of that sink's plumbing delay D (C-134); the kit offers it only at 2:1 and 1:2, whose delay is a whole number of samples (C-105). "One sample" in this epic therefore always means one domain sample, 1/96 000 s = 10.4 µs, whatever the capture rate. A default stream carries 6 channels at 4 bytes per sample (C-03, C-04: 192 bytes per channel per slice, 1 152 bytes of audio, no per-channel id byte), about 1 320 bytes per packet on the wire (C-20) and about 21.1 Mbit/s per stream (C-21); the device's 100 Mbit port carries about 19 channels (C-26), so the rig's marker stream fits it.

**The ladder `[decision]`.** L0 = loopback in one process. L1 = two machines, one wire. L2 = one reference sink per platform, certified alone. **L3 = a hosted sink and an embedded sink together.**

**The claim under test.** Every sink in a presentation group emits sample p at

`emit(p) = T(p) + A`

where A is the group's audition offset (operator configuration, one number per group, no default, C-55), and hands it to its hardware at `submit(p) = emit(p) − D`, D being that sink's declared plumbing delay, its total delay from hand-off to emission: driver buffer, resampler and converter (C-170). If every sink is honest about D, all sinks emit the same sample at the same instant, whatever their path through the network and whatever their hardware.

Each sink publishes **E**: the 99.9th percentile of |actual emission − (T(p) + A)| over a one-hour run, with the maximum reported beside it (C-73). On hosted platforms E is a percentile claim; an embedded device may claim a bound. The Linux reference sink's target is 50 µs (C-70, pending measurement at L2). The device's E is declared per device. Only the certification state commissioned gives a numeric E; an uncertified or self-monitored sink declares E as unknown (C-140, C-103). Such a sink cannot be judged at L3, so both sinks enter the rig commissioned, with a numeric E.

**Pass criterion (C-79, pending measurement; owner L3).** The **99.9th percentile of the absolute skew** between the two sinks' outputs is **at most the sum of their two E values, for 1 hour, and across restarts of either**; the maximum absolute skew is reported beside it and is not asserted. This is the same statistic as E itself. The E values in the limit are the targets — C-70 for the Linux sink and the device's declared E — never the E measured in the same run (C-149).

**Why one capture device.** Both outputs go into two channels of a **single** capture device. One capture clock means the skew is read directly as a time difference between two channels, with no clock comparison and no capture-delay calibration (any capture delay is common to both channels, provided the device's inter-channel skew is known and small). The capture runs at 96 kHz, the rate at which the capture device's inter-channel skew is to be measured (open sub-question below).

**Why the two sinks differ in everything else.** Different crystals (sound card versus device oscillator), different timestamp sources (on Linux `kernel` software stamps; on the device `mac-hardware` or `interrupt` stamps, C-131), different resampler ratios (the device's 48 kHz hardware converts 2:1 at the edge, C-80, while the wire carries only 96 kHz, C-01), different D. Agreement despite all that is the proof.

**Background rules the rig exercises.**

- Both sinks follow the same clock holder with the core servo: slew at most 10 ppm (C-60), rate change at most 1 ppm/s (C-61), lock bound at most E/2, 25 µs against C-70 (C-62, pending simulation). Handover between holders is frequency-continuous; the L2 handover test uses a 50 ppm frequency difference (C-78), made up of the measured natural difference between the two holders' crystals plus an injected drift, both recorded in the run report (C-223); the L3 handover injects no drift and records the measured natural difference. A handover is triggered as a holder loss: the old holder stops answering, each sink passes through about 2 s of holdover (C-37 plus C-33) and the new holder takes over; a run in which holdover was not observed is failed (C-212). At L3 the run asserts that the skew between the two sinks stays within the sum of their E values (C-79, C-202) through holdover and takeover, and that each sink's frequency is continuous at takeover, a rate change within C-61 and no phase step; per-sink emission error against the timeline is an L2 assertion only, which the L3 rig cannot observe (C-220). The rig therefore needs a second holder-eligible Linux node (C-212). A holder-eligible module may take over only if its time is usable: Locked, or in holdover with its alignment claim not lapsed (C-65, C-169); it takes over with frequency continuity and the timeline epoch is kept, and the epoch is bumped (C-113) only when no candidate has usable time (C-213).
- In holdover the clock bound grows 1 µs/s (C-64); alignment claims lapse after about 25 s (C-65), and the clock state then becomes Unlocked with time unusable: one-way figures and alignment claims stop (C-162). C-65 is derived as (C-70 − C-62) / C-64: the claim lapses when the clock bound, starting from at most the lock bound, reaches the sink's E. The **margin** of the holdover criterion is therefore the sink's own E: about 25 s into holdover for the Linux sink, and (E_device − lock bound) / (1 µs/s) for the device. Holdover begins after the holdover entry delay, pending simulation (epic 008b) and expected at or below the 1.5 s election silence threshold (C-195). A commissioned sink whose clock is unlocked stays commissioned: its alignment claim is suspended and resumes once the clock is locked again, without re-commissioning (C-180). The clock state and bound are published by the servo core (epic 008a) through its time view (C-187).
- A sink whose E exceeds the group's **required E** is refused at admission with a named reason. The reason is `RequiredEExceeded` (C-103, which also refuses a sink whose E is unknown from any group that sets a required E and admits it to groups that set none).
- Liveness: interrupted after 8 consecutive missed slices, 4 ms at defaults (C-38), dead after 2 s (C-39) counted from the last slice received with content or a marked gap (C-111). Only slices absent at the deadline count as misses — missing, or arriving late; a marked gap or a slice on another timeline epoch resets the count (C-102); an other-epoch slice also restarts the dead clock and has its own counter (C-135). An absent slice is counted missing at the deadline and reclassified as late if it arrives while its position is still within the ring span (C-108). Resume from interrupted is immediate at the frozen offset — so a short interruption must not shift a sink's output in time. Interrupted and dead describe the stream a sink receives; the liveness of a sink-only device as seen by its producer is judged by the producer timeout, 2 s without receiver reports (C-168, C-41).
- A timeline step is never slewed: it is an announced timeline-epoch change; both sinks restart at the announced position (announcement lead 64 slices, C-43; re-issue every 16, C-44). The source bumps the session number on restart and on timeline epoch change, and every consumer of a stream sees the same value (C-19).
- Default slice: 48 frames = 0.5 ms (C-02); one sample at 96 kHz is 10.4 µs.
- Runs whose numbers enter the constants table run in strict mode (C-147): real-time scheduling on the data and clock threads and memory locked (C-127), an unverified capability counting as absent (C-128). Strict mode is set in the operator policy of the commissioning configuration (C-165) and is hosted-only: the device stamps real-time scheduling and memory lock as present by construction. L3 writes C-79 and the device's E back, so every rig run is strict.
- A for each run is computed by the harness with the commissioning rule — the largest over sinks of maximum lateness + D + headroom (headroom C-52) — using the maximum lateness since the connection went live, never the rolling 60 s figure of C-182, and recorded with the results (C-148).
- Expected loss floor on a clean network: about one lost packet per stream per link every 13 hours (C-90), so a clean one-hour run normally shows none.

**How the rig observes the sinks.** Each sink's regime stamp is carried by a dedicated control message split into sections that each fit 1 252 bytes (C-17); it is multicast to the control group on any regime change, refreshed every 10 s, and fetched immediately by a unicast request (C-18). The rig fetches both stamps by unicast request at the start of each scenario and logs every multicast update during the run. The stamp carries per-connection counters — received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159, completing C-47) — the observed DSCP (C-47), and lateness percentiles 50th, 99th, 99.9th and maximum (C-160) over a rolling 60 s window, with the maximum since the connection went live beside them (C-182); the rig uses them to confirm what each disturbance did. The receive-timestamp source is one of `kernel`, `userspace`, `mac-hardware`, `interrupt` (C-131), each with one of the capability states present, absent, unverified, not requested (C-129). The byte layout of the stamp message belongs to the wire specification (epic 004).

**How the rig disturbs the sinks.** Packet loss on one path uses the reference source's public injection API (epic 017). Beyond it, the test-only hooks are: force a module id and nonce, force a stream onto a given timeline epoch, send a raw packet on a media or control socket; every injection is stamped (C-120). Soft restarts, link interruption and device power cycles are rig hardware (switch port control, a switchable outlet or PoE port) chosen and documented by this epic.

**Where things live.** Tool code is in the `hadronyche-tools` crate of the design set's repository (C-133). Rig documentation and `l3-results.md` go in `doc/design/reports/`; the constants table is `doc/design/02-constants.md`; nothing is written into the frozen POC tree (C-130).

**What L2 already established** (task 020, per sink alone): E, restart spread of D (Linux at most 25 µs over 20 restarts, C-74), marker offset stable within ±2 samples (C-75), zero slip after disturbances (C-76), wander (C-77). L3 does not repeat those; it measures the *difference* between two certified sinks.

**Decided since the ticket split.** The pass statistic is the 99.9th percentile with the maximum reported (C-79). The holdover margin is the sink's E (C-65, C-64, C-62). The refusal reason is `RequiredEExceeded` (C-103). Stamps are read at run time by unicast request (C-18), with health counters (C-47) and timestamp-source values (C-131, C-129). Results live in `doc/design/reports/` (C-130). The capture and tolerance unit is the single domain rate (C-01, C-80). The power-cycle and admission cases are now named scenarios, and every acceptance criterion carries an id. In the final round: the limit uses target E values (C-149); only commissioned sinks carry a numeric E (C-140); rig runs are strict (C-147); the harness computes and records A (C-148); the handover tolerance, comparison windows, loss patterns and post-restart window are the C-166 proposals; a resampler node lives only in an edge module and its delay is part of D (C-134, C-105); the follower cap C-35 is owned by L1 (task 019), so L3 has no follower-cap duty. After the final round: the remaining test parameters are the C-191 proposals — loss bursts 1 s apart on the media path only (sync loss is the holdover scenario), an announced step of 5 ms, each 10 s comparison window starting when the injected event ends, the handover triggered 60 s into the run, capture calibration repeatability judged against the mean of 10 calibrations, and the embedded device declaring its own restart spread, measured here at L3; the admission test sets the required E below the device's own declared E and the device is refused on its own figure whatever the Linux sink's E is (C-194). Since then: the no-step and recovery tolerance for the Linux–device pair is the larger of the two sinks' own tolerances — ±1 sample for the Linux sink (C-166), the device's declared restart spread for the device (C-166, C-191) — and both sinks' figures are reported beside each verdict (C-203). In the latest round: the handover is triggered as a holder loss with holdover observed, so no on-demand handover control is needed (C-212, with C-195 on holdover entry); a commissioned sink stays commissioned through holdover and its claim resumes after re-lock (C-180); the clock state and bound come from the servo core's time view (C-187); the stamp's lateness figures use a rolling 60 s window with the since-live maximum beside them, and A is computed from the since-live maximum (C-182); group-certification results taken inside a commissioning window are valid commissioning results (C-178). After that: C-212 applies to L3 explicitly, so the L3 handover run itself asserts emission error within the target E during holdover (C-202) and each sink's frequency continuity at takeover, a rate change within C-61 and no phase step, and the second holder-eligible Linux node is a required rig part; only a holder-eligible module with usable time (Locked, or in holdover with its claim not lapsed) may take over, keeping the epoch, which is bumped only when no candidate has usable time (C-213); A uses the since-live maximum lateness, never the rolling 60 s figure (C-148, now stated in the rule itself). Most recently: the L3 handover asserts the skew within the sum of the two E values through holdover and takeover and each sink's frequency continuity at takeover, and per-sink emission error is an L2 assertion only (C-220, replacing the per-sink C-202 assertion at L3); the 50 ppm of the L2 handover test is the measured natural crystal difference plus an injected drift, both recorded (C-223).

## Deliverables

**The rig** (documented well enough to rebuild, in `doc/design/reports/`):

- One Linux host: clock holder, reference source module emitting a marker stream, and the certified ALSA reference sink (task 018).
- One embedded reference device (task 026) on the same switch, with a switchable power supply (outlet or PoE port) for the power-cycle scenario.
- One two-channel capture device at 96 kHz, independent of both sinks, with a known inter-channel skew; Linux output on channel 1, device output on channel 2.
- A second Linux host on the same switch, holder-eligible, as the new holder for the handover scenario (required by C-212).
- All hosts on one IPv4 subnet (C-100).

Crate `hadronyche-tools`, binary `hadronyche-l3`:

- `skew.rs` — locates the same marker in both capture channels with sub-sample interpolation; produces the skew series.
- `run.rs` — a run eligibility gate (strict mode, commissioned sinks; C-147, C-140), the computation of A (C-148), and the scenarios:
  - `steady` — 1 hour.
  - `restart-linux`, `restart-device` — 20 restarts of each, skew re-measured after each; `restart-device` also measures the device's restart spread against the spread it declares (C-191).
  - `power-cycle-device` — 20 power cycles of the device, skew re-measured after each.
  - `handover` — clock holder lost 60 s into the run (C-191) and replaced by the second Linux host after both sinks pass through holdover (C-212, C-213); each sink's rate change and phase at takeover are measured on its own capture channel against the capture clock (C-212, C-61).
  - `loss` — injected packet loss on one sink's media path only, bursts 1 s apart (C-166, C-191).
  - `interrupt` — one sink's link interrupted for under 2 s, then restored.
  - `epoch` — an announced timeline step of 5 ms (C-191).
  - `holdover` — holder removed for 60 s; both sinks must stop claiming alignment once their bound exceeds their margin (their E).
  - `admission` — the group's required E set below the device's own declared E; the device is refused on its own figure whatever the Linux sink's E is (C-194).
- `stamp.rs` — fetches both sinks' regime stamps by unicast request and logs multicast updates (C-18).
- `report.rs` — skew percentiles and maximum per scenario, both sinks' stamps, declared E values, the A used, pass/fail.

```rust
pub struct SkewSample { pub position: u64, pub skew_ns: i64 }   // channel 2 minus channel 1
pub fn measure_skew(capture: &[[f32; 2]], rate: u32, markers: &MarkerPlan) -> Vec<SkewSample>;
pub struct SkewReport { pub p50_ns: i64, pub p999_abs_ns: i64, pub max_abs_ns: i64, pub limit_ns: i64, pub pass: bool }
```

`pass` is `p999_abs_ns <= limit_ns` with `limit_ns = E_linux + E_device`, both target values (C-79, C-149).

Document `doc/design/reports/l3-results.md` (C-130): the measured skew, the rig, the A used in each run (C-148), and the values written back to the constants table (C-79, the device's E).

## Dependencies

- **Needs:** the regime stamp message layout (004); the control mechanisms to announce a timeline step on demand and set a group's required E (005; the handover needs no control, being a holder loss, C-212); reference source with markers and the injection API (017); the Linux ALSA sink (018); its L2 certification and the marker analysis code (020); the embedded reference device with its declared D and E (026). The macOS and Windows reference sinks (021, 022) are needed only for the decision on including a hosted edge sink at L3.
- **Provides:** the cross-sink evidence and the group-E certification procedure that the commissioning tool (028) packages for operators. When packaged, the procedure drives test traffic, so it runs inside a declared commissioning window (C-107); its group-certification results are then valid commissioning results, each recorded with the window it was taken in, while ordinary production measurements are unqualified during the window (C-178).

## Acceptance criteria

1. **027-AC-1 (AC-29).** The 99.9th percentile of absolute skew between the hosted and the embedded sink is at most the sum of their E values, for 1 hour (`steady`), **and** after every one of 20 restarts of each sink (C-79); the maximum is reported.
2. **027-AC-2 — No drift.** Over the hour the skew shows no trend: a linear fit has a slope below 1 µs per hour (two crystals, one timeline).
3. **027-AC-3 — Handover.** Across a holder change the skew stays within the limit and shows no step: within the pair tolerance, the larger of the two sinks' own no-step tolerances (±1 sample for the Linux sink, C-166; the device's declared restart spread, C-191), with both figures reported beside the verdict (C-203), measured on 10 s comparison windows against the median of the first 60 s (C-166); the handover is triggered 60 s into the run as a holder loss and the comparison window starts when the new holder has taken over (C-191, C-212); a run in which either sink's holdover between holders was not observed is failed, not passed (C-212). At takeover each sink shows frequency continuity: its rate change, measured on its own capture channel against the capture clock, is within C-61 (1 ppm per second) and it shows no phase step beyond its own no-step tolerance (C-212, C-166, C-191), and the timeline epoch is kept (C-213). Through holdover and takeover the skew stays within the sum of the two sinks' E values, its p99.9 and maximum over that interval reported with the verdict (C-220, C-79, C-202); per-sink emission error is an L2 assertion only and is not reported here (C-220).
4. **027-AC-4 — Loss and interruption.** After injected loss on one path, and after an interruption shorter than 2 s, the skew returns to its prior value within the pair tolerance, the larger of the two sinks' own recovery tolerances (±1 sample, 10.4 µs, for the Linux sink, C-166; the device's declared restart spread, C-191), with both figures reported beside each verdict (C-203): the median of the 10 s window that starts when the event ends is compared with the median of the first 60 s of the run (C-166, C-191); a one-sample slip passes and is reported as a slip event (C-166). Injected loss is 1 % random plus bursts of 8 slices (C-166), the bursts 1 s apart and on the media path only (C-191).
5. **027-AC-5 — Epoch.** After an announced timeline step both sinks resume within the limit; neither plays content aligned across the two epochs.
6. **027-AC-6 — Holdover honesty.** With the holder removed, each sink stops claiming presentation alignment once its bound exceeds its margin, its own E (C-65); skew measured after that point is reported but not asserted. Both sinks stay commissioned throughout, and their claims resume after re-lock without re-commissioning (C-180).
7. **027-AC-7 (AC-30, rig part).** With the group's required E set below the device's own declared E, the device is refused at admission with the reason `RequiredEExceeded` (C-103) on its own figure, whatever the Linux sink's E is (C-194); the Linux sink plays alone when its own E is within the required E.
8. **027-AC-8 — Harness self-check.** On a synthetic two-channel file with a known 37 µs skew the tool reports 37 µs within 2 µs.
9. **027-AC-9 — Capture calibration.** The capture device's own inter-channel skew is measured (same signal into both channels) and subtracted; the value is in the report. Repeatability is judged against the mean of 10 calibrations (C-191).
10. **027-AC-10 — Stamps.** Both sinks' complete stamps (profile, timestamp source with its capability state, D and method, E, certification state) are in the report.
11. **027-AC-11 — Power cycles.** After each of 20 power cycles of the device, the 99.9th percentile of absolute skew is at most the sum of the E values, or the device's D or E is revised with a stated reason.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The skew analysis runs in CI on recorded two-channel files
- [ ] The rig is photographed or diagrammed and its parts listed
- [ ] `l3-results.md` is committed with a `[measured]` skew figure, or the device's E is revised with a stated reason

**Not applicable in this epic (C-125):** none. Every line has a subject here: the parsers are the capture file reader, the stamp file reader and the stamp message decoder (fuzzed for 60 s on every change and 1 hour nightly, corpus kept, C-95); the regime stamp fields touched are the ones the report reads and reproduces; the frozen POC line is satisfied because all code and documents go to the design set's repository (C-130, C-133).

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Is the skew within the sum of the two E values? | This task (measurement) | Skew exceeds the sum: one sink's D or E is dishonest; find which with L2 |
| Does the device's declared D hold across power cycles (not only soft restarts)? | This task (measurement, `power-cycle-device`) | Skew changes after a power cycle beyond the limit |
| Should L3 also include a macOS or Windows sink against the device? | This task, after tasks 021 and 022 | A hosted pair passes its L2 cross-check but a hosted-to-device pair fails |
| Inter-channel skew of the capture device at 96 kHz | This task (measurement) | Larger than 2 µs and unstable: choose another capture device |
| How does the marker content identify its timeline epoch, so the epoch check can tell pre-step from post-step markers? | Tasks 017 and 020 (marker stream and `MarkerPlan`) | Two markers from different epochs cannot be told apart in a capture |
| Which regime stamp fields expose a sink's clock state (the claim lapses when it becomes Unlocked, C-162) and its current clock bound? The values are published by the servo core (008a) through its time view (C-187) | Task 004 (stamp layout), with task 005 | The holdover scenario cannot read the clock state or the bound from the stamp |
| How does a test trigger an announced timeline step and a change of a group's required E on demand? (The handover is a holder loss and needs no control, C-212.) | Task 005 | None of the existing controls or the C-120 hooks can produce the event on demand |
| Recording file format (working assumption: uncompressed WAV/RIFF) | This task (ticket 027-02) | A capture tool in the rig cannot write the chosen format losslessly |
| Is the per-channel rate measurement against the capture clock fine enough to judge a rate change of 1 ppm per second (C-61)? | This task (ticket 027-25) | Measured resolution coarser than C-61, or apparent rate changes common to both channels (capture clock not stable enough) |
| Lock bound C-62 is pending simulation; the holdover margin and the expected 25 s lapse depend on it. The holdover entry delay (C-195) is also pending simulation and shifts when the lapse is expected after removal | Simulation (C-195: epic 008b) | Simulation changes C-62, and with it C-65; or sets the entry delay above C-37 |
