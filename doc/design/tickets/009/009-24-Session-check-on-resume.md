# 009-24 — Session check on resume
**Epic:** 009-Gap-and-reassembly-core · **Effort:** XS · **Depends on:** 009-06, 009-15

## Context
Gap contract `[decision]`: resume from Interrupted is immediate at the frozen offset when slices return on the same session.

- C-19 `[decision]`: the session number is assigned by the source, one per stream; it is bumped on restart and on timeline epoch change; every consumer of a stream sees the same value. It is 32 bits (C-10) and is carried on every slice (`SliceRef::session`, 009-06).
- "Same session" therefore means the same session number as the input's last slice before the interruption.
- A timeline epoch change also bumps the session; such slices are first handled as other-epoch slices (009-10, 009-12), and the expected session is updated when the consumer switches epoch.

Remaining gap of the epic, owned by the control state machines (epic 005): what the consumer does when an interrupted input returns on a different session. This ticket implements only what the epic states: no resume on a new session, and a `SessionChanged` event so the state machine can decide. Falsifier: a restarted source is resumed at an offset measured for its previous session.

## Work
Extend `FanIn`:
- Per input: the expected session number, set at admission (`readmit` or construction) from the first slice, and updated when the consumer switches timeline epoch.
- A slice whose session differs from the expected one while the epoch matches: not stored, the input stays in its current health state, `HealthEvent::SessionChanged(input)` is emitted once per new session value.
- From `Interrupted`, only a slice with the expected session leads to `Resumed` (009-15).

## Acceptance criteria
1. Interrupted input, slices return with the same session → `Resumed`, offset unchanged (EC-4).
2. Interrupted input, slices return with a new session on the same epoch → stays `Interrupted`, one `SessionChanged` event, no slice stored.
3. Epoch switch accepted by the consumer → the new session of that epoch becomes expected; its slices are stored.
4. A Live input whose session changes emits `SessionChanged` once, not once per slice.
5. Runs under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The connection-state response to `SessionChanged` (epic 005, consumed through epic 012).
- Assigning and bumping session numbers (source side, other epics).
