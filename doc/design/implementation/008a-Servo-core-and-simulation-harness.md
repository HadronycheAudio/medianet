# 008a — Servo core and simulation harness

**Phase:** P2 · **Scope:** L (16 tickets, about 22 days) · **Depends on:** 004, 006 · **Blocks:** 008b, 009, 011, 014, 015, 017b, 018, 019, 020, 022a, 028b, 029

## Goal

Design and implement the redesigned clock servo, the seqlock publication and the deadline request pacer in `hadronyche-core`, and build the deterministic simulation harness that drives them: clock, link, wall-clock and multi-holder models, a trace recorder and a closed-loop scenario runner. This is the first of the two parts of epic 008 (C-143); part 008b runs the named simulation cases on this harness, confirms or replaces the clock constants C-60 to C-63 and writes the servo specification. A reference elsewhere to "epic 008" means both parts.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's, over IPv4 only (C-100). Modules discover each other, agree on one clock holder and exchange *slices* addressed by *position* on a domain-wide timeline (position = network time counted in samples at the domain rate of 96 kHz, the only rate on the wire, C-01; one slice is 48 frames = 0.5 ms, C-02). Linux hosts are the backbone and the only holders in v1; macOS and Windows machines are edges; embedded devices are followers. Sources pace position from network time, so servo behaviour reaches hardware edges as resampler ratio; inside the network, alignment is by position and servo wobble only consumes deadline margin.

**Evidence tags.** `[code]` read in the POC · `[measured]` observed · `[kernel-doc]` documented · `[memory]` recalled · `[decision]` design choice · `[standard]` cited from a named, published standard with the clause (C-132).

**Where outputs live (C-130, C-133).** The servo, seqlock and pacer live in `hadronyche-core`. The simulation harness and its binaries live in the `sim` module of the `hadronyche-tools` crate. This part creates `doc/design/spec/servo.md` with its wired-jitter-profile section; part 008b completes it and writes results to `doc/design/reports/`. The constants table is `doc/design/02-constants.md`. Nothing is written into the frozen POC tree.

**Constants table.** Every number below comes from `doc/design/02-constants.md`; where this file and the table differ, the table wins.

**Relation to 008b.** This part delivers deliverables 1 and 2 of epic 008 and the first section of `servo.md`. Part 008b depends on this part and runs every simulation case on it. Criterion labels A1 to A12 are those of the unsplit epic: A9 is verified here; A1 to A8, A10 (cross-machine), A11 and A12 are verified in 008b. This part adds A13 to A16 for the unit-level behaviour it delivers.

### How sync works `[decision]`

The holder's monotonic clock defines network time. A follower sends requests (50 per second by default, C-36; accepted range 10 to 200 per second, C-112) to the holder's sync port; the reply echoes the request's own send timestamp plus the holder's receive and send times, so each reply is a self-contained four-timestamp exchange. A stale or duplicate reply is a valid exchange with an honestly large round trip. Every reply carries the timeline epoch (16 bits, C-10).

With t1 = follower send, t2 = holder receive, t3 = holder send and t4 = follower receive, all in nanoseconds: round trip = (t4 − t1) − (t3 − t2); offset of the holder relative to the follower = ((t2 − t1) + (t3 − t4)) / 2. If the follower-to-holder delay exceeds the holder-to-follower delay by an asymmetry a, the offset estimate exceeds the true offset by exactly a / 2.

**Wall-clock step detection lives in the platform backend (C-121).** The backend that builds each exchange from receive stamps detects a wall-clock step between kernel stamp and read and discards that sample. The check concerns the receive-timestamp source `kernel` (a `CLOCK_REALTIME` stamp); the other sources of C-131 (userspace, mac-hardware, interrupt) are reported by the backend with their capability state; the servo only ever receives exchanges that passed this check. This part writes the reference detection function and models the backend, including the check, in the simulation.

**Holder stall (C-136).** The holder compares its monotonic clock with a clock that keeps running through suspend; a divergence above the step threshold (2 ms, C-67) bumps the timeline epoch. The servo sees this as an ordinary epoch change; part 008b models the stall (case 17).

### The POC servo, which this part replaces `[code]` (`clock.rs`)

- Estimator: within each 200 ms window, use only the exchange with the smallest round trip (`Servo::feed`); keep a 30 s history of minimum delay (`DMIN_HISTORY_NS`); weight a measurement by how far its delay is above the floor.
- Gains: start as a least-squares line fit over the measurements so far (`alpha = 2(2n−1)/(n(n+1))`, `beta = 6/(n(n+1))`) and settle at floors `alpha = 0.1`, `beta = 0.005`.
- **Actuation steps the phase:** `theta0 = pred + alpha·w·err` at every update, a step in published time of a fraction of the error every 200 ms.
- Lock: at least 8 windows (so 1.6 s minimum) and smoothed error below max(20 µs, path delay / 4) (P-04).
- Step: error above 2 ms on a wired network resets the servo.
- Drift estimate clamped to ±500 ppm; this is a clamp on the estimate, not a slew limit, and no slew limit exists.
- Holdover: entered after 2 s without updates, **never expires**, and stays "usable" with no growing bound.
- Pacing: at `poc-untouched` requests are paced by a 5 ms socket receive timeout (task 003 replaced it by a deadline-paced wait in `poc-baseline`, P-07); measured 36.1 exchanges/s at a configured 50 and 71.4 at a configured 1 000 on a `CONFIG_HZ=250` kernel `[measured]` (P-02, P-03).
- Receive time is derived from a `CLOCK_REALTIME` kernel stamp `[kernel-doc]`; a wall-clock step between stamp and read corrupts the sample and is clamped to look like zero delay.
- Handover: the new holder carries the phase on but switches to its own crystal, a rate step of up to the crystal difference.
- Publication: a seqlock (`SharedSync`) with one writer, read by the data threads.

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
| Holdover | In holdover the bound grows by 1 µs per second (C-64, decided); with a 25 µs lock bound against a 50 µs E, alignment claims lapse after about 25 s (C-65, derived from C-70, C-62 and C-64). When E is unknown (a sink without a commissioned E, C-103, C-140), the lapse reference is twice the lock bound (2 × C-62, 50 µs), so the lapse still comes about 25 s into holdover (C-169). Holdover is entered after a configurable delay without measurements, at most the election silence threshold of 1.5 s (C-37): a hard limit (C-195), so a follower is always in holdover with a growing bound when it calls an election. The value within that limit is pending simulation; if no value at or below 1.5 s works, that is reported as a finding (C-195). Once the lapse is reached the clock is Unlocked, with time unusable: one-way figures and alignment claims stop (C-162) |
| Steps | A step larger than 2 ms (C-67) is never slewed; it is a timeline epoch change |
| Handover | Frequency-continuous: the new holder starts at the old holder's rate and relaxes to its own crystal within C-61. A candidate is a holder-eligible module whose time is usable: Locked, or in Holdover with its alignment claim not lapsed (C-65, C-169); such a candidate takes over with frequency continuity and the timeline epoch is kept (C-213). Among several candidates with usable time a Locked candidate is preferred over one in holdover, and among equals the lowest module id wins (C-221). Only when no candidate has usable time does the holder-eligible candidate with the lowest module id win and bump the timeline epoch (C-113) |
| Asymmetry | One-way figures carry the worst-case asymmetry bound; the estimate is biased by exactly half the path asymmetry |

**E is the target E.** Wherever the servo uses E (the lapse, the case 13 threshold) it is the platform's target E, C-70 for the Linux reference sink, never an E measured in the same run (C-149). If the E is unknown, the lapse uses 2 × the lock bound instead (C-169).

**Handover through holdover (C-195, C-212).** A holder loss is silence: followers enter holdover, the election is called after 1.5 s of reply silence (C-37) and runs for 400 ms (C-33), so the new holder takes over about 2 s after the loss and every follower, the successor included, has passed through holdover. `begin_holding` and `rebase` therefore start from `Holdover` as well as from `Locked`, and the multi-holder model scripts handover with this delay. A successor in `Holdover` whose claim has not lapsed is eligible, so a holder loss is a frequency-continuous takeover with the epoch kept, not a C-113 bump (C-213). The takeover shows a rate change within C-61 and no phase step; a handover run in which holdover was not observed is failed, not passed (C-212).

**C-60, C-61, C-62 and C-63 are proposals** (status pending-simulation). Frequency-only correction changes the loop dynamics that the POC's least-squares gains assume, so this is effectively a new servo. Part 008b confirms the four values or replaces them; this part makes every one of them a configuration field so that it can. The inherited C-66 (200 ms window, minimum-round-trip exchange) and C-67 (2 ms step threshold) are configuration fields too.

**Simulation profile `[decision]`.** The wired jitter profile used for AC-2 is C-94, provisional and synthetic: base one-way delay 100 µs, exponential jitter with 20 µs mean, 1 % of packets in bursts delayed by 1 ms. C-94 names no loss, duplication, reordering, asymmetry or interrupt coalescing, so the preset has none; the cases that need them add them explicitly. L1 (task 019) replaces C-94 with the measured profile; its falsifier is a measured profile that changes a simulation verdict.

**Drift-ramp scenario (C-119).** The injected crystal drift sweeps ±100 ppm at no more than 0.5 ppm/s; the two-clock model must be able to produce it.

### Seqlock invariant `[decision]`

The servo publishes its view (offset at a reference time, drift, state, error bound, timeline epoch) through a seqlock. The writer must never be preempted mid-write by a higher-priority reader on the same core, or the reader spins forever. Strict mode grants real-time scheduling to the data and clock threads (C-127), which is the configuration where this invariant matters most on a hosted platform; a device has real-time scheduling present by construction (C-147). The writer is the clock thread and the readers are data threads; with the default priorities (clock above data) the invariant never depends on core pinning. A thread layout whose seqlock safety does depend on a CPU pin refuses to start in every mode, strict or not, if that pin fails (C-177) or is unverified (C-201); the refusal is enforced by the wiring epics (e.g. 015), and the type documents the rule.

## Deliverables

1. In `hadronyche-core`:

```rust
// shape, names may be refined
pub struct Exchange { pub t1: i64, pub t2: i64, pub t3: i64, pub t4: i64, pub epoch: TimelineEpoch }

pub struct TimeView {
    pub offset_ns: i64, pub at_local_ns: i64, pub rate_ppb: f64,
    pub state: LockState, pub bound_ns: i64, pub epoch: TimelineEpoch,
}
pub enum LockState { Unlocked, Acquiring, Locked, Holdover }

pub struct Servo { /* fixed size, no allocation */ }
impl Servo {
    pub fn new(cfg: ServoConfig) -> Self;
    pub fn feed(&mut self, ex: Exchange);
    pub fn tick(&mut self, now_local_ns: i64);
    pub fn view(&self) -> TimeView;
    pub fn begin_holding(&mut self, now_local_ns: i64);   // frequency-continuous takeover
    pub fn rebase(&mut self);                              // new holder, same time
}

pub struct PublishedView { /* seqlock */ }
impl PublishedView {
    pub fn write(&self, v: &TimeView);   // single writer
    pub fn read(&self) -> TimeView;      // wait-free in practice
}

pub struct RequestPacer { /* deadline-based */ }
impl RequestPacer { pub fn next_deadline(&mut self, now_ns: i64) -> i64; }
```

   `ServoConfig` holds every servo parameter as a field, never a literal: measurement window (C-66), slew limit (C-60), rate-of-change limit (C-61), lock bound (C-62), E used for the lapse (the target C-70, C-149; unknown E means 2 × the lock bound, C-169), holdover bound growth (C-64), step threshold (C-67), holdover entry delay (at most C-37, a hard limit, C-195; the value within it pending simulation; see below), worst-case asymmetry bound, and the loop gains chosen by case 1 in 008b. After the lapse the state is `Unlocked` and `is_usable()` is false (C-162). `rebase` is used by a follower whose holder changed without an epoch change: it keeps published time and rate and drops the measurement history tied to the old holder. `begin_holding` is used by the follower that becomes holder. The `no_std` check of criterion A9 uses one bare-metal target with no allocator, recorded in the crate README. A reference wall-clock step detection function (backend-side, C-121) is also delivered here, outside the servo module.

2. A simulation harness (hosted, dev-only) in the `sim` module of the `hadronyche-tools` crate, where harness and tool code lives (C-133), with:
   - a seeded, deterministic core (versioned PRNG, integer-nanosecond simulated time, scenario description, trace digest);
   - a two-clock model (holder and follower crystals with offset, drift, drift rate, thermal wander, and the C-119 ramp);
   - a link model (base delay, jitter distributions for a wired LAN, bursts, asymmetry, loss, duplication, reordering, interrupt-coalescing delay), with the C-94 preset `WIRED_PROFILE` and sensitivity presets;
   - a wall-clock model that can step, and a simulated platform backend that applies wall-clock step detection before building exchanges (C-121);
   - a multi-holder model for handover (holder loss with followers in holdover until takeover, C-195, C-212), usable-time holder eligibility (C-213), the choice among several candidates with usable time (Locked before holdover, then lowest module id, C-221), the C-113 election and merge;
   - recording of true error, published bound, state, phase steps, rate, the correction term and its rate of change, and MTIE/TDEV of published time;
   - a closed-loop scenario runner wiring pacer, links, clocks, backend, servo, seqlock and recorder.

   The harness's presentation details (trace and summary file formats, output lines, file names within `doc/design/reports/`) are decided here and documented with the harness (C-206); only what 008b's reports depend on is fixed in `servo.md`.

3. The first section of `doc/design/spec/servo.md`: the wired jitter profile (C-94) with its source and falsifier, and the sensitivity presets.

## Dependencies

- Needs from 004: the sync exchange fields. From 006: the timeline epoch, the handover/bump distinction and the merge rule used by the multi-holder model.
- Provides to 008b: the servo, seqlock, pacer, every simulation model, the recorder and the scenario runner.
- Provides to 011: `begin_holding` and the usable-time eligibility predicate (Locked, or Holdover with the claim not lapsed, C-213) (the answer on handover without a good rate estimate comes from 008b).
- Provides to 015: the pacer, the servo, and the reference wall-clock step detection for the platform backend (C-121) to wire to sockets.
- Provides to 009, 018, 019, 020 (with 008b): the published view and the clock bound. This part is the publisher of the clock lock state and bound (C-187); every epic that reports one-way figures depends on it.

## Acceptance criteria

1. **A9**: the servo, seqlock and pacer compile for a `no_std` target with no allocator.
2. **A13** (unit part of A1 and A2): with exact synthetic exchanges and drift of +50 ppm, the servo reaches Locked with bound at most 25 µs (C-62) within 10 s (C-63) of the first exchange; while Locked, every update has a 0 ns phase step, the correction term stays within 10 ppm (C-60) and changes by at most 1 ppm/s (C-61), and under a 0.5 ppm/s ramp the drift estimate follows while only the correction term is clamped (C-119).
3. **A14** (unit parts of A4 and A5): the holdover bound grows by 1 µs/s (C-64) and, past the lapse (C-65; with unknown E at 2 × the lock bound, C-169), the state is `Unlocked` with `is_usable()` false (C-162); an exchange with a new epoch resets the servo to Acquiring with no view mixing epochs; `begin_holding` and `rebase`, called from `Locked` or from `Holdover` (C-212), produce a 0 ns phase step and a rate change of at most 1 ppm/s; the eligibility predicate is true in `Locked` and in `Holdover` before the lapse, false otherwise (C-213).
4. **A15** (unit part of A6): the pacer's achieved rate is within 2 % of configured at 10, 50 and 200 per second under random wake-up lateness, rates outside 10 to 200 per second (C-112) are rejected, and the reported rate matches the issued deadlines.
5. **A16** (single-machine part of A10): every model passes its analytic checks; the closed-loop smoke scenario reaches Locked with 0 honest-lock violations; two runs with the same seed give identical trace digests on one machine.

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
- "Every parser touched is fuzzed": this part touches no parser. Sync messages are parsed by the wire specification work (004) and exchanges reach the servo already built by the platform backend (C-121), so the fuzz budget C-95 has no subject here.
- "Regime stamp fields touched are complete and documented": this part adds and changes no regime stamp field (C-18). The servo publishes only through the seqlock; the receive-timestamp source and its capability state (C-129, C-131) are stamped by the platform backend, outside this epic; and the simulation harness in `hadronyche-tools` is dev-only and never stamped.
- "The results table for C-60 to C-67 is in `servo.md` and reflected in the constants table": no simulation result is produced in this part; the table is written by 008b (ticket 008b-18).

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| How long without a measurement before Locked enters Holdover? At most the election silence threshold, 1.5 s (C-37), a hard limit (C-195); the value within it is pending simulation, and the POC's 2 s is above the limit. This part exposes it as `ServoConfig.holdover_after_ns`, default 1.5 s (the C-37 limit) until 008b proposes a value, and rejects a value above 1.5 s | 008b (C-195): proposes the value from cases 4, 5 and 10 and records it in the constants table with the results (008b-18) | Bursts of the C-94 profile push a locked run into holdover, or case 10 finds a counter-example at the chosen value; if no value at or below 1.5 s works, that is reported as a finding (C-195) |
| What is the worst-case asymmetry bound in the production `ServoConfig`? The simulation uses scenario values | L1 (task 019, measurement) | Measured asymmetry exceeds the configured bound |
| The merge rule and the handover/bump distinction used by the multi-holder model | Epic 006 | The multi-holder merge event disagrees with epic 006's rule |
| Which wired jitter profile is representative? **Decided provisionally (C-94)**; L1 (task 019) replaces it with the measured profile | L1 (task 019) | L1 measurements differ enough to change a simulation verdict |
