# 004-12 — Content messages: sparse keyed controllers and transport state
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-11
## Context
Content messages (rule 5 `[decision]`) include **sparse keyed controllers** — one field of one voice moving linearly to a value over the slice — and **read-only transport state**. They use the common content-message header defined in 004-11.

Facts:
- C-02: slice length 48 frames, at 96 000 Hz, the only rate on the wire (C-01).
- No constant gives the full field set of read-only transport state; this ticket decides it (open item owned by epic 004; falsifier: a producer cannot express a value it must send without a major change). The set includes a tempo: before any transport state has arrived a receiver assumes stopped, playhead at zero, tempo 120 beats per minute (C-184, C-199). This is receiver behaviour; the message needs no "unset" value.
- C-205: the controllers of a bundle source always ride in their own member stream, so sparse keyed controller messages from a bundle source never appear in an audio member's slices. A single-stream source may always put its controllers in a second stream and must do so beyond the 100-byte cap of C-137; it may share the audio packet while they fit (C-181, revised: permissive).
- C-141, C-200: on a gap the receiver holds the last known transport state and advances the playhead as that state dictates: a stopped transport stays where it is, a playing one moves on at its tempo; the output is marked degraded. This is receiver behaviour; the transport state message needs no gap value.
- The POC's 8-byte musical-event part `Part` is a reasonable starting point for keyed entries `[code]`.
- Every message has a fixed maximum size; parsing is bounded and allocation-free (`no_std` core) `[decision]`.

## Work
1. Sparse keyed controller entry: voice key, field id, target value, value format, and any start offset within the slice; linear ramp over the slice; maximum entries per message.
2. Read-only transport state: fields (playing/stopped and tempo, C-184, C-199, plus e.g. reference position; decide the rest of the set and tag `[decision]`), widths, units and ranges; the tempo field can express the 120 beats-per-minute default; state that receivers never write it back.
3. Validity rules with outcomes: entry count above maximum, length mismatch (`BadLength`), unknown field id, offset beyond slice frames, duplicate (voice, field) in one slice, invalid transport state value, a sparse keyed controller message in an audio member stream of a bundle source (C-205).
4. Add tables to the machine-readable file.

## Acceptance criteria
1. Sparse keyed entry table with byte size and maximum entries per message.
2. Transport state table with byte size, including a tempo field (C-199).
3. At least 6 invalid forms with outcomes across the two types.
4. Passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Audio and dense controllers (004-11).
- Transport reference change announcement (004-09).
