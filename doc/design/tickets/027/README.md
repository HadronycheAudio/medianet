# Epic 027 — L3 cross-sink rig: tickets

## Summary
Epic 027 (phase P6, depends on epics 004, 005, 017, 018, 020, 021, 022, 026; blocks 028) builds the rig and runs the measurement that proves cross-sink simultaneity, the top rung (L3) of the certification ladder `[decision]`. The certified Linux ALSA reference sink and the embedded reference device play the same marker stream; their analogue outputs go into channels 1 and 2 of one capture device at 96 kHz, so skew is read on one capture clock. The pass criterion (C-79, pending measurement) is that the 99.9th percentile of absolute skew is at most the sum of the two sinks' target E values (Linux target 50 µs, C-70; device E declared per device; never the E measured in the same run, C-149), for 1 hour and across restarts of either, with the maximum reported beside it. The work is the `hadronyche-l3` binary in crate `hadronyche-tools` (`skew.rs`, `run.rs`, `stamp.rs`, `report.rs`), a documented rig, ten hardware scenarios (`steady`, `restart-linux`, `restart-device`, `power-cycle-device`, `handover`, `loss`, `interrupt`, `epoch`, `holdover`, `admission`), and `doc/design/reports/l3-results.md` with values written back to the constants table.

The epic's acceptance criteria now carry ids 027-AC-1 to 027-AC-11; 027-AC-1 is also AC-29 and 027-AC-7 is also AC-30. Tickets cite them by these ids.

Epic scope (C-126): **L (25 tickets, about 28.75 days)**.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 027-01 | Crate skeleton and skew types | XS | epic 020 |
| 027-02 | Two-channel capture file reader | S | 027-01 |
| 027-03 | Marker location and skew series | M | 027-01, epic 017, epic 020 |
| 027-04 | Synthetic file generator and harness self-check | S | 027-02, 027-03 |
| 027-05 | Skew statistics and pass/fail | S | 027-01 |
| 027-06 | Drift, step and recovery checks | S | 027-05 |
| 027-07 | Report rendering with sink stamps | S | 027-05, 027-06, epic 018, epic 026 |
| 027-08 | CI analysis on recorded files | S | 027-02, 027-03, 027-04, 027-07 |
| 027-09 | Build and document the rig | S | 027-08, epic 017, epic 018, epic 020, epic 026 |
| 027-10 | Capture inter-channel skew calibration | S | 027-03, 027-07, 027-09 |
| 027-11 | Scenario runner and steady hour | M | 027-05, 027-06, 027-07, 027-08, 027-09, 027-10, 027-22, 027-23, 027-24 |
| 027-12 | Restart scenarios | S | 027-11 |
| 027-13 | Device power-cycle check | S | 027-12 |
| 027-14 | Handover scenario | S | 027-11, 027-22, 027-25, epic 004 |
| 027-15 | Loss scenario | S | 027-11, epic 017 |
| 027-16 | Interrupt scenario | S | 027-11 |
| 027-17 | Epoch scenario | S | 027-11, epic 005, epic 017, epic 020 |
| 027-18 | Holdover scenario | S | 027-11, 027-22, epic 004 |
| 027-19 | Admission refusal check | S | 027-11, epic 005 |
| 027-20 | L3 results document and constants | M | 027-09 to 027-19, 027-22 to 027-25 |
| 027-21 | Decide hosted edge sinks in L3 | XS | 027-20, epic 021, epic 022 |
| 027-22 | Fetch regime stamps at run time | M | 027-07, epic 004 |
| 027-23 | Compute and record the audition offset A (new) | S | 027-07, 027-22 |
| 027-24 | Run eligibility gate: strict mode and certification state (new) | XS | 027-07, 027-22 |
| 027-25 | Per-sink frequency continuity at takeover (new) | S | 027-03, 027-06 |

Changes since the split: 027-22 added (C-18 makes the stamp a control message fetched by unicast request, with C-47 counters, which the runner now needs). In the final round, 027-23 added (C-148: the harness computes A with the commissioning rule and records it) and 027-24 added (C-147, C-127, C-128, C-140: rig runs must be strict and both sinks commissioned). No ticket was deleted: no ticket in this epic dealt with the withdrawn rational-L rule (C-105 revised) and no ruling removed work. Every ticket's Definition of done now lists its not-applicable lines with reasons (C-125). After C-191 and C-194, no ticket was added or deleted: 027-06, 027-10, 027-12, 027-14 to 027-17, 027-19 and 027-20 were corrected in place, and the tally is unchanged. After C-203, no ticket was added or deleted: 027-06, 027-14, 027-15, 027-16 and 027-20 were corrected in place, and the tally is unchanged. After C-178 to C-212, no ticket was added or deleted: 027-07, 027-09, 027-14, 027-18, 027-20, 027-22 and 027-23 were corrected in place (C-178, C-180, C-182, C-187, C-195, C-212), and the tally is unchanged. After C-212 (revised), C-213 and C-148 (revised), 027-25 added (C-212 now applies to L3 explicitly and asserts each sink's frequency continuity at takeover, rate change within C-61 and no phase step, which the skew series cannot show for a change common to both sinks); 027-09, 027-14, 027-18, 027-20 and 027-23 corrected in place; tally S 17 → 18. After C-220 and C-223, no ticket was added or deleted: 027-14, 027-20 and 027-25 were corrected in place, and the tally is unchanged.

## Effort tally
- XS: 3 (0.75 day)
- S: 18 (18 days)
- M: 4 (10 days)
- Total: 25 tickets, about 28.75 days → scope L (C-126)

## Coverage

### Deliverables

| Epic deliverable | Tickets |
|---|---|
| Rig: Linux host (holder, marker source, ALSA reference sink) | 027-09 |
| Rig: embedded reference device on the same switch, with switchable power | 027-09, 027-13 |
| Rig: two-channel capture device at 96 kHz with known inter-channel skew, ch1 Linux / ch2 device | 027-09, 027-10 |
| Rig: second holder-eligible Linux host for handover (required, C-212) | 027-09, 027-14 |
| Rig: all hosts on one IPv4 subnet (C-100) | 027-09 |
| Rig documented well enough to rebuild, in `doc/design/reports/` | 027-09 |
| Crate `hadronyche-tools`, binary `hadronyche-l3` | 027-01 |
| `skew.rs` (marker location, sub-sample interpolation, skew series) | 027-01, 027-03 |
| `run.rs` eligibility gate (strict mode C-147, commissioned sinks C-140) | 027-24 |
| `run.rs` computation of A (C-148) | 027-23 |
| `run.rs` `steady` (1 h) | 027-11 |
| `run.rs` `restart-linux`, `restart-device` (20 each; device restart spread measured, C-191) | 027-12 |
| `run.rs` `power-cycle-device` (20) | 027-13 |
| `run.rs` `handover` (per-sink rate change and phase at takeover, C-212, C-61) | 027-14, 027-25 |
| `run.rs` `loss` | 027-15 |
| `run.rs` `interrupt` | 027-16 |
| `run.rs` `epoch` | 027-17 |
| `run.rs` `holdover` | 027-18 |
| `run.rs` `admission` | 027-19 |
| `stamp.rs` (unicast stamp fetch, multicast update log, C-18) | 027-22 |
| `report.rs` (percentiles, max, stamps, declared E, the A used, pass/fail) | 027-05, 027-06, 027-07, 027-23 |
| `SkewSample`, `measure_skew`, `SkewReport` signatures | 027-01, 027-03, 027-05 |
| `doc/design/reports/l3-results.md` with measured skew, rig, A per run, constants write-back (C-79, device E) | 027-20 |

### Acceptance criteria

| Epic acceptance criterion | Tickets |
|---|---|
| 027-AC-1 (AC-29): p99.9 \|skew\| ≤ sum of target E for 1 h and after each of 20 restarts of each sink | 027-05, 027-11, 027-12, 027-23, 027-24 |
| 027-AC-2: no drift, slope < 1 µs/h | 027-06, 027-11 |
| 027-AC-3: handover within limit, no step (pair tolerance: the larger of the two sinks' own tolerances, both reported, C-203; ±1 sample on Linux, C-166; device's declared restart spread, C-191; handover 60 s into the run, window from the event end, C-191; triggered as a holder loss, holdover observed or the run fails, C-212; each sink's rate change within C-61 and no phase step at takeover, C-212; takeover by a holder with usable time, epoch kept, C-213; skew within the sum of the two E values through holdover and takeover, per-sink emission error an L2 assertion only, C-220) | 027-06, 027-14, 027-25 |
| 027-AC-4: loss and interruption, back to prior value within the pair tolerance (the larger of the two sinks' own tolerances, both reported, C-203), C-166 windows and loss pattern, C-191 burst spacing, media path only, window from the event end | 027-06, 027-15, 027-16 |
| 027-AC-5: epoch, resume within limit, no cross-epoch alignment | 027-17 |
| 027-AC-6: holdover honesty, claim lapses when the bound exceeds the sink's E (counted from holdover entry, C-195); sinks stay commissioned and claims resume after re-lock (C-180) | 027-18 |
| 027-AC-7 (AC-30): required E below the device's own E, device refused with `RequiredEExceeded` on its own figure whatever the Linux E (C-194); Linux plays alone when within the required E | 027-19 |
| 027-AC-8: harness self-check, 37 µs reported within 2 µs | 027-04, 027-08 |
| 027-AC-9: capture inter-channel skew measured, subtracted, reported; repeatability against the mean of 10 calibrations (C-191) | 027-07, 027-10 |
| 027-AC-10: both sinks' complete stamps in the report | 027-07, 027-22 |
| 027-AC-11: skew within the limit after each of 20 device power cycles, or D/E revised | 027-05, 027-13 |

### Definition of done

| Epic DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged (per-change CI, C-146) | all code tickets (027-01 to 027-08, 027-10 to 027-19, 027-22 to 027-25) |
| Every parser touched is fuzzed | 027-02, 027-07, 027-22 (budget C-95 scheduled by 027-08) |
| Regime stamp fields touched are complete and documented | 027-07, 027-11, 027-18, 027-20, 027-22, 027-23, 027-24 |
| Design documents updated where behaviour differs, constants table first | 027-20, 027-21 |
| No new open item without an owner and a falsifier | 027-02, 027-06, 027-10, 027-11, 027-12, 027-13, 027-18, 027-19, 027-20, 027-21, 027-22, 027-23, 027-25 |
| The frozen POC is not modified | 027-01 to 027-20, 027-22 to 027-25 (027-21 marked not applicable) |
| Every new factual claim carries an evidence tag | most tickets; consolidated in 027-20 |
| The skew analysis runs in CI on recorded two-channel files | 027-04, 027-08 (fixtures added by 027-10 to 027-18) |
| The rig is photographed or diagrammed and its parts listed | 027-09, 027-20 |
| `l3-results.md` committed with a `[measured]` skew figure, or device E revised with reason | 027-20 |

At epic level no DoD line is not applicable (see the note after the epic's checklist).

### Open sub-questions

| Question | Handled in |
|---|---|
| Is the skew within the sum of the two E values? | 027-11, 027-12, 027-20 |
| Does the device's declared D hold across power cycles? | 027-13 |
| Should L3 also include a macOS or Windows sink? (after epics 021, 022) | 027-21 |
| Inter-channel skew of the capture device at 96 kHz | 027-10 |
| Marker epoch identity | epics 017, 020; used by 027-03, 027-17 |
| Stamp fields for clock state (Unlocked after the lapse, C-162) and clock bound (published by epic 008a, C-187) | epic 004; used by 027-14, 027-18, 027-22, 027-23 |
| On-demand announced step, required-E setting (handover settled by C-212) | epic 005; used by 027-17, 027-19 |
| Recording file format | 027-02 |
| Lock bound C-62 and holdover entry delay C-195 (pending simulation) | simulation (C-195: epic 008b); affects 027-18 |
| Rate resolution of the per-channel measurement against C-61 | 027-25 |

## Remaining gaps

1. **Marker epoch identity.** How the marker content (and `MarkerPlan`) identifies its timeline epoch, so the epoch check can tell pre-step from post-step markers. C-19 gives the session number in the stamps, not in the audio. Affects 027-03, 027-17. Owner: epics 017 and 020.
2. **Stamp fields for the clock state and the clock bound.** C-162 makes the holdover lapse visible as the clock state becoming Unlocked, and C-18 fixes how stamps are fetched, but which fields carry the clock state and the current bound is not fixed. The values themselves are published by the servo core (epic 008a) through its time view (C-187). Affects 027-14, 027-18, 027-22, 027-23. Owner: wire specification epic 004, with epic 005.
3. **On-demand controls not covered by C-120 or the epic-017 injection API:** triggering an announced timeline step, setting a group's required E. Affects 027-17, 027-19. Owner: control state machines epic 005. (Forcing a handover is no longer needed: C-212.)
4. **Recording file format.** Working choice uncompressed WAV/RIFF. Affects 027-02, 027-08. Owner: this epic (027-02), recorded in `l3-results.md`.
5. **Lock bound C-62 and holdover entry delay C-195 are pending simulation.** The holdover margin and the expected 25 s lapse (C-65) depend on C-62; when that lapse is expected after the holder's removal, and the length of the holdover in the handover run, depend on C-195 (expected at or below 1.5 s, C-37). Affects 027-11, 027-14, 027-18. Owner: the servo simulation (C-195: epic 008b).

## Settled since the split

- Former gap 3, capture rate and the unit of "±1 sample": the wire carries only 96 kHz (C-01) and 48 kHz hardware converts at the edge (C-80); one sample is 10.4 µs whatever the capture rate, and the capture runs at 96 kHz (epic Background).
- Former gap 4, pass statistic: the 99.9th percentile of absolute skew, maximum reported (C-79).
- Former gap 8, holdover margin: the margin is the sink's E; the claim lapses when the bound reaches E, about 25 s for Linux (C-65, C-64, C-62), and the clock state then becomes Unlocked (C-162). The fetch mechanism is C-18. Residue: remaining gap 2.
- Former gap 9, rig control and stamp reading: stamps are fetched by unicast request and updates multicast (C-18), with health counters (C-47, completed by C-159), lateness percentiles (C-160) and timestamp-source values (C-131, C-129); test hooks are C-120; restarts, link interruption and power cycles are rig hardware chosen in 027-09. Ticket 027-22. Residue: remaining gap 3.
- Former gap 11, admission scenario missing: `admission` is now in the epic's scenario list and the refusal reason is `RequiredEExceeded` (C-103), with a connection alarm (C-173). Residue settled by C-194 (below).
- Former gap 12, power-cycle scenario missing: `power-cycle-device` is now in the epic's scenario list, with criterion 027-AC-11.
- Former gap 13, location of `l3-results.md`: `doc/design/reports/` (C-130); tool code in `hadronyche-tools` (C-133).
- Former gap 14, criteria without ids: all epic criteria now carry 027-AC-1 to 027-AC-11 (epic change).
- Former remaining gaps 4 to 7, test parameters: no-step tolerance at handover ±1 sample on Linux (then read as applying to the pair because the device had no restart spread in the table; C-191 removed that premise; settled by C-203, below); comparison windows of 10 s; sample tolerances against the median of the first 60 s; a one-sample slip passes and is reported as a slip event; injected loss 1 % random plus bursts of 8 slices; measurement window after a restart or power cycle 60 s; D spread as max − min (C-166, proposals pending first runs; 027-20 reports whether they held). Applied in 027-06, 027-12 to 027-16.
- Former remaining gap 9, role of L3 in C-35: C-35's owner is now L1 (task 019), and C-114 tests the follower load with synthetic followers. L3 has no follower-cap duty; the open sub-question is removed from the epic.
- Final round, also applied: pass limits use target E values, never the E measured in the same run (C-149); only commissioned sinks have a numeric E (C-140); rig runs are strict (C-147, C-127, C-128, set by operator policy C-165), new ticket 027-24; the harness computes and records A (C-148, headroom C-52, "cannot meet A" alarm C-174), new ticket 027-23; other-epoch slices restart the dead clock and have their own counter (C-135, used in 027-17); a sink-only device's liveness is judged by the producer timeout (C-168, C-41, checked in 027-16); D is defined by C-170, and a resampler node exists only inside an edge module with its delay in D (C-134; C-105 revised: L is always whole samples, the resampler is offered only at 2:1 and 1:2). The withdrawn rational-L rule left nothing to remove here: no ticket or statement in this epic relied on it. The per-change CI line "unprivileged" is read with C-146. C-150 figures added to 027-21.
- After the final round: C-194 settles former remaining gap 4 (027-AC-7 when E_device ≤ E_linux): the required E is set below the device's own declared E and the device is refused on its own figure whatever the Linux sink's E is; the scenario now always runs, and the Linux sink's result is asserted only when its E is within the required E (027-19, 027-20). C-191 completes the test parameters: loss bursts 1 s apart on the media path only, sync loss being its own scenario (027-15); announced step 5 ms (027-17); each 10 s comparison window starts when the injected event ends (027-06, 027-14, 027-15, 027-16); handover triggered 60 s into the run, replacing "at the middle" (027-14); calibration repeatability against the mean of 10 calibrations, replacing "at least 5 runs" (027-10); the embedded device declares its own restart spread, measured at L3 by `restart-device` (027-12, reported in 027-20). The C-191 clause on "both stamps" in a certification record (the sink's and its holder's regime stamps) has no subject here: this epic produces no certification record, and 027-AC-10 (both sinks' stamps in the report) is unaffected. All C-191 values are proposals pending first runs; 027-20 reports whether they held.
- After C-191: C-203 settles former remaining gap 6, the no-step and recovery tolerance for the Linux–device pair: the larger of the two sinks' own tolerances (±1 sample for the Linux sink, C-166; the device's declared restart spread, C-191), with both sinks' figures reported beside each verdict. The provisional default "±1 sample for the pair" is withdrawn: `step_at` and `recovered` take the pair tolerance from a new `pair_tolerance_ns` (027-06); the handover, loss and interrupt scenarios read the device's declared spread and report both figures (027-14, 027-15, 027-16); `l3-results.md` records the tolerance used instead of listing it as a working assumption (027-20). Epic 027-AC-3 and 027-AC-4 and its sub-question table updated.
- After C-203: C-212 settles how the handover is triggered (part of former remaining gap 3): as a holder loss, the old holder stops answering, both sinks pass through about 2 s of holdover (C-37 plus C-33) and the second Linux host takes over; a run without observed holdover fails. C-212 is stated for L2; L3 uses it because C-191 sets one handover procedure for both rungs, and C-202's holdover assertion becomes the skew staying within the limit through the holdover. `change_holder` by an epic-005 mechanism is replaced by `lose_holder`, 027-14 now depends on 027-22 and epic 004 instead of epic 005, and the rig procedure documents the method (027-09). C-195 places the holdover entry before the election, and the expected 25 s lapse is counted from holdover entry (027-18). C-180: sinks stay commissioned through holdover and their claims resume after re-lock without re-commissioning (027-18, 027-AC-6). C-187: the clock state and bound are published by the servo core (epic 008a); the stamp layout stays with epic 004 (remaining gap 2). C-182: stamp lateness figures use a rolling 60 s window with the since-live maximum beside them; `SinkStamp` carries both (027-07, 027-22) and `compute_a` uses the since-live maximum, the maximum observed lateness of C-148 and C-51 (027-23). C-178: group-certification results taken inside a commissioning window are valid commissioning results recorded with the window (epic Provides, 027-20). The other rules C-177 to C-212 and the late revisions of C-08, C-134, C-141, C-153, C-169, C-181, C-184 have no subject here or were already stated: the marker stream has no controllers and no bundle (C-08, C-181, C-205, C-209); no transport state (C-141, C-184, C-199, C-200); both sinks carry a numeric E, so C-169 does not arise; A is fixed per run, so the changed-A rules (C-186, C-198, C-208, C-211) do not arise; the resampler statements already match C-134, C-189, C-207; C-206 leaves report and file formats with this epic, as gap 4 already states.
- After C-212: C-212 (revised) applies to L3 explicitly, replacing the reading that L3 used it through C-191 with the skew no-step check standing in for frequency continuity and the skew limit standing in for C-202. Frequency continuity is now asserted per sink — rate change within C-61, no phase step against the sink's own no-step tolerance (C-166, C-191) — on each capture channel against the capture clock, new ticket 027-25, used by 027-14 and reported in 027-20; the second holder-eligible Linux node is a required rig part (027-09, epic Deliverables). The C-202 emission-error assertion leaves remaining gap 6. C-213 settles who may take over: a holder-eligible module with usable time (Locked, or in holdover with its claim not lapsed, C-65, C-169), with the epoch kept, and the epoch bumped (C-113) only when no candidate has usable time; 027-14 replaces "no candidate locked" with it and checks the epoch kept, 027-18 now requires no holder-eligible module during the removal and checks no takeover and no epoch change, 027-09 documents holder eligibility per scenario. C-148 (revised) states directly that A uses the since-live maximum lateness, never the rolling 60 s figure of C-182; 027-23 cites it in place of the earlier derivation and gains a test for it (epic Background).
- After C-220 to C-225: C-220 settles former remaining gap 6, per-sink emission error during holdover at L3: L3 asserts that the skew between the two sinks stays within the sum of their E values (C-79, C-202) through holdover and takeover and that each sink's frequency is continuous at takeover; per-sink emission error against the timeline is an L2 assertion only, which the L3 rig cannot observe (C-212 reworded to point at C-220). 027-14 now reports the skew over holdover and takeover as that assertion with a verdict, and no "not evaluated" emission-error line; 027-20 drops the working assumption; 027-25 and the epic's 027-AC-3 and sub-question table updated. C-223 fixes how the 50 ppm of the handover test (C-78) is made up, the measured natural difference between the two holders' crystals plus an injected drift, both recorded; the L3 handover injects no drift and records the measured natural difference (027-14, 027-20). C-221, C-222, C-224 and C-225 add no rig work: in `handover` the only holder-eligible module with usable time is the second Linux host (027-14), so no tie-break arises, and A is fixed per run, so the changed-A rules do not arise. No ticket was added or deleted; the tally is unchanged.
- Also applied earlier: fuzz budget 60 s per change and 1 hour nightly, corpus kept (C-95); IPv4 only, TTL 1 (C-100); liveness counting (C-102, C-108, C-111); not-applicable DoD lines stated (C-125); scope label from the tally (C-126); epic header lists epics 004, 005, 021 and 022.

Former gaps 1, 2 and 15 of the split appear above as remaining gaps 1, 4 and 5; former remaining gap 6 (C-191) is settled by C-203; the later remaining gap 6 (per-sink emission error at L3) is settled by C-220.
