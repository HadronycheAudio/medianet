# 017a-02 — Port the phrase generator
**Epic:** 017a-Reference-modules · **Effort:** S · **Depends on:** 017a-01, epic 002

## Context
- The reference sender produces musical-event *parts* for the echo (synth). A part is one field of one voice moving linearly to a value over the block — a sparse keyed controller, repaired by re-sending current values `[code]` (POC `src/music.rs`).
- Deliverable: `hadronyche-reference/src/instrument/phrase.rs`, ported from `music.rs`.
- Rules `[decision]`: hand-written Rust; no libm in any reference module's signal path; a ported piece cites its POC source and the archaeology (epic 002) finding that cleared it.

## Work
1. Port the phrase/part generation code from POC `music.rs` into `instrument/phrase.rs`, producing the parts consumed by `instrument::voices` (017a-01).
2. Make the generator deterministic from an explicit seed and a block position (no wall-clock or global RNG), so sender and replica produce identical parts.
3. Provide "current values" re-send of every keyed field (repair after loss) as an explicit function.
4. Header comment citing POC file/lines and the epic 002 finding.

## Acceptance criteria
1. Same seed and same position range give identical part sequences in two runs and in debug vs release.
2. Parts generated for 10 s at 0.5 ms slices (C-01, C-02: 48 frames at 96 kHz) match the POC's parts on epic 002's captured fixture, if provided.
3. The "current values" output, applied to a fresh `Voices`, yields the same state as applying the full history (test over 1 000 random positions).
4. No libm call in `phrase.rs` (same CI check as 017a-01).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Each ported POC piece cites its source file and the archaeology finding that cleared it

## Out of scope
- Voice rendering and the sine table (017a-01); the sender module (017a-09).
