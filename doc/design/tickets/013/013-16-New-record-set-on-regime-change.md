# 013-16 — New record set on regime change
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-05, 013-15, 013-17

## Context
- Rule 1: **a change of regime mid-run starts a new record set**; figures are never averaged across regimes. A timeline epoch change is a regime change. `[decision]`
- The record-set writer in `hadronyche-std` **starts a new record set whenever the local or the peer stamp id changes**.
- The `StampId` covers regime fields only; a change of counters (Drops, per-connection summaries) leaves it unchanged and starts no record set (013-05, epic 013 settled from C-18 and C-47).
- 013-AC-4: changing any regime field (for example toggling an injection, losing real-time scheduling, a timeline epoch change, a path-MTU fallback) changes the `StampId` and starts a new record set **before any record carrying the new local id, and within 10 s of a peer's change**. The 10 s bound comes from C-18: a peer multicasts its stamp on any regime change and refreshes it every 10 s, so a lost change notice is repaired by the next refresh.
- Peer stamps arrive through the collector (013-17).
- `[measured]` In the POC a run where real-time scheduling silently failed looked identical to a good one.

## Work
1. In the record-set writer, track the current local `StampId` and, per peer, the current peer `StampId` (from the collector).
2. When either changes, close the current record set and open a new one headed by the new stamp, before writing any record carrying the new id.
3. Ensure aggregates (if the writer computes any) are reset at the boundary so nothing averages across regimes.

## Acceptance criteria
1. Integration test for each of the 4 named changes (injection toggled, `rt_scheduling` Present → Absent, timeline epoch change, path MTU lowered): the output contains exactly 2 record sets, with different stamp ids; for a local change no record with the new id precedes the boundary.
2. A peer stamp change alone (local unchanged) starts a new record set within 10 s, including when the peer's change multicast is dropped and only the next refresh arrives.
3. A counter-only change (local or peer) starts no new record set.
4. No record set contains records with two different local stamp ids or two different peer stamp ids.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- Not applicable (C-125): "Every parser touched is fuzzed" (no new parser); "Regime stamp fields touched are complete and documented" and "Every stamp field has a stated source on each of the four profiles" (no field added); "Design documents updated where behaviour differs, constants table first" (follows C-18 unchanged); "No new open item without an owner and a falsifier" (none opened); "The stamp format is versioned independently of the media format" (no format change).

## Out of scope
- How the stamp is refreshed when a field changes (producers in other epics; the publication schedule is 013-24 and sending is epic 015, C-144); this ticket reacts to the id.
