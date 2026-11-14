# Epic 025b — Replay, cross-target fingerprint and cost calibration: tickets

## Summary
Epic 025b (phase P5, depends on epics 007, 013, 015, 017, 024 and part 025a; blocks 026; scope L, 13 tickets, about 17 days) is the second part of epic 025, split under C-143. It proves that exactness and cost promises hold beyond the host: the record-and-replay verifier (`hadronyche-std/record.rs` writing through the epic 015 single-writer file service, C-144, and `hadronyche-replay` in `hadronyche-tools`), the CI job `dsl-cross-target` (exact vectors and fingerprint on x86-64, AArch64 and one 32-bit device target; cross-machine pair per C-151), fingerprint fallback and mutation coverage, fixed-point resampler exactness (2:1 and 1:2 only, C-105), per-target calibration factors from a reference workload (C-106, C-176) and their regime stamp field (stamp message of epic 013, sent by epic 015, C-144), the CI job `dsl-cost`, and the published listed-target set. It uses part 025a's corpus, host vectors, `dsl-conformance` job and DSL echo.

## Tickets
| Id | Title | Effort | Depends on |
|---|---|---|---|
| 025b-01 | Recording format and parser | S | epic 007 |
| 025b-02 | Runtime recorder `record.rs` | S | 025b-01, epic 015 |
| 025b-03 | Replay core | M | 025b-01 |
| 025b-04 | `hadronyche-replay` CLI and user documentation | S | 025b-02, 025b-03 |
| 025b-05 | Replay acceptance: DSL echo under injected loss | S | 025a-18, 025b-04, epic 017 |
| 025b-06 | CI job `dsl-cross-target`: exact vectors and fingerprint gating | M | 025a-09, 025a-10, epic 024 |
| 025b-07 | Fingerprint mismatch fallback and regime stamp | S | 025b-06, epic 013, epic 015 |
| 025b-08 | Fingerprint mutation coverage | M | 025b-06, 025b-07 |
| 025b-09 | Resampler fixed-point exactness across targets | XS | 025a-04, 025b-06 |
| 025b-10 | Cost measurement and per-target calibration | M | 025a-10, 025b-06 |
| 025b-11 | CI job `dsl-cost` | S | 025b-10 |
| 025b-12 | Calibration factor in the regime stamp | XS | 025b-10, epic 013, epic 015 |
| 025b-13 | Publish the listed-target set | XS | 025b-06, 025b-08, 025b-09, 025b-11, 025b-12 |

Former ids (epic 025 before the split): 025b-01 to 025b-11 were 025-18 to 025-28; 025b-12 was 025-30; 025b-13 was 025-29.

No ticket was deleted: none existed only for the withdrawn rational-L rule. In 025b-09 the long-run vectors for the 147/320 ratio were removed (no such ratio in v1, C-105), so it went from S to XS. No ticket was added in this part: the new duties (C-144, C-151, C-176) fit existing tickets.

## Effort tally
- XS: 3
- S: 6
- M: 4
- Total: 13 tickets, about 17 days (16.75; XS 0.25, S 1, M 2.5 days), scope L (C-126)

## Coverage
### Deliverables
| Deliverable | Tickets |
|---|---|
| `hadronyche-replay` (incl. `ReplayOutcome` with excluded-window count, `replay`) | 025b-01, 025b-03, 025b-04 |
| `hadronyche-std/record.rs` (through the epic 015 file service, C-144) | 025b-01, 025b-02 |
| CI job `dsl-cross-target` | 025b-06 |
| CI job `dsl-cost` | 025b-10, 025b-11 |
| Fingerprint mutation suite and coverage map | 025b-08 |
| Provides: listed-target set for exact-float | 025b-06, 025b-13 |
| Provides: per-target calibration factors for the stamp and admission (C-106, C-176) | 025b-10, 025b-12 |

### Acceptance criteria
| Epic AC | Tickets |
|---|---|
| 1. AC-33, cost (time against bound × factor) | 025b-10, 025b-11 |
| 2. AC-32, exact-fixed | 025b-06, 025b-09 (vectors from 025a-09) |
| 3. AC-32, exact-float | 025b-06 |
| 4. AC-32, fallback | 025b-07 |
| 5. Fingerprint coverage | 025b-08 (failed-suite rule also enforced in 025b-06) |
| 6. Replay | 025b-03, 025b-05 |
| 7. Calibration in the stamp | 025b-12 |

### Definition of done
| Epic DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all except 025b-13 (documentation only) |
| Every parser touched is fuzzed (C-95 budget) | 025b-01, 025b-04 |
| Regime stamp fields touched are complete and documented | 025b-07, 025b-12 |
| Design documents updated where behaviour differs, constants table first | 025b-07, 025b-08, 025b-09, 025b-10, 025b-12, 025b-13 |
| No new open item without an owner and a falsifier | 025b-03, 025b-05, 025b-06, 025b-08, 025b-09, 025b-10 |
| The frozen POC is not modified | all |
| Every new factual claim carries an evidence tag | all |
| Listed-target set published with verifying CI job | 025b-13 |
| Cost-bound calibration recorded | 025b-10 |
| Probe and replay user documentation with one worked example each | 025b-04 (replay); probe in 025a |

### Open sub-questions
| Question | Handling |
|---|---|
| Cost-bound calibration per target | unit decided (C-106), procedure decided (C-176); factors measured in 025b-10, stamped in 025b-12 |
| Home of the single-writer file service | settled by C-144 (epic 015); ticket 025b-02 |
| Fingerprint vector coverage | ticket 025b-08 |
| Is the resampler node exact-fixed in fixed-point form? | ticket 025b-09 |
| Other open items in the epic table | see Remaining gaps |

## Remaining gaps
Only what neither the epic nor the constants table answers.

| # | Gap | Tickets affected | Who answers |
|---|---|---|---|
| 1 | Names and encoding of the stamp fields for fingerprint result, effective class and calibration factor (transport C-18; message defined by epic 013, sent by epic 015, C-144) | 025b-07, 025b-12 | Regime stamp epic 013 |
| 2 | Which 32-bit device target is listed, and whether CI has it on hardware (emulation gives no calibration factor under C-106) | 025b-06, 025b-09, 025b-10, 025b-11, 025b-12, 025b-13 | Embedded device epic 026 |
| 3 | L0 injected-loss rate and pattern | 025b-05 | Epic 017 |
| 4 | Recording format and output hash (reusing epic 007 where possible) | 025b-01, 025b-02, 025b-03 | This epic, ticket 025b-01 |
| 5 | Excluded-window alignment (timeline or recording-relative) | 025b-03, 025b-05 | This epic, ticket 025b-03 |
| 6 | How to build the contraction and extended-precision mutants | 025b-08 | This epic, ticket 025b-08 |
| 7 | Content of the calibration reference workload (C-176) | 025b-10 | This epic, ticket 025b-10 |

## Settled since the split
- Former gap 2, home of the single-writer file service: the hosted backend, epic 015 (C-144); 025b-02 now depends on epic 015.
- Former gap 8, stamp fields: the stamp message is defined by epic 013 and sent by epic 015 (C-144), so field names are now owned by epic 013 (was epic 004); 025b-07 and 025b-12 depend on 013 and 015.
- Calibration procedure: a reference workload run once per listed target, time per operation recorded and carried in the stamp (C-176); 025b-10. The calibration report states whether the run was in strict mode (C-127, C-147).
- Cross-machine bit-exactness pair: x86-64 and 64-bit ARM (C-151); 025b-06.
- Rational L withdrawn (C-105 revised): the 147/320 long-run resampler vectors are dropped from 025b-09.
- Epic split into 025a and 025b because the tally exceeded 40 days (C-143); scope recomputed from the tally (C-126).
- Two kinds of resampler (C-189): 025b-09 tests only the fixed 2:1 / 1:2 node; the steerable rate actuator and its trait are built and tested by the kit epic 024a (C-197, C-210). Resampler conversion delay is counted only in D, never in L (C-134, C-207); stated in 025b-09.
- Fuzzing tool is the implementing team's choice (C-192); 025b-01.
- Presentation details of `hadronyche-replay` (line format, documentation file name) are decided by this epic and recorded in its user documentation; only the JSON schema and exit statuses, which scripts depend on, go to `doc/design/spec/` (C-206); 025b-04 Work and acceptance criteria updated.
- Rules C-177 to C-212 checked; the others have no subject in this epic. No ticket added or deleted; tally unchanged.
- Earlier rulings kept: C-18, C-95, C-106, C-122, C-123, C-130, C-132, C-133.
