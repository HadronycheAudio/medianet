# 011-18 — Follower cap at 64
**Epic:** 011-Gossip-and-discovery-core · **Effort:** XS · **Depends on:** 011-15, epic 005

## Context
- C-35 (decided): at most 64 followers per holder; excess refused with a named event.
- Rule `[decision]`: a holder serving 64 followers refuses the 65th with a named event.
- Epic AC 5, AC-25 (core part): with 64 synthetic followers, the 65th is refused with a named event: `ElectionEvent::FollowerRefused(ModuleId)`.
- C-114 (decided): with at most 64 modules (C-34) a holder has at most 63 real followers; the 64-follower load and the refusal of a 65th are tested with synthetic followers.
- `Candidacy { followers: u8, .. }` carries the holder's current follower count.
- C-36 (inherited) 50 sync requests per follower per second, and C-112 (decided) an accepted rate of 10 to 200 per second: load context only; serving and rate checks are the clock thread's (epic 008), not this ticket's: the holder answers sync requests on the clock thread, never on housekeeping (C-188), and requests beyond 200 per second from one follower are dropped there before any work is done and counted per follower in the holder's stamp (C-204).

Open sub-question owned by L1, task 019 (owner of C-35): can a holder really serve 64 followers at 50 requests/s each (and up to 200/s, C-112)? Falsifier (C-167): holder serving lateness, the 99.9th percentile of reply send time minus request receive time, grows by more than 20 % from 1 to 64 followers. If falsified, only the constant changes; keep it a single named `const MAX_FOLLOWERS: u8 = 64`.

Still open (epic open sub-question, owner epic 005 with 008, whose clock thread receives the sync requests, C-188): how the holder learns of a follower attach, that is, the input form that leads to `FollowerRefused`. Implement the input epic 005 defines; do not invent one.

## Work
1. `pub const MAX_FOLLOWERS: u8 = 64;`
2. In `Election::step`, when I am holder and a follower attaches (input form per epic 005) while `me.followers >= MAX_FOLLOWERS`, emit `FollowerRefused(id)` and a refusal message in the `Outbox`.
3. A synthetic-follower test helper that sets `followers` to any count and generates attach inputs from fabricated ids (C-114).

## Acceptance criteria
1. With 64 synthetic followers (`followers = 64`), a 65th attach yields `FollowerRefused(id)` with the requester's id.
2. With `followers = 63` (the real maximum in a 64-module medianet), an attach yields no refusal.
3. `MAX_FOLLOWERS` is the only place the value 64 for followers appears.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

Not applicable (C-125): "Every parser touched is fuzzed" (no new parser); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "Design documents updated" (follows C-35 and C-114); "The frozen POC is not modified" (no code near the POC); "No election path reachable from hello expiry" (refusal does not open elections).

## Out of scope
- Counting followers on the clock thread: epic 008. Dropping and counting requests above 200 per second per follower (C-204): epic 008. Serving-load measurement: L1, task 019 (C-35, C-167).
- Reporting the DSCP observed on each follower's sync requests (C-152): the holder's clock side, epic 008.
