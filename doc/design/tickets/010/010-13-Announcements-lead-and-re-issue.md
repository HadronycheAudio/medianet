# 010-13 — Announcements: lead and re-issue
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** S · **Depends on:** 010-06, 010-09, epic 004
## Context
Announcements `[decision]`:
- Kinds: end-of-stream, re-alignment, timeline epoch switch, transport reference change.
- An announcement names a future position: current position + 64 slices (C-43, flat), converted to samples with the stream's slice length. At C-02 (48 frames) that is 3 072 samples of position.
- It is re-issued every 16 slices (C-44) until the named position passes.
- Consumers process positions in order, so an announcement is always processed before the position it names; the lead only has to survive loss.
- Announcements are optional in the packet layout (after the slice header, before the measurement extension); in the multi-packet regime they go in packet 0. Encoding from epic 004.
- Gap slices are header-only and never carry announcements (settled in the epic).
- C-137: on a fallback path announcements fit inside the 100-byte header cap or are omitted from the slice; that omission is implemented in 010-23, on top of this ticket's schedule.
- Open item owned by epic 004 (falsifier: a descendant misses an announcement despite four re-issues, C-43):
  - whether the slice at the target position still carries the announcement;
  - what happens to a re-issue that falls on a gap slice (dropped, or carried by the next slice with content);
  - what happens if the stream's slice length changes while an announcement is pending.
  Until epic 004 rules, implement the provisional answers below and label them provisional in the doc comments.

Signature:
```rust
pub fn announce(&mut self, kind: Announcement, now_position: u64); // target = now + 64 slices
```

## Work
- In `hadronyche-core`, add `Announcement` (the four kinds, encoding per epic 004) and a fixed-capacity pending table in `Packetiser` (no allocation). Choose the capacity (at least one entry per kind plus room for relayed entries, 010-15) and make a full table return an error from an internal insert that `announce` reports through a counter or a debug assertion; document the choice. This is an implementation choice, not a design question.
- `announce` stores `(kind, target = now_position + 64 × frames_per_slice)`, `issued_at = now_position`.
- In `begin_slice`/`finish`, include a pending announcement in the slice at `issued_at` and at every slice whose position is `issued_at + k × 16 slices` (k ≥ 1) while the slice position is below the target; drop it once the slice position reaches the target (provisional: the slice at the target does not carry it).
- Provisional: a re-issue that falls on a gap slice is not moved; the next scheduled re-issue covers it.
- Provisional: the slice length used is the one in force at `announce`; the doc comment states it.

## Acceptance criteria
1. `announce(kind, 10_000)` with 48 frames per slice yields a target of 13 072 (10 000 + 64 × 48) in the encoded announcement (epic AC-7).
2. The announcement appears in slices at positions 10 000, 10 768, 11 536, 12 304 (every 16 slices = 768 samples) and in no slice between them (epic AC-7).
3. No slice with position ≥ 13 072 carries it (epic AC-7, provisional at-target rule).
4. In a multi-packet slice it appears only in packet 0.
5. A gap slice at 10 768 carries no announcement and the one at 11 536 still does.
6. Two announcements of different kinds pending at once are both carried on their own schedules.
7. Epic 007 conformance vectors that include announcements encode byte for byte (epic AC-1, partial).
8. Filling the pending table beyond capacity gives the documented behaviour without panic.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Presence in every bundle member (010-14).
- Relaying a received announcement unchanged (010-15).
- Epoch-switch behaviour at the target (010-16).
- Omitting announcements that do not fit the C-137 cap on a fallback path (010-23).
