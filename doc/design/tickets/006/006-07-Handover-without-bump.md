# 006-07 — Handover without an epoch bump
**Epic:** 006-Timeline-epoch-specification · **Effort:** XS · **Depends on:** 006-05
## Context
Part of `doc/design/spec/timeline.md` (epic 006), section "Handover without a bump". Contributes to system criterion AC-5: a handover with a 50 ppm crystal difference does not bump the epoch (006-AC-7). The 50 ppm figure is the L2 handover test (C-78). It also gives epic 008 (simulation) the handover/bump distinction it must exercise.

Facts (`[decision]`):
- A normal handover is frequency-continuous: the new holder starts at the old holder's rate and relaxes to its own crystal at no more than 1 ppm per second (C-61, pending simulation in epic 008). Phase is carried on. The timeline epoch does not change.
- The timeline epoch is distinct from the election epoch: an election increments the latter only.
- Bumps happen only on: partition merge (losing side); a new holder that was **not locked** when it took the clock (C-113); holder stall (C-136); a step > 2 ms (C-67).
- The L2 handover is triggered as a holder loss: the old holder stops answering, the sink passes through about 2 s of holdover (C-37 plus C-33) and the new holder takes over; the run asserts emission error within the target E during holdover and frequency continuity at takeover (C-212, C-202). The holdover entry delay is at or below C-37 (pending simulation, epic 008b), so the candidate that takes over is itself in holdover when the election runs (C-195). A candidate that was locked to the old holder and takes the clock from holdover, before its claim lapses (C-65, C-162), therefore makes a continuous handover, not an unlocked takeover (C-113).
- The L2 measurement of this property uses a no-step tolerance at handover of ±1 sample on Linux and the platform restart spread elsewhere (C-166, proposal pending first runs); this section states the rule, L2 measures it.

Arithmetic to include: with a 50 ppm difference (C-78) and relaxation at ≤ 1 ppm/s (C-61), the new holder reaches its own crystal in ≥ 50 s. Because it starts at the old holder's rate and phase, the phase step at handover is zero by construction and stays below the 2 ms (C-67) step threshold, so no bump cause applies. During the about 2 s of holdover before takeover (C-212) the candidate's bound grows by about 2 µs at 1 µs per second (C-64), three orders of magnitude below 2 ms.

## Work
1. Write the handover rule: conditions for a continuous handover (the new holder was locked to the old one, and takes the clock from holdover after the old holder's loss, before its claim lapses: C-195, C-212, C-65, C-162), what is carried (rate and phase), relaxation slope C-61.
2. Show by the arithmetic above that a 50 ppm handover does not meet any of the 4 bump causes.
3. State the worst-case accumulated phase drift seen by followers during the 50 s relaxation and compare it with C-67 (2 ms); followers track it with the servo (slew), not a step.
4. Record C-61 as pending, with owner epic 008 (simulation) and its falsifier from the constants table ("lock bound unreachable under this limit", as for C-60).

## Acceptance criteria
1. The section states that a locked handover never changes the timeline epoch, with the 50 ppm example (C-78) triggered as a holder loss through about 2 s of holdover (C-212) and the ≥ 50 s relaxation figure (contributes to system AC-5).
2. The section lists, for the 50 ppm case, each of the 4 bump causes and why it does not apply.
3. C-61 appears as a pending item with owner and falsifier.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Simulating the handover (epic 008).
- Election and servo design.
