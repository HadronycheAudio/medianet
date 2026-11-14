# Epic 018 — Linux ALSA edge: tickets

## Summary
Epic 018 (phase P4, scope L: 25 tickets, about 29 days; depends on epics 004, 008, 009, 012, 013, 015, 016, 024a; blocks 020, 021, 022, 026, 027) builds the first physical edge: a Linux ALSA sink and capture module in `hadronyche-os-linux` (`alsa/sink.rs`, `capture.rs`, `status.rs`, `declared.rs`). It also builds the shared two-loop timebase in `hadronyche-core` `edge/` (`no_std`, no allocation, no libm): device-clock regression (`DeviceLine`, loop 2: S → H), resampler-ratio control (`EdgeTimebase`, with an explicit `update` step and a `reset` for departures), and it consumes the rate-actuator trait that the kit defines with its implementation (epic 024a, ratio range ±1 000 ppm, C-210), implemented in v1 only by the kit's steerable rate actuator, separate from the fixed-ratio 2:1 / 1:2 resampler node (C-189). This epic builds no resampler: both come from the arithmetic kit, epic 024a (C-197); their delays count in D, never in L, and the manifest reports the conversion delay on the device-facing port (C-207). The wire carries only the 96 kHz domain rate (C-01); a 48 kHz device is converted 2:1 at the edge through that fixed node (C-80). The sink follows the edge contract `emit(p) = T(p) + A`, `submit(p) = emit(p) − D`, self-monitors its emission error E (99.9th percentile plus maximum, C-73; Linux target 50 µs, C-70), stops claiming alignment after 25 s of holdover (C-65), or once its published bound exceeds 50 µs when its E is unknown (C-169), while keeping its certification state (C-180), re-aligns on a changed A at its own position one lead ahead and reports its current A upstream (C-186, C-198, C-214), applies the settled input-health rules (C-101, C-102, C-108, C-109, C-111, C-135), raises the connection alarms of C-173 including "cannot meet A" (C-174), is refused at admission when its declared E exceeds the group's required E or is unknown (AC-30, C-103; numeric only when commissioned, C-140), honours strict mode (C-127, C-128, C-129), treats sleep as a departure (C-124), and fills the regime stamp including the C-47 and C-159 counters and the C-160 lateness percentiles over a rolling 60 s window with the maximum since live (C-182). This is the reference sink that every other platform is measured against.

Epic acceptance criteria are identified 018-A1 to 018-A13, to avoid confusion with the global AC-n criteria (018-A11 strict mode and 018-A12 "cannot meet A" added earlier; 018-A13 changed A added in this update).

## Tickets
| Id | Title | Effort | Depends on |
|---|---|---|---|
| 018-01 | Edge module skeleton and traits | XS | epic 024a |
| 018-02 | DeviceLine regression (loop 2) | M | 018-01 |
| 018-03 | EdgeTimebase ratio and quality | S | 018-02, epic 008 |
| 018-04 | Emission error computation | XS | 018-03, epic 008 |
| 018-05 | E statistics accumulator | S | 018-01 |
| 018-06 | Alignment claim and certification state | S | 018-03, epic 008 |
| 018-09 | No-libm symbol inspection in CI | XS | 018-02, epic 024a |
| 018-10 | ALSA status pairs and DeviceClock | S | 018-01, epic 015 |
| 018-11 | Declared delay D and commissioning persistence | S | 018-10, epic 024a |
| 018-12 | AlsaSink PCM output path | M | 018-10, 018-11, epic 015, epic 016, epic 024a |
| 018-13 | AlsaSink position ring and submit scheduling | M | 018-12, epic 009 |
| 018-14 | AlsaSink rate loop, E self-monitoring and alarms | M | 018-03, 018-04, 018-05, 018-06, 018-13 |
| 018-15 | Admission with self-declared E | XS | 018-14, epic 012 |
| 018-16 | AlsaCapture read path and stamping | M | 018-03, 018-10, 018-11, epic 004, epic 008, epic 015, epic 024a |
| 018-17 | AlsaCapture bundle above 6 channels | S | 018-16, epic 004 |
| 018-18 | Regime stamp fields for ALSA edge | S | 018-11, 018-14, 018-16, epic 013 |
| 018-19 | Emission trace export for L2 verification | S | 018-14 |
| 018-20 | Reference hardware and D method documentation | S | 018-02, 018-03, 018-11, 018-18 |
| 018-21 | Sleep as departure (C-124) | S | 018-06, 018-14, 018-16, 018-25, epic 012 |
| 018-22 | AlsaSink input health: missing, late, interrupted, dead, Demoted, other-epoch; connection alarms (C-101, C-102, C-108, C-109, C-111, C-135, C-159, C-173) | S | 018-13, epic 009 |
| 018-23 | Stream health counters in edge stamps (C-47, C-159) | S | 018-16, 018-18, 018-22, epic 013 |
| 018-24 | Sink input lateness statistics and "cannot meet A" alarm (new, C-52, C-160, C-173, C-174) | S | 018-11, 018-14, 018-22, epic 009, epic 013, epic 015 |
| 018-25 | Strict mode and capability states for the ALSA edge (new, C-127, C-128, C-129, C-146, C-147, C-165) | S | 018-12, 018-16, 018-18, epic 015, epic 016 |
| 018-27 | Conversion delay in the manifest, outside L (new, C-207) | XS | 018-11, 018-12, 018-16, epic 012 |
| 018-28 | AlsaSink re-alignment on a changed A (new, C-186, C-198, C-214) | S | 018-13, 018-24, epic 009 |

Deleted in this update (C-197): 018-07 (SoftwareResampler, the steerable rate actuator), 018-08 (resampler filter length decision) and 018-26 (fixed-ratio 2:1 / 1:2 resampler node). The arithmetic kit (epic 024a, moved to phase P4 ahead of this epic) builds both resamplers and their filter design once; every edge uses the kit's. Their numbers are not reused. Earlier, the withdrawn rational-L rule (C-105 revised) left nothing to remove: no ticket handled rational latency, rounding of L or fractional delay lines.

## Effort tally
- XS: 5 (5 × 0.25 = 1.25 days)
- S: 15 (15 days)
- M: 5 (5 × 2.5 = 12.5 days)
- Total: 25 tickets, about 29 days (28.75) → scope L (C-126)

## Coverage

### Deliverables
| Deliverable | Tickets |
|---|---|
| `edge/` `DevicePair`, `DeviceClock` trait | 018-01 |
| `edge/` `DeviceLine` (update, frames_at, residual_ns, reset) | 018-02 |
| Rate-actuator trait | not this epic: defined by epic 024a with its implementation (C-210); a test double in 018-01; consumed by 018-11, 018-14, 018-16 |
| `edge/` `EdgeTimebase::update`, `ratio`, `quality_ns` | 018-03 |
| `edge/` `EdgeTimebase::emission_error_ns` (with `TimeView`, returns `Option`) | 018-04 (statistics: 018-05) |
| `edge/` `EdgeTimebase::reset`, departure (C-124) | 018-21 |
| `edge/` `EmissionStats`, `AlignmentClaim`, `CertificationState` | 018-05, 018-06 |
| `hadronyche-kit` steerable rate actuator and fixed-ratio 2:1 / 1:2 node, filter design and cost (C-106, C-189) | not this epic: built by epic 024a (C-197); consumed by 018-11, 018-12, 018-14, 018-16 |
| `alsa/sink.rs` — position ring (sized to A, C-96) → resampler → PCM write | 018-12, 018-13 |
| `alsa/sink.rs` — `submit(p)` scheduling; A not configured | 018-13 |
| `alsa/sink.rs` — re-alignment on a changed A at its own position one lead ahead (C-214); current A in receiver reports (C-186, C-198) | 018-28 |
| `alsa/sink.rs` — input health (C-101, C-102, C-108, C-109, C-111, C-135); connection alarms on interrupted, dead, Demoted (C-173) | 018-22 |
| `alsa/sink.rs` — self-monitoring of E; claim-lapsed alarm | 018-05, 018-06, 018-14 |
| `alsa/sink.rs` — input lateness percentiles over a rolling 60 s window plus maximum since live (C-160, C-182); "cannot meet A" alarm (C-174, C-182) | 018-24 |
| `alsa/sink.rs`, `capture.rs` — strict mode and capability states (C-127, C-128, C-129, C-165); refusal in every mode on a failed or unverified pin that seqlock safety depends on (C-177, C-201) | 018-25 (best effort: 018-12) |
| `alsa/sink.rs`, `capture.rs` — "the interface is held" (C-175) | 018-12, 018-16 |
| Admission: numeric E only when commissioned (C-140); refusal alarm (C-173); commissioned kept, claim suspended while unlocked (C-180) | 018-06, 018-15 |
| `alsa/sink.rs`, `capture.rs` — sleep as departure | 018-21 |
| `alsa/capture.rs` — PCM read → resampler → stamped slices; session numbers (C-19) | 018-16 |
| `alsa/capture.rs` — bundle when wider than 6 channels (C-04, C-08, C-209) | 018-17 |
| `alsa/status.rs` — (frames, host time) pairs; ALSA `DeviceClock` | 018-10 |
| `alsa/declared.rs` — D composition (C-170), including both resampler delays (C-189); commissioning persistence inside a commissioning window, recorded with its window (C-107, C-153, C-178) | 018-11 |
| Manifests: conversion delay on the device-facing port, kept out of L (C-207) | 018-27 |
| Stamp fields (device, driver, rate, buffer/period, D + components + method, E p99.9 + max with unqualified flag (C-153, C-178), residual, ratio statistics, certification state, capability states (C-129)) | 018-18 (state: 018-06; capabilities: 018-25) |
| Stamp per-connection counters (C-47, C-159), unqualified while a window is open (C-178) | 018-23 |
| Stamp lateness percentiles and maximum since live (C-160, C-182), unqualified while a window is open (C-178) | 018-24 |
| Verification tooling in `hadronyche-tools` (C-133): trace, E, MTIE, D spread; C-120 scenarios with C-166 and C-191 parameters; holder-loss handover (C-202, C-212), with frequency continuity per C-61 and a check for the rig's second holder-eligible Linux node; formats and output documented here (C-206) | 018-19 |
| Open sub-questions owned by this epic: phase term; residual statistic (resampler quality versus cost moved to epic 024a, C-197) | 018-03; 018-02 |
| Open sub-questions owned by others (epic 020, 008a, 004, 013, 009) | notes in 018-02, 018-03, 018-06, 018-11, 018-17, 018-18, 018-19, 018-22 |
| Reference hardware proposal (C-172) | 018-20 |

### Acceptance criteria
| Epic criterion | Tickets |
|---|---|
| 018-A1 Core unit tests (±100 ppm, 50 µs jitter, < 1 ppm after 30 s; 20 µs jitter step < 0.1 ppm) | 018-02, 018-03 |
| 018-A2 Self-monitored E per submitted period, blank until locked | 018-04, 018-05, 018-14, 018-18 |
| 018-A3 Holdover lapse after 25 s, or bound above 50 µs when E is unknown (C-169); certification state kept (C-180) | 018-06, 018-14 |
| 018-A4 Refusal (AC-30) and unknown E (C-103) | 018-15 |
| 018-A5 Capture stamping (checked at L2) | 018-16 (L2 run by epic 020) |
| 018-A6 Contribution to AC-26, including drift ramp (C-119), target E (C-149), strict mode (C-147), C-166 and C-191 parameters, holder-loss handover with E held in holdover (C-202, C-212) (checked at L2 by epic 020) | 018-11 (D spread), 018-13 (loss, step), 018-14 (handover, drift ramp, E), 018-16 (marker), 018-19 (trace, MTIE, spread tools, scenarios) |
| 018-A7 No libm in resampler or regression | 018-02, 018-09 (kit resamplers as linked; the resamplers are epic 024a's, C-197) |
| 018-A8 Unprivileged best-effort, stamps what it lacks (C-129) | 018-12, 018-18 |
| 018-A9 Input health, C-173 alarms, C-47 and C-159 counters | 018-22, 018-23 |
| 018-A10 Sleep as departure (C-124) | 018-21 |
| 018-A11 Strict mode (C-127, C-128, C-129); pin refusal in every mode (C-177, C-201) | 018-25 |
| 018-A12 "Cannot meet A" (C-174, rolling 60 s window C-182) | 018-24 |
| 018-A13 Changed A (C-186, C-198, C-214) | 018-28 |

### Definition of done
| DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all (privileged strict-mode test in the privileged nightly job, C-146: 018-25) |
| Every parser touched is fuzzed (C-95 budget) | 018-10, 018-11, 018-18, 018-19, 018-23 |
| Regime stamp fields complete and documented | 018-18, 018-21, 018-23, 018-24, 018-25 |
| Design documents updated, constants table first | 018-20 |
| No new open item without owner and falsifier | 018-02, 018-03, 018-06, 018-14, 018-20 |
| Frozen POC not modified | not applicable in this epic (C-125: nothing is ported from the POC); kept as a check in 018-01, 018-20 |
| Every new factual claim carries an evidence tag | most; final check 018-20 |
| `edge/` builds `no_std` with no allocation | 018-01 to 018-06, 018-09, 018-21 |
| Named reference hardware configuration recorded | 018-20 (proposed by this epic after P0, approved by the product owner, recorded in the constants table, C-172) |
| D measurement method documented | 018-20 |

## Remaining gaps
1. **`TimeView` interface.** The epic states what it needs (domain rate in host time, clock bound, lock and holdover state, `T(p)`), but the type is defined by the servo core. C-187 settles that the lock state and bound are published by epic 008a through the time view; the rest of the type is that epic's. Affects 018-03, 018-04, 018-06, 018-14. Answer: servo core epic 008a.
2. **Phase correction in the ratio.** Whether a bounded phase term is needed beyond the slope quotient. Affects 018-03, 018-14. Answer: this epic, ticket 018-03, by simulation on fakes (open sub-question with falsifier).
3. **Rate-loop bandwidth value** (C-68 says only "well below 1 Hz", pending measurement). Affects 018-02, 018-03, 018-14. Answer: a measurement, L2 Linux (epic 020).
4. **Regression residual statistic** (RMS, percentile or maximum; no-libm constraint). Affects 018-02, 018-03, 018-18. Answer: this epic, ticket 018-02 (open sub-question with falsifier).
5. **Stamp change classes and "ratio statistics".** Which edge fields are regime changes (immediate multicast) and which ride the 10 s refresh (C-18), and what "ratio statistics" contains. 018-18 assumes a split and min/max/mean ppm over 10 s. Affects 018-18, 018-23, 018-24. Answer: stamp format epic 013.
6. **Bundle wire format for captures wider than 6 channels.** Channel structure is settled (C-04, C-08); the wire format (shared position per slice, member order) is not yet published. Affects 018-17. Answer: wire specification epic 004.
7. Settled by C-225 (sink and the ratchet); see "Settled since the split", item 36. The number is kept so that references stay valid.

## Settled since the split
1. *`EdgeTimebase::ratio(&self, …)` cannot hold filter state* — epic change: explicit `update(&mut self, view, line)`, `ratio(&self)` returns the filtered value.
2. *`emission_error_ns` has no `TimeView`* — epic change: the method takes `view: &TimeView` and returns `Option<i64>` (blank until locked).
3. *Persistence format and location of the commissioned part of D* — epic change (versioned file per device, driver, device rate in the state directory) plus C-107 (written only inside a commissioning window) and C-153 (window life cycle: announced with expiry, closes at expiry or explicit close; measurements in a window unqualified).
4. *AC-5 and AC-6 listed here but checked at L2* — epic change: criteria 018-A5 and 018-A6 marked "checked at L2"; this epic delivers mechanisms, fake-backend tests and tooling.
5. *No owner for MTIE tooling* — epic change: verification tooling is a deliverable of this epic (018-19), in the `hadronyche-tools` crate (C-133).
6. *Behaviour when A is not configured* — C-55 plus epic change: the sink does not start emitting and reports a named reason until A is set.
7. *How the self-declared E is configured* — C-103: a configured value; absent means unknown, refused with `RequiredEExceeded` where a required E is set, admitted otherwise.
8. *Constants table not consulted* — epic aligned with the table: single 96 kHz wire rate (C-01), 6-channel streams without per-channel id byte (C-04), 2:1 conversion at the edge (C-80), plus C-18, C-19, C-47, C-95, C-96, C-101 to C-111, C-119, C-120, C-123 to C-126 written into the epic Background.
9. *Which certification state allows a numeric E* (former gap 6, first half) — C-140: *commissioned* only; *uncertified* and *self-monitored* declare unknown. 018-06 adds `declares_numeric_e()`; 018-15 declares unknown unless commissioned. The fallback state on a lapse was settled later by C-180 (item 22).
10. *Threshold for "cannot meet A"* (former gap 7) — C-174: maximum lateness + D + headroom exceeds A in each of 10 consecutive 1 s windows; headroom per C-52. Moved out of 018-14 into new ticket 018-24.
11. *Reference hardware selection* (former gap 9) — C-172: proposed by the implementing team after P0, approved by the product owner, recorded in the constants table. 018-20 now makes the proposal.
12. *Resampler and structural latency* — C-105 (revised): L is always a whole number of samples and the resampler node is offered only for 2:1 and 1:2; C-134: a resampler node appears only inside an edge module, and its delay is part of D; C-170 defines D. Cited in the epic, 018-03, 018-07, 018-08, 018-11, 018-16, 018-20. No rational-L handling existed in this epic, so nothing was removed.
13. *Real-time features lacking* — C-127 to C-129, C-165: strict mode set by operator policy, requires RT scheduling on data and clock threads and memory lock, refuses unverified; four capability states in the stamp. 018-12 and 018-18 now use the four states; new ticket 018-25; new criterion 018-A11. C-146: privileged tests in the nightly privileged job. C-147: L2 runs are strict.
14. *Counter set and lateness statistics* — C-159 completes C-47 (adds other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds); C-135 defines other-epoch handling; C-160 sets lateness percentiles. 018-22, 018-23 updated; 018-24 new.
15. *Alarms* — C-173: connection alarm on refusal, interrupted, dead, demoted, cannot-meet-A (control-degraded is the hosted runtime's). 018-15, 018-22, 018-24.
16. *Holdover lapse* — C-162: Unlocked with time unusable after the lapse; E blank. 018-06, 018-14.
17. *L2 parameters and pass meaning* — C-149 (target E, not measured E), C-148 (A computed by the epic 020 harness), C-166 (loss pattern, windows, slip events, D spread max − min). 018-13, 018-14, 018-16, 018-19.
18. *Interface held* — C-175: device opened by another process; reported by name in 018-12 and 018-16.
19. *Orphan duties* — C-144: the hosted backend (epic 015) emits the gap slice when the capture misses its deadline, raises the processing-overrun event, and sends the regime stamp message. Noted as out of scope in 018-16, 018-18, 018-23; no ticket here.
20. *Calibration factor owner* — C-176: the DSL conformance suite (epic 025). 018-08 out of scope updated.
21. *Locations* — C-130, C-171: specifications in `doc/design/spec/`, reports in `doc/design/reports/`. 018-08, 018-19, 018-20.
22. *Certification state after a claim lapse* (former gap 6) — C-180: a sink whose clock is unlocked stays *commissioned*; its alignment claim is suspended and does not count towards a group's required E until relock, then resumes without re-commissioning. 018-06 drops the provisional "Uncertified until restored" branch; 018-14, 018-15, 018-18 updated.
23. *Window of the lateness figures* — C-182: percentiles (C-160) and the "cannot meet A" alarm (C-174) use a rolling 60 s window, with the maximum since the connection went live beside them. 018-24 updated (reclaimable latency, C-157, is not published by this epic).
24. *Two kinds of resampler* — C-189: the fixed-ratio 2:1 / 1:2 DSL node (whole-sample delay, C-105) is separate from the edge's steerable rate actuator (ppm around nominal); both delays are part of D. Corrects the provisional reading that one resampler did the 2:1 conversion and the ppm correction: 018-07 became the actuator only and ticket 018-26 the fixed node; both were deleted later by C-197 (item 25).
25. *Who builds the resamplers* — C-197: the arithmetic kit (epic 024a, moved to phase P4 ahead of this epic) builds both the fixed 2:1 / 1:2 node and the steerable rate actuator once; every edge uses the kit's. Tickets 018-07, 018-08 and 018-26 deleted (the open sub-question on resampler quality versus cost moves to epic 024a); dependency on epic 024a added to the epic, 018-09, 018-11, 018-12, 018-16; 018-01, 018-03, 018-14, 018-20 updated.
26. *Resampler delay and L* — C-207: an edge's resampler delay is counted only in D, never in L; the manifest reports the conversion delay separately on the device-facing port. Cited in the epic, 018-11, 018-16, 018-20; new ticket 018-27 for the manifest field.

27. *Home of the rate-actuator trait* (former gap 8) — C-210: defined in the kit epic 024a with its implementation, ratio range ±1 000 ppm around the nominal ratio; every edge consumes it. The trait definition is removed from the epic Deliverables and from 018-01 (now a test double of the kit's trait; 018-01 depends on epic 024a); 018-14 and 018-16 cite the kit's trait and range.
28. *L2 handover and parameters* — C-191 (loss bursts 1 s apart, media path only; 5 ms announced step; comparison window from the end of the event; handover at 60 s), C-202 (E within target during holdover; no separate sync-loss scenario), C-212 (handover triggered as a holder loss, holdover must be observed). Epic 018-A6, 018-13, 018-14 and 018-19 updated; the trace now records the clock state.
29. *Commissioning window qualification* — C-178 (completes C-153): production figures in the stamp (E, lateness, loss counters) are unqualified while a window is open; commissioning measurements (D, restart spread) are valid and recorded with their window. 018-11, 018-18, 018-23, 018-24 updated.
30. *Pin failure and seqlock safety* — C-177, C-201: a failed or unverified pin on which seqlock safety depends refuses start in every mode. 018-25 and 018-A11 updated; 018-12 points to it.
31. *Holdover lapse with unknown E* — C-169: an uncertified or self-monitored sink takes E as 50 µs and stops claiming once its published bound exceeds it. 018-06 and 018-A3 updated.
32. *Changed A* — C-186 (the sink re-aligns at an announced position, ring resized) and C-198 (its receiver reports carry its current A). New ticket 018-28 and criterion 018-A13; the sink's re-alignment position was gap 8, since settled by C-214 (item 34).
33. *Smaller points* — C-187: the lock state and bound are published by epic 008a (gap 1 owner refined); C-209 with revised C-08: the controller member counts towards 32, and the capture has none, so 192 channels stands (epic, 018-17); C-206: trace format and tool output are this epic's, documented in 018-19.

34. *Re-alignment position of a sink after a changed A* (former gap 8) — C-214: the sink switches at its own position, one lead (C-43) ahead, as a fan-in does (C-208); it has nobody to announce it to. Corrects the provisional reading in 018-28 (position taken from the epic 009 ring as an announced position): 018-28 Work and Acceptance criteria now pick the switch position one lead ahead of the sink's own position and emit no announcement; epic Background, 018-A13 and the open sub-question row updated.
35. *Second holder-eligible node for the handover* — C-212 (revised): frequency continuity at takeover means a rate change within C-61 and no phase step, and the rig needs a second holder-eligible Linux node. 018-14 and 018-19 acceptance criteria now check C-61 and the phase step; the 018-19 handover script refuses to start without the second node; providing the node is epic 020's. No new ticket.
36. *Sink and the ratchet* (former gap 7, owned by epic 009) — C-225: a sink has no output to announce in, so it takes a ratchet step at its own position one lead (C-43) ahead, as it does for a changed A (C-214). Corrects the provisional reading that a sink cannot ratchet and Demotes a persistently late input directly: the epic Background and open sub-question row, 018-22 (Context, Work 4, AC 4) and 018-28 now ratchet first and Demote only at the ceiling (C-109).
37. *Clamp in a sink and how a change of its own A arrives* — C-222: the clamp of C-211, and C-216 with it, applies to the sink; C-224: a commissioned change of the sink's own A comes through the commissioning tool inside a declared window, after the buffer-memory check, as an announcement to the group's members. Epic Background and 018-28 (Context, Work 4 and 5, AC 6, Out of scope) updated.
38. *Handover difference and L2-only assertion* — C-223: the 50 ppm is the measured natural difference between the two holders' crystals plus an injected drift, both recorded in the run report; the 018-19 handover script injects the remainder and writes both parts (Work 4, AC 9), and 018-14 AC 4 notes that on fakes the whole 50 ppm is injected. C-220: per-sink emission error in holdover stays an L2 assertion (018-19 Context); C-212 unchanged. C-221 has no subject here. No new ticket; the tally is unchanged.

Partly answered and kept above: ratio statistics (refresh interval from C-18; content still open, gap 5), bundle format (channel structure from C-04 and C-08, wire format open, gap 6).

New work created by rulings: 018-21 (C-124), 018-22 (C-101, C-102, C-108, C-109, C-111; split from 018-13), 018-23 (C-47), 018-24 (C-52, C-160, C-173, C-174), 018-25 (C-127, C-128, C-129, C-146, C-147, C-165), 018-26 (C-189, since deleted by C-197), 018-27 (C-207), and in this update 018-28 (C-186, C-198).
