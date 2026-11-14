# 007b-16 — Gap semantics vectors: transport state and dense controllers
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** S · **Depends on:** 007a-07, 007b-15

## Context
- Read-only transport state and dense controllers are content messages (valid encodings in 007a-07). What a consumer does when a slice carrying them is gapped is settled `[decision]`:
  - transport state on a gap: the last known state is held and the playhead advanced as that state dictates: a stopped transport stays where it is, a playing one moves on at its tempo; the output is marked degraded (C-141). A stopped playhead does not advance through a gap (C-200);
  - dense controller with no prior value: each dense controller declares an initial value in the manifest (default 0), used when a gap precedes any received value (C-158); the reference dense controller is the mixer's volume (C-192);
  - where controllers travel (C-181, C-205): a bundle source's controllers always ride in their own member stream, so a gap in that member gaps the controllers while the audio members are unaffected; a single-stream source may always put its controllers in a second stream, and must do so when headers plus controllers exceed the 100-byte cap (C-137); it may share the audio packet while they fit. The compiler checks this from the manifest (C-181 revised);
  - transport state before any has arrived: stopped, playhead at zero, default tempo (C-184), which is 120 beats per minute (C-199).
- Every stream is at the domain rate of 96 000 Hz (C-01); a slice is 48 frames at defaults (C-02).
- The vectors are stored in `vectors/transport/gap-*.json` beside the POC transport scenario and fed through the driver path added in 007b-15.

## Work
1. Write transport-state gap vectors, each for a playing and for a stopped known state: the known state, then one gapped slice, then several consecutive gapped slices; for a playing state the expected state is the last known state with the playhead moved on at its tempo by 48 positions per gapped slice (C-141); for a stopped state the expected state is the last known state with the playhead unchanged (C-200); in both, every output slice over the gap is marked degraded; a fresh state after the gap replaces the held one; a gap before any transport state has arrived: the expected state is stopped, playhead at zero, tempo 120 beats per minute (C-184, C-199), and the playhead stays at zero through the gap (C-200).
2. Write dense-controller vectors on the reference mixer's volume (C-192): a gap before any received value, with a manifest that declares an initial value and with one that declares none; the expected value is the declared initial value, or 0 when none is declared; a gap after a received value is handled as the tables state. Each vector is run with the controller carried as the bundle's controller member (gap in that member only, audio members intact) and, for a single-stream source, both in the audio packet and in a second stream while headers plus controllers fit the cap (C-181, C-205).

## Acceptance criteria
1. Across a gap of n slices the expected playhead of a playing transport advances by n × 48 positions, that of a stopped transport does not advance (C-200), and the held state is otherwise unchanged.
2. Every output slice produced over a transport-state gap carries the degraded flag.
3. The no-prior-value vectors expect the declared initial value, and 0 when the manifest declares none, for the mixer's volume.
4. All files carry `spec_version`.
5. The vector with a gap before any transport state expects stopped, playhead at zero throughout the gap and a tempo of 120 beats per minute (C-184, C-199, C-200).
6. The controller-member vectors expect the gap semantics on the controller value only, with no degraded audio member, when only the controller member is gapped (C-181, C-205).
7. For a single-stream source both carriages are accepted while the controllers fit the cap, and each gives the same expected controller value across a gap; no vector expects a single-stream source that fits the cap to be rejected for using a second stream (C-181).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The POC transport scenario (007b-15).
- Audio gap handling in the reference mixer (007b-18).
- The compiler's manifest check of where controllers travel (C-181; the compiler's epic).
