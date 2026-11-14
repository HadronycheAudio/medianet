# 023-24 — Expressiveness: reference echo instrument and metronome type-check
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-23, 023-30

## Context
- 023-AC-8: **the reference echo's instrument (voices driven by sparse keyed controllers, linear ramps over the slice, table sine, phase reset at gate zero) and a metronome reading transport state both type-check.**
- Sparse keyed controllers: one field of one voice moving linearly to a value over the slice, repaired by state refresh (the POC's musical events, `proto::Part` = voice, field, value, `src/music.rs:17-25` `[code]`).
- Transport state is read-only: playing, bar, beat, tempo, time code at position p. Transport authoring is deferred past v1. Its gap default is decided (C-141): the last known state is held and the playhead advanced as that state dictates (a stopped playhead does not advance, C-200), and the output is marked degraded; the metronome relies on that default and needs no override (023-30).
- Sine must come from a shipped deterministic table (no libm, no transcendental functions).
- Both programs are ordinary (non-edge) modules: their ports are at the domain rate, 96 kHz (C-01).
- This is also the falsifier of the 023-01 syntax decision: if either program needs an escape hatch, the decision is falsified and must be reopened.
- Epic 025 writes the full reference echo; this ticket writes test programs that prove the language can.

## Work
1. `tests/expressive/instrument.<ext>`: N voices (constant N), per-voice fields driven by sparse keyed controllers, linear ramp over the slice, table-sine oscillator, phase reset when gate field reaches zero, audio output.
2. `tests/expressive/metronome.<ext>`: reads transport playing/bar/beat/tempo, produces a click on audio output.
3. Integration tests running `check()` on both.

## Acceptance criteria
1. Both programs return `Ok` from `check()` with 0 diagnostics.
2. Neither program uses an escape hatch (no raw Rust, no foreign calls).
3. The instrument's manifest determinism class is `ExactFixed` if written in Q1.31 or `ExactFloat` if it uses f32, never `Bounded` (C-123), and the test records which.
4. The metronome's manifest lists a transport-state input port and no transport-state output, and no gap override on that input (C-141).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The full reference echo application and its conformance vectors (epic 025).
