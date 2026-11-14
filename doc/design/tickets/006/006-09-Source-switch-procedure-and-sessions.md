# 006-09 — Source-side switch procedure and session numbering
**Epic:** 006-Timeline-epoch-specification · **Effort:** S · **Depends on:** 006-05, 006-06, epic 004
## Context
Part of `doc/design/spec/timeline.md` (epic 006), section "Source-side switch procedure and session numbering". This procedure is what epic 010 implements.

Facts (`[decision]`):
- The holder is a clock server, not a stream source, so it cannot announce in-band. Each module's clock thread learns the new epoch (and the new line) from sync replies, and raises a timebase-discontinuity event (old epoch, new epoch, step size). Each of that module's **sources** then announces in-band, in its own streams, the future position at which it switches, and restarts its session there.
- An announcement names a future position, with a lead of 64 slices (C-43; 32 ms at defaults), and is re-issued every 16 slices (C-44; 8 ms at defaults) until the named position passes, in every bundle member. The lead is flat: the same 64 slices for every source at every depth of the graph (C-43).
- Session numbers (C-19): assigned by the source, one per stream; bumped on restart and on every timeline epoch change; every consumer of a stream sees the same value. Session is 32 bits, stream number 16 bits, timeline epoch 16 bits (C-10).
- Slice = 48 frames = 0.5 ms at 96 kHz (C-02), the only rate on the wire (C-01).
- Every stream of the source switches. A bundle source's controllers ride in their own member stream (C-205), which counts towards the 32 members (C-08, C-209), so "every bundle member" includes it; a single-stream source whose controllers do not fit the 100-byte header cap with its headers emits them in a second stream (C-181), which announces and switches like the audio stream, with its own session (C-19).
- Announcement formats and the slice-header epoch field come from epic 004.
- Sources pace position from network time, not from their local crystal.

## Work
1. Write the procedure as numbered steps: **learn** (discontinuity event from the clock thread) → **choose switch position** = current position + 64 slices (state: on which line "current position" is computed, and that the switch position is slice-aligned) → **announce** in every bundle member (the controller member included, C-205) and every other stream of the source (C-181) at the choice and every 16 slices until the named position passes → **switch**: at the switch position, end the old session and start the new one with a bumped session number (C-19), stamping slices with the new epoch.
2. State on which line the switch position is expressed (old epoch's or new epoch's) and how the source maps it to the new line; the map is never extrapolated across epochs, so this must be explicit.
3. Session numbering across a switch, applying C-19: the session is bumped (state the increment and its 32-bit wrap), the stream number (16-bit) is kept, and the first slice of the new session carries the new epoch and the new session. State that every consumer sees the same session value because the source alone assigns it.
4. Specify what happens when a second epoch change is learned before the first switch position is reached (supersede: re-announce with a new position, or chain), including how many session bumps result.
5. Count transmissions: with lead 64 and period 16, the announcement is sent at 64, 48, 32 and 16 slices before the named position: 4 transmissions.

## Acceptance criteria
1. The procedure has the 4 steps learn → choose (current + 64 slices) → announce (every 16 slices, every bundle member) → switch (end old session, start new with bumped session and new epoch).
2. The section states the 4 transmission offsets (64, 48, 32, 16 slices before the switch) and the lead in ms at defaults (32 ms).
3. Session numbering across a switch cites C-19 and has one worked example including the 32-bit wrap.
4. The superseding case (a second change before the switch) is specified.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Behaviour when the module is in Locking or holdover (006-10).
- Consumer side (006-11, 006-21) and the ordering/loss argument (006-12).
- Implementation in a source (epic 010); announcement wire format (epic 004).
