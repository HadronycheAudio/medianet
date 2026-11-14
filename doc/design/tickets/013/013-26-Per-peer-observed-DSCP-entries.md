# 013-26 — Per-peer observed DSCP entries (new, C-152)
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-02, 013-03, 013-05, 013-07

## Context
- Rule 8: **DSCP marking is verified by the receiving peer**, which reads the received byte and reports it; no platform is trusted to self-report. Requested classes (C-13): sync CS6 (48), media and one-way probes EF (46), control CS5 (40). `[decision]`
- Peer verification of sync and control DSCP (C-152): **the holder reports in its stamp the DSCP observed on each follower's sync requests; every module reports the DSCP observed on each peer's hellos.** `[decision]` Media DSCP is already reported by each consumer in its per-connection summary (C-47, 013-23); this ticket covers the two classes C-152 adds.
- 013-AC-8 (AC-34, stamp part) includes these values: the holder's stamp shows the DSCP observed on each follower's sync requests, and every module's stamp the DSCP observed on each peer's hellos.
- A medianet has at most 64 active modules (C-34), so a stamp carries at most 63 peer entries, and a holder at most 63 real followers (C-114). The section is repeatable; if it exceeds one control message (1 252 bytes, C-17) it is split by the rule of 013-13.
- Where the sync value is recorded (C-188): the holder answers sync requests on the clock thread, never on housekeeping, and records the DSCP observed on each follower's requests there. The builder takes the value as delivered; it adds no work on the clock thread.
- Epic 014 reads the received DSCP byte and delivers the observed values into the builder (split settled in the epic). Where a platform cannot read the received byte, the value is "unverified", never 0.
- Kind: these entries are reports from peers that join and leave during a run, like the observed DSCP of the C-47 summary. By the epic's definition (reported values that move during a run) they are **counters**: they do not change the `StampId` (013-05), so a peer joining or leaving starts no record set. `[decision]`

## Work
1. In `hadronyche-core`, define a repeatable per-peer section flagged "counter" for 013-05: peer module id (64 bits, C-11), DSCP observed on that peer's hellos or "unverified", and, in a holder's stamp only, DSCP observed on that follower's sync requests or "unverified".
2. Implement a builder method (proposed `StampBuilder::peer_dscp(peer, hello: Option<u8>, sync: Option<u8>)`) and an allocation-free iterator reader on `Stamp`.
3. Extend the fuzz corpus (C-95 budget via the 013-03 harness).
4. Add the field table to `doc/design/spec/stamp.md` with the source of each field on each of the four profiles, or "unverified" where the received byte cannot be read.

## Acceptance criteria
1. Round-trip of 0, 1 and 63 peer entries preserves every field, with hello DSCP 40 and "unverified", and sync DSCP 48, "unverified" and absent (non-holder).
2. A non-holder stamp carries no sync-request value; a holder stamp carries one per follower entry.
3. An "unverified" value reads back as unverified, distinct from 0.
4. Adding, removing or changing a peer entry leaves `Stamp::id()` unchanged.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every stamp field has a stated source on each of the four profiles, or is marked absent there
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Design documents updated where behaviour differs, constants table first" (follows C-13, C-34, C-152, C-188 as written); "The stamp format is versioned independently of the media format" (uses 013-02 framing unchanged).

## Out of scope
- Reading the received DSCP byte on hellos and sync requests and feeding it into the builder (epic 014; on the holder's clock thread for sync requests, C-188).
- Splitting an oversize section (013-13); the publication schedule (013-24) and sending (epic 015, C-144).
- The media-class observed DSCP (013-23) and the module's requested classes (013-07).
