# Epic 025a — DSL conformance corpus, probe and reference programs: tickets

## Summary
Epic 025a (phase P5, depends on epics 004, 007, 017, 017b, 023, 024; blocks 025b and 026; scope L, 22 tickets, about 26 days) is the first part of epic 025, split under C-143. It proves on the host, in CI, that the DSL compiler's compile-time promises hold: the structural latency L (always a whole number of domain-rate samples, C-105), the cost bound as an executed abstract-operation count (C-106), stack and state sizes, inserted join delays and flags (C-104), the determinism class it assigns (exact-fixed or exact-float, C-123), and the gap behaviour it generates (C-122, C-141, C-158, C-184, C-200), and controller placement with the bundle cap (C-181, C-205, C-209). It delivers the conformance corpus (at least 40 positive programs, float and fixed variants counting as one (C-192), and at least 25 negative programs, hand-derived expectations, checked-in host vectors) in `hadronyche-dsl/conformance/`, the CI job `dsl-conformance`, the latency probe `hadronyche-probe` in `hadronyche-tools`, and the reference echo and mixer as DSL programs, shown bit-identical to the hand-written modules of epic 017. Part 025b (replay, cross-target fingerprint, cost calibration) depends on this part.

## Tickets
| Id | Title | Effort | Depends on |
|---|---|---|---|
| 025a-01 | Conformance harness and expect format | S | epic 023, epic 024 |
| 025a-02 | Corpus: gains, mixes, delays, lookahead, buffering, ramps | M | 025a-01 |
| 025a-03 | Corpus: linear-phase FIR, recursive filters, table oscillators | M | 025a-01 |
| 025a-04 | Corpus: resampler chains (edge modules, 96↔48) | S | 025a-01 |
| 025a-05 | Corpus: controller instruments and transport readers | S | 025a-01 |
| 025a-06 | Corpus: latency joins and padding | S | 025a-01 |
| 025a-07 | Negative corpus | S | 025a-01 |
| 025a-08 | Negative corpus: resampler placement and ratio (new, C-134, C-105) | XS | 025a-07, epic 023 |
| 025a-09 | Exact-class conformance vectors | S | 025a-05, 025a-06, epic 007, epic 024 |
| 025a-10 | CI job `dsl-conformance` | XS | 025a-07, 025a-08, 025a-09, 025a-21, 025a-22 |
| 025a-11 | Executed operation count against the cost bound | S | 025a-10, epic 024 |
| 025a-12 | Latency probe core | M | epic 017, epic 024 |
| 025a-13 | Probe: non-constant delay detection | S | 025a-12 |
| 025a-14 | `hadronyche-probe` CLI and user documentation | S | 025a-13 |
| 025a-15 | Latency conformance (AC-33) | S | 025a-04, 025a-06, 025a-10, 025a-13 |
| 025a-16 | Gap default conformance per content type | S | 025a-03, 025a-05, 025a-10 |
| 025a-17 | Memory conformance: stack and state size | S | 025a-10 |
| 025a-18 | Reference echo as a DSL program | M | 025a-10, epic 017, epic 024 |
| 025a-19 | Reference mixer as a DSL program | S | 025a-10, epic 017, epic 024 |
| 025a-20 | Reference equivalence with hand-written modules | S | 025a-18, 025a-19, epic 017, epic 017b |
| 025a-21 | Negative corpus: even-tap linear-phase FIR and device-side joins (new, C-183, C-190) | XS | 025a-07, epic 023 |
| 025a-22 | Corpus: controller placement and the bundle cap (new, C-181, C-205, C-209) | S | 025a-05, 025a-07, epic 004, epic 023 |

Former ids (epic 025 before the split): 025a-01 to 025a-05 were 025-01 to 025-05; 025a-06 was 025-31; 025a-07 was 025-06; 025a-08 was added as 025-33; 025a-09 was 025-07; 025a-10 was 025-08; 025a-11 was 025-32; 025a-12 to 025a-20 were 025-09 to 025-17.

No ticket was deleted: none existed only for the withdrawn rational-L rule. Its rational parts were removed from 025a-01, 025a-02, 025a-03, 025a-04, 025a-06, 025a-12, 025a-13, 025a-14, 025a-15 and 025a-18: the 96→44.1 and 44.1→96 chains, rational L and its reduction, the probe's fractional measurement method. As a result 025a-04 went from M to S (fewer chains) and 025a-15 from M to S (no fractional method). Ticket 025a-08 was added for the rejections that C-134 and C-105 create, ticket 025a-21 for those that C-183 and C-190 create, and ticket 025a-22 for the controller placement and bundle-cap checks that C-181, C-205 and C-209 create.

## Effort tally
- XS: 3
- S: 15
- M: 4
- Total: 22 tickets, about 26 days (25.75; XS 0.25, S 1, M 2.5 days), scope L (C-126)

## Coverage
### Deliverables
| Deliverable | Tickets |
|---|---|
| `hadronyche-dsl/conformance/corpus/` (≥ 40 positive programs, float and fixed variants counting as one, C-192; ≥ 25 negative) | 025a-01 to 025a-08, 025a-21, 025a-22 |
| `conformance/expect/` (whole-sample L and non-constant flag, class, deps, join delays and flags, initial values, edge conversion delays (C-207), controller placement (C-181, C-205), finite cost bound, diagnostics) | 025a-01 to 025a-08, 025a-21, 025a-22 |
| `conformance/vectors/` (inputs and output hashes for every positive program, host runner) | 025a-09 |
| `hadronyche-probe` (incl. `Marker`, `Delay`, `ProbeResult`, `probe`) | 025a-12, 025a-13, 025a-14, 025a-15 |
| CI job `dsl-conformance` | 025a-10 (extended by 025a-11, 025a-15 to 025a-20) |
| `hadronyche-reference/dsl/echo.hdsl` | 025a-18 |
| `hadronyche-reference/dsl/mixer.hdsl` | 025a-19 |
| Provides to 025b: corpus, vectors, job, DSL echo, operation counts | 025a-09, 025a-10, 025a-11, 025a-18 |

### Acceptance criteria
| Epic AC | Tickets |
|---|---|
| 1. AC-33, latency (whole-sample L, C-105; resampler delay outside L, C-207) | 025a-12, 025a-13, 025a-15 |
| 2. AC-33, cost (operation count) | 025a-11 |
| 3. Reference equivalence | 025a-20 |
| 4. Gap behaviour (C-122, C-141, C-158, C-184, C-200) | 025a-16 (content types, degraded at q + L), 025a-19 (reference mixer) |
| 5. Memory | 025a-17 |
| 6. Negative corpus (incl. C-134, C-105, C-183, C-190, C-209) | 025a-07, 025a-08, 025a-21, 025a-22 |
| 7. Latency joins (C-104, C-190) | 025a-06, 025a-15, 025a-21 |
| 8. Controller placement (C-181, C-205, C-209) | 025a-22 |

### Definition of done
| Epic DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all |
| Every parser touched is fuzzed (C-95 budget) | 025a-01, 025a-09, 025a-14, 025a-22 |
| Regime stamp fields touched are complete and documented | not applicable in this part (C-125): no stamp field; see 025b |
| Design documents updated where behaviour differs, constants table first | 025a-01 (expect format spec), 025a-22 (its extension) |
| No new open item without an owner and a falsifier | 025a-04, 025a-15, 025a-16, 025a-11 |
| The frozen POC is not modified | all |
| Every new factual claim carries an evidence tag | all except 025a-10 |
| Listed-target set published with verifying CI job | not applicable in this part (C-125): part 025b |
| Cost-bound calibration recorded | not applicable in this part (C-125): part 025b |
| Probe and replay user documentation with one worked example each | 025a-14 (probe); replay in 025b |

### Open sub-questions
| Question | Handling |
|---|---|
| Structural latency for non-integer ratios; probe method for a rational L | withdrawn by C-105 (revised): L is whole, other ratios out of v1 |
| Resampler node outside an edge module | settled by C-134; ticket 025a-08 |
| Resampler delay inside L or separate | settled by C-207: only in D, reported separately on the device-facing port; tickets 025a-01, 025a-04, 025a-15 |
| Join with a non-constant-delay path | settled by C-104 (revised); ticket 025a-06 |
| Degraded flag for non-integer or non-constant L | settled by C-105, C-104, C-122; ticket 025a-16 |
| Transport-state gap default | settled by C-141 (revised), C-200, C-184; ticket 025a-16 |
| Controller placement and the bundle cap | settled by C-181, C-205, C-209; ticket 025a-22 |
| Even-tap linear-phase FIR | settled by C-183; ticket 025a-21 |
| Float and fixed variants towards the 40 | settled by C-192: they count as one; ticket 025a-05 |
| L0 baseline scenario | settled by C-192: defined by epic 017b; ticket 025a-20 |
| Do POC voice windows compare bit for bit? (owner: task 002) | note in 025a-18 and 025a-20 |
| Other open items in the epic table | see Remaining gaps |

## Remaining gaps
Only what neither the epic nor the constants table answers.

| # | Gap | Tickets affected | Who answers |
|---|---|---|---|
| 4 | Refresh interval of sparse keyed controller state | 025a-05, 025a-16 | Wire specification epic 004 |
| 6 | Diagnostic ids of the negative-corpus rejections | 025a-07, 025a-08, 025a-21 | Epic 023 |
| 7 | Definition of an abstract operation and the instrumented build that counts executed ones | 025a-11 | Epic 024 |
| 8 | Expect-file syntax | 025a-01, 025a-22 | This epic, ticket 025a-01 |
| 9 | Stack measurement method | 025a-17 | This epic, ticket 025a-17 |

## Settled since the split
- Transport-state gap rule revised (C-141, C-200): the playhead advances as the held state dictates, so a stopped transport stays where it is and a playing one moves on at its tempo; the former "advanced by the positions elapsed" reading is corrected in the epic, 025a-05 (transport reader exposes play/stop state and tempo) and 025a-16 (Work and AC 7 test a stopped and a playing transport). Transport state before any has arrived is stopped, playhead at zero, tempo 120 beats per minute (C-184, C-199); 025a-16 gains AC 9. No new ticket; efforts unchanged.
- Controller placement and the bundle cap (C-181, C-205, C-209, C-08): a bundle source's controller member counts towards the 32 members, a wider source with controllers is rejected at compile time, and a single-stream source's controllers share the packet or get a second stream as the compiler checks from the manifest. These are compile-time promises, so the earlier note that no ticket of this part concerns C-205 is superseded: new ticket 025a-22 (S), epic AC 8, and 025a-10 depends on it.
- Presentation details of the probe (table layout, JSON field names, file names) are decided by 025a-14 and recorded in its documentation (C-206).
- Former gap 2, resampler delay and L: an edge's resampler delay is counted only in D and never in L; L is the module's structural delay in the domain-rate timeline, and the manifest reports the conversion delay separately, on the device-facing port (C-207). The provisional "inside L or separate" wording is replaced: resampler delay removed from the L definition (025a-01, 025a-12), `conversion_delay` added to the expect format and harness (025a-01), resampler expectations split into L and conversion delay (025a-04), probe compared with L plus conversion delay on resampler paths (025a-15). No new ticket; efforts unchanged.
- New rule, controllers of a bundle source always in their own member stream (C-205): superseded by the controller-placement entry above (025a-22).
- Former gap 3, even-tap linear-phase FIR: rejected by the compiler in v1 with a diagnostic suggesting an odd-tap design (C-183); 025a-03, new ticket 025a-21.
- New rule, join on the device side of an edge module: paths differing by an odd number of domain-rate samples are rejected, with the per-join opt-out suggested (C-190); new ticket 025a-21.
- Former gap 1, float and fixed variants towards the 40: they count as one program (C-192); 025a-01, 025a-05.
- Former gap 5, L0 baseline scenario: defined by epic 017b (C-192); 025a-20. Also from C-192: the reference dense controller is the mixer's volume (025a-19).
- Rational L withdrawn (C-105 revised): L is always a whole number of domain-rate samples. Removed: rational expect values and reduction (025a-01), the 96→44.1 and 44.1→96 chains (025a-04), the probe's fractional measurement method (025a-15, formerly remaining gap 13), `Rational` in the probe shape (now `Samples`, 025a-12, 025a-13, 025a-14). Ratios other than 2:1 and 1:2 are now rejected (025a-08).
- Former gap 5, resampler node outside an edge module: rejected by the compiler (C-134); ticket 025a-08.
- Former gap 6, join padding with a non-constant-delay path: padded on its structural part only, join flagged in the manifest (C-104 revised); 025a-06, 025a-15.
- Former gap 4, degraded flag for non-integer or non-constant L: L is always whole (C-105) and a non-constant-delay output follows its structural L (C-104, C-122); 025a-16.
- Former gap 1, gap default for read-only transport state: last known state held, output degraded (C-141; playhead rule revised, see above); 025a-05, 025a-16.
- New rule, dense controller with no prior value: declared initial value, default 0 (C-158); 025a-01, 025a-05, 025a-16.
- Epic split into 025a and 025b because the tally exceeded 40 days (C-143); scope recomputed from the tally (C-126).
- Earlier rulings kept: C-95, C-122, C-123, C-130, C-133, C-01 to C-04.
