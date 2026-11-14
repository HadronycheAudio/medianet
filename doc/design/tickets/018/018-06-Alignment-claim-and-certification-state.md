# 018-06 — Alignment claim and certification state
**Epic:** 018-Linux-ALSA-edge · **Effort:** S · **Depends on:** 018-03, epic 008

## Context
- In holdover the clock bound grows by **1 µs/s** (C-64); the alignment claim **lapses after about 25 s** (C-65, derived as (C-70 − C-62) / C-64) and the sink must stop claiming presentation alignment `[decision]`.
- A sink that cannot bound E cannot claim presentation alignment; certification failures go into the regime stamp.
- Certification is two-tier: a sink self-monitors clock bound and rate tracking; the analogue part of D needs an external commissioning measurement. Certification states: *uncertified*, *self-monitored*, *commissioned*.
- A sink with no certified E declares E as unknown (C-103). Only *commissioned* gives a numeric E; *uncertified* and *self-monitored* declare E as unknown (C-140). **C-180:** a sink whose clock is unlocked stays *commissioned*; its alignment claim is suspended and does not count towards a group's required E until the clock is locked again, then resumes without re-commissioning. A lapse therefore changes the claim, not the certification state.
- **C-169:** the lapse rule needs a fixed reference, so a sink whose E is unknown (*uncertified* or *self-monitored*, C-140) takes E as twice the lock bound (2 × C-62 = 50 µs) and stops claiming alignment when the clock bound it currently publishes exceeds that value, about 25 s into holdover at the growth of C-64.
- After the holdover claim lapses the clock state is Unlocked with time unusable: one-way figures and alignment claims stop (C-162).
- Sleep is a departure (C-124): the claim lapses and both loops reset (018-21).
- A sink that cannot meet its audition offset A raises the connection alarm "cannot meet A" (C-173); its condition is C-174 (maximum lateness + D + headroom exceeds A in each of 10 consecutive 1 s windows), evaluated in 018-24.
- Shared by all edges, so it lives in `hadronyche-core` `edge/` (`no_std`, no allocation).

## Work
1. Add `AlignmentClaim` state in `edge/`, driven by the `TimeView` lock/holdover state (epic 008), by elapsed holdover time and, when E is unknown, by the published clock bound (C-169):
   - `Claimed` while locked and quality within budget;
   - `Lapsed { reason }` after 25 s (C-65) of holdover for a sink with a numeric E; for a sink whose E is unknown, when the published clock bound exceeds 50 µs (2 × C-62, C-169); when the clock is not locked; or on departure (C-124).
2. Add `CertificationState { Uncertified, SelfMonitored, Commissioned }`. Transitions, all in one documented function: `SelfMonitored` once locked and E statistics exist; `Commissioned` only when a commissioning-measured D component is loaded (from 018-11). Add `declares_numeric_e()`, true only for `Commissioned` (C-140). A lapse of the claim (clock not locked, holdover lapse) leaves the certification state unchanged: a `Commissioned` sink stays `Commissioned` with its claim `Lapsed` (suspended), and returns to `Claimed` on relock without re-commissioning (C-180).
3. Add an `Alarm` enum for "cannot meet A" and "claim lapsed", for the sink to raise (018-14).
4. Make the 25 s lapse, the 1 µs/s growth and the 50 µs unknown-E reference named constants referencing C-65, C-64 and C-169.

## Acceptance criteria
1. **Epic criterion 018-A3:** for a *commissioned* sink with a fake view entering holdover at t = 0, the claim is `Claimed` at t = 24.9 s and `Lapsed` at t = 25 s, with a named reason.
2. Returning to lock restores `Claimed` within one update.
3. `CertificationState` transitions match a table-driven test covering all three states; `declares_numeric_e()` is true for `Commissioned` only (C-140).
4. An unlocked view at start gives `Lapsed`, and `Uncertified` when no commissioning-measured D is loaded.
5. A departure event gives `Lapsed { reason: Departure }`.
6. **C-180:** a `Commissioned` sink whose view goes unlocked (including the 25 s holdover lapse) stays `Commissioned` with `declares_numeric_e()` true and its claim `Lapsed`; on relock the claim returns to `Claimed` and the state is still `Commissioned`, with no commissioning step in between.
7. **C-169:** a *self-monitored* sink (E unknown) whose fake view enters holdover with a published bound of 10 µs growing 1 µs/s still claims at a bound of 50 µs and is `Lapsed` once the bound exceeds 50 µs.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The `edge/` module builds `no_std` with no allocation
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Wiring alarms into the sink and runtime (018-14).
- Writing the state into the regime stamp (018-18).
- Discounting a suspended claim from a group's required E on the group side (C-180; admission and required E, epic 012).
- Evaluating the "cannot meet A" condition (018-24).
