# Epic 017b — L0 harness and scenarios: tickets

## Summary
Epic 017b (phase P4, scope L) is the second part of epic 017, split under C-143; it depends on epic 017a (the reference modules and analyzer). It builds the **L0 loopback harness**: a public ground-truth injection API (`hadronyche-std::inject`) plus the test-only hooks of C-120 and C-145, all recorded in the regime stamp, and a binary `hadronyche-l0` (crate `hadronyche-tools`, C-133) that runs 15 named scenarios in one process on IPv4 loopback, non-strict and unprivileged (C-147), writes one record file through epic 015's single-writer file service (C-144), asserts content, loss accounting against the injector's log, lock, classification and ground-truth error (never latency or sync-rate timing), fails on any inert check family, and runs in CI in under 10 minutes as the regression gate for all later tasks.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 017b-01 | Injection API types | XS | epic 015 |
| 017b-02 | Clock injection | M | 017b-01, epic 015 |
| 017b-03 | Link and hop-delay injection | M | 017b-01, epic 010, epic 015 |
| 017b-04 | Partition, hello suppression and thread-kill injection | S | 017b-01, epic 015, epic 016 |
| 017b-05 | Injection recorded in the regime stamp | S | 017b-01, epic 013, epic 015 |
| 017b-06 | Test-only hooks: forced identity and forced epoch | S | 017b-05, epic 010, epic 015 |
| 017b-07 | Test-only hook: raw packet send | S | 017b-05, epic 015 |
| 017b-08 | Test-only hook: thread panic injection (new, C-145) | S | 017b-05, epic 015, epic 016 |
| 017b-09 | `hadronyche-l0` runner, record file and inert-family failure | M | 017b-05, 017a-04, epic 013, epic 015 |
| 017b-10 | Scenario `baseline`: offset, drift and sync-rate invariance | M | 017b-02, 017a-09, 017b-09, epic 008a, epic 014 |
| 017b-11 | Scenario `loss` and the resynchronisation bound | M | 017b-03, 017a-05, 017b-10 |
| 017b-12 | Scenario `diamond` | S | 017b-03, 017a-07, 017a-08, 017a-09, 017b-09 |
| 017b-13 | Scenario `late-vs-lost` | S | 017b-03, 017a-08, 017b-09, epic 009 |
| 017b-14 | Scenario `gap-classes` | M | 017b-03, 017b-04, 017a-08, 017a-09, 017b-09, epic 009 |
| 017b-15 | Scenario `realign` | M | 017b-02, 017b-03, 017a-08, 017b-09, epic 009, epic 010 |
| 017b-16 | Scenario `epoch` | S | 017a-08, 017b-09, 017b-06, epic 009 |
| 017b-17 | Scenario `blackhole` and forced small path MTU | S | 017b-03, 017a-09, 017b-09, epic 010, epic 012 |
| 017b-18 | Scenario `collision` | S | 017b-09, 017b-06, epic 012, epic 015 |
| 017b-19 | Scenario `malformed` | S | 017b-09, 017b-07, epic 007, epic 010 |
| 017b-20 | Scenario `partition` | S | 017b-04, 017a-09, 017b-09, epic 010 |
| 017b-21 | Scenarios `flood` and `refusal` | M | 017a-08, 017a-09, 017b-09, 017b-07, epic 012 |
| 017b-22 | Scenario `thread-death` | S | 017b-04, 017b-08, 017a-09, 017b-09, epic 015, epic 016 |
| 017b-23 | Scenario `housekeeping` (new, C-139, C-145) | S | 017b-04, 017b-08, 017a-09, 017b-09, epic 015, epic 016 |
| 017b-24 | Fuzz the injection record and the record-file reader (new, from the split) | S | 017b-05, 017b-09 |
| 017b-25 | L0 suite as the unprivileged CI gate | S | 017b-10 to 017b-23 |
| 017b-26 | Design documents and constants for the L0 harness (new, from the split) | S | 017b-01 to 017b-25 |

Origin in the former epic 017: 017b-01 to 017b-05 were 017-01 to 017-05; 017b-06 and 017b-07 were 017-33 and 017-34; 017b-09 to 017b-21 were 017-15 to 017-27; 017b-22 is the thread-death half of 017-28; 017b-25 was 017-30. New: 017b-08 (panic hook, C-145) and 017b-23 (`housekeeping`, now defined by C-139), which replace the provisional housekeeping half of 017-28; 017b-24 and 017b-26 take the harness half of 017-29 (fuzzing) and 017-32 (design documents).

No ticket was deleted: nothing in this part was written for the withdrawn rational-L rule (C-105 as revised), so there was no rational-number handling, rounding of L or fractional delay line to remove.

## Effort tally
- XS: 1
- S: 17
- M: 8
- Total: 26 tickets, about 37 days (37.25; XS 0.25, S 1, M 2.5 days) — scope L (C-126)

## Coverage

### Deliverables

| Deliverable | Tickets |
|---|---|
| `hadronyche-std/inject.rs` (`ClockInjection`, `LinkInjection`, `Injector`) | 017b-01, 017b-02, 017b-03, 017b-04 |
| Test-only hooks `TestHooks` (C-120, C-145) | 017b-06, 017b-07, 017b-08 |
| Injection and hook use always written into the regime stamp (sent by epic 015, C-144) | 017b-05 |
| Binary `hadronyche-l0` in `hadronyche-tools` (named scenario, duration, one record file with stamp through epic 015's single-writer service, non-zero exit) | 017b-09 |
| Scenarios `baseline` / `loss` | 017b-10 / 017b-11 |
| Scenario `diamond` / `late-vs-lost` / `gap-classes` / `realign` / `epoch` | 017b-12 / 017b-13 / 017b-14 / 017b-15 / 017b-16 |
| Scenario `blackhole` (forced small path MTU, PathMtuTooSmall) | 017b-17 |
| Scenario `collision` / `malformed` / `partition` | 017b-18 / 017b-19 / 017b-20 |
| Scenarios `flood`, `refusal` | 017b-21 |
| Scenario `thread-death` / `housekeeping` | 017b-22 / 017b-23 |
| Fuzz targets for the injection record and record-file reader | 017b-24 |
| CI gate | 017b-25 |
| Open sub-question: resync bound (owner: this part) | 017b-11 |
| Open sub-questions: reorder slot, record-file format (owner: this part) | 017b-03, 017b-09 |
| Open sub-questions: handles and sync-rate mechanism (epics 015, 016); echo degraded timing (epic 009) | 017b-01, 017b-09, 017b-14 |

### Acceptance criteria

| Epic AC | Tickets |
|---|---|
| 1. AC-8 trustworthy smoke (with C-152 DSCP) | 017b-02, 017b-03, 017b-10, 017b-11 |
| 2. AC-9 inert family exits non-zero | 017a-04, 017b-09 |
| 3. AC-10 diamond (processor = reference echo) | 017b-12 |
| 4. AC-15 late vs missing (C-108) | 017b-13 |
| 5. AC-16 gap classes and alarms (C-102, C-111, C-122, C-173) | 017b-14 |
| 6. AC-17 announced switch, ratchet, last step, Demoted (C-101, C-109, C-163) | 017b-15 |
| 7. AC-18 foreign epoch as gap, never dead (C-135) | 017b-06, 017b-16 |
| 8. AC-19 (L0 part) blackhole, live at 1 280, PathMtuTooSmall | 017b-03, 017b-17 |
| 9. AC-22 id collision and third-party quarantine (C-46) | 017b-06, 017b-18 |
| 10. AC-23 malformed / wrong-version / foreign-domain, domain 0 (C-173) | 017b-07, 017b-19 |
| 11. AC-36 partition heal, session bump (C-19) | 017b-04, 017b-20 |
| 12. AC-31 complete stamp, injection labelled, one writer | 017b-05, 017b-06, 017b-07, 017b-08, 017b-09 |
| 13. AC-017.13 L0 unprivileged in < 10 min CI | 017b-25 |
| 14. AC-017.14 flood (C-16, C-156) | 017b-07, 017b-21 |
| 15. AC-017.15 refusal (C-23, C-45, C-103, C-117, C-140, C-142, C-173, C-185) | 017b-21 |
| 16. AC-017.16 thread-death (C-42) | 017b-04, 017b-22 |
| 17. AC-017.17 housekeeping (C-139, C-145, C-161, C-185, C-188) | 017b-08, 017b-23 |

### Definition of done

| DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all; gate in 017b-25 (privileged tests: none, C-146) |
| Every parser touched is fuzzed | 017b-24 (also 017b-05, 017b-09) |
| Regime stamp fields touched are complete and documented | 017b-05 to 017b-09, 017b-26 |
| Design documents updated, constants table first | 017b-11, 017b-26 |
| No new open item without owner and falsifier | 017b-26 (and where raised: 017b-03, 017b-09) |
| Frozen POC not modified | 017b-02, 017b-26 |
| Evidence tags on new claims | all; audit in 017b-26 |
| Bit-exact with replicas on two machines (float and fixed) | not applicable (C-125): epic 017a (017a-10) |
| Ported pieces cite source file and archaeology finding | not applicable (C-125): this part ports no POC code; epic 017a |

## Remaining gaps

1. **`ModuleHandle`, `StreamHandle`, `ThreadRole`, `SocketKind` and the module API** are used but defined elsewhere. Affects 017b-01, 017b-04, 017b-06, 017b-07, 017b-08, 017b-22, 017b-23. Answer: epics 015 and 016.
2. **Reorder "held one slot"** is undefined. Affects 017b-03. Answer: this part (017b-03), recorded in the constants table through 017b-26.
3. **Resynchronisation bound after loss** (inherit 3 s?). Affects 017b-11 and 017a-07. Answer: a measurement in 017b-11, informed by epic 002.
4. **Record-file format** on top of epic 013's stamp encoding. Affects 017b-09, 017b-24. Answer: this part (017b-09).
5. **Sync-rate configuration mechanism** (the range is settled by C-112). Affects 017b-09, 017b-10, 017b-11. Answer: epic 015.
6. **When an arrival-driven module emits the degraded output for a position that never arrives.** Affects 017b-14 through the echo (017a-07). Answer: epic 009.

None of these is for the design owner. The former design-owner question, which module exercises the dense-controller domain, is settled by C-192 (the mixer's volume).

## Settled since the split

- Former gap 6, what the `housekeeping` scenario asserts → C-139 (restart policy), C-145 (panic hook), C-161 (control isolation), C-173 (control-degraded alarm), with C-144 placing the implementation and AC-20 (c) in epic 015; new tickets 017b-08 and 017b-23.
- Former gap 14, whether a foreign-epoch slice defers the dead threshold → C-135 (it restarts the dead clock, is not a miss and has its own counter); 017b-16 now holds the foreign epoch for 3 s and asserts never dead.
- "Complete stamp" (former gap 15) extended → C-159 (full counter list), C-160 (lateness percentiles), C-152 (observed DSCP on sync requests and hellos); 017b-09, 017b-10.
- Record file single writer and stamp sending → epic 015's duties (C-144); 017b-05, 017b-09 use them.
- Strict mode and measurements → C-147 (L0 is never strict; its numbers never enter the constants table); privileged tests → C-146 (separate nightly job; L0 has none); operator policy → C-165.
- Mixer ceiling in L0 → C-142 (explicit budget, since the mixer feeds no presentation group); 017b-13, 017b-15, 017b-21.
- Ratchet last step → C-163; demoted rounds counter → C-159; Demoted and Pending shown unsatisfied → C-154; 017b-15, 017b-21.
- Alarms → C-173 (refusal, interrupted, dead, demoted, control-degraded); 017b-14, 017b-15, 017b-17, 017b-21, 017b-22, 017b-23.
- Admission details → C-173: PathMtuTooSmall (017b-17), domain-0 hello ignored and ForeignDomain (017b-19), BadMemberList (017b-21); uncertified sinks have E unknown → C-140 (017b-21).
- Drain cycle → C-156 (one slice period); 017b-21.
- Header cap → C-137; 017b-17.
- Bad-input processing bound → C-164 is pending L1 and is deliberately not asserted in 017b-19.
- C-122 as now worded (stamps denote content position) → restated in 017b-14.
- C-104, C-105 (revised) and C-134 → no effect: no ticket handled rational L, rounding of L or fractional delays.
- C-143 → epic 017 split into 017a and 017b; 017-28 split into 017b-22 and 017b-23; fuzzing and design documents split between the parts.
- Earlier settlements carried over from epic 017: published bound C-62 with C-119; exact loss accounting against the injector's log; diamond processor = reference echo; C-53, C-55, C-101, C-109 for "stops at A" and "flapping"; C-120 hooks for foreign epoch, collision and malformed packets; C-16, C-23, C-45, C-103, C-117 for `flood` and `refusal`; C-42, C-111 for `thread-death`; C-18, C-47, C-129, C-131 for the stamp; C-95 for fuzzing; C-100, C-17, C-116, C-15, C-110 for the fallback path; C-112 for the sync-rate range; C-127 to C-133 applied throughout.
- Rules C-177 to C-212 (this epic was skipped when they were first applied):
  - C-177, C-201 (a failed or unverified pin refuses start in every mode when seqlock safety depends on it; non-strict L0 included) → epic rules, 017b-09, 017b-25.
  - C-181, C-205 (controllers inside the 100-byte header cap or in their own stream) → epic rules, 017b-17 context and AC 2.
  - C-182 (rolling 60 s window for lateness percentiles, maximum since live beside them) and C-204 (over-rate sync requests dropped and counted per follower in the holder's stamp) → "complete stamp" in the epic AC-31, 017b-09 context and AC 1; C-204 also asserted zero at both sync-rate ends in AC-8 and 017b-10.
  - C-141 (revised), C-184, C-199, C-200 (transport held, playhead moves only while playing; initial state stopped, zero, 120 bpm) → epic background, 017b-14 context. The former wording "held and its playhead advanced" is corrected.
  - C-185 (18 refusal reasons) → `refusal` case (d) is refused with NoSkewBudget (epic AC-017.15, 017b-21 AC 6); `housekeeping` phase (d) refuses a new connection with ControlDegraded (epic AC-017.17, 017b-23 AC 4); VersionMismatch is a discovery-time event (017b-19, 017b-21).
  - C-187 (lock state and bound published by the servo core's time view) → epic depends on 008a; 017b-10 depends on epic 008a.
  - C-188 (holder answers sync on the clock thread) → epic DSCP rule, 017b-10 context; `housekeeping` now targets the clock holder and asserts its followers stay locked (017b-23 AC 7).
  - C-192 → dense-controller question closed (the mixer's volume; epic, 017b-12); the L0 baseline is defined by 017b-10 (also listed in 017b-26).
  - C-193 (reject reasons from epic 004 through epic 007's vectors) → 017b-19.
  - C-206 (presentation details of tools) → the record-file format goes into the specifications because epics 019, 020 and 027 read it; the runner's other output formats and file names are 017b-09's decision (epic, 017b-09, 017b-26).
  - C-209 (controller member counts towards the 32 of C-08) → epic constants, 017b-21 context.
  - No effect on this part: C-178 (L0 opens no commissioning window), C-179, C-180, C-183, C-189, C-190, C-191, C-194, C-195, C-196, C-197, C-202, C-203, C-207, C-210, C-211, C-212 (ladder runs, commissioning, compiler, edges, resamplers, servo simulation); C-186, C-198, C-208 (L0's mixer has an explicit budget, never an inherited A, and no scenario changes A); C-08, C-134, C-153, C-169 as revised (beyond C-209, nothing in this part depends on them).
- C-35's owner change (L1, task 019) has no effect on this part. C-136, C-138, C-148 to C-150, C-153, C-155, C-157, C-162, C-166 to C-172, C-174 to C-176 concern other epics (servo and holder, OS probes, ladder runs, commissioning, packetiser, edges, DSL).
