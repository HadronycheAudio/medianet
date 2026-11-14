# 023-06 — Content types, rates and channel counts model
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-02, epic 004

## Context
- Part of `hadronyche-dsl/types/`: content types (audio f32 / Q1.31, dense controller, sparse keyed controller, transport state), rates, channel counts; checking (checking itself is 023-08 and 023-29).
- Content domains, v1 `[decision]`:
  - **Audio**, samples f32 or Q1.31, 4 bytes each (C-03).
  - **Dense controllers**: one value per tick.
  - **Sparse keyed controllers**: one field of one voice moving linearly to a value over the slice, repaired by state refresh (the POC's musical events, `proto::Part` = voice, field, value `[code]`).
  - **Read-only transport state** as an input: playing, bar, beat, tempo, time code at position p. Input only; no output port of this type (transport authoring deferred past v1).
- Rates: the domain rate, 96 kHz, is **the only rate on the wire** (C-01). Another rate (typically 48 kHz, C-80) appears only on the device-facing side of an edge module and is declared there, so it is a value, not a constant (C-105). Slice = 48 frames = 0.5 ms at 96 kHz (C-02).
- Channels: 6 channels per stream at defaults, 192 bytes per channel per slice, 1 152 bytes of audio per slice; **no per-channel id byte — the channel map is in the manifest** (C-04). Wider ports become bundles of at most 32 members, a source's controller member included (C-08, C-209); at most 256 streams per source (C-09).
- These types must line up with the content message definitions of epic 004.

## Work
1. `types::ContentType { Audio(SampleFormat::{F32, Q1_31}), DenseController, SparseKeyedController, TransportState }`.
2. `types::Rate` as a whole number of hertz, with a helper `ratio_to_domain()` that returns 2:1 for 48 kHz and fails for any rate the kit's resampler cannot convert (only 2:1 and 1:2 exist in v1, C-105; no rational-ratio arithmetic is needed).
3. `types::Channels(u32)` and constants `DOMAIN_RATE_HZ = 96_000` (C-01), `SLICE_FRAMES = 48` (C-02), `SAMPLE_BYTES = 4` (C-03), `STREAM_CHANNELS = 6` (C-04), `CHANNEL_BYTES_PER_SLICE = 192` (C-04), `STREAM_AUDIO_BYTES_PER_SLICE = 1_152` (C-04), `MAX_BUNDLE_MEMBERS = 32` (C-08; the controller member included, C-209), `MAX_STREAMS_PER_SOURCE = 256` (C-09). The two byte constants are derived from the others and a test asserts it.
4. Map each content type to its epic 004 message definition (a comment or a conversion function).

## Acceptance criteria
1. The 8 constants above exist with exactly these values and their C-nn ids in doc comments; there is no device-rate constant.
2. A test asserts `CHANNEL_BYTES_PER_SLICE == SLICE_FRAMES * SAMPLE_BYTES` and `STREAM_AUDIO_BYTES_PER_SLICE == CHANNEL_BYTES_PER_SLICE * STREAM_CHANNELS`.
3. `Rate` equality is exact: 96 000 Hz ≠ 48 000 Hz; `ratio_to_domain(48 000)` is 2:1; `ratio_to_domain(44 100)` fails (C-105).
4. `TransportState` is rejected as an output content type by a helper `is_valid_output()` (unit test).
5. Each of the 4 content types has a unit test of its size per sample/value where defined (audio = 4 bytes, C-03).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Bundle layout computation (023-07); the checker (023-08); edge-module rate rules (023-29).
