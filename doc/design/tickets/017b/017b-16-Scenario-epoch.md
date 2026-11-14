# 017b-16 — Scenario `epoch`
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017a-08, 017b-09, 017b-06, epic 009

## Context
- AC-18: an input on another timeline epoch is **treated as a gap until it switches**.
- `position = (network_time − epoch_origin) × rate`; a different epoch means a different `epoch_origin`, so positions are not comparable.
- **How to place the input on another epoch (settled, C-120):** the test-only hook `force_epoch(stream, epoch)` (017b-06), stamped like any injection.
- **Interrupted count (settled, C-102):** a slice on another timeline epoch proves the upstream alive and resets the count, so the input never goes interrupted while it sends foreign-epoch slices.
- **Session number (settled, C-19):** the source bumps a stream's session number on timeline epoch change; every consumer sees the same value.
- Reference mixer on a gap `[decision]`: missing input contributes zero, output marked degraded.
- **Foreign-epoch slices and the dead threshold (settled, C-135):** a slice on another timeline epoch restarts the dead clock (C-111), does not count as a miss (C-102), and has its own per-connection counter (other-epoch, C-159). The foreign-epoch phase therefore runs longer than the 2 s dead threshold (C-39) to prove it.

## Work
1. Scenario `epoch`: force one mixer input's stream onto a different epoch for 3 s; then force it back to the mixer's epoch.
2. Assertions: while epochs differ, every slice from that input is treated as a gap (zero contribution, degraded output) and the input is never interrupted and never dead; the consumer's other-epoch counter equals the number of foreign-epoch slices sent; after the switch, contributions resume, output is non-degraded, and the session number seen by the mixer equals the one the sender reports.

## Acceptance criteria
1. `hadronyche-l0 epoch --duration 30` exits 0.
2. Zero samples from the foreign-epoch input are mixed while epochs differ; one degraded output slice per foreign-epoch input slice.
3. The input is never interrupted (C-102) and never dead (C-135) during the 3 s foreign-epoch phase.
4. The consumer's other-epoch counter equals the number of foreign-epoch slices sent, and those slices are not counted missing (C-135, C-159).
5. After the switch, degraded output slices stop; sender and mixer report the same session number (C-19).
6. The hook use appears in the stamp's injection record.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Holder election after partition heal (017b-20).
- The hook itself (017b-06).
