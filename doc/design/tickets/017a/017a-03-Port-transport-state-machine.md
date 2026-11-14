# 017a-03 — Port the transport state machine
**Epic:** 017a-Reference-modules · **Effort:** S · **Depends on:** epic 002

## Context
- Deliverable: `hadronyche-reference/src/transport/` — the ported transport state machine and its **two existing tests** (POC `src/transport.rs:864`) `[code]`.
- Content-domain rule `[decision]`: read-only transport state is a content domain all reference modules may consume; transport **authoring** stays in the reference sender only (it is deferred in the DSL).
- The body of `transport.rs` was never read by the design; epic 002 (archaeology) reads it. The POC's transport check family reported "0 of 0 windows" in a 25 s run that looked green `[measured]`, so the port must not reproduce an inert check.
- **Transport state on a gap (settled, C-141):** the last known state is held and the playhead advanced as that state dictates: a playing transport moves on at its tempo over the positions elapsed; a stopped one does not advance (settled, C-200); the output is marked degraded. The read-only view therefore offers a hold-and-advance step that the echo (017a-07) and mixer (017a-08) call for a gapped position.
- **Transport state before any has arrived (settled, C-184):** stopped, playhead at zero, default tempo. The read-only view starts in this state, so a module that has received no transport state (or whose first transport positions are gapped) reads it rather than an undefined state. **Default tempo (settled, C-199):** 120 beats per minute; it replaces the ported state machine's default if that differs. Because the initial state is stopped, an early gap leaves the playhead at zero (C-200).

## Work
1. Port the state machine from POC `transport.rs` into `hadronyche-reference/src/transport/mod.rs` (split into files as needed).
2. Split the API into a read-only view (for echo and mixer) and an authoring interface (visible to the sender only, e.g. `pub(crate)` or a separate type owned by the sender).
3. Port the two existing tests at `transport.rs:864` unchanged in intent.
4. Add `hold_and_advance(positions)` to the read-only view: keeps the last known state; if that state is playing, advances the playhead over the given number of positions at its tempo; if it is stopped, leaves the playhead where it is (C-141, C-200).
5. Initialise the read-only view to the C-184 state: stopped, playhead at zero, tempo 120 beats per minute (C-199), as a named constant rather than the ported state machine's default.
6. Header comment citing POC file/lines and the epic 002 finding that cleared it.

## Acceptance criteria
1. Both ported POC tests pass.
2. The echo and mixer modules cannot call authoring methods (compile-fail test or visibility check).
3. Applying the same sequence of authoring events yields the same read-only state on two instances (property test, 1 000 random sequences).
4. After a gap of n positions with a playing held state, `hold_and_advance(n)` leaves every state field unchanged except the playhead, which has advanced by exactly n positions at the held tempo (C-141).
5. After a gap of n positions with a stopped held state, `hold_and_advance(n)` leaves every state field unchanged, playhead included (C-200); in particular a fresh view gapped before any state has arrived still reads playhead zero.
6. A freshly created read-only view, before any transport state has arrived, reports stopped, playhead at zero and tempo 120 beats per minute (C-184, C-199).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Each ported POC piece cites its source file and the archaeology finding that cleared it

## Out of scope
- The transport check family in the analyzer (017a-04 counts it; 017a-05 compares it).
- Transport authoring in the DSL (deferred; task 025).
