# Epic 008b — Servo simulation cases and results: tickets

## Summary
Epic 008b (phase P2, depends on epic 008a) is the second part of epic 008, split by C-143. On the harness of 008a it runs seventeen named simulation cases and two studies that confirm or replace the proposed clock constants C-60 to C-63 (and the inherited C-66, C-67), check the decided C-64 and derived C-65 with the C-162 state after the lapse, test the C-136 holder stall, and test the falsifiers of the fan-in offset estimator (C-50 to C-52) and ratchet (C-53, C-54, with C-101, C-109 and the C-163 last step) used by epic 009. It checks determinism on x86-64 and 64-bit ARM (C-151). The specification is completed in `doc/design/spec/servo.md`, results go into `doc/design/reports/`, replaced values into `doc/design/02-constants.md` (C-130). Epic scope: L (20 tickets, about 24.5 days, C-126).

Tickets were renumbered from the unsplit epic: 008-17 to 008-35 became 008b-01 to 008b-19 in the same order; the new ticket 008-36 became 008b-20.

## Tickets
| Id | Title | Effort | Depends on |
|---|---|---|---|
| 008b-01 | Case 01: convergence and AC-2 constants | M | 008a-07, 008a-15 |
| 008b-02 | Case 02: step response | S | 008a-13, 008a-15 |
| 008b-03 | Case 03: asymmetry bias | S | 008a-15 |
| 008b-04 | Case 04: delay bursts | XS | 008a-15 |
| 008b-05 | Case 05: holdover | S | 008a-12, 008a-15, 008b-04 |
| 008b-06 | Case 06: handover at 50 ppm | S | 008a-16, 008b-05 |
| 008b-07 | Handover without a good rate estimate | S | 008b-06 |
| 008b-08 | Case 07: stale, duplicate and reordered replies | S | 008a-15 |
| 008b-09 | Case 08: achieved request rate | S | 008a-03, 008a-15 |
| 008b-10 | Case 09: wall-clock step | XS | 008a-08, 008a-15 |
| 008b-11 | Case 10: honest lock property test | M | 008a-12, 008a-13, 008a-15 |
| 008b-12 | Cases 11-12: continuity and slew in every scenario | S | 008b-01 to 008b-06, 008b-08 to 008b-11, 008b-19, 008b-20 |
| 008b-13 | Case 13: wander, MTIE and TDEV | S | 008a-09, 008a-15 |
| 008b-14 | Interrupt-coalescing bias study | S | 008a-06, 008a-15 |
| 008b-15 | Case 14: fan-in offset estimator | M | 008a-15 |
| 008b-16 | Case 15: ratchet with a flapping input | M | 008b-15 |
| 008b-17 | Cross-machine determinism check | S | 008b-12 to 008b-16, 008b-19, 008b-20 |
| 008b-18 | Servo spec, results table and constants update | S | 008a-07, 008b-01, 008b-02, 008b-05, 008b-07, 008b-12 to 008b-16, 008b-19, 008b-20 |
| 008b-19 | Case 16: drift ramp | S | 008a-15, 008b-01 |
| 008b-20 | Case 17: holder stall (new, from C-136) | S | 008a-05, 008a-13, 008a-15 |

No ticket was deleted: no ruling removed work, and epic 008 never handled the withdrawn rational-L rule (C-104, C-105 revised; C-134). Rulings narrowed 008b-16 (C-163 removes the second last-step variant) without changing its effort.

## Effort tally
- XS: 2 (0.5 day)
- S: 14 (14 days)
- M: 4 (10 days)
- Total: 20 tickets, about 24.5 days → scope L (C-126)

## Coverage

### Deliverables
| Epic deliverable | Tickets |
|---|---|
| 1.1 Convergence / steady-state error | 008b-01 |
| 1.2 Step response | 008b-02 |
| 1.3 Asymmetry bias | 008b-03 |
| 1.4 Delay bursts | 008b-04 |
| 1.5 Holdover (with C-162) | 008b-05 |
| 1.6 Handover at 50 ppm, triggered as a holder loss (C-212) | 008b-06 (study: 008b-07) |
| 1.7 Stale, duplicate, reordered replies | 008b-08 |
| 1.8 Achieved request rate | 008b-09 |
| 1.9 Wall-clock step | 008b-10 |
| 1.10 Honest lock property test | 008b-11 |
| 1.11 Zero phase step while locked | 008b-12 |
| 1.12 Correction term and its rate of change | 008b-12 |
| 1.13 Wander MTIE/TDEV | 008b-13 |
| 1.14 Offset estimator (C-160 percentiles reported) | 008b-15 |
| 1.15 Ratchet (C-163) | 008b-16 |
| 1.16 Drift ramp | 008b-19 |
| 1.17 Holder stall (C-136) | 008b-20 |
| 1. Studies: poor rate estimate, coalescing bias | 008b-07, 008b-14 |
| 1. Cross-machine determinism (C-151) | 008b-17 |
| 2. `servo.md` with C-60..C-67 and C-50..C-54 tables | 008b-18 |

### Acceptance criteria
| Epic criterion | Tickets |
|---|---|
| A1 (AC-2: bound ≤ 25 µs within 10 s, C-94, ±100 ppm, or replace) | 008b-01, 008b-18 |
| A2 (AC-3: zero phase step, correction ≤ 10 ppm, ≤ 1 ppm/s, every scenario) | 008b-12 |
| A3 (AC-4: honest lock property) | 008b-11 |
| A4 (AC-5: handover at 50 ppm, holder-loss trigger, holdover observed and within E, C-202, C-212) | 008b-06 |
| A5 (AC-6: holdover 1 µs/s, never usable beyond lapse, Unlocked after it) | 008b-05 |
| A6 (AC-7: achieved rate within 2 %, 10 to 200 per second) | 008b-09 |
| A7 (case 3 bias = half asymmetry within 1 %) | 008b-03 |
| A8 (AC-17 simulation part: ratchet, C-101, C-163, C-109) | 008b-16 |
| A10 (deterministic on x86-64 and 64-bit ARM, C-151) | 008b-17 |
| A11 (case 14: zero late slices after 4 s alignment) | 008b-15 |
| A12 (case 16: drift ramp stays Locked, honest, within C-62) | 008b-19 |
| A17 (case 17: holder stall, C-136) | 008b-20 |

### Definition of done
| Epic DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all code tickets |
| Every parser touched is fuzzed | not applicable (C-125): no parser in this part; noted in the epic |
| Regime stamp fields touched are complete and documented | not applicable (C-125): no stamp field; noted in the epic and in 008b-18 |
| Design documents updated where behaviour differs, constants table first | 008b-18 |
| No new open item without an owner and a falsifier | 008b-01, 008b-05, 008b-06, 008b-07, 008b-14, 008b-15, 008b-16, 008b-18 |
| The frozen POC is not modified | all tickets |
| Every new factual claim carries an evidence tag | 008b-01, 008b-02, 008b-05, 008b-06, 008b-07, 008b-13 to 008b-16, 008b-18, 008b-19, 008b-20 |
| Results table for C-60..C-67 in `servo.md` and constants table | 008b-18 |
| Wired jitter profile written down with its source | 008b-18 (section created by 008a-07) |
| Seqlock priority invariant documented at the type | not applicable (C-125): the seqlock is in 008a (008a-02) |

### Open sub-questions
| Question | Ticket |
|---|---|
| 10 ppm and 1 ppm/s on the correction term compatible with 25 µs and 10 s? | 008b-01 |
| Can a successor with usable time (C-213) have a rate estimate too poor for C-61? | 008b-07 |
| Which wired jitter profile is representative (decided provisionally, C-94; L1 replaces it) | 008b-01, 008b-18 |
| Does interrupt-coalescing bias survive the min-RTT filter | 008b-14 |
| Is 4 s of aligning enough | 008b-15 |
| Holdover entry delay: value of C-195, at or below C-37 (hard limit) | 008b-04, 008b-05, 008b-11, 008b-18 |
| Production worst-case asymmetry bound | 008b-03, 008b-14 |

## Remaining gaps
1. **Production worst-case asymmetry bound.** The constants table has no value. Affects 008b-03, 008b-14. Answer: a measurement, L1 (task 019), informed by 008b-14.
2. **Measured wired profile.** C-94 is synthetic and provisional. Affects 008b-01 and every case using `WIRED_PROFILE`. Answer: a measurement, L1 (task 019).
3. **E of the Linux reference sink.** C-70 (50 µs) is pending measurement, and C-62, C-65 and the case 13 threshold derive from it. Affects 008b-01, 008b-05, 008b-13, 008b-19. Answer: a measurement, L2 Linux.
4. **Merge rule.** The merge scenarios used by the multi-holder cases follow epic 006's rule. Affects 008b-06, 008b-07 through 008a-16. Answer: epic 006.

## Settled since the split
1. Case 2 "stated time" → epic change: re-lock is an acquisition, bounded by C-63 from the first exchange of the new epoch.
2. E only for Linux → C-62 is defined against C-70; case 13 uses C-77 with C-70; the threshold is the target E, never a measured E (C-149).
3. "Unchanged" in case 7 → epic change: bit-identical when the same packet wins a window, within 1 µs otherwise.
4. Ratchet "persistently late", value of A, case 14 target, location of production code → C-101, C-55 (A is a scenario parameter), C-109 (Demoted), C-108 (late counting), C-50..C-52 falsifier (target zero, criterion A11); production code is epic 009.
5. Where C-50..C-54 results are recorded → a second table in `servo.md`, raw results in `doc/design/reports/` (C-130).
6. "Regime stamp fields" and "parser fuzzed" → C-125: marked not applicable with reasons.
7. Which two machines for determinism → C-151: x86-64 and 64-bit ARM (008b-17; the earlier "different OS images" fallback is removed).
8. Four-timestamp formulas and asymmetry sign → in the Background.
9. State after the holdover lapse → C-162: `Unlocked`, time unusable (008b-05 asserts it; the design-owner question is closed).
10. Last ratchet step when A − offset < one slice → C-163: the offset steps to A exactly (008b-16 runs one variant; the design-owner question is closed).
11. Lateness percentiles reported by case 14 → C-160: 50th, 99th, 99.9th and maximum (008b-15).
12. Holder stall → C-136: a divergence above C-67 bumps the epoch; new case 17, ticket 008b-20; C-67's result in 008b-02 notes that C-136 reuses it.
13. Strict mode for runs whose numbers enter the constants table → C-147: no subject in a simulated-time run (noted in 008b-18).
14. Location of the architecture specification → C-171: `doc/design/01-architecture.md`, named in 008b-18.
15. Oversized epic → C-143: epic 008 split into 008a and 008b.
16. Holdover entry delay → C-195: a constant whose value is pending simulation, proposed by this epic and expected at or below the 1.5 s election silence threshold (C-37), so a follower is in holdover with a growing bound when it calls an election. 008b-05 sweeps up to and around 1.5 s, proposes a value at or below it (or records the counter-example) and checks the view 1.5 s into reply silence; 008b-04, 008b-11 and 008b-18 cite C-195. The design-owner gap is closed except for recording the proposed value.
17. Publisher of the lock state and bound → C-187: the 008a servo core, through the published time view; the epic's relation to 008a and its dependencies cite it. No ticket changes.
18. How the handover case is triggered → C-212 with C-202: as a holder loss, followers passing through holdover between holders with the bound within the target E, frequency continuity at takeover, a run without observed holdover failed. 008b-06 now uses this trigger with the C-195 value from 008b-05 (new dependency) and asserts it; 008b-07 repeats its degraded-estimate runs under it; epic deliverable 1.6 and A4 updated. Effort unchanged.
19. Lapse reference when E is unknown → C-169 (revised): 2 × C-62. 008b-05 adds an unset-E run and criterion; 008b-18 recomputes the C-169 figures if C-62 is replaced; the epic's holdover requirement cites it.
20. Window of the stamp's lateness figures → C-182: case 14 reports its C-160 percentiles over a rolling 60 s window with the maximum since aligning beside them (008b-15).
21. Presentation details of this epic's tools → C-206: file names and tables under `008-simulation/` and the `sim-digests` format are decided here and recorded in the `hadronyche-tools` README (epic Background, 008b-17).
22. Candidacy of a follower in holdover → C-213 (with C-113 revised to cite it): a holder-eligible module whose time is usable (Locked, or in holdover with its claim not lapsed, C-65, C-169) takes over with frequency continuity and the epoch kept; the epoch is bumped only when no candidate has usable time. 008b-06 now asserts a takeover from Holdover with the epoch unchanged and keeps the other reading only as a labelled comparison run; 008b-07 restates eligibility as usable time, puts lapsed-holdover candidates on the C-113 path and adds a criterion for the lapse boundary; epic handover requirement, deliverable 1.6, A4 and open sub-questions updated. The design-owner gap is closed. Efforts unchanged.
23. Holdover entry delay limit → C-195 (revised): at most C-37 (1.5 s) is a hard limit, the value within it pending simulation; if no value at or below 1.5 s works, that is reported as a finding. Wording updated in the epic, 008b-04, 008b-05, 008b-11 and 008b-18; 008b-05 already never proposes a larger value. No ticket changes in effort.
24. Handover trigger → C-212 (rechecked): unchanged for this epic (item 18); its requirement of a second holder-eligible Linux node is for the L2 rig, which the simulation's multi-holder model already covers.
25. Several candidates with usable time → C-221: a Locked candidate is preferred over one in holdover; among equals the lowest module id wins. Case 6 gives the successor the lowest module id among holder-eligible nodes and asserts that it takes over (epic Requirements, case 6, A4; 008b-06 Context, Work, AC 1).
26. Source of the 50 ppm handover difference → C-223: on the L2 rig, the measured natural crystal difference plus an injected drift, both recorded in the run report; in this simulation the whole 50 ppm is injected, and case 6 and its results file say so (epic case 6, 008b-06 Context, Work, AC 7). No new ticket.

Also applied earlier: C-119 (new case 16, 008b-19), C-113 (008b-07), C-64 decided, C-78, C-10, C-100, C-01/C-02, C-132, C-130 and C-133. Reviewed with no effect on this part: C-104, C-105 and C-134 (no structural latency or resampler node in this epic; C-134 revised to cite C-207, still none), C-35, C-127 to C-131, C-135, C-137 to C-142 (C-141 revised: transport, no subject), C-144 (duties of epic 015 only), C-145, C-146 (all tests here are unprivileged), C-148, C-150, C-152 to C-159 (C-153 revised: commissioning, no subject), C-161, C-164 to C-168, C-170, C-172 to C-176. From C-177 to C-212: C-177 and C-201 (seqlock and pinning are 008a's; the simulation has no threads), C-178 to C-181 and C-183 to C-185 (commissioning, loss, certification, controllers, FIR, transport, refusal reasons; C-181 and C-184 revised, still no subject), C-186, C-198, C-208 and C-211 (a change of A while inputs are live is production fan-in behaviour, epic 009; A is fixed per scenario here), C-188 (holder threads, none in simulation), C-189, C-190, C-197, C-207 and C-210 (resamplers), C-191, C-194 and C-203 (L2/L3 parameters with no simulation counterpart), C-192, C-193, C-196 (no dependency on a later phase), C-199, C-200, C-204 (holder-side drop above 200 per second; case 8 tests the pacer within C-112), C-205, C-209 and C-08 (revised; bundles).
