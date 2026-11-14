# 009-27 — Transport-state concealment
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-19, 009-12, epic 004

## Context
Default concealment is per content type (009-19). C-141 `[decision]` (as revised) fixes the rule for transport state on a gap: the last known state is held and the playhead advanced as that state dictates: a stopped transport stays where it is, a playing one moves on at its tempo; the output is marked degraded.

- C-200 `[decision]`: the playhead of a stopped transport does not advance through a gap. This replaces the earlier reading of this ticket, which added the positions elapsed whatever the state.
- C-184 `[decision]`: transport state before any has arrived is stopped, with the playhead at zero and the default tempo.
- C-199 `[decision]`: the default tempo is 120 beats per minute.
- Positions are counted in domain-rate samples (96 kHz, C-01); one slice advances the position by 48 (C-02).
- The transport content type and the layout of its state (including the playhead field) come from epic 004.
- `Conceal::fill` carries `last` (the last received state) and `positions_elapsed` (positions since that state's slice) since 009-19.
- The round already sets `degraded` when an input is gapped (009-12); this ticket checks that a concealed transport output keeps it.

## Work
In `rx::conceal`, extend `DefaultConceal`:
- Transport content type: copy `last` into `out`, then, using epic 004's layout, read the held state's run state and tempo:
  - stopped: leave the playhead unchanged (C-141, C-200);
  - playing: advance the playhead by the time of `positions_elapsed` domain-rate samples (`positions_elapsed` / 96 000 s, C-01) at the held tempo, converted to epic 004's playhead unit (C-141).
- With no `last` (gap before any received state), write the C-184 state: stopped, playhead zero, tempo 120 beats per minute (C-199), encoded with epic 004's layout; being stopped, it does not advance (C-200). The manifest initial value of dense controllers (009-19) is not used for transport.
- The caller (`FanIn` round handling) computes `positions_elapsed` as the round position minus the position of the last received transport slice.
- Wrap-around of the playhead follows epic 004's field width (documented).

## Acceptance criteria
1. Last state playing at 120 beats per minute, gap of 2 000 slices (2 000 × 48 = 96 000 positions = 1 s, C-01, C-02) → concealed playhead advanced by 2 beats, in epic 004's playhead unit; every other field unchanged (C-141).
2. Last state stopped with playhead P, gap of 2 000 slices → concealed playhead still P; every other field unchanged (C-200).
3. A gap of one slice followed by received state: the received state is used unchanged.
4. Gap before any received state, after several slices → `out` is stopped with playhead 0 and tempo 120 beats per minute (C-184, C-199), whatever `initial` holds; no advance (C-200).
5. The output round for a concealed transport input has `degraded == true` (C-141).
6. Runs under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The transport content type and state layout (epic 004).
- What a module does with transport state (module code).
