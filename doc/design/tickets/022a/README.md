# Epic 022a — Windows backend and WASAPI edge: tickets

## Summary
Epic 022 was split under C-143 into 022a (this part) and 022b (L2 certification). 022a (phase P4, second half; depends on epics 001, 004, 005, 008a, 015, 016, 018, 020, 024a; blocks 022b) builds the Windows **edge**: a sink and a physical-input capture module on WASAPI in exclusive mode (96 kHz or 48 kHz devices only, C-105), plus a Windows implementation of the hosted backend traits (QPC host clock, IPv4 Winsock sockets with interface-pinned multicast at TTL 1, connection-reset disabled, receive timestamps, QoS-flow DSCP with peer verification of all three classes, C-47 and C-152, MMCSS real-time threads, memory lock), sync-exchange building with the wall-clock step check (C-121), strict mode set by operator policy (C-127 to C-129, C-165), firewall and power handling (sleep is a departure, C-124), a deadline clock driven by the audio callback, and the complete `windows-std` regime stamp (C-140, C-159 to C-161). The edge never holds the clock, never falls back to shared mode, uses no spinning ticker, and declares E unknown unless its certification state is *commissioned* (C-140, C-103).

**Scope:** L (21 tickets, about 30.75 days), computed per C-126.

## Tickets

| Ticket | Title | Effort | Depends on | Former id |
|---|---|---|---|---|
| 022a-01 | Crate skeleton and Windows CI (per-change and privileged nightly jobs, C-146) | S | epic 015, epic 016 | 022-01 |
| 022a-02 | QPC host clock | S | 022a-01, epic 015 | 022-02 |
| 022a-03 | UDP socket baseline (IPv4, buffers, connection-reset, don't-fragment, pinned multicast) | M | 022a-01, epic 016 | 022-03 |
| 022a-04 | Receive timestamps and batch receive | M | 022a-02, 022a-03, epic 001, epic 016 | 022-04 |
| 022a-05 | DSCP marking through QoS flows and TOS-byte receive | M | 022a-03, 022a-04, epic 004 | 022-05 |
| 022a-06 | `WinNet`: OsNet assembly and interface statistics | S | 022a-03, 022a-04, 022a-05, epic 016 | 022-06 |
| 022a-07 | `WinRealtime`: MMCSS registration, timer resolution, seqlock pin check (C-177, C-201) and wake-latency self-test | S | 022a-02, epic 016 | 022-07 |
| 022a-08 | Firewall blocking detection at startup | S | 022a-06 | 022-08 |
| 022a-09 | Power: stay awake while live; sleep is a departure | S | 022a-01, epic 005, epic 018 | 022-09 |
| 022a-10 | WASAPI exclusive-mode acquisition and "interface held" reporting | S | 022a-01 | 022-10 |
| 022a-11 | `DeviceClock` from (device position, QPC) pairs | M | 022a-02, 022a-10, epic 018 | 022-11 |
| 022a-12 | Media deadline clock from the audio callback | S | 022a-11, epic 018 | 022-12 |
| 022a-13 | WASAPI exclusive-mode sink | M | 022a-06, 022a-07, 022a-10, 022a-11, 022a-12, epic 008a, epic 018, epic 024a | 022-13 |
| 022a-14 | WASAPI exclusive-mode capture with batched emission | M | 022a-06, 022a-07, 022a-10, 022a-11, 022a-12, epic 018, epic 024a | 022-14 |
| 022a-15 | `RawCapture` for the L2 harness | S | 022a-10, 022a-11, epic 020 | 022-15 |
| 022a-16 | Edge-only: hello never carries the holder-eligible bit | XS | 022a-01, epic 018 | 022-16 |
| 022a-17 | Windows regime stamp (`windows-std`) — now M (C-140, C-152, C-159 to C-161) | M | 022a-06, 022a-07, 022a-10, 022a-13, 022a-14, 022a-20, 022a-21, epic 004, epic 008a, epic 015 | 022-17 |
| 022a-18 | No-ticker check: idle live sink CPU below 0.1 core | S | 022a-13 | 022-18 |
| 022a-19 | Sync-exchange building and wall-clock step check (C-121) | S | 022a-02, 022a-04, epic 016, epic 018 | 022-27 |
| 022a-20 | Strict mode, memory lock and capability states (C-127 to C-129, C-165) | S | 022a-07, epic 016 | 022-28 |
| 022a-21 | Peer verification of sync and control DSCP (new, C-152) | S | 022a-05, 022a-06, epic 004 | — |

No ticket was deleted. The withdrawal of the rational-L rule (C-105 revised) removed no ticket: no ticket handled rational latency, rounding of L or fractional delay lines. It removed the "any ratio is representable" clause from 022a-10, which now accepts only 96 kHz and 48 kHz devices.

## Effort tally
- XS: 1 (0.25 day)
- S: 13 (13 days)
- M: 7 (17.5 days)
- Total: 21 tickets, about 30.75 days → scope L (C-126: XS 0.25 d, S 1 d, M 2.5 d; up to 5 = S, 15 = M, 40 = L, more = XL)

## Coverage

### Deliverables

| Deliverable | Tickets |
|---|---|
| Crate `hadronyche-os-windows` (+ edge-only feature, Windows CI build, privileged nightly job) | 022a-01, 022a-16 |
| `clock.rs` — `HostClock` on QPC (`impl HostClock for QpcClock`) | 022a-02 |
| `net.rs` — IPv4, interface-pinned multicast with TTL 1, connection-reset disabled, don't-fragment, buffers sized to rate × A | 022a-03 |
| `net.rs` — receive timestamps (kernel if 001 confirms, else userspace; C-131 states), batch receive | 022a-04 |
| `net.rs` — QoS-flow DSCP, receive of the TOS byte, media verification from consumer stamps (C-47) | 022a-05 |
| `net.rs` — per-peer hello DSCP report, sync and control verification (C-152) | 022a-21 |
| `net.rs` — `impl OsNet for WinNet` (open_media, open_discovery, iface_stats), C-120 raw-send hook | 022a-06 |
| `net.rs` — sync-exchange building and step check (C-121) | 022a-19 |
| `rt.rs` — `impl OsRealtime for WinRealtime`, MMCSS, timer resolution, clock-above-data priorities and seqlock pin check (C-177, C-201), self-test, *bounded real-time: absent* | 022a-07 |
| `rt.rs` — memory lock, C-129 capability states, strict-mode refusal (C-165 policy) | 022a-20 |
| `firewall.rs` | 022a-08 |
| `power.rs` (stay-awake, throttling opt-out, sleep as departure) | 022a-09 |
| `wasapi/device.rs` — exclusive acquisition at 96 or 48 kHz (C-105), "interface held" (C-175) | 022a-10 |
| `wasapi/device.rs` — `impl DeviceClock for WasapiDevice` | 022a-11 |
| `deadline.rs` | 022a-12 |
| `wasapi/sink.rs` (C-135 counting, C-156 drain, C-140 E declaration, kit resamplers C-197 through the C-210 trait, conversion delay outside L C-207, A carried in receiver reports C-198, re-alignment on a changed A C-186 at its own position one lead ahead C-214, certification kept while unlocked C-180) | 022a-13 |
| `wasapi/capture.rs` (batch, batch size recorded, session numbers, C-137 header cap, kit resamplers C-197 through the C-210 trait, conversion delay outside L C-207) | 022a-14 |
| `impl RawCapture for WasapiRawCapture` (harness binary in `hadronyche-tools`, C-133) | 022a-15 |
| `windows-std` stamp builder and reference-list lookup | 022a-17 |

### Acceptance criteria

| Epic AC | Tickets |
|---|---|
| 022-AC-3: exclusive mode / "interface held" (C-175) / *uncertified*; rates other than 96 and 48 kHz refused (C-105) | 022a-10, 022a-17 |
| 022-AC-4: connection-reset | 022a-03 |
| 022-AC-5: firewall, named failure within 5 s | 022a-08 |
| 022-AC-6: no ticker; idle live sink < 0.1 core | 022a-12, 022a-18 |
| 022-AC-7: edge only (hello) | 022a-16 |
| 022-AC-8 (AC-31): stamp | 022a-17 (fields from 022a-04, 022a-05, 022a-06, 022a-07, 022a-10, 022a-13, 022a-14, 022a-20, 022a-21; calibration factor loaded by 022b-09) |
| 022-AC-11: sleep is a departure (C-124), new session numbers (C-19) | 022a-09, 022a-14 |
| 022-AC-12: E unknown unless *commissioned* (C-140, C-103); commissioned kept while unlocked (C-180) | 022a-13, 022a-17 |
| 022-AC-13: strict mode (C-127, C-128, C-165); seqlock pin refusal in every mode (C-177, C-201) | 022a-20, 022a-07 |
| 022-AC-14: step check (C-121) | 022a-19 |
| 022-AC-16: peer DSCP verification (C-152) | 022a-21 (hardware check in 022b-06) |

022-AC-1, 022-AC-2, 022-AC-9, 022-AC-10 and 022-AC-15 belong to epic 022b.

### Definition of done

| DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged (per-change; privileged tests nightly, C-146) | all tickets; job setup in 022a-01 |
| Every parser touched is fuzzed (C-95 budget) | 022a-04, 022a-05 |
| Regime stamp fields touched are complete and documented | 022a-06, 022a-10, 022a-14, 022a-17, 022a-20, 022a-21 |
| Design documents updated, constants table first | 022b-08 (for both parts) |
| No new open item without owner and falsifier | 022a-08 (firewall measurement); all others via the open sub-questions table |
| Frozen POC not modified | Not applicable in this part (C-125): no subject in the POC tree |
| Every new factual claim carries an evidence tag | all |
| Every `[memory]` fact confirmed or corrected | 022a-02, 022a-03, 022a-07 … 022a-11, 022a-20 (measurement); 022b-08 (text) |
| Reference list published; everything else *unverified* | 022a-17 (lookup, non-listed → *unverified*); list published in 022b-07 |
| CI builds on Windows; hardware tests a documented manual job | 022a-01; manual job in 022b-01 |

### Open sub-questions

| Question | Handling |
|---|---|
| UDP receive-timestamping control present? | Owner task 001 (C-138); note in 022a-04, closed in 022b-08 |
| DSCP without administrator rights? | 022b-06 (feeds epic 028) |
| Firewall query alone or active probe? | 022a-08 (measurement) |
| Trait signatures, interface selection, D measurement, batch-size format, departure | Other epics; see Remaining gaps |
| ASIO-class backend | Deferred past v1; no ticket |

## Remaining gaps
1. **Trait signatures** (exact signatures, name of the core deadline-clock trait, the exchange type; the rate-actuator trait is settled as epic 024a's, C-210). Affects 022a-02, 022a-06, 022a-07, 022a-11, 022a-12, 022a-15, 022a-19. Owner: epics 015, 016, 018, 020.
2. **Medianet interface selection** — how the interface index is configured and chosen. Affects 022a-03, 022a-06. Owner: epic 016 (hosted configuration surface).
3. **Firewall detection method** — whether the API query alone detects a dismissed first-run prompt. Affects 022a-08. Owner: a measurement (022a-08 itself).
4. **Field measurement of D and D_capture** — who measures them and how; D is defined by C-170 and read from a configuration key, measured inside a commissioning window (C-107, C-153), where it is a valid commissioning result recorded with its window (C-178). Affects 022a-13, 022a-14. Owner: epic 028.
5. **Batch-size stamp field format** (single number or minimum / typical / maximum). Affects 022a-14, 022a-17. Owner: epic 004 (stamp message layout, C-18).
6. **Departure announcement on sleep** — whether a sleeping edge announces departure so peers see an end rather than dead after 2 s. Affects 022a-09. Owner: epic 005.

## Settled since the split
- Former gap 7, peer verification of sync and control DSCP: C-152 (holder reports observed sync DSCP per follower; every module reports observed hello DSCP per peer). New ticket 022a-21; 022a-05, 022a-06 and 022a-17 updated; a sink-only edge now has two verifiable classes.
- Former gap 9, calibration benchmark: C-176 (epic 025 runs the reference workload once per listed target). The ticket that loads it moved to 022b (022b-09).
- Rational-L withdrawal (C-104, C-105 revised) and resampler placement (C-134): L is a whole number of samples; the resampler node exists only inside an edge module at 2:1 or 1:2; its delay is part of D (C-170). 022a-10 now refuses devices at other rates; 022a-13 and 022a-14 updated. No ticket was rational-L only.
- Who builds the resamplers: C-197 (the arithmetic kit, epic 024a, builds both the fixed 2:1 / 1:2 node and the steerable rate actuator of C-189; the edge builds none). Epic 024a added to the dependencies of the epic, 022a-13 and 022a-14. 022a-13 and 022a-14 corrected: they had run the fixed node at the slope-quotient ratio; drift is now tracked by the kit's rate actuator and the fixed node only converts 48 kHz devices. 022a-11 wording aligned. No new ticket, efforts unchanged.
- Resampler delay and L: C-207 (counted only in D, never in L; the manifest reports the conversion delay separately on the device-facing port). Work item and acceptance criterion added to 022a-13 and 022a-14.
- Certification state and numeric E: only *commissioned* gives a numeric E (C-140); 022a-13 and 022a-17.
- Per-connection counters, lateness percentiles and control isolation: C-159, C-160, C-161, now in 022a-17 (raised from S to M) and 022a-13.
- Other-epoch slices: C-135 (restart the dead clock, own counter); 022a-13.
- Drain cycle: one slice period (C-156); 022a-13.
- "Interface held": C-175; 022a-10.
- Header cap: C-137; 022a-14.
- Privileged tests: separate nightly job (C-146); 022a-01.
- Strict mode set by operator policy (C-165), hosted-only and required for measurement runs (C-147); 022a-20.
- Duties of epic 015 (C-144): gap slice on own deadline miss, processing-overrun event, housekeeping restart (C-139, AC-20 (c)) and sending the stamp are out of scope here; noted in 022a-07, 022a-12, 022a-17.
- Supported OS versions after task 001 (C-138) and the C-150 clock allotment; 022a-04.
- Commissioning-window life cycle (C-153); 022a-17.
- Pass for C-177 to C-212 (this epic was skipped when C-177 to C-196 were first applied; C-197 and C-207 had been applied):
  - Seqlock safety and pins: C-177, C-201 (a failed or unverified pin on which seqlock safety depends refuses startup in every mode; clock-above-data priorities never depend on pinning). Work item and acceptance criterion added to 022a-07; 022a-20 corrected, since it said every real-time failure outside strict mode is non-fatal; epic strict-mode text and 022-AC-13 updated.
  - What an open commissioning window disqualifies: C-178 (production figures in the stamp unqualified; commissioning results such as D valid). 022a-17 and the epic's stamp text and 022-AC-8.
  - Certification while the clock is unlocked: C-180 (stays commissioned, alignment claim suspended until relock). 022a-13, 022a-17, epic edge contract and 022-AC-12.
  - Publisher of the lock state: C-187 (servo core, epic 008a). Epic 008a added to the dependencies of the epic, 022a-13 and 022a-17.
  - Window of the lateness figures: C-182 (rolling 60 s, maximum since live beside them). 022a-13, 022a-17, epic stamp text and 022-AC-8.
  - Changed A: C-186, C-198 (the sink's receiver reports carry its current A; a live sink re-aligns with ring and ceiling resized). 022a-13; 022a-03 already re-sizes buffers on a change of A.
  - Rate-actuator trait: C-210 (epic 024a's, ±1 000 ppm). 022a-13, 022a-14, epic resampler ruling, trait-signature gap.
  - No ticket added or deleted; efforts unchanged.
- Where a sink switches to a changed A: C-214 (at its own position, one lead, C-43, ahead, as a fan-in does, C-208; nothing announced). Replaces the provisional "announced position" reading of C-186 in 022a-13 (context, work item 7, acceptance criterion 10) and the epic's engine-period text. No ticket added; efforts unchanged.
- Earlier settlements kept: receive-buffer sizing (C-96, C-21), sleep as departure (C-124), idle-sink CPU method, wake-latency self-test, discovery DSCP (C-13, C-17, C-18), timer versus power ownership, scope label from the tally (C-126), and C-100 to C-133 as applied before.
- C-35's owner change and C-136, C-141, C-142, C-151, C-154, C-157, C-158, C-162 to C-164, C-167 to C-169, C-173, C-174 create no work in this part (holder, module, consumer-core, device or L1 rules).
- C-179, C-181, C-183 to C-185, C-188 to C-196, C-199, C-200, C-202 to C-206, C-208, C-209, C-211, C-212 and the revised C-08 and C-141 create no work in this part (L1, compiler, controller, transport, holder, fan-in, specification-epic or L2/L3 rules; C-191, C-202, C-203 and C-212 are 022b's). C-206 leaves no presentation item with the design owner here.
