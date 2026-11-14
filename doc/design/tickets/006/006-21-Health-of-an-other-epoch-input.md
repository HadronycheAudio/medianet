# 006-21 — Health accounting of an input on another epoch
**Epic:** 006-Timeline-epoch-specification · **Effort:** XS · **Depends on:** 006-11
## Context
Part of `doc/design/spec/timeline.md` (epic 006), section "Health of an input on another epoch". The consumer-side procedures (006-11) say that an input still on another timeline epoch is treated as a gap at a fan-in until it switches. Rules settled after the ticket split fix how such an input interacts with the stream-health machinery; no other ticket of the epic states them, and the scenarios 006-16 and 006-18 depend on them.

Facts (`[decision]`):
- Interrupted threshold: 8 consecutive missed slices, 4 ms at defaults (C-38). Only slices absent at the deadline count as misses; a marked gap or a slice on another timeline epoch proves the upstream is alive and resets the count (C-102).
- Late versus missing: an absent slice at the deadline is counted missing; if it arrives later within the ring span it is reclassified late and discarded (C-108).
- Dead threshold: 2 s without data (C-39), counted from the last slice received with content or a marked gap (C-111). A slice on another timeline epoch restarts the dead clock (C-135).
- Degraded flag: an output at position `p` is degraded if its input at `p − L` was gapped (C-122).
- Third-party observability: producer and consumer each publish per-connection summary counters in their own regime stamp, one per gap class, plus interrupted events, dead, and the observed DSCP (C-47). A slice on another epoch has its own per-connection counter (C-135): the other-epoch counter in the list received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159).

The former open sub-question Q-8 (counter class and dead-threshold effect) is answered by C-135 and C-159.

## Work
1. Write the section with one rule per item: interrupted (C-102: reset, never a miss), late/missing (C-108: never counted missing or late, since it arrived in time), degraded flag of a consuming output (C-122), counters (C-47; the other-epoch counter, C-135, C-159), dead threshold (C-111; restarted by an other-epoch slice, C-135).
2. List Q-8 as closed by C-135 and C-159.
3. Give one worked example: an input on another epoch for 20 slices, then switching; list per slice the interrupted count, the counter incremented (other-epoch while on the old epoch, received after the switch), the dead clock restart, and the degraded flag of the fan-in output.

## Acceptance criteria
1. The section states that a slice on another epoch resets the interrupted count and is never counted as missing or late (C-102, C-108).
2. The degraded-flag rule for the consuming output cites C-122.
3. The section states that an other-epoch slice restarts the dead clock and increments the other-epoch counter, citing C-135 and C-159; Q-8 is not listed as open.
4. The worked example covers at least 20 slices, more than the 8-slice interrupted threshold, and shows no Interrupted event.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The fan-in combining rule itself (006-11).
- The regime stamp layout and counter encoding (epic 013, C-144).
- Health-state machines (epic 005).
