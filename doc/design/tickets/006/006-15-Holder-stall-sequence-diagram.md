# 006-15 — Holder stall sequence diagram
**Epic:** 006-Timeline-epoch-specification · **Effort:** XS · **Depends on:** 006-05, 006-06, 006-09
## Context
Epic 006 requires a mermaid sequence diagram for each of: partition merge; unlocked holder takeover; holder stall. This ticket draws the holder stall in the "Sequence diagrams" section of `doc/design/spec/timeline.md`.

Facts (`[decision]`):
- A holder that detects its own stall (for example its host was suspended) bumps the timeline epoch (16 bits, C-10). Holders are Linux hosts in v1.
- Stall detection (C-136): the holder compares its monotonic clock with a clock that keeps running through suspend; a divergence above the step threshold (2 ms, C-67) bumps the timeline epoch.
- Step threshold 2 ms (C-67): a larger step is never slewed. Election on reply silence 1.5 s (C-37); followers in holdover grow their bound at 1 µs per second (C-64) and their alignment claim lapses after about 25 s (C-65), after which they are Unlocked with time unusable (C-162).
- "Holdover" is a clock state defined by epic 005. The holdover entry delay is pending simulation (epic 008b) and at or below C-37, so followers are in holdover before any election starts (C-195). A candidate locked to H that takes the clock from holdover is a continuous handover, without a bump (C-212).
- Each module's clock thread learns the new epoch and line from sync replies and raises a timebase-discontinuity event (old epoch, new epoch, step size). Each source announces in every bundle member a switch at current position + 64 slices (C-43), re-issued every 16 slices (C-44), and switches to a new session (bumped, C-19) with the new epoch.

## Work
1. Add a mermaid `sequenceDiagram` with participants: holder H, follower (clock thread and one source), downstream consumer.
2. Show: H suspended → followers enter holdover → H resumes, compares its monotonic clock with its suspend-continuing clock and finds a divergence above 2 ms (C-136) → bumps epoch, new origin in sync replies → follower discontinuity event → announcement and switch.
3. Add an alternative (`alt`) branch for a stall longer than 1.5 s (C-37), where followers, already in holdover (C-195), start an election before H resumes; state the outcome (a locked candidate takes over from holdover without a bump, C-212; an unlocked one bumps, C-113).

## Acceptance criteria
1. The diagram renders in a mermaid renderer without error.
2. The diagram shows the stall detection by H itself, the bump, the discontinuity event and the announced switch.
3. The diagram has a branch for a stall longer than 1.5 s.
4. The stall detection is labelled with its rule and cites C-136 (divergence above 2 ms, C-67); no unsourced number.

## Definition of done
- [ ] The three sequence diagrams render
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- The other two diagrams (006-13, 006-14).
- Holdover servo design.
