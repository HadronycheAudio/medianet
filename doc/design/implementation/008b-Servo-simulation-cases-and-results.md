# 008b — Servo simulation cases and results

**Phase:** P2 · **Scope:** L (20 tickets, about 24.5 days) · **Depends on:** 008a (and through it 004, 006) · **Blocks:** 009, 011, 015, 018, 019, 020

## Goal

Run the named simulation cases on the harness built by 008a, and use the same harness to validate the fan-in offset estimator and ratchet. This is the simulation milestone of the roadmap and the second of the two parts of epic 008 (C-143): it either confirms the proposed clock constants C-60 to C-63 or replaces them with values it can defend, checks the decided C-64 and the derived C-65, tests the falsifiers of C-50 to C-54, and writes the servo specification. Everything about alignment margins and edge accuracy rests on this result. A reference elsewhere to "epic 008" means both parts.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's, over IPv4 only (C-100). Modules discover each other, agree on one clock holder and exchange *slices* addressed by *position* on a domain-wide timeline (position = network time counted in samples at the domain rate of 96 kHz, the only rate on the wire, C-01; one slice is 48 frames = 0.5 ms, C-02). Linux hosts are the backbone and the only holders in v1; macOS and Windows machines are edges; embedded devices are followers. Sources pace position from network time, so servo behaviour reaches hardware edges as resampler ratio; inside the network, alignment is by position and servo wobble only consumes deadline margin.

**Evidence tags.** `[code]` read in the POC · `[measured]` observed · `[kernel-doc]` documented · `[memory]` recalled · `[decision]` design choice · `[standard]` cited from a named, published standard with the clause (C-132).

**Where outputs live (C-130, C-133, C-171).** The specification goes to `doc/design/spec/servo.md` (its wired-jitter-profile section is written by 008a); simulation results go to `doc/design/reports/` (`008-simulation-results.md` and the tables under `008-simulation/`); the constants table is `doc/design/02-constants.md`; the architecture specification is `doc/design/01-architecture.md`. The simulation tests and the `sim-digests` binary live in the `hadronyche-tools` crate. Nothing is written into the frozen POC tree. File names and table layouts inside `008-simulation/` and the format of the `sim-digests` output are presentation details decided by this part and recorded in the `hadronyche-tools` README (C-206); only what other epics depend on (the C-60 to C-67 and C-50 to C-54 tables) goes into `servo.md`.

**Constants table.** Every number below comes from `doc/design/02-constants.md`; where this file and the table differ, the table wins.

**Relation to 008a.** 008a delivers, and this part uses: the servo (`Exchange`, `TimeView`, `LockState`, `Servo` with `feed`, `tick`, `view`, `begin_holding`, `rebase`, and `ServoConfig` holding C-60 to C-67, the target E, the holdover entry delay (C-195), the asymmetry bound and the loop gains as fields), the seqlock `PublishedView` (the servo core is the sole publisher of the lock state and bound, through the published time view, C-187), the deadline `RequestPacer`, and in the `sim` module of `hadronyche-tools` the seeded core, two-clock, link (with the C-94 preset `WIRED_PROFILE` and sensitivity presets), wall-clock and simulated-backend, and multi-holder models, the recorder (true error, bound, state, phase steps, rate, correction term and its rate of change, MTIE/TDEV) and the closed-loop scenario runner. Criterion labels A1 to A12 are those of the unsplit epic; A9 and A13 to A16 are verified in 008a; this part verifies A1 to A8, A10 to A12 and adds A17.

### How sync works `[decision]`

The holder's monotonic clock defines network time. A follower sends requests (50 per second by default, C-36; accepted range 10 to 200 per second, C-112) to the holder's sync port; the reply echoes the request's own send timestamp plus the holder's receive and send times, so each reply is a self-contained four-timestamp exchange. A stale or duplicate reply is a valid exchange with an honestly large round trip. Every reply carries the timeline epoch (16 bits, C-10).

With t1 = follower send, t2 = holder receive, t3 = holder send and t4 = follower receive, all in nanoseconds: round trip = (t4 − t1) − (t3 − t2); offset of the holder relative to the follower = ((t2 − t1) + (t3 − t4)) / 2. If the follower-to-holder delay exceeds the holder-to-follower delay by an asymmetry a, the offset estimate exceeds the true offset by exactly a / 2; this is the sign convention used by case 3.

**Wall-clock step detection lives in the platform backend (C-121).** The backend that builds each exchange from receive stamps detects a wall-clock step between kernel stamp and read and discards that sample; the check concerns the receive-timestamp source `kernel` (C-131). The simulation models the backend, including the check, so that case 9 can show the estimate is unaffected.

**Holder stall (C-136).** The holder compares its monotonic clock with a clock that keeps running through suspend; a divergence above the step threshold (2 ms, C-67) bumps the timeline epoch. Case 17 runs it, so a C-67 replacement also changes the stall threshold.

### The POC servo, which epic 008 replaces `[code]` (`clock.rs`)

- Estimator: per 200 ms window, the exchange with the smallest round trip; a 30 s minimum-delay history; weighting by delay above the floor.
- Gains: least-squares line fit settling at `alpha = 0.1`, `beta = 0.005`.
- **Actuation steps the phase:** `theta0 = pred + alpha·w·err` at every update.
- Lock: at least 8 windows and smoothed error below max(20 µs, path delay / 4) (P-04).
- Step: error above 2 ms resets the servo. Drift estimate clamped to ±500 ppm; no slew limit.
- Holdover: entered after 2 s without updates, never expires, no growing bound.
- Pacing by a 5 ms socket receive timeout: 36.1 exchanges/s at a configured 50 and 71.4 at a configured 1 000 on a `CONFIG_HZ=250` kernel `[measured]` (P-02, P-03).
- A wall-clock step between `CLOCK_REALTIME` stamp and read is clamped to look like zero delay.
- Handover: the new holder switches to its own crystal at once, a rate step of up to the crystal difference.

### Requirements on the new servo `[decision]`

| Requirement | Statement |
|---|---|
| Continuity | Published time is continuous while locked: zero phase step across updates, checked as 0 ns on the integer-nanosecond published time. Corrections act on frequency only |
| Slew | While locked, the servo's correction term is at most 10 ppm from the holder's rate (C-60) and changes by at most 1 ppm per second (C-61). These limits bound the correction term, not the drift estimate it tracks (C-119). Acquisition is exempt |
| Lock is a number | The published view carries an error bound. Locked requires bound ≤ 25 µs (C-62, which is E/2 for the Linux reference sink E = 50 µs, C-70) |
| Acquisition | Locked within 10 s of the first exchange (C-63). After an epoch change the estimator restarts, so re-lock is an acquisition and the same 10 s applies from the first exchange of the new epoch |
| Honest lock | Never reports Locked while true error exceeds its published bound |
| Pacing | Requests are paced by deadline; the achieved rate equals the configured rate within 2 % over the accepted range 10 to 200 per second (C-112), and is reported |
| Wall clock | A wall-clock step between kernel stamp and read is detected and the sample discarded, by the platform backend before the servo (C-121) |
| Holdover | In holdover the bound grows by 1 µs per second (C-64, decided); with a 25 µs lock bound against a 50 µs E, alignment claims lapse after about 25 s (C-65, derived from C-70, C-62 and C-64). Once the lapse is reached the clock is Unlocked, with time unusable: one-way figures and alignment claims stop (C-162). Holdover is entered after the holdover entry delay (C-195), at most the election silence threshold of 1.5 s (C-37) as a hard limit, the value within that limit pending this part's proposal, so a follower is already in holdover with a growing bound when it calls an election. When the target E is unknown, the lapse reference is twice the lock bound (2 × C-62, 50 µs at current values, C-169) |
| Steps | A step larger than 2 ms (C-67) is never slewed; it is a timeline epoch change. A holder stall above the same threshold bumps the epoch (C-136) |
| Handover | Frequency-continuous: the new holder starts at the old holder's rate and relaxes to its own crystal within C-61. Candidates are holder-eligible modules whose time is usable: Locked, or in holdover with the alignment claim not lapsed (C-65, C-169); such a candidate takes over with frequency continuity and the timeline epoch is kept (C-213). Among several candidates with usable time a Locked candidate is preferred over one in holdover, and among equals the lowest module id wins (C-221). Only when no candidate has usable time does the holder-eligible candidate with the lowest module id win and bump the timeline epoch (C-113) |
| Asymmetry | One-way figures carry the worst-case asymmetry bound; the estimate is biased by exactly half the path asymmetry |

**E is the target E.** Every threshold that uses E (the lapse, case 13) uses the platform's target E, C-70 for the Linux reference sink, never an E measured in the same run (C-149).

**C-60, C-61, C-62 and C-63 are proposals** (status pending-simulation). They have never been simulated. Frequency-only correction changes the loop dynamics that the POC's least-squares gains assume, so this is effectively a new servo. This part confirms the four values or replaces them, and updates the constants table first.

Inherited starting points, also to be confirmed by this part: measurement window 200 ms using the minimum-round-trip exchange (C-66); step threshold 2 ms (C-67). C-64 is decided and C-65 is derived: the simulation checks them (case 5) and reports a counter-example to the design owner rather than replacing them.

**Simulation profile `[decision]`.** The wired jitter profile used for AC-2 is C-94, provisional and synthetic: base one-way delay 100 µs, exponential jitter with 20 µs mean, 1 % of packets in bursts delayed by 1 ms. C-94 names no loss, duplication, reordering, asymmetry or interrupt coalescing, so the AC-2 preset has none; the cases that need them add them explicitly. L1 (task 019) replaces C-94 with the measured profile; its falsifier is a measured profile that changes a simulation verdict.

**Drift-ramp scenario (C-119).** The injected crystal drift sweeps ±100 ppm at no more than 0.5 ppm/s. L2 passes it if E is held throughout; the simulation runs it first (case 16).

**Determinism (C-151).** Cross-machine bit-exactness is checked on x86-64 and 64-bit ARM.

**Strict mode (C-147)** applies to runs whose numbers enter the constants table; it has no subject in this simulation, which runs in simulated time, independent of host scheduling (criterion A10), with no data or clock thread.

### Alignment estimator and ratchet, validated with the same harness `[decision]`

A fan-in consumer measures origin-to-arrival lateness per input during a 4 s aligning state (C-50). Offset = maximum observed lateness plus headroom (C-51; percentiles are reported, not used; the reported set is the 50th, 99th and 99.9th percentiles and the maximum, C-160), where headroom is the largest of 25 % of the observed maximum, one slice period (0.5 ms), and twice the published clock bound (C-52). The offset is a ratchet: it grows by one slice period (C-53) when an input is persistently late, after a dwell of 10 s (C-54). "Persistently late" means at least one late slice in each of 10 consecutive 1 s windows (C-101). The offset never shrinks automatically; its ceiling is the presentation group's audition offset A, which has no default and is set per presentation group by commissioning (C-55), so the simulation takes A as a scenario parameter. When the gap between offset and A is less than one slice period, the last step goes to A exactly (C-163). An input that is persistently late with the offset already at A enters the health state Demoted: treated as gapped, counted separately, cleared only by re-admission (C-109).

A slice absent at its deadline is counted missing; if it arrives later while its position is still within the ring span, it is reclassified as late (missing −1, late +1) and discarded (C-108). Rings are sized to A (C-96). The falsifier of C-50, C-51 and C-52 is a late slice after alignment on a clean network, so the target of case 14 is zero late slices.

C-50 to C-54 are decided, owned by simulation (then L1 for C-50 to C-52) for their falsifiers. This part reports for each one either "falsifier not triggered" or the counter-example with a proposed replacement for the design owner. The production estimator and ratchet are built by epic 009; this part builds reference models of them in the simulation crate.

## Deliverables

1. The simulation suite in the `sim` module of `hadronyche-tools`, one named test per case, on the 008a harness:
   1. convergence time and steady-state error under constant drift up to ±100 ppm, across jitter distributions;
   2. step response: a stepped holder causes an epoch change and re-lock within C-63 (10 s) from the first exchange of the new epoch;
   3. asymmetric delay produces a bias of exactly half the asymmetry;
   4. delay bursts do not pull the estimate;
   5. holdover: bound grows by 1 µs/s, true error stays inside it, and after the lapse the state is Unlocked with time unusable (C-162);
   6. handover with a 50 ppm crystal difference (C-78), the whole 50 ppm injected through the two-clock model (C-223 splits it into a measured natural difference plus an injected drift only on real hardware): no phase step, rate change within the limit; triggered as a holder loss, as the L2 run is (C-212): followers pass through holdover between holders, with true error within the bound and the bound within the target E during it (C-202), and a run with no observed holdover fails; the successor, in holdover with its claim not lapsed, takes over with frequency continuity and the epoch kept (C-213), and since every candidate is then in holdover it is the one with the lowest module id (C-221);
   7. stale, duplicate and reordered replies leave the estimate unchanged: published values bit-identical to a clean run whenever a window's minimum-round-trip exchange is the same packet, and true error within 1 µs of the clean run otherwise;
   8. achieved request rate equals the configured rate within 2 % over the whole accepted range, 10 to 200 per second (C-112);
   9. a wall-clock step during a run does not reach the estimate;
   10. honest lock, as a property test over randomised scenarios;
   11. zero phase step across updates while locked;
   12. maximum correction term and its rate of change in every scenario (C-119);
   13. wander: MTIE at most 10 µs for intervals up to 1 s and at most E (C-70) for any interval (C-77); TDEV reported;
   14. offset estimator: late-slice rate after alignment on a clean-network lateness model, target zero;
   15. ratchet: one flapping (bimodal) input, persistently late by C-101; last step to A (C-163), termination at the ceiling A and demotion (C-109);
   16. drift ramp: crystal drift sweeping ±100 ppm at no more than 0.5 ppm/s (C-119);
   17. holder stall: a stall above C-67 bumps the epoch and followers re-lock within C-63; a shorter stall does not bump (C-136).
   Plus two studies: handover without a good rate estimate, and interrupt-coalescing bias; and a `sim-digests` cross-machine determinism check on x86-64 and 64-bit ARM (C-151).
2. `doc/design/spec/servo.md` completed (specifications live in `doc/design/spec/`, C-130): the algorithm, its parameters, a results table giving for each of C-60 to C-67 either "confirmed" (or, for C-64 and C-65, "checked") or the replacement value with its justification, and a second table giving for each of C-50 to C-54 the outcome of its falsifier. Replaced values go into `doc/design/02-constants.md` first. The raw results behind both tables are in `doc/design/reports/` (C-130).

## Dependencies

- Needs from 008a: the servo, seqlock, pacer, all simulation models, the recorder and the scenario runner; and, through it, the epic 004 exchange fields and the epic 006 epoch, handover/bump distinction and merge rule.
- Provides to 009: the validated estimator and ratchet parameters (and, with 008a, the published view the fan-in deadline reads).
- Provides to 011: the answer on handover without a good rate estimate (tighter eligibility, if any).
- Provides to 015: the confirmed servo parameters behind the pacer and servo it wires to sockets.
- Provides to 018, 019, 020: the clock bound that the edge error budget and the ladder thresholds assume (published at run time by the 008a servo core through the time view, C-187).

## Acceptance criteria

1. **A1** (system AC-2): bound at most 25 µs (C-62) within 10 s (C-63) under the wired jitter profile C-94 and drift up to ±100 ppm — or the task replaces the numbers and says why.
2. **A2** (system AC-3): zero phase step across updates; correction term at most 10 ppm (C-60); its rate of change at most 1 ppm/s (C-61), in every scenario while locked.
3. **A3** (system AC-4): no counter-example to honest lock in the property test.
4. **A4** (system AC-5): handover at 50 ppm difference (C-78) stays within 1 ppm/s with no step; the handover is triggered as a holder loss, holdover between holders is observed at every follower, and the bound stays within the target E during it (C-202, C-212); the successor takes over from holdover with the timeline epoch kept (C-213), chosen by C-221.
5. **A5** (system AC-6): holdover bound grows by 1 µs/s (C-64); the state is never reported usable beyond the lapse (C-65), and is Unlocked after it (C-162).
6. **A6** (system AC-7): achieved rate within 2 % of configured over the whole accepted range, 10 to 200 per second (C-112).
7. **A7**: case 3 shows bias equal to half the injected asymmetry within 1 %.
8. **A8** (system AC-17, simulation part): the ratchet steps one slice after the C-101 test (10 consecutive 1 s windows each with a late slice), takes its last step to A exactly (C-163), stops at A, and demotes a flapping input to Demoted (C-109); no oscillation.
9. **A10**: the simulation is deterministic: same seed, same result, on x86-64 and 64-bit ARM (C-151).
10. **A11**: case 14 shows zero late slices after a 4 s alignment (C-50) on the clean-network model — or the task records the counter-example and a proposed replacement for the design owner.
11. **A12**: case 16 (drift ramp, C-119) stays Locked throughout with true error inside the bound and the bound within C-62.
12. **A17**: case 17 (holder stall, C-136) bumps the epoch exactly when the stall exceeds C-67, with no view mixing epochs, re-lock within C-63 and no honest-lock violation.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The results table for C-60 to C-67 is in `servo.md` and reflected in the constants table
- [ ] The wired jitter profile used for AC-2 is written down with its source (measured at L1 later, assumed now)
- [ ] The seqlock priority invariant is documented at the type

**Not applicable in this part** (C-125):
- "Every parser touched is fuzzed": this part touches no parser; exchanges reach the servo already built by the platform backend (C-121), so the fuzz budget C-95 has no subject here.
- "Regime stamp fields touched are complete and documented": this part adds and changes no regime stamp field (C-18); the simulation harness is dev-only and never stamped.
- "The seqlock priority invariant is documented at the type": the seqlock is written and documented in 008a (ticket 008a-02); this part does not touch it.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Are 10 ppm and 1 ppm/s on the correction term (C-119) compatible with a 25 µs bound and 10 s acquisition? | This part | No parameter set meets all four; the constants are replaced |
| At handover, can a successor with usable time still have a rate estimate poor enough to break C-61 at followers? Decided part (C-213): a candidate whose time is usable (Locked, or in holdover with its claim not lapsed, C-65, C-169) takes over with frequency continuity and the epoch kept, so after a holder loss the successor takes over from holdover; only when no candidate has usable time does the lowest-id holder-eligible candidate win and bump the epoch (C-113) | This part, then 011 for any tightened eligibility | A successor with usable time causes a correction-rate change above 1 ppm/s at a follower |
| Which wired jitter profile is representative? **Decided provisionally (C-94)**; L1 (task 019) replaces it with the measured profile | L1 (task 019) | L1 measurements differ enough to change a simulation verdict |
| Does interrupt coalescing bias survive the minimum-round-trip filter? | This part (model), L1 (measure) | Offset shifts when coalescing is toggled |
| Is 4 s of aligning enough for maximum-plus-headroom? | This part, then L1 | Late slices after alignment on a clean network |
| How long without a measurement before Locked enters Holdover? **Constant C-195**, at most the 1.5 s election silence threshold (C-37) as a hard limit; the value within it is pending simulation (POC: 2 s) | This part proposes the value from cases 4, 5 and 10; the design owner records it in C-195 | Bursts of the C-94 profile push a locked run into holdover at every value at or below C-37 (reported as a finding, never answered by a larger value), or case 10 finds a counter-example at the chosen value |
| What is the worst-case asymmetry bound in the production `ServoConfig`? The simulation uses scenario values | L1 (task 019, measurement) | Measured asymmetry exceeds the configured bound |
