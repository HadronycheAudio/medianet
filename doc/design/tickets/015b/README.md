# Epic 015b — Hosted runtime verification and strict mode: tickets

## Summary
Epic 015b is part b of epic 015 (split under C-143) and depends on part a (015a), which builds the Linux hosted runtime. This part proves that runtime against the thread rules L-1, L-4, L-5, L-7 and L-8 (flood isolation, housekeeping isolation, fail-stop, static lock check), checks the achieved sync rate, the absence of allocation on the data thread, wall-clock step immunity, don't-fragment and one writer per file; it adds strict mode and the capability states (C-127 to C-129, C-131, C-161, C-165) with the pin-dependent seqlock refusal (C-177, C-201) and the test-only hooks (C-120, C-145); it takes in strict mode every measurement whose numbers enter the constants table (C-147); and it updates the design documents for both parts.

## Tickets
| Id | Title | Effort | Depends on |
|---|---|---|---|
| 015b-01 | Achieved sync rate verification (AC-7) | S | 015a-10, 015b-12, 015b-13, epic 016 |
| 015b-02 | Static check: no lock shared across thread classes (AC-20(e)) | S | 015a-17 |
| 015b-03 | Flood isolation test (AC-11) | S | 015a-14, 015a-17, 015b-12 |
| 015b-04 | Housekeeping isolation tests (AC-20 a–d) | M | 015a-17, 015b-12, 015b-14 |
| 015b-05 | Fail-stop test (AC-21) | S | 015a-17, 015b-14 |
| 015b-06 | Wall-clock step test | S | 015a-10 |
| 015b-07 | Don't-fragment capture test | XS | 015a-17, 015a-18 |
| 015b-08 | One writer per file test (AC-31) | XS | 015a-17 |
| 015b-09 | No allocation on the data thread after go-live | S | 015a-17 |
| 015b-10 | Per-packet receive cost measurement | S | 015a-11, 015b-13, epic 016 |
| 015b-11 | Design documents update | XS | 015a-14, 015a-15, 015a-16, 015a-18, 015a-19, 015a-20, 015a-21, 015b-01, 015b-02, 015b-10, 015b-12, 015b-13, 015b-14, 015b-15 |
| 015b-12 | Test-only injection hooks (C-120) | S | 015a-05, 015a-17, 015a-18 |
| 015b-13 | Strict mode and capability states (C-127 to C-129, C-131, C-177, C-201) | S | 015a-06, 015a-17, 015a-18 |
| 015b-14 | Panic-injection hook (C-145) — new | XS | 015a-16, 015b-12 |
| 015b-15 | Strict-mode host clock measurements (C-147) — new | S | 015a-03, 015b-13, epic 016 |

Renumbered from epic 015: 015-11 → 015b-01, 015-19 → 015b-02, 015-20 → 015b-03, 015-21 → 015b-04, 015-22 → 015b-05, 015-23 → 015b-06, 015-24 → 015b-07, 015-25 → 015b-08, 015-26 → 015b-09, 015-27 → 015b-10, 015-28 → 015b-11, 015-31 → 015b-12, 015-32 → 015b-13, new 015-34 → 015b-14, new 015-36 → 015b-15.

Changes in the last refinement: 015b-14 added (C-145 creates the panic hook); 015b-15 added (C-147: the host clock measurements formerly in 015a-03 run in strict mode). 015b-01 and 015b-10 now record strict-mode runs and depend on 015b-13 and epic 016. 015b-04 and 015b-05 use the panic hook and are no longer provisional; 015b-06's real step runs in the privileged nightly job (C-146). No ticket was deleted: the withdrawn rational-L rule had produced no work in this epic.

Changes in the C-177 to C-212 sweep (no ticket added or deleted, tally unchanged): 015b-13 refuses to start in every mode when seqlock safety depends on a failed or unverified CPU pin (C-177, C-201); 015b-04 adds the holder's sync replies through a housekeeping restart (C-188) and the restart stop with the ControlDegraded refusal (C-139, C-185); 015b-01 adds the over-rate follower case (C-204) and depends on 015b-12; 015b-04 and 015b-07 record their tools' output formats (C-206); 015b-11 lists C-182, C-185, C-188, C-201, C-204, C-206, C-209 and the stamp fields they touch; 015b-06's out-of-scope line now points at 015b-15 for the detector tolerance.

## Effort tally
- XS: 4 (1 day)
- S: 10 (10 days)
- M: 1 (2.5 days)
- Total: 15 tickets, about 14 days — epic scope M (C-126)

## Coverage
| Epic item | Tickets |
|---|---|
| Strict mode and capability states (C-127 to C-129, C-131, C-161, C-165); pin-dependent seqlock refusal (C-177, C-201) | 015b-13 |
| Test-only hooks (C-120, C-145) | 015b-12, 015b-14 |
| Asserting allocator for 017 | 015b-09 |
| Static lock check | 015b-02 |
| Scripted peer and DF checker in `hadronyche-tools` | 015b-04, 015b-07 |
| Strict-mode reports (C-147) | 015b-01, 015b-10, 015b-15 |
| Design documents for both parts | 015b-11 |
| 015b-A1 — Flood isolation (AC-11) | 015b-03, 015b-12 |
| 015b-A2 (a)–(d) — Housekeeping cannot damage a live stream (AC-20), with C-188 and the C-139/C-185 restart stop | 015b-04, 015b-14 |
| 015b-A2 (e) — static lock check | 015b-02 |
| 015b-A3 — Fail-stop (AC-21) | 015b-05, 015b-14 |
| 015b-A4 — Achieved sync rate (AC-7) and over-rate drop (C-204) | 015b-01 |
| 015b-A5 — One writer per file (AC-31) | 015b-08 |
| 015b-A6 — No allocation on the data thread | 015b-09 |
| 015b-A7 — Wall-clock step | 015b-06 |
| 015b-A8 — Don't-fragment | 015b-07 |
| 015b-A9 — Strict mode | 015b-13 |
| 015b-A10 — Measurements in strict mode (C-147) | 015b-01, 015b-10, 015b-15 |
| 015b-A11 — Test-only hooks | 015b-12, 015b-14 |
| DoD — Tests green in CI, unprivileged (per-change CI, C-146) | all code tickets; 015b-06's real step in the privileged nightly job; 015b-11 and 015b-15 marked not applicable |
| DoD — Every parser touched is fuzzed | not applicable in this part (C-125): no parser |
| DoD — Regime stamp fields documented | 015b-11, 015b-12, 015b-13, 015b-14 |
| DoD — Design documents updated, constants table first | 015b-01, 015b-02, 015b-10, 015b-11, 015b-15 |
| DoD — No new open item without owner and falsifier | most tickets; 015b-11 checks all |
| DoD — Frozen POC not modified | every ticket |
| DoD — Evidence tags | 015b-01, 015b-06, 015b-07, 015b-10, 015b-11, 015b-13, 015b-15 |
| DoD — Trait surface compiles with a stub backend | not applicable in this part (C-125): no trait change (015a-01, 015a-17, 015a-21) |
| DoD — Per-packet receive cost recorded for task 019 | 015b-10 |
| DoD applicability (C-125) | every ticket lists its not-applicable lines with reasons |
| Open sub-question: jitter and tolerance (measurement in this part) | 015b-15 |
| Open sub-questions owned elsewhere (receive cost: 019; CI namespaces: check on the CI host) | noted in 015b-07, 015b-10 |

## Remaining gaps
1. **Value of `declared_jitter_ns()` and the step-detector tolerance.** Affects 015b-06, 015b-15. Answer: a measurement, 015b-15 in strict mode (C-147), recorded in the constants table; the jitter is confirmed at L2.
2. **Unprivileged user namespaces on the CI host**, needed for the capture of 015b-07. Answer: a check on the CI host.

## Settled since the split
Settled for this part:
- Former 2, no-allocation check placed in the L0 harness (017) — epic change: the check runs here (015b-09); 017 reuses the allocator.
- Former 5, accepted sync range for AC-7 — C-112: 10 to 200 per second.
- Former 17, capture privilege — epic change: the capture runs unprivileged in a user and network namespace.
- Panic-injection hook (formerly gap 7, design owner) — C-145: a test-only hook injects a panic into a data, clock or housekeeping thread. Built in 015b-14; 015b-04 and 015b-05 use it and are no longer provisional.
- Real `CLOCK_REALTIME` step (formerly gap 8, design owner) — C-146: privileged tests run in a separate privileged nightly job; "unprivileged" applies to per-change CI. 015b-06.
- New rules applied without a former gap: C-147 (strict mode for measurement runs: 015b-01, 015b-10, 015b-15), C-161 (control isolation present on hosted platforms), C-165 (strict mode set in the commissioning configuration), C-139 (restart timing used by 015b-04), C-135 (other-epoch slices in the fail-stop criteria).
- Applied in the C-177 to C-212 sweep: C-177 and C-201 (failed or unverified pin under pin-dependent seqlock safety refuses in every mode: 015b-13; 015b-13's non-strict criterion now assumes the default layout), C-185 with C-139 (restart stop and ControlDegraded refusal: 015b-04), C-188 (holder answers sync on the clock thread: 015b-01, 015b-04), C-204 (over-rate sync requests dropped and counted per follower: 015b-01), C-206 (report names and tool output formats decided here: epic, 015b-04, 015b-07), C-182 and C-209 (listed and documented in 015b-11).
- Settled for part a: see 015a's index (C-136, C-137, C-138, C-139, C-144, C-152, C-155 to C-160, C-173 and others).
- Not applicable to this epic: C-104 and C-105 as revised and C-134 — no rational-L handling existed here, so no ticket was deleted. Also without a subject here: C-178 to C-181, C-183, C-184, C-186, C-187, C-189 to C-200, C-202, C-203, C-205, C-207, C-208, C-210 to C-212, and the late revisions of C-141, C-153 and C-169.
