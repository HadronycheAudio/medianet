# Epic 014 — Measurement path: tickets

## Summary
Epic 014 implements in `hadronyche-core` (module `measure`, `no_std`, no allocation) the reverse unicast measurement path, the lineage extension and the per-connection summaries each end publishes in its regime stamp. Because media packets carry no per-receiver state, measurement uses a separate reverse port whose v1 message set is exactly two kinds: the receiver report (one per bundle every 100 ms, C-40, DSCP 40, sent from the consumer's data thread, unicast only, C-47, at most 1 252 bytes of UDP payload over IPv4, C-17, C-100; per member: stream, session (C-19), received / late / missing / marked-gap / interrupted counts in slices and input health including Demoted (C-109); socket and interface drop counters; observed DSCP of received media; each member record also carries the other C-159 counts; once per report, the consumer's current A, its own or inherited, so a changed A propagates upstream one hop per report, C-198) and the one-way probe request / reply (DSCP 46, four timestamps, one-way figures only while the clock's time is usable, C-162, with an asymmetry bound of half the round trip). The report is the producer's only evidence its consumer is alive: the producer stops after 2 s without reports (C-41). Missing slices that arrive within the ring span are reclassified as late (C-108). The optional lineage extension carries hop count, accumulated residence time and accumulated declared delay as an integer sum of whole domain-rate samples (C-105), with the critical input's values propagating at a fan-in. The epic also delivers vectors, fuzz targets at the C-95 budget, the C-164 bad-input bound for its parsers and a loss-floor comparator on aggregate loss over all streams (C-118).

## Tickets
| Id | Title | Effort | Depends on |
|---|---|---|---|
| 014-01 | Measurement module scaffold and constants | XS | epic 004 |
| 014-02 | Counter widths and wrap handling | S | 014-01, epic 004 |
| 014-03 | Receiver report encoder | S | 014-02, epics 004, 009, 012, 013 |
| 014-04 | Receiver report parser | S | 014-03, epic 004 |
| 014-05 | Receiver report vectors and fuzzing | S | 014-04, epic 007 |
| 014-06 | Report timer | S | 014-01 |
| 014-07 | Producer watch | XS | 014-01, epics 005, 012 |
| 014-08 | Probe request and reply codec | S | 014-01, epic 004 |
| 014-09 | Probe one-way figures | S | 014-08, epic 008a |
| 014-10 | Probe vectors and fuzzing | S | 014-08, epic 007 |
| 014-11 | Probe initiation policy | S | 014-08, epic 012 |
| 014-12 | Reverse-port message classifier | S | 014-04, 014-08, epic 004 |
| 014-13 | Lineage through a hop | S | 014-01, epics 004, 010 |
| 014-14 | Lineage fan-in, vectors and fuzzing | S | 014-13, epic 007 |
| 014-15 | Loss-floor comparator | S | 014-04 |
| 014-16 | Report path bounded time and no allocation | S | 014-03, 014-06 |
| 014-17 | Producer liveness scenario (AC-21) | S | 014-03, 014-04, 014-06, 014-07, epic 012 |
| 014-18 | Report counters and rate scenario (AC-15, AC-2) | M | 014-03, 014-04, 014-06, epic 009 |
| 014-19 | Linux drop counter and DSCP tests (AC-37, AC-34) | S | 014-03, 014-04, epic 013 |
| 014-20 | No per-receiver state in media packets (AC-10) | XS | 014-04, epic 004 |
| 014-21 | Design documents and constants update | S | 014-01 to 014-20, 014-22, 014-23, 014-25, 014-26 |
| 014-22 | Per-connection stamp summary (C-47, C-159) | S | 014-02, 014-04, 014-07, epics 012, 013 |
| 014-23 | Late reclassification scenario (C-108) | S | 014-18, epic 009 |
| 014-25 | Parser bad-input bound (C-164) | S | 014-04, 014-08, 014-12, 014-13 |
| 014-26 | Current A in the receiver report (C-198) | S | 014-03, 014-04, 014-05, epics 004, 007 |

Added since the split: 014-22 (C-47 creates the per-connection summaries), 014-23 (C-108 creates the reclassification checks), 014-25 (C-164 creates the bad-input cost bound for this epic's four parsers), 014-26 (C-198 makes the report carry the consumer's current A upstream).

Deleted: 014-24 "Rational declared-delay accumulation". Its only purpose was the withdrawn rational-L rule; C-105 as revised makes every L a whole number of samples, so the accumulated declared delay is a plain integer sum, now done inside 014-13. The number 014-24 is not reused. Earlier, C-100 removed only the IPv6 path from 014-19 and the split rule from 014-03.

Extended in place by the latest rulings (effort unchanged): 014-02, 014-03, 014-04, 014-05, 014-22 carry the full C-159 counter set; 014-18 adds an other-epoch scenario (C-135); 014-13 drops rational handling (C-105), adds the header-cap omission (C-137) and points processing overrun to epic 015 (C-144); 014-09 withholds figures once the holdover claim lapses (C-162); 014-11 applies the commissioning window life cycle (C-153); 014-15 records the C-91 rule (C-167). Last sweep (C-177 to C-212 and the late revisions): 014-11 now treats tool-driven probe figures as valid commissioning results recorded with their window (C-178, replacing "unqualified"); 014-15 ignores in-window observations when configured for production, counts them for a commissioning soak (C-178), and checks the 10 × floor line of C-179; 014-22 notes that in-window summaries are unqualified (C-178); 014-09 adds the unknown-E lapse (C-169); 014-01 and 014-03 count the controller member among the 32 (C-08 as revised, C-209, C-205); 014-05, 014-10, 014-12, 014-14 leave the fuzzing tool to the team, keep vectors at the workspace root (C-192) and expect epic 004's reject reasons (C-193); 014-21 lists the new ids.

## Effort tally
- XS: 3 (0.75 day)
- S: 21 (21 days)
- M: 1 (2.5 days)
- Total: 25 tickets, about 24 days; scope L (C-126)

## Coverage
### Deliverables
| Deliverable | Tickets |
|---|---|
| `MemberCounters` (with per-member session and the C-159 counts), `DropCounters` | 014-02, 014-03 |
| `encode_report` with `EncodeError` | 014-03 |
| `ReceiverReport` (`parse`, `at_position`, `members`, `drops`, `observed_dscp`) | 014-04 |
| Consumer's current A in the report (`current_a` in `encode_report` and `ReceiverReport`) and the change hand-off to C-142 / C-186 (C-198) | 014-26 |
| `ReportTimer` (`new`, `due`) | 014-06 |
| `ProducerWatch` (`new`, `on_report`, `silent_for_2s`) | 014-07 |
| `ConnectionSummary` for the regime stamp (C-47, C-159) | 014-22 |
| `Probe`, `round_trip_ns`, `encode_probe_request`, `answer_probe`, `close_probe` | 014-08 |
| `ClockStatus`, `OneWay`, `Probe::one_way` (withheld when time is unusable, C-162) | 014-09 |
| `ReverseMessage`, `classify_reverse` | 014-12 |
| `Lineage::through` with integer declared delay (C-105), extension encode/parse within the header cap (C-137) | 014-13 |
| `Lineage::critical` (fan-in, `Option`) | 014-14 |
| Vector tests (report, probe, extension) | 014-05, 014-10, 014-14 |
| Fuzz targets at the C-95 budget (report, probe, classifier, extension) | 014-05, 014-10, 014-12, 014-14 |
| Bad-input cost bound for every parser (C-164) | 014-25 |
| Loss-floor comparator / cable-health flag | 014-15 |
| Closed reverse-port message set (DoD) | 014-12 |
| Bounded time, no allocation, no lock (DoD) | 014-16 |
| `no_std` build without allocator | 014-01 |
| Regime stamp fields: drop counters, DSCP, capability states (C-129), summaries | 014-19, 014-22, 014-21 |
| Design documents, constants, open sub-questions (DoD) | 014-02, 014-11, 014-21 |
| Open sub-question 1 owned here: counter semantics and widths | 014-02 |
| Open sub-question 4 owned here: probe initiator and default rate | 014-11 |

### Acceptance criteria
| Epic AC | Tickets |
|---|---|
| 1. Vectors both directions (AC-1); parsers fuzzed at C-95 budget | 014-05, 014-10, 014-12, 014-14 |
| 2. One report per bundle per 100 ms ±1 slice, data thread; 16 members → 10/s | 014-06, 014-18 |
| 3. 32-member report fits 1 252 bytes of UDP payload | 014-02, 014-03 |
| 4. AC-21 producer side | 014-07, 014-17 |
| 5. AC-15 refined by C-108; other-epoch slices counted apart, not a miss (C-135) | 014-18, 014-23 |
| 6. AC-37 socket overflow in drop counters on Linux | 014-19 |
| 7. AC-34 observed DSCP 46 reported as 46 | 014-19 |
| 8. Lineage through three hops, integer declared delay (C-105); fan-in critical input; bad index gives no value | 014-13, 014-14 |
| 9. Probe round trip exact; one-way withheld while time is unusable (C-162); asymmetry bound half the round trip | 014-08, 014-09 |
| 10. No receiver-report field in media packets | 014-20 |
| 11. `no_std` without allocator | 014-01 |
| 12. Producer and consumer summaries agree over every C-159 counter; stale sessions ignored (C-47, C-159, C-19) | 014-22 |
| 13. Closed reverse-port message set | 014-12 |
| 14. Bad-input processing linear in length, at most 10 µs per packet (C-164) | 014-25 |
| 15. Report carries the consumer's current A; a change travels one hop per report (C-198) | 014-26 |

## Remaining gaps
1. **Wire layouts.** Message type ids and byte layouts of the report (including widths of the C-159 counts and the unit and width of the current A, C-198), probe and lineage extension (within the C-137 header cap). Affects 014-02, 014-03, 014-04, 014-08, 014-12, 014-13, 014-26. Answer: the wire specification, epic 004.
2. **Counter semantics.** Cumulative or per-interval, and widths (rates, the C-108 decrease and the C-159 set are now known). Affects 014-02, 014-03, 014-15, 014-22. Answer: epic 004 with this epic (014-02).
3. **Authentication reservation on the reverse port.** Whether reverse-port messages carry the C-12 reservation (with the v1 values of C-115); it changes the report size budget. Affects 014-03, 014-08, 014-12. Answer: epic 004.
4. **Counters across a session change.** Whether per-input counters restart when a stream's session is bumped (C-19). Affects 014-02, 014-15, 014-22. Answer: epic 009.
5. **Definitions of the new C-159 counts** (what exactly counts as too-early, duplicate, incomplete reassembly and a demoted round). Affects 014-03, 014-18, 014-22. Answer: epic 009, which owns the per-input counters.
6. **`ProducerWatch` start before the first report.** Affects 014-07, 014-17. Answer: the control state machines, epic 005 (open sub-question 5).
7. **What a "connection" covers for C-47 summaries** (one stream or one bundle). Affects 014-22. Answer: epic 012.
8. **Admission accounting granularity** for the probe falsifier. Affects 014-11. Answer: epic 012.
9. **Loss comparator minimum sample size.** Affects 014-15. Answer: a measurement, L1 (task 019) (open sub-question 7). C-91's value (10 × the measured clean-wire loss, C-167) and C-90's check stay with L1 and task 001.
10. **Cable-health event.** Which event is raised from the comparator flag (not among the C-144 duties or the C-173 connection alarms). Affects 014-15. Answer: epic 015.
11. **Numeric time budget for report generation.** Affects 014-16. Answer: a measurement (014-16 records it), then epic 015, which owns the data thread's cycle budget.
12. **Lineage range, saturation and negative residence**, now including the integer declared delay and the fit within the C-137 cap. Affects 014-13, 014-21. Answer: epic 004 with this epic (open sub-question 10).
13. **C-164 figure.** 10 µs per bad packet is a proposal. Affects 014-25. Answer: a measurement, L1 (task 019).

## Settled since the split
- Former gap 1 (module name): settled by the epic, module `measure`. The layouts stay with epic 004 (remaining gap 1).
- Former gap 2 (counter semantics, packets per slice): counters count slices, 2 000/s per member at defaults, one packet per default slice (C-02, C-05, C-22); socket load up to 64 000 packets/s (C-92); missing can decrease (C-108). Cumulative versus per-interval remains (remaining gap 2).
- Former gap 3 (MTU accounting): IPv4 only, 1 280-byte fallback gives 1 252 bytes of UDP payload, and the report must fit without a split rule (C-100, C-17, C-06).
- Former gap 4 (encoder error path): settled by the epic, `Result<usize, EncodeError>` with nothing written on error.
- Former gap 5 (timestamp resolution): settled by the epic, criteria 8 and 9 are exact with injected integer timestamps.
- Former gap 6 (lineage units): settled by C-01 and C-105 as revised: every L is a whole number of domain-rate samples, so declared delay is an integer sum convertible to time at the single rate. The earlier rational-L rule is withdrawn; resampling exists only inside edge modules and its delay is part of D (C-134). Saturation and negative residence remain (remaining gap 12).
- Former remaining gap 13 (representation of a sum of rationals, design owner): withdrawn with the rational-L rule (C-105); ticket 014-24 deleted.
- Former remaining gap 5 (separate count for Demoted): settled by C-159, the per-connection demoted-rounds counter, carried per member in the report. The exact counting rule is epic 009's (remaining gap 5).
- Former remaining gap 15 ("processing overrun" event owner): settled by C-144, the hosted backend, epic 015.
- Former gap 7 (`Lineage::critical` bad index): settled by the epic, returns `Option`, `None` for a bad index.
- Former gap 9 (one-way figures API): asymmetry bound settled by the epic as half the round trip; `ClockStatus` defined; receive-timestamp source reported by the backend (C-131); figures continue in holdover with the growing bound and stop once the claim lapses (C-162). The clock publisher is now settled (C-187, below).
- Former gap 10 (loss comparator): statistic is aggregate loss over all streams (C-118) against C-90 per default packet; uses missing after reclassification (C-108); threshold rule proposed as 10 × the measured clean-wire loss (C-167). Minimum sample size and the event remain (remaining gaps 9, 10).
- Former gap 12 (admission unit): admission is a fraction of link rate, ceiling 70 % (C-23); tool-driven probes only inside a commissioning window (C-107) with the C-153 life cycle, measurements in a window unqualified. Granularity remains (remaining gap 8).
- Former remaining gap 14 (clock dependency, design owner, open sub-question 9): settled by C-187, the servo core, epic 008a, publishes the lock state and clock bound through the published time view; 014-09 and the epic now depend on 008a.
- C-182 (rolling 60 s window for lateness percentiles, reclaimable latency and the cannot-meet-A alarm, with the maximum since the connection went live beside them): checked; those stamp fields are not computed in this epic (014-22 notes it), so no ticket changes.
- C-198 (the receiver report carries the consumer's current A, own or inherited; a change propagates one hop per report and each inheriting fan-in re-aligns, C-142, C-186): no earlier ticket or gap held a provisional reading; it creates work, ticket 014-26 (effort S), and epic criterion 15. Deciding the inherited A and the re-alignment stay outside this epic; the field's unit and width join remaining gap 1.
- C-178 (with C-153 as revised): the epic and 014-11 had read C-153 as "every measurement in a window is unqualified", which C-178 contradicts. Now: production figures in stamps (including the 014-22 summaries) are unqualified while a window is open; the commissioning tool's probe figures and soak are valid commissioning results recorded with their window. 014-15 gains an in-window flag on `observe` (effort unchanged).
- C-179 (loss above 10 × the C-90 floor reopens redundancy; 1 × to 10 × is a cabling or host investigation): read by the L1 soak (task 019); 014-15 already takes its threshold as a parameter and now checks the 10 × line without overflow.
- C-169 (holdover lapse with unknown E): applied by epic 008a's time view; 014-09 and the epic state it.
- C-08 as revised and C-209, C-205 (controller member counts among the 32, as its own member stream): the 32-member worst case of 014-03 is unchanged; constants rows updated.
- C-192 and C-193: fuzzing tool left to the team (no longer `cargo fuzz` by name), vectors crate at the workspace root, reject reasons from epic 004 (014-05, 014-10, 014-12, 014-14).
- Checked, no subject in this epic: C-177, C-180, C-181, C-183 to C-185, C-188 to C-191, C-194 to C-197, C-199 to C-204, C-206, C-208, C-210 to C-212, and C-134, C-141, C-184 as revised (014-13 and the epic already keep the edge's resampler delay in D, never in L, consistent with C-207; no tool presentation detail is left to the design owner, C-206).
- Former gap 14 (AC-6/AC-7 need sockets): settled by the epic and C-133, the tests live in the `hadronyche-tools` crate.
- Former gap 15 (system AC wording): settled by the epic, which now states AC-1, AC-15, AC-21, AC-34, AC-37 and L-1 inline.
- Epic open sub-question 2 (report interval for the A proposal): settled by C-160 and C-148, the tool uses the stamp's lateness maximum and percentiles, not report counters.
- Other rulings applied to tickets: per-member session (C-19, 014-03, 014-04, 014-22); reports unicast only and summaries in the stamp (C-47, 014-22) with the full counter set (C-159); other-epoch slices (C-135, 014-18); header cap for the extension (C-137, 014-13); Demoted health (C-109); fuzz budget (C-95); capability states for drop counters and DSCP (C-129, 014-19); output locations (C-130, C-171); `[standard]` tag for C-90 once cited (C-132); strict mode for figures entering the table (C-147, 014-16, 014-21); sink-only device liveness by the producer timeout (C-168, 014-07); housekeeping panic and restart policy end to end left to epic 015 (C-139, C-145, 014-17); peer verification of sync and control DSCP outside this epic (C-152, 014-19).
- Unchanged and still open: former gap 8 (remaining gap 6), former gap 11 (remaining gap 11).
