# 010-14 — Announcements in every bundle member
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** S · **Depends on:** 010-04, 010-13
## Context
- A wide source emits a **bundle** of aligned streams (members); at most 32 members (C-08).
- C-19: each member stream has its own session number, assigned by the source; every consumer of a stream sees the same value. (This replaces the earlier wording "one session per bundle".)
- Announcements are re-issued in every member, because subscribers may take a subset of members (C-44: every 16 slices until the named position passes, in every bundle member).
- Announcement target = issue position + 64 slices (C-43).
- `Packetiser` holds per-stream state (session, epoch, announcements pending). Settled in the epic: bundle-level state lives in a bundle packetiser holding one `Packetiser` per member in a fixed array.

## Work
- In `hadronyche-core`, add `BundlePacketiser<const M: usize>` (name may be refined) holding one `Packetiser` per member of a `BundleLayout<M>`, each built with the session the source assigns to that member stream.
- `announce` on the bundle registers the same `(kind, target)` in every member's pending table, so every member carries it on the same positions.
- No allocation: member packetisers live in a fixed array.

## Acceptance criteria
1. For a 16-member bundle, after `announce(kind, p)` every member's slices at p, p + 768, p + 1 536, p + 2 304 (48 frames) carry the announcement with target p + 3 072 (epic AC-7).
2. A consumer subscribed to any single member alone sees the announcement (test with each member 0..15).
3. Each member carries the session value it was built with, and it is the same in every packet of that member (C-19).
4. A gap slice on member 3 at position p does not stop members 0–2 and 4–15 carrying the announcement at p.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Member subscription and admission (epic 012).
- Choosing session values: the source supplies them (C-19).
- Per-member interrupted/dead signalling on the receive side.
