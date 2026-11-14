# 007b-04 — Holder stall and holdover-lapse scenario vectors
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** M · **Depends on:** 007b-02

## Context
- Scenario vectors are ordered lists of `(time, input) → expected (outputs, events, resulting state)`, stored in `vectors/control/*.json`, driven through `UnderTest::control_step` (007a-04). Every transition row must be exercised (epic criterion 007-AC3). The transition tables come from epic 005.
- Settled rules `[decision]`:
  - holder stall: the holder compares its monotonic clock with a clock that keeps running through suspend; a divergence above the step threshold of 2 ms (C-67) bumps the timeline epoch (C-136);
  - a timeline epoch change bumps the session number of every stream of the source (C-19);
  - after the holdover alignment claim lapses, the clock state is Unlocked with time unusable: one-way figures and alignment claims stop (C-162). The lapse comes about 25 s into holdover with the current figures: (C-70 − C-62) / C-64, with a holdover bound growth of 1 µs per second (C-64, C-65).
  - a device whose E is unknown (C-103, C-140) takes E as twice the lock bound (2 × C-62, that is 50 µs) for the lapse rule: it stops claiming alignment when the bound it currently publishes exceeds that value, about 25 s into holdover at the growth of C-64 (C-169);
  - a commissioned sink whose clock is unlocked stays commissioned; its alignment claim is suspended and does not count towards a group's required E until the clock is locked again, then resumes without re-commissioning (C-180);
  - the holdover entry delay is at most the election silence threshold of 1.5 s (C-37), a hard limit, so a follower is always in holdover with a growing bound when it calls an election; the value within that limit is pending simulation (epic 008b proposes it; the POC used 2 s), and if no value at or below 1.5 s works, epic 008b reports that as a finding (C-195);
  - until its claim lapses a follower in holdover still has usable time and may take over as holder with the timeline epoch kept; after the lapse it has not (C-213). The takeover scenarios are in 007b-02.
- C-62 (lock bound), C-70 (E of the Linux reference sink) and the value of the holdover entry delay within its 1.5 s limit (C-195) are still pending, so the lapse time is derived, not fixed. Each holdover scenario states the lock bound, E (or that E is unknown), growth and entry delay it uses and computes the lapse time with the C-65 formula, or with 2 × C-62 as the reference when E is unknown (C-169). No scenario uses the POC's 2 s entry delay, which exceeds C-37.
- `ControlInput` carries the reading of the clock that keeps running through suspend (007a-04).

## Work
1. Write scenarios in `vectors/control/holder-stall-*.json`:
   - divergence between the two clocks of exactly 2 ms: no epoch bump; divergence of 2 ms + 1 ns: the holder bumps the timeline epoch, and the session number of every stream it sources is bumped;
   - a run with no suspend (both clocks advance together): no epoch bump.
2. Write scenarios in `vectors/control/holdover-*.json`:
   - a follower enters holdover; at the lapse time computed from the scenario's stated figures minus 1 ns it is still in holdover with its alignment claim; at the lapse time it is Unlocked, with time unusable, and its one-way figures and alignment claims stop;
   - one scenario with the current figures (lapse at about 25 s, C-65);
   - a device with E unknown: the claim holds while the published bound is at or below 2 × C-62 and stops at the step where it first exceeds it (about 25 s at the current figures, C-169);
   - holdover entry: sync replies stop; at the stated entry delay (at most 1.5 s, C-195) the follower is in holdover with a bound growing at C-64; when the 1.5 s silence triggers the election (C-37) the follower is already in holdover; one entry scenario uses the limit itself, 1.5 s, and still expects holdover at the election call;
   - a commissioned sink lapses to Unlocked: its certification state stays commissioned, its alignment claim is suspended and it does not count towards the group's required E; when its clock locks again the claim resumes with no re-commissioning step (C-180).
3. Add one scenario per remaining holder-stall or holdover transition row in the tables.

## Acceptance criteria
1. The stall threshold is tested at 2 ms (no bump) and just above (bump).
2. The bump scenario expects a new timeline epoch and a new session number for every stream of the holder.
3. The holdover scenarios expect no state change 1 ns before the computed lapse and Unlocked with time unusable at it.
4. After the lapse, no expected output carries a one-way figure or an alignment claim.
5. Every holdover scenario states the lock bound, E (or E unknown), growth and entry delay it was computed with; no entry delay above 1.5 s is used (C-195).
6. All files carry `spec_version`.
7. The unknown-E scenario expects the claim to stop exactly when the published bound first exceeds 2 × C-62 (C-169).
8. Every entry scenario, including the one at the 1.5 s limit, expects holdover, with a growing bound, no later than the election call (C-195).
9. The relock scenario expects the sink to stay commissioned throughout, no alignment claim and no contribution to the group's required E while unlocked, and the claim back after relock with no re-commissioning (C-180).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Election (007b-02), holder follower cap (007b-03).
- Servo behaviour and the values of C-60 to C-63 and of the holdover entry delay (simulation, epic 008b, C-195).
- Wall-clock step detection in the platform backend (C-121).
