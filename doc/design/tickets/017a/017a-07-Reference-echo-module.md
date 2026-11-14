# 017a-07 — Reference echo module
**Epic:** 017a-Reference-modules · **Effort:** M · **Depends on:** 017a-01, 017a-03, epic 009, epic 015

## Context
- The reference echo is the POC's synth (`src/synth.rs` with `music::Voices`, `src/music.rs:287`) `[code]` ported onto the runtime's module API (epic 015) as `hadronyche-reference/src/echo.rs`.
- Rules `[decision]`:
  - The reference echo is **arrival-driven** with a **single input**: it renders when the parts slice arrives.
  - Content domains it consumes: sparse keyed controllers (parts) and read-only transport state; it produces audio.
  - No libm in its signal path. Hand-written Rust first (DSL re-expression is task 025).
- Slice = 48 frames = 0.5 ms at 96 kHz, the only rate on the wire (C-01, C-02); 6 channels per stream, 192 bytes per channel and 1 152 bytes of audio per slice, no per-channel id byte, channel map in the manifest (C-04). Slices are addressed by position: `position = (network_time − epoch_origin) × rate`.
- **Structural latency (settled, C-105):** the echo's L is a whole number of domain-rate samples, declared in its manifest. The echo contains no resampler node: a resampler may appear only inside an edge module (C-134).
- **Degraded flag (settled, C-122):** stamps denote content position. While the echo processes input position p it emits the output stamped p − L, whose content came from input position p − L; that output is degraded if the input at p − L was gapped. The echo emits a degraded output slice for such a position rather than emitting nothing, so downstream sees a marked gap, which proves the upstream alive (C-102) and defers the dead threshold (C-111).
- **Multi-packet input (settled, C-15):** as an arrival-driven input, a multi-packet parts slice is reassembled within 2 slice periods; otherwise it is gapped.
- The echo is the "processor" of the diamond scenario (017b-12): its output feeds the mixer.
- Loss handling inherited from the POC `[code]`: stateful divergence heals itself (state refresh: parts re-send current values; oscillator phase resets when the gate is zero).
- **Transport state on a gap (settled, C-141):** the last known transport state is held and the playhead advanced only if that state is playing; a stopped transport's playhead does not advance (C-200; 017a-03's hold-and-advance); the output is marked degraded. Before any transport state has arrived, the view reads stopped, playhead at zero, 120 beats per minute (C-184, C-199).
- **Open sub-question (owner: epic 009):** when an arrival-driven module emits the degraded output for an input position that never arrives (on the next arrival, or at some deadline). Falsifier: consumers downstream of the echo count interrupted while the parts source is alive. Implement whatever epic 009 specifies; until then, emit on the next arriving slice and record the choice.
- **Open sub-question (owner: epic 017b, ticket 017b-11):** the resynchronisation bound after loss; inherited value 3 s (`SETTLE_SECS`).

## Work
1. Implement `echo.rs` against the module API of epic 015: one input (parts), one output (audio), using `instrument::voices` (017a-01) and the read-only transport view (017a-03).
2. Stamp each output slice with the content position of the input it came from: the output stamped q is emitted while processing input position q + L, L being the echo's declared structural latency, a whole number of samples (C-105, C-122).
3. Support float and fixed-point rendering modes, selected by configuration.
4. On a gapped input position (gap classes from epic 009, including reassembly expiry under C-15), emit a degraded output slice stamped with that position (C-122), hold the transport state and advance its playhead only while playing (C-141, C-200), and keep voice state; on the next slice, continue.
5. Header comment citing POC `src/synth.rs` and the epic 002 finding that cleared it.

## Acceptance criteria
1. Driven by a test harness with 20 000 parts slices (10 s), the echo's output is bit-identical to `instrument::voices` rendering the same parts offline, in float and in fixed mode.
2. Each output slice carries the content position of the input it came from, and the declared L is a whole number of samples (C-105, C-122).
3. For each dropped input slice, exactly one output slice is marked degraded, stamped with the dropped input's position (C-122).
4. After 100 dropped input slices and a parts re-send of current values, the output becomes bit-identical to the offline reference again within the resynchronisation bound (3 s until 017b-11 decides).
5. A parts slice split over several packets with one packet missing for more than 2 slice periods is gapped (C-15).
6. No libm call in `echo.rs` (CI check from 017a-01).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Each ported POC piece cites its source file and the archaeology finding that cleared it
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The mixer (017a-08) and sender (017a-09).
- L0 scenarios that use the echo (017b-10 onwards).
