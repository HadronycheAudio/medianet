# 012-16 — Gossip isolation and pinned destinations
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** 012-14, 012-15

## Context
Epic 012 rules `[decision]`:
- **L-1:** a live connection becomes dead only from data-plane evidence. Gossip silence never tears it down.
- **L-5:** destinations are pinned at admission and change only by re-admission. A hello with a new address never redirects a live stream.
- **L-4:** housekeeping admits unknown sources only; it never touches a live data path. Peer-map changes reach the engine only through `on_peer_map`, which may move Pending to Requested and nothing else (012-14).
- The hello carries only a regime-stamp version (C-18); a new stamp version from a peer is gossip and changes no connection.
- Hello expiry for non-live peers is 3 s (C-31); a live peer is never expired by hello silence.

Retired POC behaviour `[code]`: routes were rebuilt when a peer's hello showed a new address (`module.rs:542`).

Acceptance criteria of the epic covered here: 012-A7 (AC-20 a) 30 s of hello silence leaves every Live connection Live; 012-A6 (AC-20 d) a hello with a changed address during Live changes nothing in the pinned addresses. DoD: no transition out of Live or Interrupted is reachable from a gossip input (test and table inspection).

## Work
- Tests driving `Admission` with Live, Interrupted and Demoted connections through: `on_peer_map` calls removing the peer, changing the peer's IPv4 address, changing its stamp version, and 30 s of ticks without any hello.
- Code audit: confirm that no path from `on_tick`, `on_message` or `on_peer_map` calls `step` with anything but handshake inputs, and only for Requested/Pending slots; add a debug assertion.
- Table inspection: a test iterating the 012-11 const transition table asserting every row whose source is Live, Interrupted or Demoted has a data-plane trigger.

## Acceptance criteria
1. 012-A7 (AC-20 a): 30 simulated seconds of ticks with no hello (peer absent from the map) leave every Live connection Live and Interrupted ones Interrupted (until data-plane evidence).
2. 012-A6 (AC-20 d): a peer-map change of the source or destination address during Live leaves `pinned_source` and `pinned_dest` byte-identical, and no outbox message is produced.
3. The table-inspection test passes and fails if a gossip-triggered row out of Live/Interrupted is added (verified once by a mutation).
4. Only re-admission (a new `request`) yields new pinned addresses.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: tests only
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection)
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Housekeeping itself and how it calls the engine (epic 015). Hello parsing (epic 011).
