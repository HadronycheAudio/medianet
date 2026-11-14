# 006-06 — Timebase-discontinuity event
**Epic:** 006-Timeline-epoch-specification · **Effort:** XS · **Depends on:** 006-05
## Context
Part of `doc/design/spec/timeline.md` (epic 006), section "Discontinuity event". Continuity across a merge is **not** guaranteed. What is guaranteed is that the step is never slewed silently: every affected module gets a timebase-discontinuity event carrying the step size, and its streams restart with a new session at an announced position (`[decision]`). The session number is bumped on every timeline epoch change (C-19).

The event's Rust shape given by the epic (names may be refined):
```rust
pub struct TimelineEpoch(pub u16);
pub struct Discontinuity { pub from: TimelineEpoch, pub to: TimelineEpoch, pub step_ns: i64 }
```
Bump causes that raise it: partition merge (losing side), unlocked holder takeover (C-113), holder stall (detected by the holder as a divergence above 2 ms between its monotonic clock and a clock that runs through suspend, C-136), step > 2 ms (C-67).

It answers the epic's open sub-question:

| Id | Question | Owner | Falsifier |
|---|---|---|---|
| Q-4 | What is the sign convention of `step_ns`, and its value when the step cannot be measured (stall, unlocked takeover)? | This ticket | Two modules report different `step_ns` for the same bump |

## Work
1. Specify the payload: old epoch (`from`), new epoch (`to`), step size (`step_ns`, signed nanoseconds).
2. Define the sign convention of `step_ns` (e.g. new network time minus old network time at the same local instant) and how it is computed for each of the 4 causes, including the case where the step is unknown (e.g. after a stall); record the choice as `[decision]` (closes Q-4).
3. Specify who raises the event (the module's clock thread, on the first sync reply with a newer epoch), when (before any source announces), and who consumes it (the module's sources, and any observer/log).
4. Specify that the event is raised exactly once per epoch change per module.

## Acceptance criteria
1. The section lists the 3 payload fields with types matching the shape above.
2. The sign convention is stated and illustrated by one example with a positive and one with a negative step.
3. The section states the value of `step_ns` for each of the 4 bump causes, or the sentinel used when it is unknown (Q-4 closed).
4. The "exactly once per epoch change per module" rule is stated.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The Rust type itself (006-08).
- How sources react to the event (006-09).
