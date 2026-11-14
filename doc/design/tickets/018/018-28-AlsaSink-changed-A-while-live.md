# 018-28 — AlsaSink re-alignment on a changed A (C-186, C-198, C-214)
**Epic:** 018-Linux-ALSA-edge · **Effort:** S · **Depends on:** 018-13, 018-24, epic 009

## Context
- A is the operator-configured audition offset of the sink's presentation group, with no default (C-55); the sink's input ring is sized to A (C-96), and `emit(p) = T(p) + A`, `submit(p) = emit(p) − D` `[decision]`.
- **C-186:** when A changes while inputs are live, every fan-in that inherited the old A re-enters aligning at an announced position, with rings and ratchet ceiling resized, as the sinks do. The sink therefore re-enters aligning with its ring resized to the new A; from its switch position `submit(p)` uses the new A.
- **C-198:** the receiver report (C-40) carries the consumer's current A. The sink's receiver reports carry its current A, so the change propagates upstream one hop per report, on the data plane, and each fan-in that inherited the old value re-aligns (C-186). The receiver report itself is not built by this epic.
- **C-214:** a sink switches to a changed A at its own position, one lead (C-43: 64 slices, 32 ms at defaults) ahead, as a fan-in does (C-208); it has nobody to announce it to. The sink picks the switch position itself; it is not taken from an announcement or from the epic 009 ring.
- **C-224:** a commissioned change of the sink's own A reaches the sink through the commissioning tool, inside a declared window (C-153): the tool validates buffer memory first (C-211) and then announces the new A to the group's members. The sink acts on that announcement.
- **C-222 (with C-211, C-216):** the clamp applies to any consumer that buffers to meet A, sinks included. A change that does not fit the sink's buffer memory and reaches it anyway leaves the sink at the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm and keeps it emitting; the alarm stays raised until a later change of A that fits, and the receiver report carries the A it was given, not the clamped value.
- A ratchet step in the sink is taken the same way, at its own position one lead ahead, unannounced (C-225, 018-22). The "cannot meet A" test (C-174, 018-24) uses the A in force.

## Work
1. In `alsa/sink.rs`, accept a change of A while the input is live; pick the switch position one lead (C-43) ahead of the sink's own current position and re-enter aligning there (C-186, C-214), using the epic 009 ring mechanics; emit no announcement.
2. Resize the input ring to the new A rounded up to whole slices (C-96, C-186) at the re-alignment, keeping allocation off the data path.
3. From the switch position, compute `submit(p)` with the new A; positions before it keep the old A.
4. Hand the current A to the receiver report (C-198); after a clamped change, the A given, not the clamped value (C-216, C-222).
5. Check the new A against the sink's buffer memory with epic 009's `check_depth` (009-11); if it does not fit, keep the deepest ring the sink can hold as its ceiling and hold the cannot-meet-A alarm with `set_ceiling_short` (009-26) until a later change of A that fits (C-211, C-216, C-222).
6. Make 018-24 test "cannot meet A" against the new A from the switch position.

## Acceptance criteria
1. **Epic criterion 018-A13:** on fake inputs, changing A from 2 ms to 4 ms while the input is live makes the sink re-enter aligning at the position one lead (64 slices, C-43) ahead of its current position when the change is received (C-214); positions before it are emitted at T(p) + 2 ms and positions from it at T(p) + 4 ms, within 1 sample.
2. The ring span after the change equals the new A rounded up to whole slices (C-96, C-186), for an increase and a decrease of A.
3. The receiver report carries the old A before the change and the new A after it (C-198).
4. The "cannot meet A" test uses the new A from the switch position (C-174, C-186).
5. The sink emits no announcement of its switch position (C-214).
6. A change to an A deeper than the sink's buffer memory (C-222): the ring stays at its deepest size, one `CannotMeetA` at once, emission continues, no `MeetsAAgain` until a later change to an A that fits, and the receiver report carries the A given (C-216).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The alignment ring and its re-alignment mechanics (epic 009); the receiver report message (C-40).
- Re-alignment of fan-ins and the commissioning tool's memory check and announcement before applying a change of A (C-208, C-211, C-224).
