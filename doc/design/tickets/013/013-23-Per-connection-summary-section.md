# 013-23 — Per-connection summary section
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-02, 013-03, 013-05, 013-07

## Context
- Third-party observability of stream health (C-47): **producer and consumer each publish per-connection summary counters in their own regime stamp, one counter per gap class (received, late, missing, marked gap), plus interrupted events, dead, and the observed DSCP. Receiver reports are not multicast and there is no query message.** `[decision]` C-159 completes the list: **received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead**, plus the observed DSCP of C-47. The stamp is therefore the only place a third party (commissioning tool, monitor, collector) sees stream health.
- Counting rules the producers of these counters follow (this ticket only carries them):
  - late versus missing (C-108): at the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span it is reclassified as late (missing −1, late +1). **The missing counter can therefore decrease**; the encoding must not assume monotonic counters.
  - interrupted (C-38, C-102): 8 consecutive missed slices, where only slices absent at the deadline count; a marked gap or a slice on another timeline epoch resets the count.
  - dead (C-39, C-111): 2 s without a slice with content or a marked gap.
  - other-epoch (C-135): a slice on another timeline epoch has its own counter, restarts the dead clock and is not a miss.
  - demoted rounds (C-109, C-159): rounds an input spends Demoted at the ratchet ceiling; this is how Demoted inputs are "counted separately".
  - too-early, duplicates, incomplete reassemblies (C-159): counted by the reception epics that define them.
- The summaries are **counters**, not regime fields: they are refreshed with the stamp every 10 s (C-18) and **do not change the `StampId`** (013-05).
- A connection is named by its stream (source module id 64 bits, stream number 16 bits, session 32 bits; C-10, C-11, C-19, the session being the same for every consumer) and its peer module id, so a third party can pair the producer's and the consumer's entries. The `ConnId` layout is owned by epic 004; use the placeholder of 013-07 until then.
- Shape from the epic (widths fixed with epic 004's layout):
  ```rust
  pub struct ConnSummary { pub received: u64, pub late: u64, pub missing: u64, pub marked_gap: u64,
                           pub other_epoch: u64, pub too_early: u64, pub duplicates: u64,
                           pub incomplete_reassemblies: u64, pub demoted_rounds: u32,
                           pub interrupted_events: u32, pub dead: bool, pub observed_dscp: Option<u8> }
  impl<'b> StampBuilder<'b> { pub fn connection_summary(&mut self, conn: ConnId, s: &ConnSummary); }
  ```
- `observed_dscp` is the DSCP byte the consumer read on that connection's packets; the producer side writes none.
- No constant bounds the number of connections in one stamp (C-09, C-34); a summary section larger than one control message (1 252 bytes, C-17) is split by the rule of 013-13. The epic records this falsifier for C-18: the summaries of a 32-member bundle source (C-08) exceed one section.

## Work
1. In `hadronyche-core`, define `ConnSummary` with the twelve fields above (C-47, C-159), a repeatable summary section flagged "counter" for 013-05, with a role (producer or consumer) per entry.
2. Implement `StampBuilder::connection_summary` and an allocation-free iterator reader on `Stamp`.
3. Extend the fuzz corpus (C-95 budget via the 013-03 harness).
4. Add the field table to `doc/design/spec/stamp.md` with units, the counting rules above with their C-ids, and the source of each field on each of the four profiles.

## Acceptance criteria
1. Round-trip of 0, 1, 32 and 64 summaries preserves every one of the twelve fields, including counters at 0 and at their maximum value and `observed_dscp` both absent and 46.
2. A summary whose missing counter is lower than in the previous refresh (a C-108 reclassification) parses and round-trips without error.
3. Changing any summary field leaves `Stamp::id()` unchanged.
4. A producer stamp and a consumer stamp for the same stream and session pair up by connection identity in a test.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every stamp field has a stated source on each of the four profiles, or is marked absent there
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Design documents updated where behaviour differs, constants table first" (follows C-47, C-108, C-109, C-135, C-159 as written); "The stamp format is versioned independently of the media format" (uses 013-02 framing unchanged).

## Out of scope
- Counting any of the summary values in the data plane (the epics that own reception); this ticket carries the values.
- Splitting an oversize section (013-13); the publication schedule (013-24) and sending (epic 015, C-144); reading summaries in the collector (013-17).
