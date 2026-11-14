# Epic 026b — Reference device and porting guide: tickets

## Summary

Epic 026b (phase P6, scope L: 22 tickets, about 23 days) is the second lettered part of epic 026, split under C-143 (an epic above 40 days by ticket tally is split into lettered parts; a reference to epic 026 means both parts). It builds one reference device on top of the crate `hadronyche-embedded` and the host-verified device side delivered by 026a (five traits of hardware surface: `Transport`, `DeviceTime`, `AudioOut`, `Identity`, and the kit's `RateActuator` re-exported, C-210; `Device<T, C, A, I>` holds the software-ratio adapter over the arithmetic kit's steerable rate actuator; both resamplers come from epic 024a, C-197). The reference device is firmware for a named board with wired Ethernet and an audio codec, acting as a sink that receives one whole 6-channel stream and plays the channels its codec has, joining an IPv4 medianet as a follower only. It runs the unchanged `hadronyche-core` state machines and implements only transport, time and audio hardware `[decision]`. This part chooses and gets approval for the board (C-172), writes the firmware (network, time, identity, watchdog, audio output), answers the epic's measurement sub-questions on the board (memory fit, resampler CPU fit and `f64` cost, bounded control handling, timestamp source and jitter, DSCP/don't-fragment, device E), runs the epic's acceptance tests on hardware (conformance over Ethernet, exact-fixed replay, identity on two boards, holdover, cable pull, fault reset, admission refusal), and adds the remaining stamp fields: per-connection counters (C-47, C-159), lateness percentiles and reclaimable latency (C-160, C-157), alarms and the DSCP observed on each peer's hellos (C-152, C-173, C-174), the target's calibration factor (C-106, C-176), D and E (C-170, C-73) and the device's own restart spread of D (C-191). It also handles a change of A on the device, announced by the commissioning tool and switched at the device's own position, with the clamp of a sink (C-186, C-198, C-211, C-214, C-216, C-222, C-224), delivers the porting guide and updates the design documents and the constants table.

Epic acceptance criteria keep their ids 026-AC-1 to 026-AC-12 (026-AC-1 is the global AC-35) and are completed in this part; their host-verifiable share is met first in 026a (026a-AC-1 to 026a-AC-9). This part depends on 026a; no ticket of 026a depends on this part.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 026b-01 | Reference board selection | XS | none |
| 026b-02 | Firmware network, time and identity | M | 026b-01, 026a-03, 026a-05, 026a-06, 026a-11 |
| 026b-03 | Firmware audio output | M | 026b-02, 026a-10 |
| 026b-04 | Memory fit measurement | S | 026b-03 |
| 026b-05 | Resampler CPU budget | S | 026b-03 |
| 026b-06 | Bounded control handling | S | 026b-03, 026a-12 |
| 026b-07 | Timestamp source and jitter | S | 026b-02 |
| 026b-08 | Conformance vectors over Ethernet | M | 026b-03, epic 007 |
| 026b-09 | Exact-fixed replay on device | S | 026b-03, epics 024, 025 |
| 026b-10 | Identity test on two boards | XS | 026b-02 |
| 026b-11 | Per-connection counters in stamp (new, C-47) | S | 026a-08, 026a-11, 026b-02 |
| 026b-12 | Alignment figures, alarms and peer DSCP in stamp (new, C-152, C-157, C-160, C-173, C-174) | S | 026a-08, 026a-10, 026a-11, 026b-02, epics 009, 013 |
| 026b-13 | Changed A on the device (new, C-186, C-198, C-211) | S | 026a-07, 026a-08, 026b-12, epics 009, 012, 014 |
| 026b-14 | Declare D and E in stamp | S | 026b-03, 026a-11, 026a-12 |
| 026b-15 | Holdover on holder loss | S | 026b-03, 026a-06, 026b-14 |
| 026b-16 | Liveness cable pull | S | 026b-03, 026a-08, 026b-11 |
| 026b-17 | Fault reset and rejoin | S | 026b-03, 026b-02 |
| 026b-18 | Admission refusal on device | XS | 026b-03, 026a-07, 026b-11 |
| 026b-19 | Device calibration factor (new, C-106) | S | 026b-09, 026a-11, epics 024, 025 |
| 026b-20 | Declare restart spread in stamp (new, C-191) | XS | 026b-14, 026a-11, 026a-12 |
| 026b-21 | Device porting guide | S | 026b-03, 026b-04, 026b-05, 026b-07 |
| 026b-22 | Design docs and constants update | XS | 026b-01, 026b-04, 026b-05, 026b-06, 026b-07, 026b-15, 026b-14, 026b-19, 026b-13 |

The rows are in a valid execution order: every ticket comes after the tickets it depends on, and every 026a ticket comes before all of these.

## Effort tally

- XS: 5 (1.25 days)
- S: 14 (14 days)
- M: 3 (7.5 days)
- Total: 22 tickets, about 23 days (22.75) — scope L by C-126 (more than 15 and at most 40 days)

Together with 026a (13 tickets, 18.25 days) the epic still totals 35 tickets and 41.0 days.

## Coverage

### Deliverables

| Deliverable | Tickets |
|---|---|
| `examples/reference-device/` — firmware for one named board, sink for one 6-channel stream | 026b-01, 026b-02, 026b-03 |
| Board implementations of `Transport`, `DeviceTime`, `Identity` (with the stored domain id, C-173) and `AudioOut`; watchdog | 026b-02 (network, time, identity, watchdog), 026b-03 (audio) |
| Stamp fields added to `stamp_publish.rs` (026a-11) | 026b-11 (counters), 026b-12 (alignment figures, alarms, peer DSCP), 026b-14 (D and E), 026b-19 (calibration factor), 026b-20 (restart spread) |
| Changed A on the device | 026b-13 |
| `docs/device-porting.md` | 026b-21 |
| Design documents and constants table | 026b-22 |
| Open sub-question: CPU fit of the fixed node and the rate actuator (C-189), and `f64` cost | 026b-05 |
| Open sub-question: bounded control handling | 026b-06 |
| Open sub-question: timestamp source and jitter | 026b-01, 026b-07 |
| Open sub-question: DSCP, don't-fragment, received DSCP | 026b-02 |
| Open sub-question: device E value (form settled by C-73) | 026b-14, confirmed by task 027 |
| Open sub-question: reference board (approval by the product owner, C-172) | 026b-01, 026b-22 |

### Acceptance criteria

| Epic criterion | Tickets |
|---|---|
| 026-AC-1 (AC-35): follower, 63-entry peer map (C-217), bounded or report-only (control isolation per C-161), stamp published and collected | 026a-04, 026a-05, 026a-11, 026a-12, 026b-02, 026b-06 |
| 026-AC-2: memory fit ≥ 25 % headroom at A = 10 ms (23 040-byte ring), figure recorded | 026b-04 (early estimate 026b-01) |
| 026-AC-3: follower only even with lowest id, holdover 1 µs/s, claim dropped at declared E (Unlocked after, C-162; E unknown: claim dropped when the published bound exceeds 2 × C-62, C-169); stays commissioned and resumes on relock (C-180); takeover with the epoch kept and the stream not restarted (C-213, C-221) | 026a-05, 026a-06, 026b-15 |
| 026-AC-4: identity with true random disabled; id stable, nonce new; domain 0 ignored (C-173) | 026a-03, 026b-02, 026b-10 |
| 026-AC-5: conformance vectors of task 007 over real Ethernet | 026b-08 |
| 026-AC-6: exactness by replay | 026b-09 |
| 026-AC-7: liveness cable pull | 026a-08, 026b-16 |
| 026-AC-8: fault = reset, liveness judged by producer timeout (C-168), panic hook (C-145) | 026b-02 (watchdog), 026b-17 |
| 026-AC-9: refusal of a 7th channel (second stream) on buffer memory, named reason | 026a-07, 026b-18 |
| 026-AC-10: D (with method, C-170), restart spread (L3, C-191) and E (C-73 from a commissioned run per C-140, or unknown) in stamp | 026a-10, 026a-11, 026b-14, 026b-20 |
| 026-AC-11: no allocator and no libm symbol in firmware | 026a-01 (check), 026b-02 (applied to image), 026b-03, 026b-09, 026b-19 |
| 026-AC-12: stamp content (C-129 states incl. C-147 and C-161, C-131 source, C-47/C-159 counters, C-160 percentiles, C-157 reclaimable latency, both over the 60 s window of C-182, C-152 peer DSCP, C-107/C-153 window, C-106/C-176 factor) | 026a-04, 026a-13, 026b-02, 026b-11, 026b-12, 026b-19 |
| Background: changed A while live, announced by the tool (C-224), switched at the device's own position (C-214), current A in receiver reports, A beyond declared memory with the sink clamp in detail (C-186, C-198, C-211, C-216, C-222) | 026b-13 |

### Definition of done

| DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all code tickets (026b-02, 026b-03, 026b-08, 026b-09, 026b-11, 026b-12, 026b-13, 026b-20) |
| Every parser touched is fuzzed (C-95 budget, CI job in 026a-01) | 026b-11, 026b-12 |
| Regime stamp fields touched are complete and documented | 026b-02, 026b-06, 026b-07, 026b-11, 026b-12, 026b-13, 026b-14, 026b-17, 026b-19, 026b-20 |
| Design documents updated where behaviour differs, constants table first | 026b-04, 026b-05, 026b-15, 026b-22 |
| No new open item without an owner and a falsifier | 026b-01, 026b-02, 026b-04 to 026b-08, 026b-12, 026b-14, 026b-19, 026b-22 |
| The frozen POC is not modified | 026b-22 |
| Every new factual claim carries an evidence tag (`[standard]` added by C-132) | all tickets |
| Crate builds for a device target and for the host in CI | 026b-02 onward (code tickets) |
| Reference board, timestamp source and oscillator named in docs | 026b-01, 026b-02, 026b-03, 026b-07, 026b-21 |
| "What weakens on no_std" reflected field by field in the stamp | 026b-06, 026b-17, 026b-21 |

Not applicable in this part (C-125): none; every line has a subject.

## Settled since the split

- Split under C-143 — with 026b-13 (formerly 026-35) epic 026 tallied 41 days, above the 40-day line, so C-143 called for lettered parts. Settled by this split: 026a (host-testable HAL, core integration, stamp publication and collector, 13 tickets, 18.25 days) and 026b (reference device firmware, on-device measurements and tests, remaining stamp fields, porting guide, 22 tickets, 22.75 days). No ticket content changed; tickets were renumbered and their references updated. Former gap 10 is removed.
- Gap "trait count inconsistent" — epic change: five traits, `RateActuator` held inside `Device` with the software-ratio default; porting guide covers five (026b-21).
- Gap "identity mixing unspecified" — C-11, C-46: module id stable across reboots, nonce new per boot; the mixing function is an implementation detail (026a-03; verified on two boards in 026b-10).
- Gap "stamp publication details missing" — C-18: sectioned control message, multicast on change and every 10 s, unicast request, hello carries a version number (collector record format and file names: this epic, 026a-12, C-206; record location C-171, where 026b-14 stores the certificate).
- Gap "holdover margin undefined" — C-65 with C-73: the margin is the device's declared E.
- Gap "AC-7 timing ambiguous" — C-41 and C-111: each 2 s timer starts at its own last event; tests stay at 1 s and 3 s (026b-16).
- Gap "AC-9 limit ambiguous" — C-04 and C-96: a 7th channel is a second stream, refused on declared memory sized to A; the port cannot refuse it (reason name remains, gap 3).
- Gap "skew-buffer rate ambiguous" — C-01, C-80, C-96: the ring holds wire slices at 96 kHz; 23 040 bytes at A = 10 ms.
- Gap "2-channel vs 6-channel" — C-04: the stream is the unit of subscription; the device buffers 6 channels and plays what its codec has.
- Gap "task 007 not in dependencies" — epic change: 007 added to the header and Dependencies (026b-08).
- Gap "optional capture source" — epic change: stated as not a deliverable of this epic.
- Gap "fault injection mechanism unspecified" — C-120, C-145: a test-only panic-injection hook, stamped like any injection; on the device it panics inside control handling (026b-17).
- Epic sub-question "device E: bound or percentile" — C-73 (with C-103 for E unknown, C-140 for which certification state gives a number).
- Former gap 9 "026-AC-8 for a sink" — C-168: a sink-only device's liveness is judged by the producer timeout (C-41); 026-AC-8 and 026b-17 restated.
- Former gap 11 "alignment-claim margin with E unknown" — C-169 (corrected): an unknown E is taken as the fixed reference 2 × C-62 (50 µs); the device stops claiming alignment when the bound it currently publishes exceeds it, about 25 s into holdover at C-64. 026a-06 Work 5 and acceptance criterion 6 implement the lapse (the provisional last-lock reading is withdrawn); 026b-15 and the epic restated. C-162: after the lapse the clock is Unlocked with time unusable.
- Resampler and rational L — C-105 (revised): L is a whole number of samples and the resampler node is offered only at 2:1 and 1:2, which covers the device's 2:1; C-134: the node appears only inside an edge module and its delay is part of D; C-170 defines D (026a-09, 026a-10, 026b-05, 026b-14). No rational-number handling existed in these tickets, so nothing was removed.
- Control isolation — C-161: on a device, absent or unverified unless the bound of 026b-06 is shown (026a-04, 026b-06). C-164 gives the proposed per-bad-packet bound measured in 026b-06.
- Calibration procedure — C-176: epic 025 runs the reference workload per listed target (026b-19).
- Liveness details — C-135 (other-epoch slices restart the dead clock and have their own counter), C-16 with C-156 (drain bound per slice period) (026a-08, 026b-11).
- Counters — C-159 completes the C-47 list (026b-11).
- Reference board approval — C-172 (026b-01, 026b-22). User-facing guide location — C-171 (026b-22).
- Two kinds of resampler — C-189: the DSL resampler node is fixed-ratio (2:1 or 1:2, whole-sample delay, C-105); the rate actuator is a separate steerable resampler varying by parts per million, its delay part of D. The epic's "same resampler, with a steered ratio" reading is withdrawn; 026a-09 Work and acceptance criteria corrected (fixed node plus `SoftwareRatio` at nominal ratio 1 after it), 026a-10, 026b-03, 026b-04, 026b-05 and 026b-14 restated. The ABI's "frequency-dependent delay" property is epic 024's and needs no work here.
- L2 and L3 parameters, completed — C-191: an embedded device declares its own restart spread, measured at L3 (new ticket 026b-20); "both stamps" in a certification record are the regime stamps of the sink and of the holder it followed (026b-14). Its other parameters (loss-burst spacing, 5 ms announced step, comparison-window start, handover at 60 s, calibration repeatability against the mean of 10) belong to the L2/L3 rigs, not to this epic's tickets.
- Who builds the resamplers — C-197: the arithmetic kit (epic 024a) builds both the fixed 2:1 node and the steerable rate actuator; the device uses the kit's and builds none. A shorter filter or other change to the actuator is filed with epic 024a (026b-05, epic sub-question, gap 5). 026a-02, 026a-10 and 026b-21 restated; 026a-09 is the adapter.
- Tolerance for a pair of different sinks — C-203: the larger of the two sinks' own no-step and recovery tolerances, both reported beside each verdict. The rig and the verdicts are task 027's; the device's side is its declared restart spread (026b-20, epic Background restated). No new work.
- Home of the rate-actuator trait — C-210: defined in the kit epic (024a) with its implementation, ratio range ±1 000 ppm; every edge consumes it. The reading in which epic 018 owned the trait and `hadronyche-embedded` defined it is withdrawn: 026a-02 re-exports the kit's trait, 026a-09 implements it with a ±1 000 ppm range test, 026b-05, 026b-21, the epic's Deliverables, Dependencies and sub-question, and gap 5 restated with owner epic 024a.
- What an open commissioning window disqualifies — C-178: production figures in stamps are unqualified while a window is open; the tool's own measurements (D, restart spread, certification) are valid commissioning results recorded with their window. 026b-14 had required the certification run to avoid any window (from the earlier "measurements in a window are unqualified" reading); Work and acceptance criteria corrected to record the window instead. 026a-13, 026b-12 and the epic restated; 026b-20 already stated it.
- Certification of a sink whose clock is unlocked — C-180: the device stays commissioned, its claim is suspended and resumes on relock without re-commissioning. 026a-06 Work 5 and criterion 5, 026b-15 criterion 5, epic 026-AC-3 and Background restated.
- Window of the stamp's lateness figures — C-182: rolling 60 s window for percentiles, reclaimable latency and cannot-meet-A, maximum since live beside them (026b-12 Work 1 and criteria 1, 4; epic Background).
- Refusal reasons completed — C-185: 18 in all, so no buffer-memory reason is added (026a-07, 026b-18, gap 3, epic sub-question). ControlDegraded and the control-degraded alarm do not arise on a device, where any fault is a reset (026b-12).
- A changed while live, its carrier and A beyond buffer memory — C-186, C-198, C-211: the device re-aligns with ring and ceiling resized, carries its current A in receiver reports, and, if an A beyond its declared memory reaches it, keeps the deepest ring it holds, raises cannot-meet-A and keeps playing. No ticket covered this: new ticket 026b-13 (S); 026a-07, 026a-08 and 026b-22 point to it.
- Holdover entry delay — C-195: pending simulation, at or below C-37; 026a-06 Work 4 and criterion 4, 026b-15 criterion 2 corrected (they had tied holdover entry to the 1.5 s election silence); gap 4 extended.
- Rules checked and not applicable to this epic: C-177 and C-201 (seqlock and CPU pinning; the device runs one cooperative loop with no threads to pin), C-179 (redundancy decision), C-181, C-205, C-209 and revised C-08 (controller placement and the bundle cap; producer and compiler rules, the device subscribes to whole streams), C-183, C-190 (compiler rules), C-184, C-199, C-200 and revised C-141 (transport state; no subject in the device's tickets), C-188 and C-204 (holder-side; the device sends 50 requests/s, inside the accepted range), C-192, C-193 (vector provenance and crate location; 026b-08 consumes epic 007's harness as it stands), C-194, C-202, C-212 (L2 and L3 rig parameters; the L3 rig is task 027), C-196 (specification epics), C-208 (fan-ins with an output; the device has none, 026b-13 out of scope). Revised C-134, C-153 and C-169 were already applied.
- Earlier rules checked and not applicable to this epic: C-137 (header cap; met by the producer, the device only parses), C-139 (hosted restart policy; on the device any fault is a reset), C-163 (last ratchet step; applied by core), C-136 (holder stall; the device is never holder), C-138, C-141, C-142, C-144 (duties of epic 015 only), C-146 (privileged tests; no ticket here needs privilege), C-148, C-149, C-150, C-151, C-154, C-155, C-158, C-165, C-166, C-167, C-175. C-35's owner change does not affect this epic.
- Rules C-213 to C-225 — applied in place, no ticket added or deleted, efforts unchanged: C-214 (the device switches to a changed A at its own position one lead ahead, unannounced), C-224 (the change arrives through the commissioning tool's announcement inside a window; epic 012 added to the epic header and to 026b-13 for its receipt, 012-30), C-222 and C-216 (the clamp applies to the device; the alarm stays until a later A that fits; reports carry the given A): 026b-13 Context, Work 1 to 3, criteria 1 and 3, Out of scope, epic Capacity and Deliverables. C-225 (the device's own ratchet step at its own position) and C-215 (upstream steps announced): 026b-13 Context and epic Liveness, implemented in 026a-08. C-213 and C-221 (takeover by a usable-time candidate, epoch kept): 026b-15 Context, new Work 6 and criterion 6, epic Role and 026-AC-3. C-217 (63-entry peer map): 026b-01, 026b-04, 026b-06, 026b-08, 026b-12, 026b-21, epic Background, 026-AC-1 and 026-AC-2. C-219: not computed by the device, which subscribes to audio members only; the epic notes that admission would apply it. The epic's "user-facing note" is reworded "operator-facing note".
- Rules C-213 to C-225 checked and not applicable to this part: C-208 (fan-ins with an output), C-209 (bundle cap), C-212, C-220, C-223 (L2 and L3 handover rig assertions; task 027 and epic 020), C-218 (compile-time rejections; epic 023).

New work created by rulings in this part: 026b-19 (C-106, C-176), 026b-11 (C-47, C-108, C-159), 026b-12 (C-152, C-157, C-160, C-173, C-174), 026b-20 (C-191), 026b-13 (C-186, C-198, C-211).

## Remaining gaps

1. **Reference board not yet chosen.** Proposed by ticket 026b-01, approved by the product owner and recorded in the constants table (C-172); falsifier: no candidate meets 026-AC-2 with 25 % headroom. Affects 026b-01 to 026b-22. Owner: this part (026b-01) with product-owner approval.
2. **Allowed methods for "D with method".** What D contains is settled (C-170); the list of methods is not. Affects 026b-14 (and 026a-10). Owner: epic 018 (shared edge core).
3. **Which refusal reason a buffer-memory refusal carries.** The refusal rule is settled and the list is complete at 18 reasons (C-173, C-185), so no new reason is added; the table does not say which of the 18 applies. Affects 026b-18 (and 026a-07). Owner: epic 012 (admission).
4. **C-62 lock bound and holdover entry delay pending simulation.** Tests read 25 µs from the constants table and the holdover entry delay from the core servo (C-195: proposed by epic 008b, at or below C-37). Affects 026b-07, 026b-15 (and 026a-06). Owner: the servo simulation.
5. **`f64` without libm and soft-float cost.** `RateActuator::set_ratio(f64)` on a part that may lack a double-precision FPU. Affects 026b-05 (and 026a-09). Owner: measurement in 026b-05; epic 024a if the trait signature or the kit's actuator must change (C-197, C-210).
6. **Device E value.** Its form is settled (C-73, commissioned run per C-140); the value comes from the one-hour run in 026b-14 and is confirmed by task 027 (L3, C-79). Affects 026b-14, 026b-15. Owner: measurement, then task 027.
7. **How a domain id reaches a device.** C-173 makes peers ignore hellos with domain 0, so a device must hold a non-zero domain id; how it is written to the device's storage is not settled. Affects 026b-02 (and 026a-03). Owner: commissioning tooling with epic 005.
8. **How device alarms reach an operator.** C-173 and C-174 define the connection alarms; a device has no files or local log, and no rule says how its alarms are surfaced. Affects 026b-12. Owner: epic 013 (stamp wire object) with epic 005.

The encoding of the commissioning-window announcement and of the tool's announcement of a changed A (C-224 settles who sends it and when; owner epic 005 with 004) affects 026a-13 and 026b-13 and is listed in 026a's index as gap 5. None needs the design owner.

## Ticket numbers before the split

| Old ticket | New ticket |
|---|---|
| 026-01 | 026b-01 |
| 026-14 | 026b-02 |
| 026-15 | 026b-03 |
| 026-16 | 026b-04 |
| 026-17 | 026b-05 |
| 026-18 | 026b-06 |
| 026-19 | 026b-07 |
| 026-20 | 026b-08 |
| 026-21 | 026b-09 |
| 026-22 | 026b-10 |
| 026-23 | 026b-15 |
| 026-24 | 026b-16 |
| 026-25 | 026b-17 |
| 026-26 | 026b-18 |
| 026-27 | 026b-14 |
| 026-28 | 026b-21 |
| 026-29 | 026b-22 |
| 026-30 | 026b-19 |
| 026-31 | 026b-11 |
| 026-33 | 026b-12 |
| 026-34 | 026b-20 |
| 026-35 | 026b-13 |

The other 13 former tickets of epic 026 (026-02 to 026-13 and 026-32) are in 026a; its index maps them.
