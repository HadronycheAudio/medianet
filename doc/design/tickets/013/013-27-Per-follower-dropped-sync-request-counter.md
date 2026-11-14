# 013-27 — Per-follower dropped sync request counter (new, C-204)
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-02, 013-03, 013-05, 013-26

## Context
- Sync requests above the accepted rate (C-204): **requests beyond 200 per second from one follower (C-112) are dropped before any work is done and counted per follower in the holder's stamp.** `[decision]` The accepted sync request rate is 10 to 200 per second (C-112).
- The holder already carries one entry per follower in the per-peer section of 013-26 (sync-request DSCP, holder only, C-152). The dropped-request count is a second holder-only, per-follower value; it goes in the same entry.
- A holder has at most 63 real followers (C-114, C-34), so at most 63 such values per stamp. If the section exceeds one control message (1 252 bytes, C-17) it is split by the rule of 013-13.
- Kind: the count moves during a run, so by the epic's definition it is a **counter**: it does not change the `StampId` (013-05) and starts no record set. `[decision]`
- Counter width is fixed with the byte layout (epic 004; remaining gap 2); this ticket proposes one and marks it "pending epic 004".

## Work
1. In `hadronyche-core`, add to the per-peer section of 013-26 a holder-only field: number of sync requests from that follower dropped for exceeding 200 per second (C-204, C-112). Absent in a non-holder stamp.
2. Extend the builder method of 013-26 (proposed `StampBuilder::peer_dscp(peer, hello, sync, sync_dropped: Option<u64>)`, or a separate holder-only setter) and the allocation-free reader on `Stamp`.
3. Extend the fuzz corpus (C-95 budget via the 013-03 harness).
4. Add the field to the per-peer field table in `doc/design/spec/stamp.md`, with its source on each of the four profiles (a holder's sync request handling; absent on a profile that cannot be holder).

## Acceptance criteria
1. Round-trip of 0, 1 and 63 follower entries in a holder stamp preserves the dropped count, including 0 and the maximum value of the chosen width.
2. A non-holder stamp carries no dropped-request value, and its reader reports it absent, distinct from 0.
3. Changing a follower's dropped count leaves `Stamp::id()` unchanged.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every stamp field has a stated source on each of the four profiles, or is marked absent there
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Design documents updated where behaviour differs, constants table first" (follows C-112, C-114, C-204 as written); "The stamp format is versioned independently of the media format" (uses 013-02 framing unchanged).

## Out of scope
- Enforcing the 200 per second limit and counting the dropped requests (the holder's sync request handling, outside this epic); this ticket carries the count.
- Splitting an oversize section (013-13); the publication schedule (013-24) and sending (epic 015, C-144).
