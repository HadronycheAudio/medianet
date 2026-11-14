# 006-10 — New epoch learned during Locking or holdover
**Epic:** 006-Timeline-epoch-specification · **Effort:** S · **Depends on:** 006-09, epic 005
## Context
Part of `doc/design/spec/timeline.md` (epic 006), section "Modules in Locking or holdover". Deliverable: the behaviour of a module that learns of a new epoch while it is itself in Locking or holdover. It answers the epic's open sub-question:

| Id | Question | Owner | Falsifier |
|---|---|---|---|
| Q-5 | What does a source do if it is in Locking or holdover when the epoch changes? | This ticket, with the clock state definitions from epic 005 | It announces a switch computed from a stale line |

Facts (`[decision]`):
- Each module's clock thread learns a new epoch and line from sync replies; its sources then announce a switch at current position + 64 slices (C-43), re-issued every 16 slices (C-44).
- Holdover bound growth: 1 µs per second (C-64). The alignment claim lapses in holdover after about 25 s, that is (C-70 − C-62) / C-64 (C-65); for a device whose E is unknown the reference is 2 × C-62 = 50 µs, which gives the same about 25 s (C-169). When the claim lapses the clock state is Unlocked, with time unusable: one-way figures and alignment claims stop (C-162). Step threshold: 2 ms (C-67); a larger step is never slewed.
- Election on reply silence: 1.5 s (C-37). The holdover entry delay is pending simulation (epic 008b) and expected at or below C-37, so a follower is already in holdover when it calls an election (C-195).
- The module's lock state and bound are published by the servo core (epic 008a) through the published time view (C-187); the section reads the state from that view (a cross-reference, C-196) and does not define it.
- A laptop edge that sleeps is a departure: both timebase loops are reset and the edge returns through normal admission, Locking then Aligning (C-124). A sleeping edge therefore never resumes in holdover on a stale line.
- "Locking" and "holdover" are clock states owned by the control state machines (epic 005). Cite epic 005's definitions; do not redefine them here. If epic 005 has not published them when this ticket runs, use the working reading "Locking: before the module is locked to the holder; holdover: running on the last known line without fresh replies", mark it as provisional, and keep Q-5 open with owner epic 005.

Arithmetic: at 1 µs/s (C-64), holdover uncertainty would reach the 2 ms step threshold (C-67) after 2 000 s, but the alignment claim lapses much earlier, after about 25 s (C-65), and the module is then Unlocked with time unusable (C-162).

## Work
1. Specify the behaviour in **Locking**: the module has no trusted line; state whether its sources may announce, and from which line, or must wait until locked on the new epoch. Include the edge returning from sleep (C-124).
2. Specify the behaviour in **holdover**, and after the claim lapses to Unlocked with time unusable (C-162), when a sync reply with a new epoch arrives (the reply itself ends holdover) and when the epoch change is learned indirectly (e.g. from an upstream slice header carrying a newer epoch); a switch position must never be computed from a stale line (the falsifier).
3. Specify how the source's switch position is computed once the new line is trusted, and the maximum delay between learning and announcing.
4. State the effect on downstream consumers while the source has not switched (its input is "on another epoch" and is treated as a gap at a fan-in; health accounting per 006-21).

## Acceptance criteria
1. The section specifies source behaviour for each of the 2 states (Locking, holdover) × 2 ways of learning (sync reply, upstream slice header).
2. No rule in the section computes a switch position from a line that is not the new epoch's line, or shows a bound proving the stale line error is below one slice (0.5 ms).
3. The holdover arithmetic (2 000 s to reach 2 ms at 1 µs/s), the 25 s alignment-claim lapse (C-65) and the Unlocked state after it (C-162) are stated.
4. The edge-sleep case cites C-124.
5. Q-5 is closed, or left open with owner and falsifier.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Defining the clock state machine itself (epic 005).
- The normal switch procedure (006-09).
