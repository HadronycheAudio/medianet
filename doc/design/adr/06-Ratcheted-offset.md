# ADR 06 — Ratcheted offset

## Status

Accepted

## Context

A fan-in consumer processes position p at `T(p) + offset` (ADR 05). The offset must be chosen, and it must survive graph changes: a module added on a branch, a connection re-routed or a hop leaving all change the critical path while audio is playing.

Three questions need an answer: how the offset is estimated, what happens when it turns out too small, and how a change is coordinated with everything downstream.

## Decision

**Estimation.**

- During the aligning state, 4 s (C-50), the consumer measures origin-to-arrival lateness per input.
- Offset = maximum observed lateness plus headroom (C-51).
- Headroom is the largest of: 25 % of the observed maximum, one slice period, twice the published clock bound (C-52).
- Percentiles are reported, not used.

**Ratchet.**

- The offset is frozen once live.
- It grows automatically by one slice period (C-53) when an input is persistently late, after a dwell of 10 s (C-54).
- It never shrinks automatically.
- Its ceiling is the audition offset A of the presentation group it feeds (C-55, ADR 12).
- Beyond the ceiling the offending input is demoted to gapped.
- A path that became shorter leaves the offset safe; the runtime reports reclaimable latency and does nothing.

**Coordination.**

- A re-alignment is an announced discontinuity. The triggering consumer places the announcement in its own output slices, naming a future position with a lead of 64 slices (C-43), re-issued every 16 slices (C-44).
- In-band propagation reaches exactly the consumer's descendants, which are exactly the affected set. A fan-in that receives two announcements takes the later position.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| 99.9th percentile over the aligning window | Over a short window the percentile rests on a handful of tail samples and misses rare events |
| Offset frozen until an operator acts | Complicated graphs mutate while playing; the operator cannot be the trigger |
| Re-align automatically on every topology change | A path that got shorter or stayed within headroom needs no change; needless discontinuities |
| Dwell-then-re-align without a ceiling | One flapping edge re-triggers after every dwell and glitches the whole subgraph each time |
| Automatic shrink when a path gets shorter | Every shrink is a discontinuity for no correctness gain; reclaim is an operator decision |
| Gossiped topology version as the trigger | Withdrawn: in-band announcements reach exactly the descendants, and the consumer's own lateness measurement is the truth |
| Per-position deceleration (slew the offset by resampling) | The runtime would resample content, which breaks the when/what boundary and bit-exactness; available only to a sink that already has a resampler |
| Lead of A plus a margin | In-band announcements are processed in position order, so the lead only has to survive loss; a flat lead suffices |

## Consequences

Positive

- Termination is guaranteed: growth is monotone and bounded by A.
- One coordinated discontinuity per re-alignment, with no global coordination protocol.
- The estimator does not depend on tail statistics it cannot have.
- A single bad edge cannot take the graph hostage: it is demoted.

Negative

- Latency ratchets up and stays up until the operator reclaims it.
- An event rarer than the aligning window can still be missed; headroom is the only protection.
- A demoted input is audibly absent rather than late.

Neutral

- The operator sets one number per presentation group (A); it bounds every ratchet upstream.

## Falsifier

- Simulation (task 008, alignment scenarios): the ratchet oscillates, or A is exhausted in normal operation (C-53, C-54).
- Simulation then L1 (task 019): late slices after alignment on a clean network with the maximum-plus-headroom estimator (C-50, C-51, C-52).
- A descendant misses an announcement despite four re-issues within the lead (C-43).

## Links

- [01-architecture.md §4.3 Offset: estimation and ratchet](../01-architecture.md#43-offset-estimation-and-ratchet), [§4.5 In-band announcements](../01-architecture.md#45-in-band-announcements), [§9.2 Presentation groups](../01-architecture.md#92-presentation-groups)
- [02-constants.md](../02-constants.md): C-43, C-44, C-50, C-51, C-52, C-53, C-54, C-55
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-14, AC-17
- Related: [05-Position-invariant-fan-in-alignment.md](05-Position-invariant-fan-in-alignment.md), [07-Control-plane-invariants.md](07-Control-plane-invariants.md), [12-Presentation-groups-with-required-E.md](12-Presentation-groups-with-required-E.md)
