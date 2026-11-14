# 027-02 — Two-channel capture file reader
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-01

## Context
- The L3 tool `hadronyche-l3` (crate `hadronyche-tools`) analyses recordings made by one two-channel capture device: channel 1 = Linux ALSA reference sink output, channel 2 = embedded reference device output. One capture clock means skew is read directly as a time difference between the two channels, with no clock comparison `[decision]`.
- The analysis entry point takes interleaved frames: `measure_skew(capture: &[[f32; 2]], rate: u32, markers: &MarkerPlan)`. Something must turn a recorded file into `Vec<[f32; 2]>` plus its sample rate.
- The epic requires the skew analysis to run in CI on recorded two-channel files, and that every parser touched is fuzzed. This reader is a parser. Fuzz budget (C-95): each parser 60 s on every change and 1 hour nightly; the corpus is kept.
- The rig captures at 96 kHz, the domain rate and the only rate on the wire (C-01). The device's 48 kHz hardware converts at the edge (C-80) and does not affect the capture rate. The reader still takes the rate from the file header rather than assuming it.
- The recording format is still an open sub-question of the epic, owned by this ticket. Working choice: uncompressed WAV/RIFF (PCM 16/24/32-bit integer and 32-bit float). Record the choice in the module doc and in `l3-results.md` (027-20); falsifier: a capture tool in the rig cannot write it losslessly.

## Work
1. Module `capture.rs` in `hadronyche-l3` with `pub fn read_capture(bytes: &[u8]) -> Result<(Vec<[f32; 2]>, u32), CaptureError>` (the second value is the sample rate in Hz) and a file wrapper `read_capture_file(path)`.
2. Accept exactly 2 channels; reject any other channel count with a named error.
3. Convert integer PCM to `f32` in [−1, 1].
4. Never panic on malformed input; all errors are `CaptureError` variants (truncated header, unsupported format, wrong channel count, data length not a multiple of the frame size).
5. Fuzz target `fuzz_read_capture` (cargo-fuzz) over `read_capture`, with a seed corpus of a few small valid files; 60 s per change in CI, 1 hour nightly, corpus committed and kept (C-95).
6. A small writer `write_capture(frames, rate)` used by tests and by the synthetic generator (027-04).

## Acceptance criteria
1. Round trip: `read_capture(write_capture(x, 96000))` returns `x` exactly for float files and within 1 LSB for 24-bit files.
2. A mono file and a 4-channel file are rejected with the wrong-channel-count error.
3. The fuzz target runs 60 s on every change and 1 hour nightly with no crash (C-95); the corpus is kept in the repository.
4. A 60 s, 96 kHz, two-channel float file (5 760 000 frames) is read in under 2 s in CI. A one-hour file (345 600 000 frames, about 2.8 GB as float) is handled by a streaming or chunked read; tested manually, not in CI.
5. A file whose header states another rate is read and its rate returned unchanged (no resampling in the reader).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier (the format choice is recorded with its owner and falsifier)
- Not applicable (C-125): "Regime stamp fields touched" — none; "Design documents updated" — the format is recorded by 027-20; "Every new factual claim carries an evidence tag" — no factual claim beyond the tested round trip; "The skew analysis runs in CI on recorded two-channel files" — wired by 027-08; "The rig is photographed or diagrammed" — no rig work; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Marker detection (027-03).
- Live capture from the audio device (027-09, 027-11).
