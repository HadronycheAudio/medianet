# 010-21 — Send counters for the regime stamp
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** XS · **Depends on:** 010-08, 010-14
## Context
- C-47: third-party observability of stream health. Producer and consumer each publish per-connection summary counters in their own regime stamp: one counter per gap class (received, late, missing, marked gap), plus interrupted events, dead, and the observed DSCP. Receiver reports are not multicast and there is no query message.
- C-159 completes the per-connection list: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead. All but the marked-gap class are observed by a receiver; on the send side the packetiser can supply only slices sent with content and marked gaps.
- Forward packets carry no per-receiver state, so the same bytes go to every consumer of a stream: what the packetiser can count is per stream, and the producer's per-connection summary takes these per-stream figures for each connection of that stream.
- The packetiser is the only place that knows how many slices it encoded with content and how many it marked as gaps (010-08, 010-17).
- The regime stamp message itself (C-18) is defined by epic 013 and sent by the hosted backend, epic 015 (C-144).

## Work
- In `hadronyche-core`, `no_std`, no allocation, add `SendCounters { content_slices: u64, gap_slices: u64 }` (names may be refined) to `Packetiser`, incremented by `finish` and `gap_slice` on success only, and a getter on `Packetiser` and on the bundle packetiser (per member).
- Counters are monotonic and wrap only at `u64::MAX`; no atomic or lock (the packetiser is single-owner).
- Document both fields for the epic 013 regime-stamp specification (in `doc/design/spec/`, C-130) as the producer-side source of the C-47/C-159 "marked gap" class and of the slices sent.

## Acceptance criteria
1. After 10 content slices and 3 gap slices on one stream, the counters read 10 and 3 (epic AC-12).
2. A failed `finish` or `gap_slice` (`OutTooSmall`) does not change the counters.
3. On a bundle packetiser, a gap slice on member 3 changes only member 3's counters.
4. Both fields are described in the regime-stamp specification.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] No allocation and no lock anywhere on the send path
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Consumer-side counters (received, late, missing, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, C-159), interrupted and dead (receive side).
- Observed DSCP (backend, epic 015).
- Publishing the regime stamp (C-18; epic 015, C-144).
