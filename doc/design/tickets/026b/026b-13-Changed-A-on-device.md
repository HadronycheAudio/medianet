# 026b-13 — Changed A on the device
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** S · **Depends on:** 026a-07, 026a-08, 026b-12, epics 009, 012, 014

## Context

The device's alignment ring is sized to the presentation group's A (C-96) and held in a fixed array bounded by the declared buffer memory (026a-07): 23 040 bytes for one 6-channel stream at A = 10 ms. Admission refuses an A that does not fit at admission time. Rules settled after the last refinement cover a change of A while the stream is live:

- How the change reaches the device: a commissioned change of a sink's own A comes through the commissioning tool, inside a declared commissioning window (C-153); the tool validates buffer memory first (C-211) and then announces the new A to the group's members (C-224). The device takes the announced value through core admission (epic 012, ticket 012-30).
- A changed while inputs are live: every fan-in that inherited the old A re-enters aligning at an announced position, with rings and ratchet ceiling resized, as the sinks do (C-186). The device is a sink: it switches at its own position, one lead (C-43) ahead, as a fan-in does (C-208), and has nobody to announce it to (C-214). Its ring and ratchet ceiling are resized at that position.
- Carrier of a changed A upstream: the receiver report (C-40) carries the consumer's current A, its own or the one it inherited (C-198). The device sends receiver reports (026a-08), so they carry its current A.
- A larger than the buffer memory: the commissioning tool checks declared buffer memory before applying a change of A and refuses one that does not fit. If one reaches the consumer anyway, it keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm (C-174) and keeps its streams running (C-211). The clamp applies to any consumer that buffers to meet A, sinks included (C-222), so it applies to the device; on the device the deepest ring it can hold is the one its declared buffer memory allows. In detail (C-216, applied to a sink by C-222): the alarm stays raised until a later change of A that fits, and the receiver report carries the A the device was given, not the clamped value, so upstream modules size themselves to what the sinks need.
- A ratchet step in the device follows the same position rule: at its own position one lead ahead (C-225); the ceiling it steps towards is the A in force, or the clamped ceiling.

Core implements re-alignment, ring sizing and the alarm condition (epic 009) and the receiver report (epic 014); this ticket wires them into the device's static storage and proves the behaviour on the device build. No allocation: resizing means using more or less of the fixed array, never growing it.

## Work

1. Feed the announced A (C-224) through core admission into the device: re-enter aligning at the device's own position one lead (C-43) ahead, unannounced (C-214), resize the ring within the fixed array and set the ratchet ceiling to the new A (C-186).
2. When the new A needs more than the declared buffer memory, cap the ring and the ceiling at what the declared memory holds, raise cannot-meet-A through core's alarm interface (C-174, 026b-12) and keep the stream playing (C-211, C-222); keep the alarm raised until a later change of A that fits (C-216).
3. Carry the device's current A in every receiver report it sends (C-198), using epic 014's field; when clamped, that is the A it was given, not the clamped value (C-216).
4. Make sure the declared buffer memory is in the device stamp, where the commissioning tool reads it for its check (C-211, 026a-04, 026a-11).

## Acceptance criteria

1. Host test: with the stream live at A = 10 ms, a change to A = 5 ms received when the device is at position q makes it re-enter aligning at its own position q + 64 slices (one lead, C-43, C-214), announcing nothing, with the ring and ratchet ceiling sized to 5 ms; no gap other than those core's aligning state dictates (C-186).
2. Host test: the first receiver report after the change carries the new A; every report before it carries the old one (C-198).
3. Host test: a device declaring 23 040 bytes that receives A = 20 ms keeps a 10 ms ring as its ceiling, raises cannot-meet-A, and keeps the stream running (C-211, C-222, C-174); its receiver reports carry 20 ms, not 10 ms; the alarm stays raised until a later change to A = 10 ms, which clears it (C-216).
4. The decoded device stamp carries the declared buffer memory (C-211).
5. Device build: no alloc, no libm symbols.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The crate builds for at least one device target and for the host (for tests) in CI

## Out of scope

- The commissioning tool's buffer-memory check (commissioning tooling, C-211).
- Announcing the re-alignment position: the device has no output stream, so it announces nothing (C-214); fan-ins that output announce their own (C-208).
- The announcement message from the commissioning tool and its encoding (commissioning tooling and the wire specification; C-224).
- Admission-time refusal on memory (026a-07, 026b-18).
