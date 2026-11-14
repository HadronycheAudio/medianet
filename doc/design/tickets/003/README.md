# Epic 003 — POC smoke fixes: tickets

## Summary
Epic 003 (phase P0, pre-implementation check **0c**) puts the frozen proof of concept **hadropoc** (`/home/dorian/workspace/hadropoc`, Rust) under git, tags the untouched tree (plus `.gitignore`) `poc-untouched`, applies three minimal fixes so that its loopback run is a citable baseline — one process-owned writer per metrics file (bug 1: four module writers truncating and overwriting one file, ~6 of ~80 lines parseable `[measured]`), clock-sync waits bounded by the next request deadline with a 100 ms ceiling (bug 2: a fixed 5 ms receive timeout gave 37/s at `--sync-hz 50` and 77/s at `--sync-hz 1000` on a `CONFIG_HZ=250` kernel, P-02 and P-03 `[measured]`), and a non-zero headless exit naming any in-scope check family with zero windows (bug 3: music and transport inert yet green `[measured]`) — adds `tools/smoke.sh` and `BASELINE.md`, then tags `poc-baseline`, after which the POC is never modified again, and records the post-fix rates in the constants table and evidence appendix. The sync rate is checked at 50/s (C-36, inside the accepted range 10 to 200/s of C-112); 1000/s is recorded for comparison only. It depends on epic 002 for the cause of the inert families and feeds epics 007 and 017. Epic scope: **M** (10 tickets, about 5.5 days, rule C-126).

## Tickets
| Id | Title | Effort | Depends on |
|---|---|---|---|
| 003-01 | Init repository and tag poc-untouched | XS | none |
| 003-02 | Measure pre-fix sync rate | XS | 003-01 |
| 003-03 | Process-owned metrics writer | S | 003-01 |
| 003-04 | Deadline-paced clock sync wait | S | 003-01, 003-02 |
| 003-05 | Measure post-fix sync rate and lock | XS | 003-02, 003-04 |
| 003-06 | Fail headless run on zero-sample family | S | 003-01, epic 002 |
| 003-07 | Smoke script | S | 003-03, 003-04, 003-06, epic 002 |
| 003-08 | Write BASELINE note | XS | 003-02, 003-05, 003-06, 003-07 |
| 003-09 | Verify and tag poc-baseline | XS | 003-03, 003-04, 003-05, 003-06, 003-07, 003-08 |
| 003-10 | Update design set with baseline | XS | 003-05, 003-09 |

No ticket was added or deleted by the rulings, including the last sweep (C-177 to C-212, with late revisions of C-08, C-134, C-141, C-153, C-169, C-181, C-184): in it, C-147 removed the design-owner question from 003-02, 003-05 and 003-10, and C-206 added a presentation note to 003-06 and 003-08, all at unchanged effort. In the round before (C-100 to C-176): none of the epic's work depended on the withdrawn rational-L rule (C-104, C-105 revised; C-134), C-144 assigns its duties to epic 015 only, and no rule up to C-176 creates work no ticket covers. In that round 003-10 lost the file-location question (C-130), 003-02, 003-05, 003-08 and 003-10 gained the measurement-conditions note (C-147), 003-03 and 003-07 gained out-of-scope notes (C-144, C-133), all at unchanged effort. Earlier, 003-04 gained a dependency on 003-02 (it uses the measurement method 003-02 records). 003-07 no longer fuzzes its metrics check (C-125, C-95); negative tests replace it at no change in effort.

## Effort tally
- XS: 6 (1.5 days)
- S: 4 (4 days)
- M: 0
- Total: 10 tickets, about 5.5 days → scope **M** (C-126)

## Coverage
| Epic item | Tickets |
|---|---|
| Deliverable 1 — repository, `.gitignore`, first commit, tag `poc-untouched` | 003-01 |
| Deliverable 2 — metrics writer fix, layout unchanged | 003-03 |
| Deliverable 3 — sync pacing fix | 003-04 (measured in 003-02, 003-05) |
| Deliverable 4 — zero-sample failure over in-scope families | 003-06 |
| Deliverable 5 — `tools/smoke.sh` (100 ± 2 rows, 25 ± 2 per module; patterns from `poc-untouched`) | 003-07 |
| Deliverable 6 — second commit, tag `poc-baseline` | 003-09 |
| Deliverable 7 — `BASELINE.md` | 003-08 |
| Deliverable 8 — design-set update (constants table next to P-02, evidence appendix, rule exception) | 003-10 |
| 003-AC1 — both tags annotated; diff stat limited to listed parts, `.gitignore` in first commit | 003-01, 003-09 |
| 003-AC2 — 100 % of metrics lines parse | 003-03, 003-07 |
| 003-AC3 — within 2 % at 50/s (C-36, C-112); 1000/s recorded only | 003-02, 003-04, 003-05, 003-08 |
| 003-AC4 — inert in-scope family exits non-zero and is named (system AC-9) | 003-06 |
| 003-AC5 — smoke passes 3 times in a row, unprivileged | 003-09 |
| 003-AC6 — two `transport.rs` tests still pass | 003-01, 003-03, 003-04, 003-06, 003-09 |
| 003-AC7 — contributes to system AC-8 and AC-31 | 003-07 (AC-8), 003-03 (AC-31), 003-10 |
| Open sub-question: music/transport producible without code changes (owner 002) | 003-06, 003-07, 003-08, 003-10 |
| Open sub-question: deadline pacing vs servo lock (owner 003, measured) | 003-05 |
| Open sub-question: method for completed exchanges per second (owner 003, measured) | 003-02 |
| Settled (C-147): POC runs are not in strict mode; post-fix figures go beside P-02 | 003-02, 003-05, 003-08, 003-10 |
| Settled (C-206): zero-sample line wording and other presentation details decided here, recorded in `BASELINE.md` | 003-06, 003-08 |
| Dependency: align any statement of "no code goes into hadropoc" with C-130 | 003-10 |
| DoD — design documents updated, constants table first, evidence appendix records post-fix rates | 003-10 |
| DoD — both tags exist, `BASELINE.md` before/after figures | 003-08, 003-09 |
| DoD — not applicable (C-125): CI part of "tests green in CI", parser fuzzing, regime stamp fields | noted in 003-03, 003-04, 003-06, 003-07, 003-09, 003-10 |

## Remaining gaps
1. **Cause of the inert music and transport families, and activating flags if any.** Decides the in-scope family set of the zero-sample check, the smoke flags, and the known-inert list. The epic now defines both outcomes, so there is no contradiction left, only a missing input. Affects 003-06, 003-07, 003-08, 003-10. Answered by: epic 002.
2. **How completed exchanges per second are observed on the untouched POC.** Still an open sub-question of the epic. Affects 003-02, 003-04, 003-05. Answered by: a measurement (003-02, by reading the POC at `poc-untouched`).
3. **Exact text of the final summary, of the locked-state indicator and of the metrics header, and whether a metrics row identifies its module.** The epic now says where they come from (the POC at `poc-untouched`) and that the layout is unchanged, but not what they are. Affects 003-03, 003-07. Answered by: reading the POC source (003-07).
4. **Whether deadline pacing still reaches lock (P-04) within 25 s.** Open sub-question with a falsifier. Affects 003-05, 003-08, 003-10. Answered by: a measurement (003-05).

## Settled since the split
- Former gap 3, "±2 rows" ambiguous → epic deliverable 5: 100 ± 2 rows in total, and 25 ± 2 per module where a row identifies its module.
- Former gap 5, `.gitignore` placement → epic change: `.gitignore` is in the first commit and 003-AC1 no longer lists it in the diff between the tags.
- Former gap 6, CI not described → C-125 and epic change: the POC has no CI and 003-AC1 forbids adding one; the "in CI" part is marked not applicable and met by unprivileged `cargo test` and smoke runs.
- Former gap 7, fuzzing target → C-125 (with C-95): no parser is touched; the smoke field-count check is a test assertion, covered by negative tests.
- Former gap 8, "regime stamp fields" → C-18 and C-125: the regime stamp is a runtime control message; no stamp field is touched; marked not applicable.
- Former gap 10, measuring machine → P-02 and P-03: the machine of the pre-fix figures, `CONFIG_HZ=250`; 003-02 records its identity.
- Former gap 11, exit code value → epic background `[decision]`: any non-zero status plus the family's name.
- Former gap 12, no constants and AC-7 range unstated → C-36 and C-112 (accepted range 10 to 200/s; 50/s checked, 1000/s outside the range), P-02 to P-04 cited; epic criteria now carry ids 003-AC1 to 003-AC7, so they no longer collide with system criteria AC-7 to AC-9.
- Former gap 9, design-set locations → partly settled: the constants table is `02-constants.md`; the rest was remaining gap 5.
- Former remaining gap 5, evidence appendix and "no code goes into hadropoc" → C-130: the evidence appendix is `doc/design/06-evidence-appendix.md`, and C-130 itself states that nothing is written into the frozen POC tree except by task 003. 003-10 now only aligns other statements of the rule (for example in `01-architecture.md`, C-171).
- Runtime single-writer file service → C-144: owned by epic 015; this epic fixes the POC only (003-03 out of scope).
- Location of `tools/smoke.sh` and `BASELINE.md` → C-130, C-133: they stay in the POC tree under the task 003 exception, not in `doc/design/reports/` or `hadronyche-tools`.
- Privileged tests → C-146: none in this epic; the "unprivileged" line is met by unprivileged runs.
- Withdrawn rational-L rule (C-104, C-105 revised; C-134) → no effect: no ticket of this epic touched latency representation or resampling.
- Former open sub-question "Is 1000 exchanges/s reachable unprivileged?" → C-112: 1000/s is outside the accepted range; recorded only, nothing depends on it.
- Former remaining gap 5 and open sub-question "Does C-147 cover the POC reference rows?" → C-147: POC runs do not run in strict mode and their figures go only in the POC reference rows (P-nn). The post-fix figures are taken unprivileged and entered beside P-02 unconditionally; 003-10 lost its conditional work and criterion (003-02, 003-05 context updated).
- Wording of the zero-sample line, layout of `BASELINE.md`, comments of `tools/smoke.sh` → C-206: presentation details decided by this epic and recorded in `BASELINE.md` (003-06, 003-08).
- Rules C-177 to C-212 otherwise → no subject in this epic: they concern the runtime (seqlock pinning, commissioning, controllers and bundles, resamplers, alignment, the servo and holder, refusal reasons, ladder tests), which the frozen POC does not move toward; C-204 does not affect the 1000/s figure, which is a POC comparison figure outside C-112 and asserted nowhere.
