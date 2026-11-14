# 011-08 — Hello reject counters and version event
**Epic:** 011-Gossip-and-discovery-core · **Effort:** S · **Depends on:** 011-07

## Context
Epic AC 10, AC-23: foreign-domain, domain-0, malformed and wrong-version hellos cause no state change and are counted per reason; a version mismatch produces a named event.

Rules `[decision]`:
- A foreign-domain hello (a different non-zero domain) is dropped and counted.
- A hello carrying domain 0 is ignored and counted, not refused (C-173): it has its own counter and raises no event.
- A malformed hello is dropped and counted.
- A version mismatch is counted and is a named event; the version is in the hello so it is detected before any connection. It is raised here, at discovery, where it stops the peer joining (the peer is never inserted), not at admission (C-185).
- `Hello::parse` (011-03) returns `Reject::{ForeignDomain, UnconfiguredDomain, Malformed, VersionMismatch { peer, theirs }}`.
- `MapEvent` has `ForeignDomain` and `VersionMismatch { peer: ModuleId, theirs: u8 }`.
- The epic's deliverables fix the counting API: `Discovery::reject_counters() -> RejectCounters` with `foreign_domain`, `unconfigured_domain`, `malformed`, `version_mismatch` as `u64`.

## Work
1. In `Discovery::on_hello`, on `Err(reject)`: do not touch the map, timers or identity; increment the matching counter, saturating.
2. Return `MapEvent::VersionMismatch { peer, theirs }` for a version mismatch, `MapEvent::ForeignDomain` for a foreign domain, and `None` for malformed input and for a domain-0 hello (C-173).
3. Implement `Discovery::reject_counters()` as in the epic, read-only.

## Acceptance criteria
1. For each of the four reasons, feeding 100 rejecting hellos leaves the peer map byte-for-byte equal to before (compare a snapshot) and the matching counter equals 100, the others 0.
2. A wrong-version hello returns `Some(MapEvent::VersionMismatch { peer, theirs })` with the sender's id and version.
3. A foreign-domain hello returns `Some(MapEvent::ForeignDomain)`.
4. A malformed hello returns `None` and does not panic.
5. A domain-0 hello returns `None`, increments only `unconfigured_domain`, and does not raise `ForeignDomain` (C-173).
6. Counters saturate rather than wrap (test from `u64::MAX - 1`).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

Not applicable (C-125): "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "Design documents updated" (the API is the epic's); "The frozen POC is not modified" (no code near the POC); "No election path reachable from hello expiry" (no expiry or election code).

## Out of scope
- Exporting counters to metrics or logs on a host: epic 015.
- Admission refusals (epic 012): VersionMismatch is not one of them (C-185).
- Parse logic itself: 011-03.
