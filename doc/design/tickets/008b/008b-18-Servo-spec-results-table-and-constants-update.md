# 008b-18 — Servo spec, results table and constants update
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** S · **Depends on:** 008a-07, 008b-01, 008b-02, 008b-05, 008b-07, 008b-12, 008b-13, 008b-14, 008b-15, 008b-16, 008b-19, 008b-20
## Context
Deliverable 2 of 008b (deliverable 4 of the unsplit epic 008): `doc/design/spec/servo.md` — the algorithm, its parameters, a results table giving for each of C-60 to C-67 either "confirmed" (for C-64 and C-65, "checked") or the replacement value with its justification, and a second table for C-50 to C-54. Specifications live in `doc/design/spec/`, results in `doc/design/reports/`, and the constants table is `doc/design/02-constants.md` (C-130). The design documents must be updated where behaviour differs, constants table first. 008a-07 already created `servo.md` with the wired jitter profile section (C-94).

Constants under test `[decision]`:
| Id | Value | Status in the constants table |
|---|---|---|
| C-60 | correction term ≤ 10 ppm from holder rate while Locked (bounds the correction, not the drift estimate, C-119) | pending-simulation |
| C-61 | correction term changes ≤ 1 ppm/s while Locked (C-119) | pending-simulation |
| C-62 | Locked requires bound ≤ 25 µs (E/2 against C-70, E = 50 µs) | pending-simulation |
| C-63 | Locked within 10 s of first exchange | pending-simulation |
| C-64 | holdover bound growth 1 µs/s | decided: checked, not replaced |
| C-65 | alignment claims lapse ≈ 25 s into holdover | derived |
| C-66 | measurement window 200 ms, minimum-round-trip exchange | inherited, owner simulation |
| C-67 | step threshold 2 ms (larger steps are an epoch change) | inherited, owner simulation |

Also validated (provided to epic 009), all decided with simulation as falsification owner: C-50 4 s aligning, C-51 offset = max lateness + headroom, C-52 headroom = max(25 % of max, 0.5 ms, 2 × bound), C-53 ratchet step one slice (0.5 ms), C-54 dwell 10 s, tested through C-101 and C-109. For these the outcome is "falsifier not triggered" or a counter-example with a proposed replacement for the design owner.

Results come from `doc/design/reports/008-simulation-results.md` and `doc/design/reports/008-simulation/` (008b-01 C-60..C-63, C-66; 008b-02 C-67; 008b-05 C-64, C-65 and the proposed C-195 holdover entry delay; 008b-07, 008b-14, 008b-15 open questions; 008b-12, 008b-13 tables; 008b-15, 008b-16 C-50..C-54 with the C-163 last step; 008b-19 drift ramp; 008b-20 holder stall, C-136 with the C-67 threshold).

## Work
1. Complete `doc/design/spec/servo.md`: algorithm (window/min-RTT, frequency-only loop with drift estimate and clamped correction term, bound formula, states and transitions, holdover entry after the C-195 delay and lapse with the C-162 state after the lapse and the C-169 reference when E is unknown, step/epoch rule and the C-136 holder stall rule, `begin_holding`/`rebase`, pacer with the C-112 range, wall-clock detection as a backend duty per C-121, seqlock and its priority invariant); every `ServoConfig` parameter with value and unit; the C-60..C-67 results table; the C-50..C-54 table; answers or current status of every open sub-question of the epic with owner and falsifier.
2. Update `doc/design/02-constants.md` first with any replaced pending-simulation or inherited value, and recompute what derives from it: C-65 from C-62, and the figures C-169 quotes for 2 × C-62 (propose those to the design owner, as C-169 is a settled rule); propose to the design owner (do not edit) any change to a decided constant (C-50 to C-54, C-64) and the proposed value of the holdover entry delay C-195 (at or below C-37, a hard limit; if no such value works, the counter-example as a finding, never a larger value); then update any other design document whose behaviour differs, including the architecture specification `doc/design/01-architecture.md` (C-171).
3. Every factual claim carries an evidence tag; simulation results tagged `[measured]` with the case name; standard definitions (MTIE/TDEV) tagged `[standard]` with the clause (C-132).

## Acceptance criteria
1. `servo.md` has a table with exactly one row for each of C-60, C-61, C-62, C-63, C-64, C-65, C-66, C-67, each "confirmed"/"checked" or a replacement value plus justification and the case that supports it.
2. `servo.md` has a table with one row for each of C-50 to C-54 giving the falsifier outcome and the supporting case.
3. Every value in the C-60..C-67 table equals the corresponding entry in `doc/design/02-constants.md` (checked by review).
4. Every open sub-question of the epic has an answer or a current status, an owner and a falsifier in `servo.md`.
5. `ServoConfig` defaults in code equal the values in `servo.md`.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] The results table for C-60 to C-67 is in `servo.md` and reflected in the constants table
- [ ] The wired jitter profile used for AC-2 is written down with its source (measured at L1 later, assumed now)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

Not applicable (C-125): "Regime stamp fields touched are complete and documented" — epic 008 adds and changes no regime stamp field; the servo publishes through the seqlock only.

Strict mode (C-147) has no subject here: simulation results are computed in simulated time, are independent of host scheduling (criterion A10), and come from no data or clock thread.

## Out of scope
- Running the simulations (case tickets); L1 measurement (task 019).
