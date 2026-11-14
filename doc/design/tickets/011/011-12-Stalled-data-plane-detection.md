# 011-12 — Stalled data plane detection
**Epic:** 011-Gossip-and-discovery-core · **Effort:** S · **Depends on:** 011-07, epic 005

## Context
The hello carries a data-plane liveness counter: the authentication reservation's 64-bit message counter (C-12) is live in v1 because it is the liveness counter, while the scheme is 0 and the tag is zero (C-115).
- C-42 (decided): a stalled data plane is visible to peers within 2 hello periods.
- C-30: hello period 1 s, so within 2 s.
- Event: `MapEvent::Stalled(ModuleId)`.

Still open (epic open sub-question, owner epic 005): the exact detection rule — over what span the counter must fail to advance — and whether a module with no data plane (no live stream) is exempt. Falsifier: a module with no streams is reported stalled, or a stalled data plane is not reported within 2 hello periods. Implement epic 005's rule; the working assumption to check against it is "no advance across the hellos of the last 2 s".

## Work
1. In `PeerMap` bookkeeping, record the time the peer's liveness counter last changed.
2. In `Discovery::on_tick` (or `on_hello`), raise `Stalled(id)` once when the epic 005 rule is met while hellos still arrive; re-arm when the counter advances again. Apply the exemption for modules without a data plane if epic 005 defines one.
3. On the sending side, `LocalInfo` takes the own liveness counter from the platform each hello (encoded by 011-02).

## Acceptance criteria
1. A peer whose counter stops advancing at t (hellos still arriving every 1 s) raises `Stalled(id)` no later than t + 2 s (C-42).
2. `Stalled` is raised once per stall, not on every tick.
3. When the counter advances again, a later stop raises `Stalled` again.
4. A peer whose counter advances on every hello never raises `Stalled` over a 60 s run.
5. Stall detection never removes the peer from the map and never opens an election.
6. The exemption case (if any) defined by epic 005 has its own test.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] Design documents updated where behaviour differs, constants table first

Not applicable (C-125): "Every parser touched is fuzzed" (no new parser); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "The frozen POC is not modified" (no code near the POC); "No election path reachable from hello expiry" (covered by criterion 5; no expiry code here).

## Out of scope
- Incrementing the counter in the data plane: platform/data-plane epics.
- Acting on a stall (operator alarm, stream teardown): consumers of `MapEvent`.
