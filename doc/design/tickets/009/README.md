# Epic 009 — Gap and reassembly core: tickets

## Summary
Epic 009 (phase P3, scope L: 29 tickets, about 29 days; depends on epics 004, 006, 007, 008; blocks 012, 014, 015, 017, 024, 026, 028) implements the receive side of the data plane in `hadronyche-core` as pure `no_std` code with no sockets and no allocation, capacities as const generics: packet validation and source filtering by pinned IPv4 address and port plus stream (C-100), copy-index dedup (C-07), multi-packet reassembly with a deadline (C-05, C-15 arrival-driven, C-110 aligned), a per-input alignment ring sized to A (C-96), a fan-in that fires at the fixed time `T(p) + offset` and never early, an offset estimator (max lateness over 4 s plus headroom, C-50 to C-52) with a ratchet capped by A (C-53, C-54, C-101) and demotion (C-109), the three gap classes (marked gap, late, missing; C-108 reclassification), the degraded flag and its delay through a whole-sample structural latency L (C-105, C-122), the health states Live, Interrupted (C-38, C-102), Demoted (C-109) and Dead (C-39, C-111, C-135) with the session check (C-19), the C-159 stamp counters and the alignment-group figures (C-157, C-160) over a rolling 60 s window (C-182), the cannot-meet-A test (C-174), re-alignment of a fan-in when its inherited A changes (C-186) at a position it picks one lead ahead and announces (C-208; a ratchet step likewise, C-215), with the ceiling clamped to the compiled ring and cannot-meet-A raised and held until a change that fits when the new A does not fit (C-211, C-216), and the current A carried in receiver reports, the given A rather than the clamped one (C-198, C-216), default concealment (C-158, C-141, C-184, C-199, C-200), a per-cycle drain budget (C-16, cycle one slice period, C-156), a bad-input cost check (C-164), plus unit, conformance, fuzz (C-95) and scenario tests.

Epic acceptance criteria are numbered EC-1 to EC-12 in the epic; system criteria keep their AC-n ids.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 009-01 | Rx module scaffold and shared types | S | epic 004 |
| 009-02 | No-alloc test allocator | S | 009-01 |
| 009-03 | Drain budget helper | XS | 009-01, 009-02 |
| 009-04 | Source filter | S | 009-01, 009-02, epic 004 |
| 009-05 | Copy-index dedup | S | 009-01, 009-02, epic 004 |
| 009-06 | Reassembler: slice assembly and dedup | M | 009-05, epic 004 |
| 009-07 | Reassembler expiry of incomplete slices | S | 009-06 |
| 009-08 | Media packet conformance vectors | S | 009-04, 009-07, epic 007 |
| 009-09 | Fuzz targets for packet validation and reassembly | S | 009-04, 009-07 |
| 009-10 | Per-input alignment ring | M | 009-06, epic 006 |
| 009-11 | Alignment memory sizing and report | S | 009-10 |
| 009-12 | Fan-in deadline and fixed-time rounds | M | 009-07, 009-10, epic 006 |
| 009-13 | Offset estimator (aligning state) | S | 009-12, epic 008 |
| 009-14 | Gap counters and late/missing attribution | S | 009-12 |
| 009-15 | Input health: interrupted and dead thresholds | S | 009-14 |
| 009-16 | Offset ratchet and demotion | S | 009-13, 009-14, 009-15, epic 008 |
| ~~009-17~~ | ~~Decide and implement demoted-input retry~~ — removed: C-109 rules that Demoted is cleared only by re-admission, so there is no retry to decide or implement; the "stays demoted" and `readmit` tests moved to 009-16 | — | — |
| 009-18 | Unaligned input opt-out | XS | 009-13, 009-16 |
| 009-19 | Default concealment | S | 009-02, epic 004 |
| 009-20 | Diamond alignment test (EC-2, system AC-10) | S | 009-07, 009-13, 009-14, 009-19, epic 006 |
| 009-21 | Bounded drain under flood (EC-6, system AC-11 core part) | S | 009-03, 009-04, 009-07, 009-14 |
| 009-22 | Design documents and epic close-out | XS | 009-01 to 009-16, 009-18 to 009-21, 009-23 to 009-30 |
| 009-23 | Degraded flag through structural latency L (C-122; whole-sample L per revised C-105, rational-L handling removed) | S | 009-12 |
| 009-24 | Session check on resume (C-19) | XS | 009-06, 009-15 |
| 009-25 | Alignment-group stamp figures: lateness percentiles and reclaimable latency (new, C-157, C-160) | S | 009-13, 009-02 |
| 009-26 | "Cannot meet A" test (new, C-174) | XS | 009-13, 009-16 |
| 009-27 | Transport-state concealment (C-141 as revised; stopped does not advance per C-200, default tempo 120 bpm per C-199) | S | 009-19, 009-12, epic 004 |
| 009-28 | Bad-input processing bound (new, C-164) | XS | 009-04, 009-07, 009-09 |
| 009-29 | Re-alignment on a change of inherited A (C-186; position picked by the fan-in per C-208, ceiling clamped per C-211) | S | 009-11, 009-13, 009-16, 009-26 |
| 009-30 | Inherited A carried in receiver reports (new, C-198) | S | 009-11, 009-29, epic 014 |

## Effort tally
- XS: 6 (009-03, 009-18, 009-22, 009-24, 009-26, 009-28)
- S: 20
- M: 3 (009-06, 009-10, 009-12)
- Total: 29 tickets, about 29 days (XS 0.25 d, S 1 d, M 2.5 d), scope L (C-126)

No ticket was added or deleted in this update: C-215 and C-216 turned the readings of 009-16, 009-26 and 009-30 into settled rules (no change of behaviour), C-214 and C-216 were added to 009-29, C-148 to 009-25, and 009-22 lists them as rulings instead of the epic's own readings. The only removed ticket is still 009-17 (see the table).

## Coverage

### Deliverables

| Deliverable | Tickets |
|---|---|
| `SourceFilter<N>` with `admit` (IPv4, C-100) | 009-04 |
| `Reassembler<SLOTS, MAX_SLICE>::push` (dedup by stream, position, index; session carried) | 009-05, 009-06 |
| `Reassembler::expire` (C-15) / `expire_before` (C-110), incomplete reassemblies and missing packets | 009-07, 009-12, 009-14 |
| `AlignRing<DEPTH, MAX_SLICE>` `insert` / `take` (Stored, Late, OtherEpoch, Duplicate, TooEarly) | 009-10 |
| `GapClass`, `InputHealth` (Live, Interrupted, Demoted, Dead) | 009-01 (types), 009-12, 009-14, 009-15, 009-16, 009-24 (semantics) |
| `FanIn::deadline_ns`, `on_deadline` (Round, degraded flag) | 009-12 |
| `FanIn::observe_lateness`, `finish_aligning`, `reclaimable_ns` (C-157) | 009-13 |
| Lateness percentiles and alignment-group summary (C-157, C-160), rolling 60 s window and maximum since live (C-182; the latter is the figure for the commissioning proposal of A, C-148) | 009-13, 009-25 |
| Cannot-meet-A test (C-174, rolling window C-182) | 009-26 |
| `FanIn::change_inherited_a`: re-alignment on a change of inherited A (C-186), position picked and announced by the fan-in (C-208), ceiling clamped and cannot-meet-A raised and held until a change that fits when the A does not fit (C-211, C-216) | 009-29, 009-26 |
| `FanIn::current_a_ns`, `on_downstream_a`: A carried in receiver reports, the given A after a clamped change (C-198, C-216) | 009-30 |
| `FanIn::ratchet` (Stepped at an announced position one lead ahead per C-215 / AtCeilingDemoted / None, last step to A per C-163), `Round::announce`, Demoted state | 009-16 |
| `FanIn::readmit` | 009-15, 009-16 |
| `FanIn::set_unaligned` (operator policy, C-165) | 009-18 |
| `DegradedDelay<ROUNDS>` (whole-sample L, C-105) | 009-23 |
| Per-input health, session and counters in `FanIn` | 009-14, 009-15, 009-24 |
| `Conceal` trait, `DefaultConceal` (dense initial value C-158), per-input override, mixed audio-and-controller slices (C-181) | 009-19 |
| Transport-state concealment (C-141, C-200; C-184 and C-199 before any state) | 009-27 |
| `InputCounters`, C-47 / C-159 stamp counters | 009-01, 009-07, 009-14, 009-15 |
| `DrainBudget` helper (cycle one slice period, C-156) | 009-03 |
| Ring depth for A, memory report, `check_depth` incl. no-budget refusal (C-27, C-96, C-142, reason NoSkewBudget C-185; the commissioning tool's check, C-211) | 009-11 |
| Unit tests | every code ticket 009-03 to 009-30 |
| Bad-input cost check (C-164) | 009-28 |
| Conformance-vector tests (vectors crate at the workspace root, C-192, C-193) | 009-08 |
| Fuzz targets (packet validation, reassembly), C-95 budget, tool of the team's choice (C-192) | 009-09 |
| No-alloc test allocator and lock lint | 009-02 |
| Design document updates (definition of done) | 009-22 |

### Acceptance criteria

| Epic criterion | Tickets |
|---|---|
| EC-1 (system AC-1) conformance vectors 100 %, zero panics/hangs under fuzzing at C-95 | 009-08, 009-09 |
| EC-2 (system AC-10) diamond bit-exact against the defined offline reference | 009-19 (reference mixer), 009-20 |
| EC-3 (system AC-15) late only / missing only, C-108 | 009-10, 009-14 |
| EC-4 (system AC-16) marked gap + degraded; interrupted at exactly 8 (C-102); dead at 2 s (C-111, C-135); immediate resume on the same session (C-19) | 009-12, 009-15, 009-24 |
| EC-5 (system AC-18) other epoch is a gap; no round combines two epochs | 009-10, 009-12 |
| EC-6 (system AC-11 core) 10 × flood bounded by C-16, deadline met | 009-03, 009-21 |
| EC-7 fixed-time firing | 009-12 |
| EC-8 dedup: duplicate counted and discarded, first valid copy wins | 009-05, 009-06, 009-14 |
| EC-9 multi-packet: incomplete slice dropped at deadline (C-15, C-110), missing packets counted | 009-07, 009-12 |
| EC-10 `no_std` without allocator; 16-member bundle at 10 ms ≈ 370 KB by const fn; depth for A and `check_depth` | 009-01, 009-11 |
| EC-11 ratchet (C-101, last step C-163) and demotion (C-109), cleared only by `readmit` | 009-16 |
| EC-12 degraded through whole-sample L (C-122, C-105) | 009-23 |

### Open sub-questions

| Question | Handling |
|---|---|
| C-16 drain bound vs legitimate bursts (L1 / L2 Windows, tasks 019, 022) | Note in 009-03, 009-21; recorded in 009-22 |
| Cycle of C-16 | Closed by C-156 (one slice period); 009-03, 009-21 |
| Maximum skew budget a ring must support | Closed by C-96 and C-142; 009-11 sizes rings to the budget, `check_depth` refuses no budget |
| Max-plus-headroom over 4 s sufficient? (task 008, then L1) | Note in 009-13; recorded in 009-22 |
| Should a demoted input be re-tried automatically? | Closed by C-109 (no); 009-17 removed; tested in 009-16 |
| Slot eviction when all SLOTS are busy (this epic, then L1) | Implemented and recorded in 009-06 |
| Reclaimable latency definition | Closed by C-157; 009-13 computes, 009-25 publishes |
| Window of the current maximum lateness and percentiles | Closed by C-182 (rolling 60 s, also for cannot-meet-A; maximum since live beside them); 009-13, 009-25, 009-26 |
| Transport state before any has arrived | Closed by C-184 and C-199 (stopped, playhead zero, 120 beats per minute); 009-27 |
| Playhead of a held transport state through a gap | Closed by C-141 as revised and C-200 (stopped stays, playing moves on at its tempo); 009-27 corrected |
| Change of an inherited A while inputs are live | Closed by C-186 (re-align at an announced position, rings and ceiling resized); 009-29 |
| How a changed A reaches a fan-in | Closed by C-198 (current A in the receiver report, one hop per report); new 009-30 |
| Announced position of a C-198-triggered re-alignment | Closed by C-208 (the fan-in picks it one lead ahead and announces it in its output, as for a ratchet step); 009-16, 009-29, 009-30 corrected |
| A changed A deeper than the fan-in's ring | Closed by C-211 (refused beforehand by the commissioning tool; otherwise ceiling clamped, cannot-meet-A raised, streams kept); 009-11, 009-26, 009-29, 009-30 corrected |
| Does a ratchet step take effect on the next round or at an announced position? | Closed by C-215 (a re-alignment like any other, announced one lead ahead, never silently on the next round); 009-16's reading now settled |
| Cannot-meet-A and the reported A after a clamped change | Closed by C-216 (alarm raised until a later change that fits; report carries the given A, not the clamped value); 009-26, 009-29, 009-30's readings now settled |
| Where a sink switches to a changed A ("as the sinks do", C-186) | Closed by C-214 (its own position, one lead ahead, unannounced); sink epics, noted in 009-29 |
| Does the C-211 clamp, with C-216, apply to sinks? | Closed by C-222 (any consumer that buffers to meet A, sinks included); 009-26, 009-29 |
| Ratchet step in a sink, which has no output to announce in | Closed by C-225 (its own position, one lead ahead, as for a changed A, C-214); 009-16, 009-29 |
| No lateness sample during aligning (epic 005) | Assumed "stay Aligning" in 009-13 |
| Dense hold-last with no last value | Closed by C-158 (manifest initial value, default 0); 009-19 |
| Stamp class of the extra counters | Closed by C-159; 009-01, 009-14 |
| Interrupted input returning on a new session (epic 005) | Event only, in 009-24 |
| C-122 wording vs position invariant | Closed by C-122 as revised; 009-23 |

## Remaining gaps
Only what neither the epic nor the constants table answers.

1. **Reassembler slot exhaustion.** Behaviour when all `SLOTS` are busy is not in the design; 009-06 evicts the oldest incomplete slot and counts it. Affects 009-06, 009-07. Answer: this epic records it as its own decision; L1 confirms (falsifier: `slot_overflow` rises on a clean network).
2. **No lateness sample during aligning.** What a consumer does when an aligned input is silent for the whole 4 s. Affects 009-13. Answer: control state machines (epic 005).
3. **Interrupted input returning on a new session.** C-19 defines the session, but not the consumer's response when it changes; 009-24 emits `SessionChanged` and does not resume. Affects 009-15, 009-24. Answer: control state machines (epic 005).
4. **C-164 bad-input bound.** The 10 µs per packet figure is a proposal. Affects 009-28. Answer: measurement, L1 (task 019).
5. **C-16 drain bound against legitimate bursts.** Affects 009-03, 009-21. Answer: measurement, L1 / L2 Windows (tasks 019, 022).

## Settled since the split
Former "Gaps in the epic", by number:

1. `InputCounters` lacked duplicate and reassembly-missing counters — epic change: fields added; publication settled by C-159 (adds `incomplete_reassemblies`; missing packets and rejects stay receiver-report counters).
2. Late vs missing attribution — C-108.
3. Meaning of "missed slice" and "without data" — C-102 (misses; marked gap and other epoch reset the count), C-111 (dead counted from the last slice with content or a marked gap) and C-135 (an other-epoch slice also restarts the dead clock and has its own counter).
4. "Persistent lateness" undefined — C-101.
5. "Demoted to gapped" had no state — C-109 (health state Demoted, counted separately, cleared only by re-admission); also closes the demoted-retry sub-question, so 009-17 was removed.
6. `GapClass` mixed classes and states; wrong-epoch class unstated — epic change (three gap classes; Dead and Demoted are health states; `InputAt::OtherEpoch`), C-102 and C-135.
7. Reassembly deadline for aligned inputs — C-110.
8. Slot eviction — still remaining gap 1 (this epic's own decision, L1 confirms).
9. No ring outcome for a slice too far ahead — epic change (`TooEarly`, ring sized to A per C-96; C-142 for a fan-in outside any group).
10. Fate of packets beyond the drain budget — settled by the epic (they stay in the socket; the kernel counter counts overflow); the cycle is one slice period (C-156).
11. No API for the unaligned opt-out — epic change (`set_unaligned`); set by operator policy in the commissioning configuration (C-165).
12. Gap slice on a module's own deadline miss and p − L stamping — C-144 (hosted backend, epic 015); the degraded flag through L is delivered here per C-122 (009-23).
13. Reclaimable latency — C-157 (definition and publication in the alignment group; 009-13, 009-25); its window, and that of the percentiles and the cannot-meet-A test, is a rolling 60 s window with the maximum since live beside them (C-182; 009-13, 009-25, 009-26).
14. No lateness sample during aligning — still remaining gap 2.
15. 1 152 bytes per slice unexplained — C-04 (6 × 192 B, 4-byte samples, no per-channel id byte); the memory arithmetic is 16 × 20 × 1 152 B = 368 640 B ≈ 370 KB (C-27).
16. Offline reference for the diamond undefined — epic change (EC-2 defines it: same test processor and reference mixer on undelayed slices).
17. Concealment override and hold-last default — override settled by the epic (per-input `C: Conceal`); dense controller with no prior value settled by C-158 (manifest initial value, default 0).
18. Fuzz duration — C-95 (60 s per change, 1 h nightly, corpus kept); the 1 s per-input hang timeout stays a test parameter.
19. Regime stamp fields touched not listed — C-47 and C-159 (per-connection counters), C-157 and C-160 (alignment group), C-18 (stamp transport, outside this epic); section layout left to epic 004.
20. `FanIn` lacked ring const generics — epic change (`FanIn<INPUTS, DEPTH, MAX_SLICE>`).
21. "Same session" undefined — C-19 (009-24; the response to a new session is remaining gap 3).
22. No lock check named — epic change (lock lint added to the deliverables, implemented in 009-02).
23. Announced position of a re-alignment triggered by a receiver report (former remaining gap 6) — C-208: the fan-in picks it one lead (C-43) ahead and announces it in its own output (009-29, 009-30; the same rule moves a ratchet step to an announced position, 009-16).
24. The epic's own readings recorded in 009-22 — now settled rules: a ratchet step at an announced position one lead ahead (C-215, 009-16); cannot-meet-A held until a later change of A that fits and the given A, not the clamped value, carried upstream (C-216, 009-26, 009-29, 009-30). The reading on the maximum since live across a C-186 re-alignment stays this epic's own (009-29).

Rulings applied in the handover update: C-222 (the clamp of C-211, and C-216 with it, applies to any consumer that buffers to meet A, sinks included; `set_ceiling_short` and `check_depth` are named as the calls a sink epic uses, in the epic, 009-26 and 009-29) and C-225 (a sink takes a ratchet step at its own position one lead ahead, unannounced, as for a changed A; the same `Stepped` outcome serves it and the sink ignores `announce`, in the epic, 009-16 and 009-29). Both are consistent with C-214 and C-215. C-220, C-221, C-223 and C-224 have no subject in this epic (C-224 is the commissioning tool's and the sink epics'). No ticket added or deleted.

Rulings applied in the update before the handover update: C-215 (a ratchet step is a re-alignment like any other, announced one lead ahead, never silently on the next round; settles the reading in 009-16, no change of behaviour), C-216 (cannot-meet-A stays raised until a later change of A that fits; the receiver report carries the A given, not the clamped value; settles the readings in 009-26 and 009-30, cited in 009-29), C-214 (a sink switches at its own position one lead ahead, with nothing to announce; sink epics' work, noted in the epic and 009-29), C-148 (the commissioning proposal of A and the ladder runs use the maximum lateness since live, not the rolling 60 s figure; noted in 009-25, whose `max_since_live_ns` already provides it). 009-22 now lists these as rulings and drops the two readings they settle. No new work, no ticket added.

Rulings applied in the update before that: C-208 (re-alignment position picked by the fan-in one lead ahead and announced in its output; replaces the caller-supplied position in 009-29 and 009-30, closes former remaining gap 6, and makes the ratchet step of 009-16 take effect at an announced position), C-211 (a change of A deeper than the compiled ring is applied with the ceiling clamped and cannot-meet-A raised, streams kept; replaces the refusal and "keep the old A" in 009-29 and 009-30; `set_ceiling_short` in 009-26; `check_depth` named as the commissioning tool's check in 009-11), C-185 (NoSkewBudget refusal reason, 009-11), C-08 revised and C-209 (controller member counts towards the 32; 009-04, 009-11, 009-12), C-181 and C-205 (controllers in their own member stream or sharing a single-stream source's packet; section-by-section concealment in 009-19), C-178 (figures and counters unqualified while a commissioning window is open; noted in 009-14, 009-25), C-187 (clock bound from the servo core 008a; 009-13), C-192 and C-193 (vectors crate location and sources, fuzzing tool of the team's choice; 009-08, 009-09). C-177, C-179, C-180, C-183, C-188 to C-191, C-194 to C-197, C-199 (already applied), C-201 to C-207, C-210 and C-212, and the late revisions of C-134 (now citing C-207, already consistent here), C-141 and C-184 (already applied), C-153 (through C-178) and C-169, create no further work in this epic; C-206 has no subject, since the epic builds no tool.

Rulings applied in an earlier update: C-141 revised and C-200 (the playhead of a stopped transport does not advance, a playing one moves on at its tempo; corrects 009-27, which added the positions elapsed whatever the state), C-199 (default tempo 120 beats per minute; 009-27), C-198 (receiver report carries the consumer's current A, one hop per report; 009-29 updated, new 009-30; the announced position stays remaining gap 6).

Rulings applied before that: C-182 (rolling 60 s window for the percentiles, reclaimable latency and cannot-meet-A, maximum since live beside them; replaces the "since the last stamp publication" assumption and the window reset in 009-13 and 009-25, and the 1 s window maximum in 009-26), C-184 (transport state before any has arrived; corrects 009-27, which used the manifest initial value), C-186 (re-alignment of a fan-in on a change of inherited A; new 009-29).

Rulings applied in older updates: C-104 and C-105 revised (L is a whole number of samples; rational-L handling, rounding and fractional delay removed from the epic and 009-23), C-122 revised wording (closes the former wording gap), C-134 (resampler only inside an edge module, its delay is part of D), C-135 (009-10, 009-15), C-141 (new 009-27), C-142 (009-11, 009-16), C-144 (gap slice emission in epic 015), C-156 (009-03, 009-21), C-157 and C-160 (009-13, new 009-25), C-158 (009-19), C-159 (009-01, 009-07, 009-14), C-163 (009-16), C-164 (new 009-28), C-165 (009-18), C-173 and C-174 (new 009-26; alarms raised by epic 012). C-35's owner change and the other new rules (C-127 to C-133 already applied; C-136 to C-140, C-143, C-145 to C-155, C-161, C-162, C-166 to C-172, C-175, C-176) create no work in this epic.

Other rulings applied earlier: C-01 (one rate, 96 kHz), C-100 (IPv4 only, 009-04), C-94 (jitter profile in 009-20), C-96 (rings sized to A, 009-11), C-125 (definition-of-done applicability note in the epic), C-126 (scope label), C-130 and C-133 (output locations, 009-22), C-131 (arrival timestamps come from the backend), C-132 (`[standard]` tag).
