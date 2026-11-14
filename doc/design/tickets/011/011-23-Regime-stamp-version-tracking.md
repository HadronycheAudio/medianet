# 011-23 — Regime stamp version tracking
**Epic:** 011-Gossip-and-discovery-core · **Effort:** XS · **Depends on:** 011-07, epic 004

## Context
C-18 (decided): a module's regime stamp is not in the hello. It is a dedicated control message split into sections that each fit C-17, multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch. The hello carries only a stamp version number. The message is defined by epic 013 and sent by epic 015 (C-144). Discovery is therefore where a module first learns that a peer's stamp has changed.

The hello encoder and parser (011-02, 011-03) carry the version field and the peer map (011-06) stores it. The epic adds the event `MapEvent::StampVersionChanged { peer: ModuleId, version }`, which the stamp consumer uses to issue the unicast fetch. The width of the version comes from the epic 004 layout.

## Work
1. In `Discovery::on_hello`, compare the version in the hello with the version stored for that peer. If it differs, store the new value and raise `StampVersionChanged { peer, version }` (queued behind any other event of the same hello).
2. A peer's first hello (a `Joined`) also raises `StampVersionChanged`, so a consumer fetches the stamp of every new peer.
3. The own stamp version is taken from `LocalInfo` on every hello sent.

## Acceptance criteria
1. A peer whose hellos carry version 5, then 5, then 6 raises exactly one `StampVersionChanged { version: 6 }` after its join.
2. A new peer raises `Joined` followed by `StampVersionChanged` with its current version.
3. A hello that is rejected (foreign domain, malformed, wrong version) never raises `StampVersionChanged` and does not change the stored version.
4. Changing the own `LocalInfo` stamp version changes the version in the next hello sent.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

Not applicable (C-125): "Every parser touched is fuzzed" (the field is parsed by 011-03 and fuzzed by 011-05); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (handles the version number, not a stamp field); "Design documents updated" (follows C-18); "The frozen POC is not modified" (no code near the POC); "No election path reachable from hello expiry" (no expiry or election code).

## Out of scope
- Sending, sectioning, refreshing or fetching the regime stamp message: defined by epic 013 and sent by the hosted backend, epic 015 (C-144).
- Bumping the own stamp version when the regime changes: the owner of the stamp.
