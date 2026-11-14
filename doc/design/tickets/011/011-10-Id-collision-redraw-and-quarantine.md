# 011-10 — Id collision: redraw and quarantine
**Epic:** 011-Gossip-and-discovery-core · **Effort:** M · **Depends on:** 011-07, 011-24, epic 005

## Context
Identity `[decision]`: id = 64 bits from the OS random source plus a 64-bit instance nonce (C-11). A **collision** is the same id seen with a different nonce.
- The colliding pair detects it within one hello period (1 s, C-30).
- The **later joiner redraws** its id and rejoins; if both joined at once, both redraw.
- **Third parties quarantine** the id (ignore both) until only one instance nonce has been seen for the id for one hello-expiry period, 3 s (C-46, C-31).
- It is logged on both modules: in this I/O-free core that means `MapEvent::Collision(id)` is raised on both modules and the platform writes the log line (epic). A collision is never silent.
- Tests force the collision with the test-only hook that sets a module id and nonce (C-120), delivered by 011-24.

`[code]` Retired POC behaviour, which must not survive: a hello carrying one's own id was taken as one's own echo and silently swallowed (`module.rs:404-409`), so a collision merged two modules.

Epic AC 2, AC-22: a forced id collision is detected within one hello period (1 s); the later joiner redraws; third parties quarantine the id until only one nonce has been seen for 3 s; `Collision` on both modules; no merged peer.

Still open (epic open sub-question, owner epic 005): how a module decides it is the "later joiner" and what "joined at once" means; the hello carries no join time. Implement the rule from epic 005's transition table. If the table does not have it when this ticket starts, the ticket is blocked; do not invent a rule.

Events: `MapEvent::Collision(ModuleId)` on the pair, `MapEvent::Quarantined(ModuleId)` on third parties.

## Work
1. Implement `pub fn redraw_id(&mut self, rng: &mut dyn IdSource, out: &mut Outbox);` — draw a new `(ModuleId, Nonce)`, reset own join state, and emit a hello (rejoin).
2. In `on_hello`: a hello whose id equals my id
   - with my nonce: my own echo, ignored;
   - with a different nonce: collision — raise `Collision(id)`; if I am the later joiner (or both joined at once) per epic 005, call `redraw_id`.
3. In `on_hello` for a third party: a known peer id seen with a different nonce → mark the entry quarantined, ignore hellos from both nonces for admission, raise `Quarantined(id)`; a quarantined entry is never reported as one peer and never used for admission.
4. Lift quarantine once only one nonce has been seen for the id for 3 s (C-46): record, per quarantined id, the last time each nonce was seen, and lift when the other nonce has been silent for 3 s while the id is still heard.

## Acceptance criteria
1. Two simulated modules A (joined at t=0) and B (joined at t=5 s), forced to the same id with different nonces through the C-120 hook: both raise `Collision(id)` within 1 s of B's first hello; only B redraws; after the redraw both appear as distinct peers in each other's map.
2. Two modules joining "at once" (per epic 005's definition) with the same id: both redraw.
3. A third module C hearing both raises `Quarantined(id)`, and its map never holds a single entry combining fields of A and B ("no merged peer").
4. A hello with my own id and my own nonce is ignored without any event.
5. C's quarantine lifts 3 s (± one tick) after the last hello carrying the second nonce, not before; C then sees both modules under their final ids.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] Design documents updated where behaviour differs, constants table first

Not applicable (C-125): "Every parser touched is fuzzed" (no new parser); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "No election path reachable from hello expiry" (no expiry or election code).

## Out of scope
- Platform id sources (OS random; hardware unique id mixed with a persistent boot counter on devices without a true random source): epics 015, 026.
- The forced-identity hook itself: 011-24.
- Writing log lines: the host/device platform consumes `MapEvent`.
