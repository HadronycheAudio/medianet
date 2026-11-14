# 006-14 — Unlocked holder takeover sequence diagram
**Epic:** 006-Timeline-epoch-specification · **Effort:** XS · **Depends on:** 006-05, 006-06, 006-09
## Context
Epic 006 requires a mermaid sequence diagram for each of: partition merge; unlocked holder takeover; holder stall. This ticket draws the unlocked holder takeover in the "Sequence diagrams" section of `doc/design/spec/timeline.md`.

Facts (`[decision]`):
- A new holder that was not locked when it took the clock (no locked candidate existed) bumps the timeline epoch (16 bits, C-10). When no candidate is locked, the holder-eligible candidate with the lowest module id wins and bumps the timeline epoch (C-113). Contrast: a locked handover is frequency-continuous and does not bump.
- Election on reply silence: 1.5 s (C-37); election window 400 ms (C-33). The holdover entry delay is pending simulation (epic 008b) and at or below C-37, so the follower is in holdover when the election runs (C-195).
- A candidate that was locked to H1 and takes the clock from holdover is a continuous handover, not an unlocked takeover; the L2 handover is exactly that case (C-212). The diagram's H2 and H3 were therefore never locked to H1 (for example both still in Locking).
- Each module's clock thread learns the new epoch and line from sync replies and raises a timebase-discontinuity event (old epoch, new epoch, step size). Each source then announces in every bundle member a switch at current position + 64 slices (C-43), re-issued every 16 slices (C-44), and switches to a new session (bumped, C-19) with the new epoch.

## Work
1. Add a mermaid `sequenceDiagram` with participants: old holder H1 (fails), two holder-eligible candidates H2 and H3, neither locked, H2 with the lower module id, a follower (clock thread and one source), and a downstream consumer.
2. Show: H1 replies stop → follower enters holdover after the holdover entry delay (pending, at or below 1.5 s, C-195) → 1.5 s silence (C-37) → election (400 ms, C-33) → no locked candidate, H2 wins by lowest module id (C-113) → H2 bumps the epoch and publishes a new origin in sync replies → follower discontinuity event → announcement (+64 slices, 4 transmissions) → switch and new session.
3. Add a note contrasting with a locked handover: a candidate locked to H1 taking over from holdover after the same silence (about 2 s, C-37 plus C-33, C-212) does not bump.

## Acceptance criteria
1. The diagram renders in a mermaid renderer without error.
2. The diagram shows the 1.5 s silence and the 400 ms election window.
3. The diagram shows the C-113 choice (lowest module id among unlocked holder-eligible candidates), that H2 was unlocked as the reason for the bump, the discontinuity event and the announced switch.

## Definition of done
- [ ] The three sequence diagrams render
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- The other two diagrams (006-13, 006-15).
- Election design (epic 005).
