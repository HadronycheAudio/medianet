# Epic 008a — Servo core and simulation harness: tickets

## Summary
Epic 008a (phase P2, depends on epics 004 and 006) is the first part of epic 008, split by C-143 because the unsplit epic reached 36 tickets and about 46.5 days. It replaces the POC clock servo with a frequency-only servo in `hadronyche-core` (types `Exchange`, `TimeView`, `LockState`, `ServoConfig`, `Servo`, the seqlock `PublishedView`, the deadline `RequestPacer`; fixed size, `no_std`, no allocator), writes the reference wall-clock step detection used by the platform backend (C-121), and builds the deterministic, hosted, dev-only simulation harness in the `sim` module of the `hadronyche-tools` crate (C-133): seeded core, two-clock, link (with the C-94 preset), wall-clock with a simulated backend, and multi-holder models; a recorder of true error, bound, state, phase steps, rate, correction term and its rate of change (C-119), MTIE/TDEV; and the closed-loop scenario runner. It also creates `doc/design/spec/servo.md` with the wired-jitter-profile section. Part 008b depends on this part and runs every simulation case on it. Epic scope: L (16 tickets, about 22 days, C-126).

Tickets were renumbered from the unsplit epic: 008-01 to 008-16 became 008a-01 to 008a-16 in the same order.

## Tickets
| Id | Title | Effort | Depends on |
|---|---|---|---|
| 008a-01 | Core time types and no_std build | S | epic 004, epic 006 |
| 008a-02 | Seqlock PublishedView | S | 008a-01 |
| 008a-03 | Deadline RequestPacer | S | 008a-01 |
| 008a-04 | Sim crate skeleton and seeded determinism | S | 008a-01 |
| 008a-05 | Two-clock model | S | 008a-04 |
| 008a-06 | Link model | M | 008a-04 |
| 008a-07 | Wired jitter profile | S | 008a-06 |
| 008a-08 | Wall-clock model and stamp step detection | S | 008a-01, 008a-04 |
| 008a-09 | Trace recorder and MTIE/TDEV | M | 008a-04 |
| 008a-10 | Servo measurement window and feed | S | 008a-01 |
| 008a-11 | Servo frequency loop, bound and lock | M | 008a-10, 008a-02 |
| 008a-12 | Servo holdover and lapse | S | 008a-11 |
| 008a-13 | Servo step threshold and epoch change | S | 008a-11, epic 006 |
| 008a-14 | Servo handover: begin_holding and rebase | S | 008a-11, 008a-12, epic 006 |
| 008a-15 | Closed-loop scenario runner | M | 008a-02, 008a-03, 008a-05, 008a-06, 008a-08, 008a-09, 008a-11 |
| 008a-16 | Multi-holder model | S | 008a-14, 008a-15, epic 006 |

No ticket was deleted in this part. The withdrawal of the rational-L rule (C-104, C-105 revised; C-134) removed nothing: epic 008 never handled structural latency, rational numbers, rounding of L or fractional delay lines. C-35's owner change does not touch this epic.

## Effort tally
- XS: 0 (0 days)
- S: 12 (12 days)
- M: 4 (10 days)
- Total: 16 tickets, about 22 days → scope L (C-126)

## Coverage

### Deliverables
| Epic deliverable | Tickets |
|---|---|
| 1. `Exchange`, `TimeView`, `LockState`, `ServoConfig` field list | 008a-01 |
| 1. `Servo` (`new`, `feed`, `tick`, `view`, `begin_holding`, `rebase`; fixed size, no allocation) | 008a-10, 008a-11, 008a-12, 008a-13, 008a-14 |
| 1. `PublishedView` seqlock | 008a-02 |
| 1. `RequestPacer` (range C-112) | 008a-03 |
| 1. Reference wall-clock step detection (backend-side, C-121) | 008a-08 |
| 2. Seeded deterministic core in `hadronyche-tools` (hosted, dev-only, C-133) | 008a-04 |
| 2. Two-clock model (including the C-119 ramp) | 008a-05 |
| 2. Link model, C-94 preset and sensitivity presets | 008a-06, 008a-07 |
| 2. Wall-clock model and simulated backend detection (C-121) | 008a-08 |
| 2. Multi-holder model (handover, C-113 election, merge) | 008a-16 |
| 2. Recording (true error, bound, state, phase steps, rate, correction term, MTIE/TDEV) | 008a-09 |
| 2. Closed-loop scenario runner | 008a-15 |
| 3. Wired-jitter-profile section of `servo.md` | 008a-07 |

### Acceptance criteria
| Epic criterion | Tickets |
|---|---|
| A9 (servo, seqlock, pacer `no_std`, no allocator) | 008a-01, 008a-02, 008a-03, 008a-08, 008a-10 to 008a-14 |
| A13 (unit lock, continuity, C-60/C-61 clamps, C-119 ramp) | 008a-11 |
| A14 (unit holdover and lapse with C-162, epoch change, `begin_holding`/`rebase`) | 008a-12, 008a-13, 008a-14 |
| A15 (unit pacer within 2 %, C-112 range) | 008a-03 |
| A16 (models' analytic checks, smoke scenario, single-machine determinism) | 008a-04, 008a-05, 008a-06, 008a-08, 008a-09, 008a-15, 008a-16 |

### Definition of done
| Epic DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all tickets |
| Every parser touched is fuzzed | not applicable (C-125): no parser in this part; noted in the epic and in 008a-10 |
| Regime stamp fields touched are complete and documented | not applicable (C-125): no stamp field; noted in the epic |
| Design documents updated where behaviour differs, constants table first | 008a-07 (first `servo.md` section); the rest in 008b-18 |
| No new open item without an owner and a falsifier | 008a-07, 008a-12 |
| The frozen POC is not modified | all tickets |
| Every new factual claim carries an evidence tag | 008a-07, 008a-09 |
| Results table for C-60..C-67 in `servo.md` and constants table | not applicable (C-125): no result in this part; written by 008b-18 |
| Wired jitter profile written down with its source | 008a-07 |
| Seqlock priority invariant documented at the type | 008a-02 |

### Open sub-questions
| Question | Ticket |
|---|---|
| Holdover entry delay (configuration field here, default 1.5 s, values above the 1.5 s hard limit rejected; value proposed and recorded by 008b, C-195) | 008a-12 |
| Production worst-case asymmetry bound (configuration field here) | 008a-11 |
| Merge rule and handover/bump distinction | 008a-14, 008a-16 |
| Which wired jitter profile is representative (decided provisionally, C-94; L1 replaces it) | 008a-07 |

## Remaining gaps
1. **Holdover entry delay.** At most 1.5 s (C-37), a hard limit; the value within it is pending simulation (C-195). Affects 008a-12 (field `holdover_after_ns`, default 1.5 s, the C-37 limit; values above it rejected). Answer: 008b proposes the value by simulation (008b-05) and records it in the constants table with its results (008b-18); if no value at or below 1.5 s works, that is reported as a finding.
2. **Merge rule and handover/bump distinction.** The multi-holder merge event follows epic 006's rule, which this epic does not state. Affects 008a-14, 008a-16. Answer: epic 006.
3. **Production worst-case asymmetry bound.** `bound_ns` needs it; the constants table has no value. Affects 008a-11. Answer: a measurement, L1 (task 019), informed by 008b-14.
4. **Measured wired profile.** C-94 is synthetic and provisional. Affects 008a-07 and every case using `WIRED_PROFILE`. Answer: a measurement, L1 (task 019).
5. **E of the Linux reference sink.** C-70 (50 µs) is pending measurement, and C-62 and C-65 derive from it. Affects 008a-11, 008a-12. Answer: a measurement, L2 Linux.

## Settled since the split
1. `ServoConfig` fields unspecified → epic change: the field list is in deliverable 1, each tied to its C-id.
2. no_std target triple unnamed → epic change: any bare-metal target without an allocator proves A9; 008a-01 records it in the README.
3. Accepted request-rate range and "equals" vs "2 %" → C-112 (10 to 200 per second).
4. Wired jitter profile missing → C-94 (provisional; L1 replacement listed above as a measurement).
5. Location of wall-clock step detection → C-121 (platform backend); C-131 names the stamp source it concerns.
6. "Zero phase step" tolerance → epic change: 0 ns on the integer-ns published time.
7. State after the holdover lapse → C-162: `Unlocked`, time unusable, one-way figures and alignment claims stop (008a-12; the provisional rule and its design-owner question are removed).
8. Which E the servo uses for the lapse → C-149: the platform's target E (C-70), never a measured E (008a-01, 008a-12).
9. Seqlock invariant on devices → C-147: strict mode is hosted-only and a device has real-time scheduling by construction, so the invariant applies there too (008a-02).
10. Machine pair for determinism → C-151: x86-64 and 64-bit ARM (criterion A10, checked in 008b-17; 008a-04 provides single-machine determinism).
11. Holder stall → C-136 reuses the C-67 step threshold to bump the epoch; noted in 008a-13, modelled in 008b-20.
12. Oversized epic → C-143: epic 008 split into 008a and 008b.
13. Holdover entry delay owner and ceiling → C-195: pending simulation, proposed and recorded by 008b, at or below C-37; the default of `holdover_after_ns` moves from the POC's 2 s to 1.5 s and the design-owner step is removed (epic, 008a-01, 008a-12).
14. Lapse reference when E is unknown → C-169: 2 × the lock bound (50 µs); `edge_target_ns` is optional (epic, 008a-01, 008a-12).
15. Seqlock safety and pinning → C-177, C-201: default priorities (clock above data) never depend on pinning; a layout relying on a failed or unverified pin refuses to start in every mode, enforced by the wiring epics and documented at the type (epic, 008a-02).
16. Handover as a holder loss → C-212 with C-195: followers pass through about 2 s of holdover (C-37 + C-33) before takeover; `begin_holding` and `rebase` work from `Holdover`, and the multi-holder model scripts the takeover delay (epic, 008a-14, 008a-16). The eligibility question it exposed is settled by C-213 (item 19).
17. Publisher of the lock state and bound → C-187: this part, through the published view (epic Dependencies).
18. Harness presentation details → C-206: trace and summary formats and report file names are decided by this epic and documented with the harness (epic, 008a-04).
19. Holder eligibility in holdover (former remaining gap 6) → C-213: a holder-eligible module whose time is usable (Locked, or Holdover with the claim not lapsed, C-65, C-169) is a candidate and takes over with frequency continuity, epoch kept; C-113's bump applies only when no candidate has usable time. `is_eligible_holder()` follows the `is_usable()` rule, so 008a-14 now depends on 008a-12; the multi-holder takeover keeps the epoch (epic, 008a-13, 008a-14, 008a-16).
20. Holdover entry delay as a hard limit → C-195 revised: at most 1.5 s (C-37), not merely expected; a configuration above it is rejected, and if no value within it works that is reported as a finding (epic, 008a-01, 008a-12).
21. Handover run verdicts → C-212: frequency continuity at takeover is a rate change within C-61 and no phase step, and a run in which holdover was not observed is failed, not passed; the multi-holder model reports such a run failed (epic, 008a-16). Its L2 emission-error assertion (C-202) and the second holder-eligible Linux node concern the rig, not this part.
22. Several candidates with usable time → C-221: a Locked candidate is preferred over one in holdover; among equals the lowest module id wins. The multi-holder model chooses the successor this way and tests it (epic Requirements and deliverable 2, 008a-16 Context, Work 2 and new AC 3). No new ticket.
23. Epic header: the Blocks list now names every epic that depends on this part, 017b among them (also 014, 022a, 028b and 029).

Also applied earlier: C-119 (slew limits bound the correction term), C-113 (election without a Locked candidate, in 008a-14 and 008a-16), C-64 decided, C-78, C-10, C-100, C-01/C-02, C-127, C-132, C-130 and C-133. Reviewed with no effect on this part: C-104, C-105 and C-134 (no structural latency or resampler node in this epic), C-35, C-128 to C-131 beyond C-131's stamp sources, C-135, C-137 to C-142, C-144 (duties of epic 015 only), C-145, C-146 (all tests here are unprivileged), C-148, C-150, C-152 to C-161, C-163 to C-176, C-178 to C-186 (C-180 concerns certification, not the servo), C-188 (where the holder answers is epic 011/015 wiring; the simulation models no threads), C-189 to C-194, C-196 (no specification epic or later-phase dependency here), C-197 to C-200, C-202 to C-205 (C-202 is an L2 assertion; C-204 is holder-side, and the pacer already rejects rates above 200, C-112), C-207 to C-211, and the late revisions of C-08, C-134, C-141, C-153, C-181 and C-184.
