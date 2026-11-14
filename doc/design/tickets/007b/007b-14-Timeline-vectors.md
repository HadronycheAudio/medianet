# 007b-14 — Timeline vectors
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** S · **Depends on:** 007a-02, 007a-04, epic 004, epic 005, epic 006

## Context
- Slices are addressed by position (64 bits) on a domain-wide timeline. The packet header carries a 16-bit timeline epoch and a 32-bit session (C-10).
- Timeline rules `[decision]`:
  - position = (network time − epoch origin) × rate, where rate is the domain rate of 96 000 Hz, the only rate on the wire (C-01);
  - the mapping is affine per epoch and never extrapolated across epochs;
  - an epoch-switch announcement is first sent 64 slices ahead (C-43) and re-issued every 16 slices until the named position passes, in every bundle member (C-44);
  - the session number of each stream is bumped on a timeline epoch change, and every consumer sees the same value (C-19);
  - a fan-in treats an input on another epoch as a gap; such a slice is not a miss, restarts the dead clock and is counted in its own per-connection other-epoch counter (C-135, C-159).
- Epic 006 supplies the timeline and epoch scenarios, epic 004 the wire layouts (C-193). Still open, owned by epic 006 (timeline tables, C-193): the units of network time, epoch origin and position, the rounding of the product, and the behaviour before origin; and whether an announcement is also carried in the slice at the named position. This ticket takes both from the tables.
- Deliverable: `vectors/timeline/*.json` with position/time conversions and epoch-switch scenarios.

## Work
1. Write conversion vectors: (network time, epoch origin) → position at 96 000 Hz, and the inverse, including origin itself, large times near the 64-bit position limit, and times before origin if the tables define them, with the rounding the tables state.
2. Write epoch-switch scenarios for a bundle of at least 2 members: announcement first sent 64 slices ahead, re-issued at 48, 32 and 16 slices ahead in every member, switch at slice 0 (plus an issue in the switch slice if the tables require it); positions after the switch use the new epoch's affine map only; the session number of every stream is bumped at the switch.
3. Write a fan-in scenario: two inputs, one on the old and one on the new epoch; the other-epoch input is treated as a gap.
4. Check them through the driver of 007a-04.

## Acceptance criteria
1. At least 5 conversion vectors in each direction, all at 96 000 Hz.
2. The epoch-switch scenario expects announcements at 64, 48, 32 and 16 slices ahead in every bundle member, followed by the switch, and no announcement more than 64 slices ahead.
3. The session number of every stream after the switch differs from its value before, and is the same for every consumer.
4. No vector expects a position computed from a previous epoch's map after the switch.
5. The fan-in scenario expects a gap for the other-epoch input and the other-epoch counter +1 per such slice (C-135).
6. All files carry `spec_version`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Clock election and the epoch bump it causes (007b-02); the epoch bump on a holder stall (007b-04). Encoding of the epoch-switch announcement (007a-06).
