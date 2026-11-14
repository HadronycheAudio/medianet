# 005-29 — Sink switch to a changed A
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-15, 005-16, 005-28

## Context
- Machine 4 (connection states, consumer side) of `doc/design/spec/control.md`; table `connection_consumer` in `doc/design/spec/control.toml` (005-15, 005-16); the A in force per admitted input from 005-27; the receiver report's A field and the fan-in rows for an inherited A from 005-28; checker from 005-03.
- C-55: the audition offset A is one per presentation group, proposed by commissioning. C-96: alignment rings are sized to A.
- C-186: every fan-in that inherited the old A re-enters aligning at an announced position, with rings and ratchet ceiling resized, "as the sinks do".
- C-214: a sink switches to a changed A at its own position, one lead (C-43: 64 slices, 32 ms at defaults) ahead, as a fan-in does (C-208); it has nobody to announce it to.
- C-224: a commissioned change of a sink's own A reaches the sink through the commissioning tool, inside a declared window (C-153): the tool validates buffer memory first (C-211) and then announces the new A to the group's members.
- C-222: the clamp of C-211 applies to any consumer that buffers to meet A, sinks included; C-216 applies to a sink in the same way.
- C-198: the receiver report (C-40, one per bundle every 100 ms) carries the consumer's current A; a change propagates one hop per report.
- 005-28 covers only a fan-in whose A is inherited, and excludes a consumer whose A comes from its own presentation group. No ticket covered a sink's own switch to a changed A.
- L-2: re-alignment travels in-band in slices; a sink's switch position is a local decision with no announcement (C-214). L-5: a change of A changes no pinned destination.

## Work
1. Add a local input "own A changed" to `connection_consumer`, raised for a sink's connections when its presentation group's A changes (C-55); name its source as the commissioning tool's announcement of the new A to the group's members inside a commissioning window (C-224), whose message format belongs to other epics.
2. Add the rows for "own A changed": from Live, re-enter Aligning at a position the sink picks itself, one lead (C-43) ahead, with rings and ratchet ceiling resized (C-186), and no announcement output (C-214); when the sink's declared buffer memory cannot hold the new A, the row keeps the deepest ring it can hold as ceiling, raises the cannot-meet-A alarm (C-174) with `alarm = true`, as the 005-28 fan-in row does (C-222, C-211); decide the rows for every other state (for example Aligning, Interrupted, Locking, Demoted, and the pre-admission states, which only update the A in force) and record them in "Decisions on open sub-questions" with a `[decision]` tag and a falsifier.
3. State that the sink's next receiver report carries the new A (C-198), which starts the propagation handled by 005-28.
4. Add scenario S15 to the "Scenarios" section: the A of a presentation group changes; its sink switches at its own position 64 slices (32 ms) after it learns the new A, with no announcement; the fan-in feeding it learns the new A from the sink's next report and re-aligns as in S14.

## Acceptance criteria
1. Every consumer state has a row for "own A changed"; the checker of 005-03 passes.
2. From Live, the row moves to Aligning with the switch position one lead (64 slices, C-43) ahead, rings and ratchet ceiling resized (C-186), and no announcement output (C-214).
3. In S15 the sink's first receiver report after the change carries the new A (C-198), and the upstream fan-in learns it at most one report period (100 ms, C-40) later.
4. No new row changes a pinned destination (L-5); the rows decided beyond the Live case are recorded with their falsifier.

## Definition of done
- [ ] Transition tables are total (checked mechanically from the machine-readable form)
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- How commissioning chooses or changes a group's A, its buffer-memory check and the announcement message (C-55, C-211, C-224, other epics).
- Propagation upstream and fan-in re-alignment (005-28, C-198, C-208).
- Ring resizing and the alignment data path (other epics).
