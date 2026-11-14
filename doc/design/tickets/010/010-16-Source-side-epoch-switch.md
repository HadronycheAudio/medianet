# 010-16 — Source-side epoch switch
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** M · **Depends on:** 010-13, 010-14, epic 006
## Context
- Packet header carries session (32 bits) and timeline epoch (16 bits) (C-10).
- C-19: the session number is assigned by the source, one per stream, and bumped on restart and on timeline epoch change; every consumer of a stream sees the same value. So at a switch each member's session is bumped (incremented, wrapping at 32 bits); no value comes from epic 006.
- Epoch switch on the source side `[decision]`: learn the new line from sync, announce, and at the switch position end the old session and start a new one with the new epoch.
- Settled in the epic (contract between `announce` and `switch_epoch`): `switch_epoch` issues the timeline-epoch-switch announcement itself, with target T = `now_position` + 64 slices (C-43, flat), and returns T. The caller does not announce separately and cannot choose another lead.
- `Line` and the switch procedure come from epic 006.
- Epic AC-9: `switch_epoch` issues the announcement naming T; every member switches at T to a bumped session and the new epoch. Contributes to system AC-18.
- Open item owned by epic 006 (switch procedure): the boundary reading. Falsifier: consumers see a duplicate or missing position across the switch. Until ruled, use the boundary reading — the old session's last slice is the one just before T and the new session's first slice is at T — and label it provisional.

Signature:
```rust
pub fn switch_epoch(&mut self, now_position: u64, new: &Line) -> u64; // issues the announcement, returns T
```

## Work
- Implement `switch_epoch` on `Packetiser` and on the bundle packetiser (010-14), `no_std`, no allocation: issue the epoch-switch announcement (010-13 / 010-14) with target T, record the pending switch (T, new epoch from `Line`).
- At T: for every member of the bundle together, bump the session (C-19) and start carrying the new epoch.
- A second `switch_epoch` while one is pending returns an error (or is documented as replacing it); choose and test.

## Acceptance criteria
1. `switch_epoch(p, line)` returns T = p + 64 × frames per slice.
2. With that T, slices before T carry the old session and old epoch; the slice at T carries session old + 1 and the new epoch from `Line` (epic AC-9, under the provisional boundary reading).
3. The boundary is identical for every member of a 16-member bundle, and each member's session is its own value + 1.
4. The epoch-switch announcement naming T is present in every member at the C-44 re-issue positions before T.
5. No slice of the new session has a position lower than T; no slice of the old session has a position ≥ T.
6. A session of `u32::MAX` bumps to 0.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Learning the new line from sync (epic 006).
- Session bump on restart: done by the source when it builds its packetisers (C-19).
- Consumer-side switch handling and the rest of system AC-18.
