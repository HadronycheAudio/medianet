# Tickets for epic 019 — L1 two-machine harness

## Summary
Epic 019 (phase P4, depends on 008a, 015, 016, 017; blocks 028; scope L) builds and runs L1, the first rung of the test ladder that leaves one machine: two Linux hosts on one wire through one switch, IPv4 only (C-100). It delivers crate `hadronyche-tools` (C-133) with binary `hadronyche-l1`, which orchestrates both hosts over SSH, deploys one build, starts reference modules from `hadronyche-reference`, runs named scenarios (`baseline`, `two-bundles`, `asymmetry`, `coalescing`, `multi-packet`, `blackhole`, `serving`, `stress`, `rcvbuf-shrink`, `flood`, `loss-soak`, `bad-input`), computes A for each run with the commissioning rule (C-148), collects both hosts' records and regime stamps (C-18, C-47, C-129, C-131) and produces one report, plus the results file `doc/design/reports/l1-results.md` (C-130). It measures, in strict-mode runs (C-147), C-93, C-92, C-90 (24 h soak at the two-bundle load, C-118), C-91 (10 × measured clean-wire loss, C-167), C-94, C-35, C-164 and coalescing bias, checks the falsifiers of C-14, C-16 and C-50 to C-52, writes them back to `doc/design/02-constants.md`, and decides whether v1's "detect and conceal, no redundancy" stance survives.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 019-01 | Crate and CLI skeleton | S | epic 015, epic 017 |
| 019-02 | SSH orchestration and deploy | M | 019-01, epic 015, epic 016, epic 017 |
| 019-03 | Regime stamp collection | S | 019-02, epic 015 |
| 019-04 | Record ingestion and report | M | 019-03, epic 008a, epic 015, epic 017 |
| 019-05 | Loopback dry-run in CI | S | 019-02, 019-03, 019-04 |
| 019-06 | Hardware pair and results file | XS | 019-03 |
| 019-07 | Baseline scenario and DSCP check | S | 019-04, 019-05, 019-06, 019-26, epics 015, 016, 017 |
| 019-08 | Two-bundles scenario | M | 019-07, epics 015, 016, 017 |
| 019-09 | Pacing spread and switch bursts | S | 019-08 |
| 019-10 | Receive cost measurement | S | 019-07, 019-08 |
| 019-11 | Coalescing scenario | S | 019-07 |
| 019-12 | Asymmetry reference selection | S | 019-06 |
| 019-13 | Asymmetry scenario | M | 019-07, 019-11, 019-12, epic 008a |
| 019-14 | Multi-packet scenario | S | 019-07 |
| 019-15 | Blackhole scenario | S | 019-06, 019-07 |
| 019-16 | Serving scenario | M | 019-04, 019-05, 019-06, 019-23 |
| 019-17 | Stress scenario | S | 019-07 |
| 019-18 | Receive buffer overflow scenario | S | 019-07 |
| 019-19 | Flood scenario | S | 019-07, 019-18 |
| 019-20 | Loss-soak scenario | M | 019-07, 019-08 |
| 019-21 | Loss alarm and redundancy verdict | S | 019-20 |
| 019-22 | Constants write-back | S | 019-06 to 019-21, 019-24 to 019-27 |
| 019-23 | Synthetic follower generator (new, C-114, C-120) | S | 019-02, epic 015, epic 017 |
| 019-24 | Wired jitter profile (new, C-94) | S | 019-07, 019-08 |
| 019-25 | Drain-bound check (new, C-16) | XS | 019-04, 019-08, 019-11, 019-17 |
| 019-26 | Ladder A computation (new, C-148) | S | 019-03, 019-04, 019-05 |
| 019-27 | Bad-input processing bound (new, C-164) | S | 019-07, 019-19, epic 015, epic 017 |

No ticket was deleted: no ruling removed work. The withdrawal of the rational-L rule (C-104, C-105 revised; C-134) touches nothing in this epic: no ticket handled structural latency, fractional delay or resampling.

## Effort tally
- XS: 2 (0.5 day)
- S: 19 (19 days)
- M: 6 (15 days)
- Total: 27 tickets, about 34.5 days → scope L (C-126); below the 40-day split threshold (C-143)

## Coverage

### Deliverables

| Epic deliverable | Tickets |
|---|---|
| Crate `hadronyche-tools`, binary `hadronyche-l1` (C-133) | 019-01 |
| Orchestration over SSH: deploy same build, start reference modules, collect records and stamps | 019-02, 019-03 |
| Produce one report | 019-04 |
| Dry-run mode on loopback, unprivileged | 019-05 |
| Scenario `baseline` (with DSCP check) | 019-07 |
| Scenario `two-bundles` (C-92), one or two bundles, spread on/off | 019-08, 019-09 |
| Scenario `asymmetry` | 019-12, 019-13 |
| Scenario `coalescing` | 019-11 |
| Scenario `multi-packet` | 019-14 |
| Scenario `blackhole` | 019-15 |
| Scenario `serving` (real up to 63, synthetic 64 and 65, C-114) | 019-16, 019-23 |
| Scenario `stress` | 019-17 |
| Scenario `rcvbuf-shrink` | 019-18 |
| Scenario `flood` (C-120 raw-packet hook) | 019-19 |
| Scenario `loss-soak` (C-118) | 019-20 |
| Scenario `bad-input` (C-164) | 019-27 |
| A computed per run with the commissioning rule (C-148) | 019-26 |
| Short-scenario durations 10 minutes (C-167) | 019-11, 019-13 to 019-19, 019-27 |
| Hardware record (managed and unmanaged switch, coalescing-capable NIC, strict-mode capability) | 019-06, 019-12 |
| `doc/design/reports/l1-results.md` (C-130) with measured value, hardware, stamp per constant | 019-06, 019-22 |
| C-93 receive cost | 019-10, 019-22 |
| C-92 two-bundle capacity | 019-08, 019-22 |
| C-90 loss floor | 019-20, 019-22 |
| C-91 loss alarm threshold | 019-21, 019-22 |
| C-94 wired jitter profile | 019-24, 019-22 |
| C-35 followers per holder | 019-16, 019-22 |
| C-14 egress spread falsifier (off via C-165 pacing override; C-155 phase check) | 019-09, 019-22 |
| C-16 bounded drain falsifier (cycle = slice period, C-156) | 019-25, 019-22 |
| C-164 bad-input bound | 019-27, 019-22 |
| Strict mode for table-bound runs (C-147) | 019-04, 019-06, 019-22 and each measuring ticket |
| C-50 to C-52 falsifier (late after alignment) | 019-07, 019-08 |
| Coalescing bias | 019-11, 019-22 |
| Provides: switch findings to 028, jitter profile to simulation, redundancy go/no-go, C-25 decision (C-93 falsifier) | 019-09, 019-15, 019-21, 019-22, 019-24 |
| Open sub-question: asymmetry external reference | 019-12 |
| Open sub-question: clean-wire loss near floor | 019-20, 019-21 |
| Open sub-question: deadlines at 64 000 packets/s | 019-08 |
| Open sub-question: unmanaged switch absorbs bursts | 019-09 |

### Acceptance criteria

| Epic acceptance criterion | Tickets |
|---|---|
| 1. AC-13 two bundles (C-92), third bundle refused | 019-08, A from 019-26 |
| 2. AC-14 zero late slices in `baseline` | 019-07, A from 019-26 |
| 3. AC-24 asymmetry bound vs external check | 019-12, 019-13 |
| 3. AC-24 aggregate loss at or below floor | 019-20 |
| 3. AC-24 multi-packet serialisation time (±5 µs, C-167) | 019-14 |
| 3. AC-24 coalescing bias measured and recorded | 019-11 |
| 4. AC-25 64 followers, 65th refused (C-167 definition and 20 % tolerance) | 019-16, 019-23 |
| 5. AC-19 (L1 part) blackhole / 1 280 bytes / path MTU in stamp | 019-15 (stamp field: 019-03) |
| 6. AC-34 DSCP 48/46/40 as received (sync and control per C-152) | 019-07 (stamp field: 019-03) |
| 7. AC-37 receive buffer shrink visible | 019-18 |
| 8. AC-11 (L1 part) 10× flood | 019-19 |
| 9. 019-A both hosts' complete stamps in every result file | 019-03, 019-04 |
| 10. 019-B redundancy: measured loss vs C-90 and recommendation | 019-21 |
| 11. 019-C jitter profile (C-94) | 019-24 |
| 12. 019-D bounded drain (C-16) | 019-25 (flood case: 019-19) |

### Definition of done

| Epic DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all code tickets, enforced by 019-05 |
| Every parser touched is fuzzed (C-95: 60 s per change, 1 h nightly, corpus kept) | 019-01, 019-03, 019-04, 019-13, 019-14, enforced by 019-05 |
| Regime stamp fields complete and documented (C-152, C-159, C-160, C-182, C-204, C-153) | 019-03 and each scenario ticket |
| Design documents updated, constants table first | 019-22 (and 019-03 for stamp docs) |
| No new open item without owner and falsifier | 019-08, 019-09, 019-12, 019-21, 019-22, 019-24 |
| Frozen POC not modified | all; specifically checked in 019-17 |
| Evidence tag on every new claim (including `[standard]`, C-132) | all; C-90 tag in 019-21, 019-22 |
| Named hardware pair recorded | 019-06 (+ 019-12 for reference hardware) |
| Each pending constant measured or reason given | 019-08, 019-09, 019-10, 019-11, 019-16, 019-20, 019-21, 019-24, 019-25, 019-27, 019-22 |
| Harness dry-run passes in CI on loopback | 019-02, 019-05, 019-26, and each scenario ticket |

Not applicable (C-125): none at epic level; every checklist line has a subject in this epic.

## Remaining gaps
1. **Runtime instrumentation**: data-thread time in batch receive (C-93), per-input drain counter per slice period (C-16, C-156), data-thread time per bad packet (C-164). Affects 019-10, 019-25, 019-27. Answer: epic 015.
2. **Stamp section layout, receiver report layout; record file format.** Content is settled (C-18, C-47, C-152, C-159, C-160); layout is not. Affects 019-03, 019-04, 019-07, 019-18. Answer: epic 004 (stamp, receiver report), epic 015 (record file).
3. **Whether a wider-than-default stream is accepted** for `multi-packet` and `blackhole`. Affects 019-14, 019-15. Answer: epic 015.

## Settled since the split
- Former gap 1, "the commissioned A is undefined": definition by C-55, C-53, C-96; value for L1 runs by C-148 (the harness computes A with the commissioning rule and records it; new ticket 019-26).
- Former gap 2, scenarios missing for AC-11, AC-34, AC-37, C-14: epic change (scenarios `rcvbuf-shrink`, `flood` listed; DSCP check in `baseline`; spread parameter on `two-bundles`).
- Former gap 3, "C-14 25 % of what": C-14 (25 % of the slice period, 125 µs, plus per-source phase offset); phase offset in [0, 75 %) by C-155; switching it off by the pacing override in the commissioning configuration, C-165.
- Former gap 4, "well above C-90" and the C-91 margin: margin settled by C-167 (C-91 = 10 × measured clean-wire loss); "well above" settled by C-179 (above 10 × C-90, about 580 losses in the soak, reopens the redundancy decision; 1 × to 10 × is investigated as a cabling or host problem); applied in 019-20 and 019-21, which no longer propose a rule.
- Former gap 5, loss-soak power and the 1 500-byte basis: C-118 (24 h at 64 000 packets/s, aggregate, about 58 expected losses) and C-90 (per default packet of about 1 320 bytes).
- Former gap 6, serving at 64 and 65 followers: C-114 and C-34 (synthetic followers beyond 63; ticket 019-23 via C-120). Serving lateness definition and "no growth" tolerance: C-167 (reply send minus request receive at the 99.9th percentile; within 20 % from 1 to 64). C-35's owner is L1 (task 019).
- Former gap 7, asymmetry bound value: epic change (each one-way figure is tested against its own published bound).
- Former gap 8, stamp schema: transport and content by C-18, C-47, C-129, C-131, C-159, C-160; where sync and control DSCP are published by C-152; layout remains (remaining gap 2).
- Former gap 9, location of `l1-results.md` and the constants table: C-130 (`doc/design/reports/`, `doc/design/02-constants.md`).
- Former gap 10, how to force a multi-packet slice: epic change (wider stream; C-04's 6 channels are "at defaults", C-05 regime stamped), with C-110 for the reassembly deadline. Tolerance: ±5 µs (C-167).
- Former gap 11, blackhole scope and switch types: epic change and C-116; PathMtuTooSmall only when the 1 280-byte probe also fails (C-173).
- Former gap 12, run durations: short scenarios run 10 minutes (C-167).
- Previous remaining gap 6, strict mode for measurement runs: C-147 (any run whose numbers enter the constants table runs in strict mode); enforced by 019-04's `eligible_for_constants` and 019-22.
- Former gap 14, criterion for tightening C-25: C-93 falsifier ("measured cost makes C-92 fail").
- Former gap 15, privilege versus unprivileged CI: epic change (privileged steps only on the hardware pair, skipped in dry-run), consistent with C-146.
- Former gap 16, hardware not named: epic change (hardware requirements listed; naming the pair is 019-06's work).
- Former gap 17, stress loads not enumerated: epic change (taken from `tools/stress.sh`'s usage text, script unmodified).
- Former gap 18, C-92 referenced "(below)": epic change (C-92 stated in the Background; AC-13 restates it).
- Previous remaining gap 7, switch to disable the egress spread: C-165 (pacing override in the commissioning configuration).
- Previous remaining gap 11, whether a default slice fits the 1 252-byte fallback: C-137 (1 152 bytes of audio plus at most 100 bytes of headers and checksum).
- Final sweep (C-177 to C-212): stamp lateness window (C-182: percentiles over a rolling 60 s window, maximum since live beside them) in 019-03 and 019-04, and A computed from the maximum since live in 019-26 and the epic; commissioning window (C-153, C-178) stamped in 019-03 and excluded from constants-eligible runs in 019-04; publisher of the clock lock state and bound (C-187) added as a dependency on epic 008a to the epic, 019-04 and 019-13; holder serving on the clock thread (C-188) and the per-follower over-rate drop counter (C-204) in 019-03 and 019-16 (new criterion: zero drops at every N); seqlock pin failure (C-177, C-201) in 019-06's host selection; controllers in their own member stream counted in the 32-member cap (C-08, C-181, C-205, C-209), so the L1 bundle sources carry none and 019-08's load stays 32 streams. No ticket added or deleted; tally unchanged.
- Still open, carried into remaining gaps: former gap 13 / previous remaining gap 8, instrumentation (now 1, extended with C-164); previous remaining gap 9 (now 2) and 10 (now 3). Previous remaining gap 1, "well above C-90": settled by C-179.

New rulings also folded into the tickets: C-01, C-04, C-100 (one wire rate, 192 bytes per channel, IPv4 only), C-95 (fuzz budget), C-101, C-108, C-109 (ratchet, late versus missing, Demoted), C-102, C-111 (health counters), C-19 (per-session accounting), C-117 (per-direction admission), C-120 (flood, synthetic followers and malformed packets through stamped test hooks), C-94 (ticket 019-24), C-16 (ticket 019-25), C-127 to C-133 (strict mode, capability states, timestamp source, output locations, `[standard]` tag, code location). In this refinement: C-135 (other-epoch counter), C-137, C-147, C-148 (new ticket 019-26), C-152, C-155, C-156, C-159, C-160, C-164 (new ticket 019-27), C-165, C-167, C-173 (refusal alarm, PathMtuTooSmall), and C-35's owner. Last pass: C-179 (redundancy reopening threshold, in 019-20 and 019-21); C-118 re-checked, already folded in (019-06, 019-20, 019-21, 019-22). C-104, C-105 (revised) and C-134 have no subject in this epic; C-144's duties belong to epic 015. The remaining new rules (C-136, C-138 to C-143, C-145, C-146, C-149 to C-151, C-154, C-157, C-158, C-161 to C-163, C-166, C-168 to C-172, C-174 to C-176) create no work here. Final sweep: C-08, C-153, C-177, C-178, C-181, C-182, C-187, C-188, C-201, C-204, C-205, C-209 applied (see "Settled since the split"); C-180, C-183 to C-186, C-189 to C-200, C-202, C-203, C-207, C-208, C-210 to C-212 and the revised C-134, C-141, C-169 have no subject here (no sink certification, resampling, transport state, live change of A or L2/L3 scenario in this epic; C-185's refusal reasons need no change to 019-08 or 019-15; the fuzzing tool choice of C-192 is already made in 019-01); by C-206 the report, results file and CLI layouts are this epic's own and none is left to the design owner.
