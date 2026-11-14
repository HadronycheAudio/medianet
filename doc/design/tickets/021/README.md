# Epic 021 — macOS backend and Core Audio edge: tickets

Epic 021 (phase P4, second half; depends on epics 001, 015, 016, 018, 020, 024a, 025; blocks 028; scope L) makes a macOS machine a certified **edge** member of a medianet: crate `hadronyche-os-macos` implements the hosted backend traits on macOS (`HostClock` on mach absolute time; IPv4-only, interface-pinned `OsNet` sockets with multicast TTL 1, don't-fragment, `IP_TOS` DSCP and kernel-or-userspace receive stamps; sync exchange building with step filtering; `OsRealtime` with time-constraint policy, audio workgroup join, memory locking and strict mode), a power/sleep handler that treats sleep as a departure, a local-network permission check, and a Core Audio sink and physical-input capture module with device guards and declared D. The wire carries 96 kHz only; a 48 kHz device converts through the kit's resampler node and steerable rate actuator (epic 024a; this epic builds none, C-197); the node exists only inside edge modules, and its whole-sample conversion delay is counted only in D, never in L, and reported separately on the device-facing port of the manifest (C-105, C-134, C-207). The edge never holds the clock (holder bit never set). The work ends with L2 certification on macOS (E ≤ 100 µs, D restart spread ≤ 50 µs, cross-sink skew ≤ 150 µs, L2 common criteria and drift ramp), the `[memory]` facts confirmed or corrected, and the record `doc/design/reports/l2-results-macos.md` with a short certified reference list.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 021-01 | Crate scaffold and macOS CI | XS | epic 015, epic 016, epic 018 |
| 021-02 | Mach host clock | S | 021-01, epic 015 |
| 021-03 | Socket options: don't-fragment, DSCP, TOS receive | S | 021-01, epic 016 |
| 021-04 | Interface-pinned multicast and unicast sockets | M | 021-03, epic 016 |
| 021-05 | Receive timestamps, kernel or userspace | S | 021-02, 021-04, epic 001 |
| 021-06 | Interface stats and absent drop accounting | XS | 021-04 |
| 021-07 | Local-network permission detection | S | 021-04 |
| 021-08 | Local-network permission prompt for a CLI binary | S | 021-07, 021-31 |
| 021-09 | Real-time process and thread setup | S | 021-01, 021-02, epic 015 |
| 021-10 | Core Audio device layer and DeviceClock | M | 021-02, epic 018 |
| 021-11 | Device guard: refuse aggregate, Bluetooth and network devices | S | 021-10 |
| 021-12 | Device guard: rate change, device loss, held interface | S | 021-10, 021-11 |
| 021-13 | Audio device workgroup join | XS | 021-09, 021-10 |
| 021-14 | Declared D from device properties | S | 021-10, epic 018, epic 024a |
| 021-15 | Core Audio edge sink | M | 021-05, 021-12, 021-13, 021-14, epic 018, epic 024a |
| 021-16 | Core Audio edge capture | M | 021-05, 021-12, 021-13, 021-14, epic 018, epic 024a |
| 021-17 | Raw capture for the L2 harness | S | 021-10, 021-11, epic 020 |
| 021-18 | Power assertion and sleep/wake handling | S | 021-15, 021-16 |
| 021-19 | Edge only: holder bit never set | S | 021-15, 021-16 |
| 021-20 | Regime stamp `macos-std` | S | 021-05, 021-06, 021-12, 021-14, 021-27, 021-30 |
| 021-21 | Interface pinning and socket options verified on the wire | S | 021-04, 021-15, 021-16 |
| 021-22 | Commissioning measurement of D versus device properties | S | 021-14, 021-15, 021-16, 021-17, 021-31, epic 020 |
| 021-23 | L2 macOS run: E, D restart spread, cross-sink skew (AC-27) | M | 021-17, 021-18, 021-20, 021-21, 021-22, 021-31, epic 020 |
| 021-24 | L2 common criteria on macOS | M | 021-23 |
| 021-25 | Confirm `[memory]` facts and update design documents | S | 021-05, 021-08, 021-21, 021-22, 021-23, 021-24, 021-30, epic 001 |
| 021-26 | `l2-results-macos.md` and certified reference list | S | 021-20, 021-23, 021-24, 021-25, 021-27, 021-29, 021-30 |
| 021-27 | Cost calibration factor for the macOS target (new, C-106, C-176) | S | 021-01, 021-31, epic 018, epic 025 |
| 021-28 | Sync exchange building and step filter (new, C-121) | S | 021-05, epic 015, epic 016, epic 018 |
| 021-29 | L2 drift-ramp scenario on macOS (new, C-119) | S | 021-23, epic 020 |
| 021-30 | Strict mode: memory locking and capability states (new, C-127 to C-129) | S | 021-09, 021-13 |
| 021-31 | macOS reference hardware proposal (new, C-172) | XS | epic 001 |

No ticket was deleted. The withdrawn rational-L rule (C-105 revised) only touched 021-14, which had a rational conversion step and one unit test for a fractional L; both are replaced by whole-sample handling and the ticket is otherwise still needed. Rulings C-80 and C-124 changed what 021-14 and 021-18 assume, not whether they are needed. New rules that add work to existing tickets rather than a new one: C-105 (unsupported-rate refusal, 021-11), C-146 (privileged nightly job, 021-01), C-152 and C-159 (stamp fields, 021-20), C-161 and C-165 (021-30), C-147 to C-149 and C-166 (L2 tickets), C-197 (dependency on epic 024a, 021-14, 021-15, 021-16), C-207 (conversion delay outside L and reported on the device-facing port, 021-14, 021-15, 021-16), C-177 and C-201 (pin-dependent seqlock refusal, 021-30), C-178 (commissioning results recorded with their window, 021-20, 021-22), C-180 (certification across an unlocked clock, 021-18, 021-20), C-182 (stamp lateness window, 021-20), C-186 and C-198 (re-alignment on a changed A, 021-15), C-189 and C-210 (rate actuator at every device rate, its delay in D, 021-10, 021-14, 021-15, 021-16), C-191, C-202 and C-212 (L2 parameters and handover as holder loss, 021-24, 021-26), C-212 revised and C-213 (second holder-eligible Linux node in the rig, frequency continuity per C-61, epoch kept at takeover, 021-24; takeover eligibility, 021-19), C-214 (sink switch position on a changed A, 021-15).

## Effort tally

XS: 4 · S: 21 · M: 6 · total 31 tickets. At XS = 0.25 day, S = 1 day, M = 2.5 days: about 37 days, scope **L** (C-126); below the 40-day split threshold of C-143.

## Coverage

### Deliverables

| Deliverable | Tickets |
|---|---|
| Crate `hadronyche-os-macos` (edge-only build) | 021-01 |
| `clock.rs` — `HostClock` on mach absolute time (`MachClock`), media deadline clock | 021-02 |
| `net.rs` — IPv4-only, interface-pinned multicast with TTL 1 (`open_media`, `open_discovery`) | 021-04 |
| `net.rs` — receive timestamps (kernel or userspace, C-131) | 021-05 |
| `net.rs` — don't-fragment, `IP_TOS`, receive of the TOS byte | 021-03 |
| `net.rs` — `iface_stats` | 021-06 |
| `net.rs` — sync exchange building and step detection (C-121) | 021-28 |
| `rt.rs` — time-constraint policy, capability report, `setup_process`, `setup_thread`, `measure_wake_latency` | 021-09 |
| `rt.rs` — workgroup join | 021-13 |
| `rt.rs` — memory locking, four capability states, strict-mode refusal (C-127 to C-129), pin-dependent seqlock refusal (C-177, C-201) | 021-30 |
| `power.rs` — power assertion, sleep/wake → departure, loop reset, normal admission (C-124) | 021-18 |
| `coreaudio/sink.rs` (kit resamplers, C-197; rate actuator at every device rate, C-189, C-210; conversion delay on the device-facing port, C-207) | 021-15 |
| `coreaudio/capture.rs` (kit resamplers, C-197; rate actuator at every device rate, C-189, C-210; conversion delay on the device-facing port, C-207) | 021-16 |
| `DeviceClock` from callback timestamps (`CoreAudioDevice`) | 021-10 |
| D from device properties, the resampler node's whole-sample conversion delay and the rate actuator's delay, counted only in D (C-105, C-134, C-189, C-207) | 021-14 (commissioning measurement: 021-22) |
| `coreaudio/guard.rs` — refuse aggregate, Bluetooth, network, virtual and unsupported-rate devices (C-105) | 021-11 |
| `coreaudio/guard.rs` — sample-rate change, device loss, "interface held" (C-175), certification state *uncertified* (C-140) | 021-12 |
| `permissions.rs` — missing local-network permission as named startup failure | 021-07 |
| `RawCapture for CoreAudioRawCapture` (L2 harness) | 021-17 |
| Regime stamp `macos-std` fields, including calibration factor (C-176), peer hello DSCP (C-152), counters (C-159) | 021-20, 021-27 |
| Document `doc/design/reports/l2-results-macos.md` | 021-26 |
| Reference hardware for L2 (C-172) | 021-31 |
| Per-OS checklist inputs to epic 028 | 021-08, 021-26 |

### Acceptance criteria

| Epic criterion | Tickets |
|---|---|
| 021-AC1. AC-27 (E ≤ 100 µs, D restart spread ≤ 50 µs, cross-sink skew ≤ 150 µs) | 021-23 (inputs 021-14, 021-15, 021-22, 021-31) |
| 021-AC2. L2 common criteria (C-75 to C-78, C-166, C-191), handover as holder loss (C-202, C-212) with the epoch kept at takeover (C-213) and drift ramp (C-119), against the target E (C-149), in strict mode (C-147) | 021-24, 021-29 |
| 021-AC3. Interface pinning, IPv4, TTL 1 | 021-04 (mechanism), 021-21 (wire capture) |
| 021-AC4. Sleep as departure | 021-18 (clock side: 021-02, 021-28) |
| 021-AC5. Guards | 021-11, 021-12 |
| 021-AC6. Edge only | 021-01 (feature), 021-19 |
| 021-AC7. Stamp (AC-31) | 021-20 (fields from 021-03, 021-05, 021-06, 021-12, 021-14, 021-27, 021-30), 021-26 (reference list) |
| 021-AC8. Capture scope | 021-11, 021-16 |
| 021-AC9. Receive-timestamp claim replaced by `[measured]` | 021-05, 021-25 |
| 021-AC10. Strict mode | 021-30 |

### Definition of done

| Item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all code tickets; privileged tests in the nightly job of 021-01 (C-146) |
| Every parser touched is fuzzed | 021-03, 021-05 (budget C-95) |
| Regime stamp fields touched are complete and documented | 021-06, 021-12, 021-14, 021-20, 021-26, 021-27, 021-30 |
| Design documents updated where behaviour differs, constants table first | 021-08, 021-22, 021-25, 021-31 |
| No new open item without an owner and a falsifier | 021-08, 021-22, 021-23, 021-26, 021-31 |
| The frozen POC is not modified | Not applicable at epic level (C-125): no work touches the POC; 021-01 and 021-25 keep the line as a guard |
| Every new factual claim carries an evidence tag | most tickets |
| Every `[memory]` fact confirmed or corrected | 021-25 |
| Short reference list published; everything else *unverified* | 021-20, 021-26 (hardware from 021-31, C-172, C-138) |
| CI builds the crate on macOS; hardware tests are a documented manual job | 021-01 (plus hardware tickets) |

### Open sub-questions

| Question | Handling |
|---|---|
| Kernel monotonic receive stamps for UDP? (owner task 001) | Note in 021-05; result recorded in 021-25 |
| Is E = 100 µs reachable, and for which device classes? (this task) | 021-23 |
| Do device-reported latency properties match commissioning measurement? (this task; falsifier mismatch above 20 µs, C-150) | 021-22 |
| Permission prompt, pre-grant and probe detection (this task, feeding 028) | 021-07, 021-08 |
| L2 skew statistic (task 020) | 021-23 reports both |
| Wake-latency threshold (this task, then design owner) | 021-09 reports, 021-23 records |

## Remaining gaps

1. **Trait signatures.** Argument and return types of `HostClock`, `OsNet`, `OsRealtime`, `DeviceClock`, `RawCapture`, the sync exchange type and the loop-reset entry point; the API of the kit's resampler node and steerable rate actuator, including the manifest field that carries the conversion delay on the device-facing port (C-207) and the rate actuator's delay as its trait reports it (the trait and its ±1 000 ppm range are the kit's, C-210). Affects 021-02, 021-04, 021-06, 021-09, 021-10, 021-14, 021-15, 021-16, 021-17, 021-18, 021-28. Answer: epics 015, 016, 018, 020, 024a.
2. **Edge-only feature name.** Affects 021-01, 021-19. Answer: the hosted crates (epics 015, 016, 018).
3. **Declared jitter values.** No design value for `declared_jitter_ns` of host or device clock; set from measurement. Affects 021-02, 021-10. Answer: a measurement (in those tickets).
4. **Local-network permission detection method.** The self-addressed multicast probe is a working choice. Affects 021-07, 021-08. Answer: a measurement (021-08).
5. **D-method values and stamp encoding.** Allowed values for "D method" and the field encoding; the certification-state values are now fixed by C-140. Affects 021-14, 021-20, 021-22. Answer: epic 013, which defines the regime stamp message (C-144).
6. **Wake-latency threshold.** None exists; the self-test only reports. Affects 021-09, 021-23. Answer: a measurement (021-23); a threshold enters the constants table only if E misses target and the excess correlates with wake latency.
7. **L2 cross-sink skew statistic.** C-82 gives the limit but not the statistic (C-79 defines it for L3 only). Affects 021-23. Answer: the L2 harness epic 020.

## Settled since the split

- Re-admission on wake (former gap 6): C-124 — sleep is a departure, both loops reset, normal admission (Locking, Aligning); time bounded by C-63 and C-50; dead counted per C-111.
- Sleep-time clock stop (former gap 15): C-124 — no loop state crosses sleep.
- Ownership of the edge resampler (former gap 7): C-80, C-105 (revised) and C-134 — it is the kit's resampler node, allowed only inside an edge module; its conversion delay is always a whole number of domain-rate samples and is part of D, never of L (C-207); only the 2:1 and 1:2 ratios exist in v1, so 021-11 refuses devices that cannot run at one of them; the wire is 96 kHz only (C-01). The earlier rational-L handling in 021-14 is removed.
- E admission refusal and pre-certification E (former gap 12): C-103 and C-140 — E *unknown* unless the certification state is *commissioned*; refused with `RequiredEExceeded` where a required E is set; refusal is core admission.
- AC-9 editing the epic itself (former gap 14): epic change — criteria now have ids (021-AC1 to 021-AC10) and 021-AC9 names 021-25 as the ticket that edits the platform mapping.
- Stamp field values (former gap 11): C-18, C-47, C-103, C-129, C-131, C-140 (certification states), C-152 (peer hello DSCP), C-159 (full counter list) and C-160 (lateness percentiles) fix the fields; only the D-method values and encoding remain (gap 5).
- "Interface held" condition (former gap 5): C-175 — the device cannot be opened in the required mode because another process has it open (021-12).
- Which E in L2 pass criteria (former gap 7): C-149 — the platform target, 100 µs (021-24, 021-29).
- Reference device and reference list size (former gap 8): C-172 and C-138 — the implementing team proposes the reference hardware after P0 and the product owner approves it (new ticket 021-31); the list is that hardware on the supported OS versions that passes L2.
- D share of the macOS E budget (former gap 11): C-150 — 40 µs clock + 40 µs tracking + 20 µs uncertainty of D, a proposal pending L2; the 021-22 falsifier is a mismatch above 20 µs.
- Cost calibration procedure (former gap 13): C-176 — the DSL conformance suite (epic 025) runs a reference workload once per listed target (021-27).
- Stamp sending: C-144 — the hosted backend (epic 015) sends the regime stamp message that epic 013 defines.
- L2 run conditions: C-147 (strict mode), C-148 (A computed by the harness with the commissioning rule), C-166 (test parameters) applied to 021-23, 021-24, 021-29.
- Privileged tests: C-146 — a privileged nightly job (021-01) runs the wall-clock step test of 021-28 and, if needed, the DF test of 021-03.
- Strict mode details: C-161 (control isolation *present* by construction) and C-165 (strict mode set by the commissioning configuration) in 021-30.
- Commissioning window life cycle: C-153 in 021-20 and 021-22.
- Who builds the resamplers: C-197 — the arithmetic kit (epic 024a, moved to P4) builds the fixed 2:1 / 1:2 node and the steerable rate actuator; this epic uses them and builds none. Epic 024a added to the epic's dependencies and to 021-14, 021-15, 021-16.
- Resampler delay and L: C-207 — the conversion delay is counted only in D, never in L; L is the module's structural delay in the domain-rate timeline; the manifest reports the conversion delay separately on the device-facing port. 021-14 no longer reads the delay as the node's L (Work and criteria corrected); 021-15 and 021-16 report it in the manifest; 021-22 wording aligned.
- "Announced step" in 021-AC2 (former gap 5): C-191 — the step is 5 ms; loss bursts are spaced 1 s apart on the media path only; each 10 s comparison window starts when the injected event ends; the handover is triggered 60 s into the run. The step scenario itself, including how it is announced, is run as the epic 020 harness defines it. Applied in 021-24; the epic's open sub-question row is removed.
- Handover and sync loss at L2: C-202 and C-212 — the handover is a holder loss with about 2 s of holdover; the run asserts emission error within the 100 µs target E during holdover and frequency continuity at takeover, and fails if no holdover was observed; no separate sync-loss scenario (021-24, 021-AC2).
- Certification record stamps: C-191 — "both stamps" are the regime stamps of the sink and of the holder it followed (021-26).
- What a commissioning window disqualifies: C-178 — production measurements are unqualified while it is open; the commissioning measurement of D in 021-22 is a valid commissioning result recorded with its window. The earlier wording "measurements taken in a window are unqualified" is corrected in the epic, 021-20 and 021-22.
- Certification while unlocked: C-180 — a commissioned sink stays commissioned through an unlocked clock (for example after wake); its alignment claim is suspended until Locked (021-18, 021-20).
- Stamp lateness window: C-182 — rolling 60 s window, maximum since live beside it (021-20).
- Changed A on a live sink: C-186 and C-198 — the sink re-enters aligning with its ring resized; its receiver reports carry its current A; the resize happens outside the I/O callback (021-15).
- Two kinds of resampler: C-189 and C-210 — the steerable rate actuator runs at every device rate, not only off 96 kHz, and its delay is part of D; its trait and ±1 000 ppm range are the kit's. 021-14, 021-15 and 021-16 corrected (they had tied the actuator to non-96 kHz devices and left its delay out of D).
- Seqlock safety and pinning: C-177 and C-201 — a layout whose seqlock safety depends on a failed or unverified CPU pin refuses to start in every mode; default priorities never depend on pinning (021-30).
- Handover rig and takeover: C-212 (revised) and C-213 — frequency continuity at takeover means a rate change within C-61 and no phase step; the rig needs a second holder-eligible Linux node; a holder-eligible candidate with usable time (Locked, or in holdover with its claim not lapsed, C-65, C-169) takes over and keeps the timeline epoch, which C-113 bumps only when no candidate has usable time. 021-24 Context, Work and criteria extended (rig, epoch unchanged across the handover); 021-19 and the epic's edge-only ruling cite C-213 beside C-113.
- Sink switch position on a changed A: C-214 — the sink switches at its own position one lead (C-43) ahead, as a fan-in does (C-208), and announces nothing. 021-15 corrected (it had the sink re-aligning "at an announced position"); the epic's edge contract states it.
- Handover rulings and sink rules: C-223 — the 50 ppm is the measured natural difference between the two holders' crystals plus an injected drift, both recorded in the run report (epic 021-AC2, 021-24 Context and criterion 3); C-221 — after the loss every candidate is in holdover and the lowest module id wins, so the epic 020 harness gives the second Linux node the lowest id; the macOS edge is never a candidate (021-24); C-220 — per-sink emission error in holdover stays an L2 assertion (epic 021-AC2, 021-24); C-212 rechecked — the epic's 021-AC2 and 021-24 already name the rig's second holder-eligible Linux node. C-222 and C-225 — the sink clamps a change of A that does not fit and takes a ratchet step at its own position one lead ahead, unannounced, through the shared core (021-15). No new ticket; the tally is unchanged.
