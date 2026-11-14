# Epic 002 — POC archaeology: tickets

## Summary
Epic 002 is pre-implementation check **0b** (phase P0, scope M: 14 tickets, about 13 days). It diagnoses the known instrument anomalies of the frozen proof of concept **hadropoc** (inert music and transport check families; a "lost" counter of about 21.7 million in a clean run; a corrupt loopback metrics file; a sync rate of 37/s at 50 Hz and 77/s at 1000 Hz, P-02) to the level of "instrument bug, mechanism named", reads the POC files never fully read (notably `audio.rs` `analyse` / `analyse_voices`, to know whether voice windows are compared bit for bit), checks the integrity of the `experiments/` metrics behind the I/O-pressure clock-eligibility rule, and confirms by run two claims only read from code (unbounded drain starving the ticker; a phase step in published time at each servo update). It also re-measures the POC sync rate from clean metrics at 10, 50, 200 and 1000 Hz, covering the AC-7 range of 10 to 200 per second (C-112). All findings go into one document, `doc/design/reports/archaeology/REPORT-0b.md` (C-130), with a verdict line per item (a) to (e). Diagnostic logging lives in a scratch copy; the frozen POC is not modified. The epic unblocks 003 (bug confirmations, (a) answer), 007 and 017 (exact-tier citation and vector extraction) and 011 (I/O-pressure rule).

## Tickets
| Id | Title | Effort | Depends on |
|---|---|---|---|
| 002-01 | Scratch copy and report skeleton | XS | none |
| 002-02 | Reading notes: audio.rs analyse and analyse_voices | S | 002-01 |
| 002-03 | Reading notes: transport.rs and control.rs | S | 002-01 |
| 002-04 | Reading notes: sender.rs, stats.rs and tui.rs | S | 002-01 |
| 002-05 | Inert families: baseline run and transport trace | S | 002-01, 002-03 |
| 002-06 | Inert families: music trace, minimal condition and verdict | S | 002-02, 002-05 |
| 002-07 | Loss counter: per-hop logging run | S | 002-01 |
| 002-08 | Loss counter: first wrong hop and verdict | XS | 002-04, 002-07 |
| 002-09 | Experiment metrics integrity | S | 002-01 |
| 002-10 | Flood test of the unbounded drain | M | 002-01, 002-04 |
| 002-11 | Servo phase step measurement | S | 002-01 |
| 002-12 | Clean sync rate baseline | S | 002-01 |
| 002-13 | Close-out: report audit and design updates | S | 002-02, 002-03, 002-04, 002-06, 002-08, 002-09, 002-10, 002-11, 002-12, 002-14 |
| 002-14 | Sync rate baseline at the ends of the AC-7 range (new, from C-112) | XS | 002-12 |

No ticket was deleted and none was added in the C-127 to C-176 round: the epic holds no rational-L, rounding or fractional-delay work (C-104, C-105 and C-134 change nothing here), C-144 assigns nothing to it, and the new rules that touch it are absorbed by existing tickets. None was added or deleted in the C-177 to C-212 round either: C-147 and C-206 are absorbed by existing tickets.

## Effort tally
- XS: 3 (0.75 day)
- S: 10 (10 days)
- M: 1 (2.5 days)
- Total: 14 tickets, about 13.25 days → scope **M** (C-126)

## Coverage
### Deliverables
| Epic deliverable | Tickets |
|---|---|
| `doc/design/reports/archaeology/REPORT-0b.md` (C-130), one section per item, verdict line per section | 002-01 (skeleton), 002-13 (audit) |
| (a) step 1 — 25 s loopback run with `--csv`, final summary | 002-05 |
| (a) step 2 — transport trace (reference, `ctx.is_reference()`, listen-before-claim 1.5 s C-32, `events` routing) | 002-03, 002-05 |
| (a) step 3 — music trace (settling exclusions, `voices` flag, `place_voices`) | 002-02, 002-06 |
| (a) step 4 — minimal condition for both families, confirmed by a 25 s run | 002-06 |
| (b) step 1 — log `rx_lost` at each hop, 25 s run from the scratch copy | 002-07 |
| (b) steps 2-3 — first wrong hop, field holding the printed value | 002-04, 002-08 |
| (c) — per-file line counts, parse against header, monotonic `unix_ms`, single-writer rule, I/O-pressure verdict | 002-09 |
| (d) — reading notes for `audio.rs` (`analyse`, `analyse_voices`) | 002-02 |
| (d) — reading notes for `transport.rs`, `control.rs` | 002-03 |
| (d) — reading notes for `sender.rs`, `stats.rs`, `tui.rs` | 002-04 |
| (d) — verdict line (form set by the epic) | 002-02 (statement), 002-13 (line) |
| (e) 1 — nominal rate, flood one mixer input at 10×, TX pacing and mixer ticker lateness | 002-10 |
| (e) 2 — `theta0` before/after servo update, at least 200 locked updates | 002-11 |
| (e) 3 — sync rate at 50 and 1000 Hz from clean metrics, 3 × 30 s, unprivileged POC runs feeding P-02 (C-147) | 002-12 |
| (e) 3 — sync rate at 10 and 200 Hz (AC-7 range, C-112), 3 × 30 s | 002-14 |
| Background rules: scratch copy, report in `doc/design/reports/archaeology/` (C-130), checker in `hadronyche-tools` and flood probe in `probes/` (C-133), constants table `02-constants.md` first | 002-01, 002-09, 002-10, 002-13 |
| Background rule: evidence appendix `doc/design/06-evidence-appendix.md` (C-130), architecture specification `doc/design/01-architecture.md` (C-171) | 002-13 |
| Open sub-question: voice comparison bit-level | 002-02 |
| Open sub-question: I/O-pressure finding survives | 002-09 |
| Open sub-question: 21.7 M decoding or display error | 002-08 |
| Open sub-question: 5 ms behaves as about 13 ms (informational) | 002-12 |
| Hand-offs to 003, 007, 017, 011 | 002-06, 002-08, 002-13 |

### Acceptance criteria
| Epic criterion | Tickets |
|---|---|
| 002-AC-1 verdict line with mechanism and pointer or run output, (a)-(e) | 002-06 (a), 002-08 (b), 002-09 (c), 002-02/002-13 (d), 002-10/002-11/002-12 (e), 002-13 (audit) |
| 002-AC-2 (a) command line with both families non-zero, or why none | 002-06 |
| 002-AC-3 (b) source field named, log of first wrong hop | 002-07, 002-08 |
| 002-AC-4 (c) table of every metrics file | 002-09 |
| 002-AC-5 (d) voice comparison bit-level yes/no | 002-02 |
| 002-AC-6 (e) flood result, step distribution, sync rate at 10, 50, 200, 1000 Hz | 002-10, 002-11, 002-12, 002-14 |
| 002-AC-7 contributes to system AC-9 and AC-7 | 002-05, 002-06 (AC-9); 002-12, 002-14 (AC-7); 002-13 (hand-offs) |

### Definition of done
| DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged (per-change CI, C-146) | 002-09 (the metrics checker in `hadronyche-tools`, the only code of the epic), 002-13 (check) |
| Every parser touched is fuzzed | 002-09 (60 s per change, 1 h nightly, corpus kept, C-95), 002-13 (check) |
| Regime stamp fields touched are complete and documented | not applicable (C-125): no stamp field in this epic; 002-13 records the reason |
| Design documents updated where behaviour differs, constants table first | 002-13 (`02-constants.md` P-rows first) |
| No new open item without an owner and a falsifier | all |
| The frozen POC is not modified (task 003 is the only exception) | all |
| Every new factual claim carries an evidence tag | all |
| Diagnostic logging for (b) and (e) in a scratch copy | 002-01, 002-05, 002-06, 002-07, 002-10, 002-11, 002-12, 002-13, 002-14 |
| Evidence appendix updated with each verdict | 002-13 (`doc/design/06-evidence-appendix.md`, C-130) |

## Remaining gaps
1. **Nominal rate of a mixer input and reachability of 10×.** No constant gives the POC's per-input datagram rate, and at the P-01 feed (about 9.6 KB per datagram, about 150 Mbit/s) 10× may not be reachable on loopback. Affects 002-10. Answer: **a measurement** (002-10 step 1; the ticket reports the highest rate reached if 10× is not).

## Settled since the split
1. Report location — narrowed by an epic change: never inside the frozen tree (follows from the DoD). The root is now settled by C-130 (item 14).
2. Scratch copy location — epic change: any path outside `/home/dorian/workspace/hadropoc`, recorded with fingerprints in the report.
3. Design documents and constants table — epic change: the constants table is `02-constants.md`; P-02, P-03 and the loss-handling facts are updated there first, other documents found by searching the design set.
4. Regime stamp fields — C-125: line marked not applicable in this epic, with the reason.
5. CI and fuzzing for an investigation epic — C-125 and C-95: they apply to the 002-09 metrics checker only; fuzz 60 s per change, 1 hour nightly, corpus kept.
6. Metrics header and single-writer rule — epic change: the rule is stated in deliverable (c); the header is read from `module.rs:1292`.
7. Flood test parameters — epic change: procedure, durations and the mixer metric (ticker lateness, deadline against actual tick) are in deliverable (e) 1; the runtime's answer to the drain is C-16. The rate value itself is a measurement (Remaining gap 1).
8. Clean metrics for (e) 3 — epic change: separate processes or one path per module in the scratch copy; 002-12 names the column.
9. Run durations and repetitions — epic change: 25 s for (a) and (b), at least 25 s per run for (e) 1, at least 200 locked updates (60 s) for (e) 2, 3 × 30 s per rate for (e) 3.
10. Verdict for (d) — epic change: its form is defined in deliverable (d).
11. Dependency header — epic change: "Blocks" now lists 003.
12. Sizing — C-126: scope recomputed from the tally as M (14 tickets, about 13 days).
13. Ids and tags — epic change: C-ids and P-ids cited (C-01, C-16, C-30 to C-33, C-36, C-47, C-60, C-61, C-66, C-95, C-112, C-125, C-126, P-01 to P-04); `[kernel-doc]` listed; `CONFIG_HZ=250` tagged `[measured]` as P-03; epic criteria numbered 002-AC-1 to 002-AC-7 so they no longer collide with system AC-7 and AC-9.
14. Report root — C-130: reports go to `doc/design/reports/`; the report is `doc/design/reports/archaeology/REPORT-0b.md` with its companion files beside it. Closes former remaining gap 1 (002-01, 002-13).
15. Checker location and CI home — C-133: the 002-09 checker is a tool binary in the `hadronyche-tools` crate, tested and fuzzed by the design set's repository CI (002-09).
16. Flood tooling location — C-133: a standalone flood sender is a throwaway probe in `probes/` (002-10).
17. Evidence appendix — C-130: `doc/design/06-evidence-appendix.md`; entries follow the format that file already uses. Closes former remaining gap 2 (002-13).
18. Architecture specification — C-171: `doc/design/01-architecture.md` is among the documents 002-13 searches for changed statements.
19. "Cycle" of the drain bound — C-156: one slice period; cited where the epic and 002-10 contrast the POC drain with C-16.
20. Runtime loss counters — C-159 completes C-47; the full per-connection counter list is cited in the epic and 002-08.
21. "Unprivileged" in the definition of done — C-146: applies to per-change CI; the checker needs no privileged nightly job (002-09).
22. Evidence tags — C-132: `[standard]` added to the tag list in the epic.
23. C-104, C-105 (revised) and C-134 — checked: the epic has no latency, resampler or rational-L content; no change.
24. C-147 — first raised as a remaining gap (applicability to POC reference values).
25. C-147 (as now worded) — POC runs do not run in strict mode and their figures go only into POC reference rows. Closes former remaining gap 1: the open sub-question is removed from the epic and 002-01, the interim recording and conditional repeat are removed from 002-12 and 002-14, and 002-13 writes P-02 as a POC figure.
26. Presentation of the 002-09 checker — C-206: its output line format and file names are decided by this epic and recorded in the checker's documentation (epic, 002-09).
27. C-177 to C-212 and the late revisions of C-08, C-134, C-141, C-153, C-169, C-181, C-184 — checked: no other rule has a subject in this POC investigation epic (no runtime code, stamp, controller, transport, resampler, alignment, commissioning or L2 content); no change.

New work from a ruling: C-112 (AC-7 is checked over 10 to 200 per second) adds the 10 and 200 Hz measurements, ticket 002-14.
