# 006-13 — Partition merge sequence diagram
**Epic:** 006-Timeline-epoch-specification · **Effort:** XS · **Depends on:** 006-05, 006-06, 006-09
## Context
Epic 006 requires a mermaid sequence diagram for each of: partition merge; unlocked holder takeover; holder stall. This ticket draws the partition merge, in the "Sequence diagrams" section of `doc/design/spec/timeline.md`. It contributes to system criterion AC-36 (006-AC-6): after a healed partition, one holder within 400 ms + 1 s = 1.4 s; the losing side receives a timeline epoch change and its streams restart at an announced position; no silent step.

Facts (`[decision]`):
- Election window 400 ms (C-33); hello period 1 s (C-30); election on reply silence 1.5 s (C-37).
- Partition merge bumps the timeline epoch (16 bits, C-10) for the losing side, because the two timebases have diverged.
- How the losing side is determined is owned by epic 005 (epic open sub-question Q-6). The diagram uses the rule as written in 006-05; if Q-6 is still open, the diagram labels the losing holder "loser per the epic 005 election rule" and does not invent a criterion.
- Each module's clock thread learns the new epoch and line from sync replies and raises a timebase-discontinuity event (old epoch, new epoch, step size). Each source then announces, in every bundle member, a switch at current position + 64 slices (C-43), re-issued every 16 slices (C-44), and at that position ends the old session and starts a new one with a bumped session number (C-19) and the new epoch.
- Continuity across a merge is not guaranteed; the step is never slewed silently.

## Work
1. Add a mermaid `sequenceDiagram` with participants: winning holder H1, losing holder H2, follower module on the losing side (clock thread and one source), and one downstream consumer.
2. Show: link heals → hellos (≤ 1 s, C-30) → election within 400 ms (C-33) → H2 stops serving → losing follower receives sync reply with new epoch and origin → discontinuity event → source announces switch at +64 slices, 4 transmissions → switch with bumped session → consumer switches.
3. Annotate the 1.4 s bound and the "no silent step" property.

## Acceptance criteria
1. The diagram renders in a mermaid renderer (e.g. GitHub preview or `mmdc`) without error.
2. The diagram shows the 400 ms and 1 s intervals and the 1.4 s total (system AC-36).
3. The diagram shows the discontinuity event, the announcement with its 64-slice lead and re-issues, and the new session (C-19).
4. The losing-side criterion shown is the one from 006-05 / epic 005, or is labelled as pending Q-6.

## Definition of done
- [ ] The three sequence diagrams render
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- The other two diagrams (006-14, 006-15).
- The election algorithm itself (epic 005).
