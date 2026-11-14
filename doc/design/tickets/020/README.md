# Epic 020 — L2 certification harness: tickets

## Summary
Epic 020 (phase P4, scope L, depends on epics 008, 016, 017, 018; blocks 021, 022, 027, 028) builds the harness that certifies a physical edge — a sink or capture module on a real audio device — and runs it for Linux. The crate `hadronyche-tools`, binary `hadronyche-l2`, generates position markers through the sink under test, captures them raw against the hardware frame counter (no capture resampler, to avoid the shared-crystal trap), and makes the sink follow an in-process reference clock holder with injected drift and steps (to avoid the idle-servo trap). From the captured markers it computes the emission error E (99.9th percentile over one hour, with the maximum, C-73), the restart spread of the plumbing delay D, marker-offset constancy, slip after disturbances (packet loss, a 50 ppm holder handover, an announced step), a ±100 ppm drift ramp (C-119), the steered-ratio and correction-term series and MTIE. It emits a stamped certification record. The Linux run targets E ≤ 50 µs (C-70) and D spread ≤ 25 µs (C-74); results go into `doc/design/reports/l2-results-linux.md` (C-130) and the constants table. Certification runs compute A with the commissioning rule (C-148) and run in strict mode (C-147). The macOS (021) and Windows (022) tasks reuse the harness with their own backends.

## Tickets

| Ticket | Title | Effort | Depends on |
|---|---|---|---|
| 020-01 | Crate and binary skeleton | XS | none |
| 020-02 | RawCapture trait and recording format | S | 020-01 |
| 020-03 | Position marker generation | S | 020-01, epic 017 |
| 020-04 | Marker detection with sub-sample interpolation | M | 020-02, 020-03 |
| 020-05 | Capture timeline and emission error | S | 020-04, epic 018 |
| 020-06 | Emission-error statistics and slip detection | S | 020-05 |
| 020-07 | Ratio series and MTIE | S | 020-05, epic 018 |
| 020-08 | Synthetic capture and harness self-check | S | 020-03, 020-06, 020-07 |
| 020-09 | Rig description | S | 020-01 |
| 020-10 | Linux ALSA raw capture backend | S | 020-02, epic 018 |
| 020-11 | Capture-delay calibration method | S | 020-09 |
| 020-12 | Rig calibration procedure | M | 020-04, 020-05, 020-09, 020-10, 020-11 |
| 020-13 | Certification record | S | 020-06, 020-07, 020-09 |
| 020-14 | Scenario runner, steady scenario and idle-servo control | M | 020-03, 020-05, 020-06, 020-07, 020-09, 020-10, 020-13, 020-26, epic 017, epic 018 |
| 020-15 | Restarts scenario and D repeatability | S | 020-14 |
| 020-16 | Loss and step disturbance scenarios | S | 020-14 |
| 020-17 | Handover 50 ppm scenario | M | 020-14, 020-16, 020-28 |
| 020-18 | Drift-ramp scenario | S | 020-14 |
| 020-19 | MTIE mask sufficiency | S | 020-07, epic 008 |
| 020-20 | Spectral criterion on a pure tone | S | 020-02, 020-08 |
| 020-21 | Linux reference rig build and documentation | S | 020-09, 020-11, 020-12, 020-28, epic 016 |
| 020-22 | Linux steady, control and restart runs | M | 020-14, 020-15, 020-21, 020-27 |
| 020-23 | Linux disturbance runs | M | 020-16, 020-17, 020-18, 020-22 |
| 020-24 | Linux results document and constants write-back | S | 020-19, 020-20, 020-21, 020-22, 020-23 |
| 020-25 | Fuzz CI jobs | XS | 020-02, 020-09 |
| 020-26 | Audition offset A for ladder runs | S | 020-01, epic 017, epic 018 |
| 020-27 | Strict mode for certification runs | S | 020-13, 020-14 |
| 020-28 | Second holder node for the handover | M | 020-09, 020-14, 020-27, epic 017 |

No ticket was removed: no ticket of this epic existed for the withdrawn rational-L rule (C-104, C-105 revised), which only touched the epic's note on the 2:1 edge resampler. 020-25 was added at the previous refinement for the fuzz budget of C-95; 020-18 was rewritten around C-119. 020-26 (the harness computes A with the commissioning rule, C-148) and 020-27 (certification runs in strict mode, C-127, C-128, C-129, C-147) were added at the previous refinement. New in this refinement: 020-28 (the takeover holder runs on a second holder-eligible Linux node, C-212 revised, C-213).

## Effort tally
- XS: 2 (0.5 day)
- S: 19 (19 days)
- M: 7 (17.5 days)
- Total: 28 tickets, about 37 days: scope L (C-126); below the 40-day split threshold of C-143

## Coverage

### Deliverables

| Epic deliverable | Tickets |
|---|---|
| Crate `hadronyche-tools`, binary `hadronyche-l2` | 020-01 |
| `marker.rs` — marker generation (reference source module), waveform choice | 020-03 |
| `marker.rs` — detection in raw capture, sub-sample interpolation | 020-04 |
| `rig.rs` — rig description, device rates, second holder-eligible Linux node (C-212) | 020-09 |
| `rig.rs` — calibration procedure | 020-11, 020-12 |
| `run.rs` — `steady` (1 h) | 020-14 |
| `run.rs` — `restarts` (20) | 020-15 |
| `run.rs` — `loss`, `step` | 020-16 |
| `run.rs` — `handover-50ppm` | 020-17 |
| `run.rs` — `holder` mode on the second Linux node (C-212) | 020-28 |
| `run.rs` — `drift-ramp` (±100 ppm at ≤ 0.5 ppm/s, C-119) | 020-18 |
| `run.rs` — computation of A by the commissioning rule (C-148) | 020-26 |
| `run.rs` — strict-mode check (C-147) | 020-27 |
| `analysis.rs` — emission-error percentiles and maximum, marker-offset series, slip detection | 020-05, 020-06 |
| `analysis.rs` — ratio series, correction-term checks (C-60, C-61, C-119), MTIE | 020-07 |
| `report.rs` — certification record with all listed fields, A and its inputs, strict-mode capability states, both stamps | 020-13 (with 020-26, 020-27) |
| Trait `RawCapture` / `RawBlock` with `resampler_in_use` | 020-02 (trait, file backend), 020-10 (Linux backend) |
| Recording format and parser fuzzing per C-95 | 020-02, 020-09, 020-25 |
| `doc/design/reports/l2-results-linux.md` with the Linux certification record (C-130) | 020-23 (record), 020-24 (document) |
| Constants write-back C-68, C-70, C-74, C-75, C-76, C-77, C-81 | 020-24 |

### Acceptance criteria

| Epic acceptance criterion | Tickets |
|---|---|
| 020-A1 = AC-26 (Linux): E ≤ 50 µs over 1 h, in strict mode with A per C-148 | 020-05, 020-06, 020-14, 020-22, 020-26, 020-27 |
| 020-A1: restart spread (max − min, 60 s window, C-166) ≤ 25 µs over 20 restarts | 020-15, 020-22 |
| 020-A1: marker offset within ±2 samples of the median of the first 60 s (C-166) | 020-06, 020-14, 020-22, 020-23 |
| 020-A1: zero slip after loss, 50 ppm handover, announced step (parameters C-166, C-191) | 020-06, 020-16, 020-17, 020-23 |
| 020-A1: MTIE ≤ 10 µs up to 1 s and ≤ the target E overall (C-149) | 020-07, 020-14, 020-22 |
| 020-A2 = AC-5 (hardware part): ratio ≤ 1 ppm/s and no offset step (±1 sample, C-166) during the 50 ppm handover; emission error within the target E while the sink is in holdover between holders (C-202); handover triggered as a holder loss to a holder on a second holder-eligible Linux node, continuity within C-61, a run with no observed holdover failed (C-212); epoch kept at takeover (C-213) | 020-07, 020-17, 020-23, 020-28 |
| 020-A3 Trap 1 demonstrably avoided (control run idle, certified run active) | 020-14, 020-22 |
| 020-A4 Trap 2 demonstrably avoided (refuse if capture resamples) | 020-02, 020-10, 020-14 |
| 020-A5 Harness self-check (30 µs error, 1-sample slip, within 2 µs) | 020-08 (with 020-06) |
| 020-A6 Rig calibration repeatable within ±1 sample of the mean of 10 calibrations (C-191) | 020-12, 020-21 |
| 020-A7 Record has complete stamps, names reference hardware (C-172), other hardware *unverified* | 020-13, 020-21, 020-23, 020-27 |
| 020-A8 Results written back with `[measured]` or target revised with reason, from strict-mode runs only (C-147) | 020-24, 020-27 |
| 020-A9 Drift ramp: E held over ±100 ppm at ≤ 0.5 ppm/s; correction term within C-60, C-61 | 020-07, 020-18, 020-23 |

### Definition of done

| Epic DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all code tickets (020-01 to 020-20, 020-22, 020-23, 020-25 to 020-28) |
| Every parser touched is fuzzed | 020-02 (recording), 020-09 (rig file), 020-25 (CI jobs, C-95) |
| Regime stamp fields touched are complete and documented | not applicable (C-125): no regime stamp field is added or changed; 020-13, 020-21, 020-23, 020-26, 020-27 and 020-28 mark the line not applicable with the reason |
| Design documents updated where behaviour differs, constants table first | 020-11, 020-19, 020-21 (reference hardware, C-172), 020-24 |
| No new open item without an owner and a falsifier | 020-11, 020-14, 020-18, 020-19, 020-20, 020-22, 020-24 |
| The frozen POC is not modified | 020-01, 020-10, 020-24 |
| Every new factual claim carries an evidence tag | all |
| Analysis runs in CI on recorded capture files | 020-02, 020-04 to 020-08, 020-12, 020-14, 020-20, 020-22, 020-23 |
| Rig documented well enough to rebuild | 020-21 (with the second holder node of 020-28) |
| Linux certification record committed | 020-24 |

### Open sub-questions

| Sub-question | Owner | Ticket |
|---|---|---|
| Capture-delay calibration without circularity | this epic | 020-11 (decision), 020-12 (implementation) |
| 50 µs on a stock kernel or real-time kernel needed | this epic | 020-22 |
| MTIE mask: two points or full mask (with epic 008) | this epic | 020-19 |
| Spectral criterion: needed, tone and threshold | this epic | 020-20 |
| Idle/active servo threshold | this epic, by measurement | 020-14 (provisional), 020-22 (measured) |

## Remaining gaps
1. **Threshold for "servo idle" vs "active" (020-A3).** Comes from the control run. Affects 020-14, 020-22. Answer: measurement (020-22).
2. **Sub-questions this epic answers by measurement:** capture-delay calibration method (020-11), stock or real-time kernel for 50 µs (020-22), MTIE mask sufficiency (020-19, with epic 008), spectral criterion (020-20). Answer: measurement in those tickets.
3. **Interfaces of the dependencies.** The injection API for drift, steps and making one in-process holder stop answering, which is how the handover is triggered (holder loss, C-212; epic 017; election rules from the control state machines, epic 005), the reference holder with injection run on the rig's second holder-eligible Linux node (C-212, 020-28), and read access to the sink's steered ratio, drift estimate, correction term (separately, as C-119 requires), declared D, the C-47/C-159 counters, the C-160 lateness percentiles (rolling 60 s window and maximum since live, C-182), the C-129 capability states and the sink's clock state and bound, to time the holdover interval of C-202 (epic 018; the clock state and bound are published by the servo core, epic 008a, C-187). What the trigger and the takeover must do is settled by C-212 and C-213; only the interface shape remains. Affects 020-03, 020-05, 020-07, 020-14, 020-16 to 020-18, 020-26 to 020-28. Answer: epics 017 and 018 (with 005 for the election, 008a for the published time view).

## Settled since the split
- **Former gap 1, criterion ids collide with list numbers:** the epic's criteria now carry ids 020-A1 to 020-A9 (020-A1 = AC-26, 020-A2 = AC-5); tickets cite them. Epic change.
- **Former gap 3, `RawCapture` cannot report a resampler:** `resampler_in_use` is now in the epic's trait. Epic change.
- **Former gap 4, recording format unspecified:** the epic lists a versioned, self-sufficient recording format as a deliverable. Epic change.
- **Former gap 5, marker waveform unspecified:** the epic leaves the choice to this task, recorded with its reason in `marker.rs` (020-03). Epic change.
- **Former gap 7, reference hardware list not given:** the epic makes this task create the list, starting with the Linux reference rig; 021 and 022 add theirs. Epic change.
- **Former gap 9, `drift-ramp` without pass criterion or sweep rate, and in conflict with C-60:** settled by C-119 (±100 ppm at no more than 0.5 ppm/s, pass is E held throughout; C-60 and C-61 bound the correction term, not the drift estimate). Added as criterion 020-A9.
- **Former gap 10, 50 ppm between two holders on one machine:** the epic now states it is produced by injected drift. Epic change (the handover trigger API stays in remaining gap 4).
- **Former gap 14, MTIE interval grid:** the epic now fixes a logarithmic grid from one marker period to the run length, always including 1 s. Epic change.
- **Former gap 16, value of A for test runs:** now settled by C-148: the harness computes A with the commissioning rule (largest over sinks of maximum lateness + D + headroom) and records it with the results. New ticket 020-26; 020-05, 020-13, 020-14, 020-22 changed.
- **Former gap 17, spectral criterion thresholds and tone:** carried as an open sub-question of the epic with owner (this task, 020-20) and falsifier. Epic change.
- **Former gap 18, locations:** the constants table is `doc/design/02-constants.md`; `l2-results-linux.md` and the rig document go in `doc/design/reports/` (C-130); the crate `hadronyche-tools` lives in the design set's repository and protocol types come from `hadronyche-core` (C-133). Epic, 020-01, 020-13, 020-21, 020-24 changed.
- **Last refinement's remaining gap 1, reference for the ±1-sample calibration tolerance (020-A6):** settled by C-191: repeatability is judged against the mean of the 10 calibrations (the provisional reading). 020-12, 020-21 and the epic changed.
- **Last refinement's remaining gap 2, disturbance parameters not fixed by C-166:** settled by C-191: loss bursts spaced 1 s apart, on the media path only (sync loss is its own scenario, later settled by C-202); announced step 5 ms; the 10 s comparison window after an event starts when the injected event ends (no separate settling time); handover triggered 60 s into the run. 020-06 (window start, new criterion 5), 020-16, 020-17, 020-23, 020-24 and the epic changed.
- **Last refinement's remaining gap 3, meaning of "both stamps":** settled by C-191: the regime stamps of the sink and of the holder it followed. 020-13 drops the provisional pair of tier stamps for a single certification-state field beside the two regime stamps; 020-23, 020-24 and the epic changed.
- **Last refinement's remaining gap 1, sync-loss scenario:** settled by C-202: L2 has no separate sync-loss scenario; the handover run gains one assertion, emission error within the target E while the sink's clock is in holdover between holders; `loss` stays on the media path. 020-17 (work, new criterion 4), 020-16, 020-23, 020-24 (applied parameters, criterion 5) and the epic (procedure, pass numbers, 020-A2, dependency interfaces; the open sub-question is removed) changed. No new ticket: the assertion belongs to 020-17, whose effort stays M.
- **Tolerance for a pair of different sinks:** C-203: the larger of the two sinks' own no-step and recovery tolerances, both sinks' figures reported beside each verdict. Every L2 run of this epic judges one sink, so it creates no work here; noted in the epic beside the cross-sink criterion that epics 021 and 022 apply.
- **Unlocked clock of a commissioned sink:** C-180: it stays commissioned, its alignment claim is suspended until relock and resumes without re-commissioning. Stated in the epic and 020-13 (the record is not withdrawn or re-issued); no new work.
- **Restart spread of an embedded device:** C-191: declared by the device and measured at L3, not by this harness. Noted in the epic and 020-17.
- **Last refinement's remaining gap 3, does the handover trigger put the sink in holdover:** settled by C-212: the handover is triggered as a holder loss (the old holder stops answering, about 2 s of holdover, C-37 plus C-33, then the new holder takes over); the run asserts emission error within the target E during holdover and frequency continuity at takeover, and a run in which holdover was not observed is failed, not passed. 020-17 drops the provisional "not exercised" reading (Work, criteria 1 and 4, new holdover-observed check; holdover entry delay pending per C-195, so only its occurrence is checked); 020-23, 020-24, the epic (procedure, pass numbers, 020-A2, dependency interfaces) changed. No new ticket: the trigger change stays in 020-17, effort M.
- **Revised C-212, continuity threshold and second node:** frequency continuity at takeover is a rate change within C-61 with no phase step, and the rig needs a second holder-eligible Linux node. The provisional reading of two in-process holders on one machine is dropped: holder B runs on the second node. New ticket 020-28 (`holder` mode and its control link, M); 020-17 (Work, criteria 1, 5, 7, new dependency), 020-09 (`holder_node`, criterion 5), 020-21 (second node in the rig, criterion 4), 020-14 (setup note) and the epic (Trap 1 note, procedure, pass numbers, 020-A2, deliverables, dependency interfaces, scope) changed. The 50 ppm of C-78 is produced by injected drift on B set against the measured natural difference between the two nodes' crystals.
- **Handover rulings C-220, C-221, C-223:** the 50 ppm is the measured natural difference between the two holders' crystals plus an injected drift, both recorded in the run report (C-223): the certification record gains a `handover_difference` block (020-13 Work and criterion 6), 020-17 (Context, criterion 5) and 020-28 (Context, Work, criterion 3) write both parts to it, and the epic's record deliverable and 020-A2 name it. After the loss every candidate is in holdover, so the lowest module id wins (C-221): B is given the lowest id among the rig's holder-eligible modules through the C-120 hook (020-17 Context, Work, criterion 1; 020-28 Context, Work, criterion 4; epic pass numbers). Per-sink emission error in holdover stays this epic's L2 assertion; L3 asserts what C-220 states (epic pass numbers, 020-17). Rig descriptions (epic procedure and `rig.rs`, 020-09, 020-21) already name the second holder-eligible Linux node of C-212. No new ticket; the tally is unchanged.
- **Who may take over as holder:** C-213: a holder-eligible module with usable time (Locked, or in holdover with its claim not lapsed, C-65, C-169) takes over with frequency continuity and the epoch kept; the epoch is bumped (C-113) only when no candidate has usable time. 020-17 waits for B to be Locked before the loss, checks that the epoch is kept (new check, criterion 6); 020-28 reports B's clock state; the epic (procedure, pass numbers, 020-A2) changed.
- **Window of the stamp's lateness figures:** C-182: percentiles and the cannot-meet-A alarm use a rolling 60 s window, the maximum since live beside them. 020-26 computes A from the maximum since the connection went live (new criterion 5); the epic's dependency interfaces changed.
- **Publisher of the clock state and bound:** C-187: the servo core (epic 008a) through the published time view; the harness reads them through the sink. Epic and remaining gap 3 changed.
- **Rate actuator:** C-189, C-197, C-210: the steered ratio the harness records is that of the edge's rate actuator, built with its trait by the kit (epic 024a), ±1 000 ppm range, delay in D; C-207 (resampler delay only in D, never in L). Epic, 020-07, 020-21 changed; no new work.
- **Commissioning window:** C-178 (with revised C-153): measurements taken in a window are valid commissioning results recorded with that window. 020-13 gains an optional `commissioning_window` field; 020-24 and the epic's packaging note changed.
- **Presentation details:** C-206: the record's JSON and Markdown layout and file names are decided by this epic and documented in `report.rs`. 020-13 and the epic changed.
- **Former gap 19, scope label:** recomputed from the ticket tally per C-126: L (28 tickets, about 37 days).
- **Previous remaining gap 1, does a 1-sample slip pass on Linux:** settled by C-166: it passes and is reported as a slip event. 020-06, 020-08, 020-13, 020-16, 020-17, 020-23 and the epic changed.
- **Previous remaining gap 2, reference for the ±N-sample tolerances:** settled for the run tolerances (C-75, C-76, handover) by C-166: against the median of the first 60 s, with 10 s comparison windows. The calibration tolerance of 020-A6 was settled later by C-191.
- **Previous remaining gap 3, D per restart and its spread:** settled by C-166 and C-170: D is the delay from hand-off to emission, measured over a 60 s window after each restart (declared D plus the settled emission error); the spread is max − min. 020-15, 020-22 changed.
- **Previous remaining gap 4, disturbance parameters:** partly settled by C-166 (loss 1 % random plus bursts of 8 slices; 10 s comparison windows). The rest was settled later by C-191.
- **Previous remaining gap 5, "no step" threshold at the handover (020-A2):** settled by C-166: ±1 sample on Linux, the platform restart spread elsewhere. 020-17, 020-23, the epic changed. The idle/active threshold stays a measurement (remaining gap 2).
- **Which E the pass criteria mean:** C-149: the platform target E, never the E measured in the same run. 020-07 (`wander_verdict` takes the target), 020-13, 020-14, 020-18, 020-19, 020-22, 020-23 and the epic changed.
- **E budget for macOS and Windows:** C-150 (macOS 40 + 40 + 20 µs, Windows 100 + 100 + 800 µs, proposals) added to the epic's target table, 020-11 and 020-13.
- **Strict mode for certification runs:** C-127, C-128, C-129, C-147. New ticket 020-27; 020-02, 020-13, 020-14, 020-22, 020-23, 020-24 changed.
- **Reference hardware:** C-172: proposed by the implementing team, approved by the product owner, recorded in the constants table. 020-13, 020-21 and the epic changed.
- **Revised C-104, C-105 and new C-134:** L is a whole number of samples; the edge's 2:1 resampler exists only inside the edge module and its conversion delay is part of D. The epic's rates note, 020-09 and 020-21 changed. No ticket held rational-L handling, so none was deleted.
- **Former gap 20** was merged into the reference-tolerance gap, now settled except for 020-A6. **Former gaps 2, 6, 8, 11, 12, 13 and 15** are settled or narrowed as listed above; what remains is in Remaining gaps.

Other rulings applied to the tickets: C-01 and C-80 (96 kHz is the only wire rate; device rates are rig properties; tolerances in domain samples) in 020-02, 020-03, 020-04, 020-05, 020-09, 020-10, 020-21; C-95 (fuzz budget) in 020-02, 020-09, 020-25; C-103 (no certified E means E unknown) in 020-13, 020-24; C-47, C-38, C-102, C-108, C-111 (loss counters and thresholds) in 020-16; C-50, C-63 (admission after restart, lock timeout) in 020-14, 020-15; C-119 in 020-07, 020-14, 020-17, 020-18, 020-23; C-120 (stamped injections) in 020-13; C-107 (commissioning window) in 020-24; C-125 (not-applicable DoD line) in 020-13, 020-21, 020-23, 020-26, 020-27; C-133 (code locations) in 020-01; C-135, C-159 (other-epoch slices, full counter list) in 020-13, 020-16; C-140 (only commissioned gives a numeric E) in 020-13, 020-23; C-132 (`[standard]` tag) in 020-19, 020-20; C-153 (commissioning window life cycle) in the epic; C-160, C-173, C-174 (lateness percentiles, alarms) in 020-26; C-170 (definition of D and its capture mirror) in 020-05, 020-09, 020-11, 020-15, 020-21; C-180 (unlocked clock of a commissioned sink) in 020-13 and the epic; C-191 (L2 and L3 test parameters, completed) in 020-06, 020-12, 020-13, 020-16, 020-17, 020-21, 020-23, 020-24 and the epic; C-202 (sync loss at L2) in 020-16, 020-17, 020-23, 020-24 and the epic; C-203 (pair of different sinks) in the epic only; C-212 (revised) and C-213 in 020-09, 020-14, 020-17, 020-21, 020-28 and the epic; C-148 in 020-02, 020-05, 020-13, 020-14, 020-22, 020-23, 020-24, 020-26 and the epic. C-35's owner change and the other rules from C-127 to C-176 (including C-144, which assigns duties to epic 015 only) create no work in this epic. C-195 (holdover entry delay, pending) is noted in 020-17. C-177, C-179, C-181, C-183 to C-186, C-188, C-190, C-192 to C-194, C-196, C-198 to C-201, C-204, C-205, C-208, C-209, C-211 and the late revisions of C-08, C-141, C-169 and C-184 have no subject in this epic (the harness neither defines thread layouts nor changes A during a run, and its ~2 s handover holdover is far inside the claim lapse of C-65 and C-169).
