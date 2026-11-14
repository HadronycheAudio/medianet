# 012-23 — Per-connection stamp record
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** 012-14, 012-15, epic 004, epic 009

## Context
Epic 012 owns the connection table, so it supplies what the regime stamp says about each connection `[decision]`:
- Regime stamp transport (C-18): a dedicated control message split into sections that each fit 1 252 bytes (C-17), multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch. The hello carries only a stamp version number. The section layout is epic 004's.
- The path MTU of each connection goes in the stamp (epic 012 probe rule).
- Third-party observability (C-47, completed by C-159): producer and consumer each publish per-connection counters in their own regime stamp: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead; plus the observed DSCP (C-47). A slice on another timeline epoch has its own counter (C-135). Receiver reports are not multicast and there is no query message.
- Sending the stamp message is the hosted backend's duty (epic 015, C-144); this ticket only supplies the records.
- Late versus missing (C-108): an absent slice counts missing at the deadline and is reclassified late (missing −1, late +1) if it arrives while still in the ring span. Epic 009 counts; this ticket stores what it is given.
- Media and probes use DSCP EF (46) (C-13); the observed value is reported, not checked here.
- Commissioning window (C-153, C-178): while a window is open, ordinary production figures in stamps (loss among them) are unqualified. The window is stamped by the module while open, so the record carries no flag of its own and the counters are neither reset nor suppressed.

## Work
- `pub struct GapCounters { received, late, missing, marked_gap, other_epoch, too_early, duplicates, incomplete_reassemblies: u64, demoted_rounds: u32, dscp: u8 }` (C-159) and `pub struct ConnStampRecord { conn, path_mtu, state, counters, interrupted_events: u32, dead: bool }` (names may be refined).
- `Admission::on_counters(conn, GapCounters)` stores the latest counters from epic 009 for a non-terminal slot.
- The engine increments `interrupted_events` on every Live → Interrupted and sets `dead` on → Dead, on both producer and consumer slots.
- `Admission::stamp_records(&self) -> impl Iterator<Item = ConnStampRecord>` over admitted slots (Admitted to Dead/Ended), no allocation; a terminal slot stays listed until reused, so the last values are published.
- Document each field (name, unit, meaning, source) next to the type; the size of one record lets epic 004 size its sections.

## Acceptance criteria
1. 012-A13: an admitted connection's record shows its path MTU (1 280 after a failed probe), every C-159 counter as last given, `interrupted_events`, `dead` and the observed DSCP, on both sides.
2. Three Live → Interrupted → Live cycles give `interrupted_events == 3`; → Dead sets `dead`.
3. A late reclassification given by epic 009 (missing −1, late +1) is stored as given, without re-counting.
4. Refused and Pending slots produce no record.
5. No allocation; builds in the `no_std` job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser; the stamp codec is epic 004's
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: behaviour follows C-18 and C-47
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable: counters never drive a transition
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Counting gap classes, other-epoch, too-early, duplicate, incomplete-reassembly and demoted-round events (epic 009). Encoding the stamp (epic 004) and sending it (epic 015, C-144).
