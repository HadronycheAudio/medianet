# 020-02 — RawCapture trait and recording format
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-01

## Context
The L2 harness measures emission from a physical sink by capturing the device output back. To avoid the *shared-crystal trap* `[decision]` (a loopback from a sound card's output to its own input shares one crystal; if both directions resample, the two resamplers cancel each other's error), capture is read **raw against the hardware sample counter, with no capture resampler**.

Per-platform backends (Linux in 020-10, macOS in epic 021, Windows in epic 022) implement one trait. The epic gives its shape (names may be refined) `[decision]`:

```rust
/// Raw capture for measurement: frames plus the device's own frame counter and
/// the host time of that counter. No resampling.
pub trait RawCapture {
    fn open(device: &str, rate: u32, channels: u16) -> std::io::Result<Self> where Self: Sized;
    fn read(&mut self, out: &mut [f32]) -> std::io::Result<RawBlock>;
    /// True if any rate conversion is in the capture path; an unknown answer is true.
    fn resampler_in_use(&self) -> std::io::Result<bool>;
}
pub struct RawBlock { pub frames: usize, pub first_frame_count: u64, pub host_ns: i64 }
```

The analysis must run in CI on recorded capture files with no audio hardware, so this ticket also defines a recording format and a file-backed implementation of the trait.

Epic criterion 020-A4 (trap 2) requires the harness to refuse to run if the capture path reports a resampler in use. The epic's trait carries `resampler_in_use` for this; an unknown answer counts as "in use".

## Work
- In `hadronyche-tools`, define `RawCapture` and `RawBlock` as above.
- Include the resampler query `resampler_in_use` in the trait, with the rule that an unknown answer is treated as "in use".
- Define a versioned recording format for raw capture: a header (format version, device name, device rate, channels, sample format f32, resampler flag) followed by a sequence of blocks each carrying `frames`, `first_frame_count`, `host_ns` and the interleaved samples. Side channels needed by the analysis (injected ground truth, steered resampler ratio samples, the servo's drift estimate and correction term as separate series (C-119), marker schedule, the run's audition offset A and the inputs it was computed from (C-148, ticket 020-26), the strict-mode capability states read from the sink's stamp (C-129, ticket 020-27)) are stored as additional typed records in the same file, so a recording is self-sufficient.
- Implement `FileCapture: RawCapture` that replays a recording, and `Recorder` that wraps any `RawCapture` and writes what it reads.
- Fuzz the recording parser (cargo-fuzz target `recording_parse`) with the budget of C-95: 60 s on every change and 1 hour nightly, with the corpus kept (C-95); the CI jobs are 020-25. The parser must never panic, only return errors.
- Document the format in the module doc comment.

Rates: the domain rate is 96 kHz and it is the only rate on the wire (C-01); at that rate one sample is 10.4 µs `[decision]`. A capture device may run at another rate (for example 48 kHz, C-80), so the header records the device rate; capture is never resampled.

## Acceptance criteria
1. A recording written by `Recorder` from a synthetic source of 10 blocks is replayed by `FileCapture` with identical `frames`, `first_frame_count`, `host_ns` and bit-identical samples.
2. `FileCapture::open` on a file with an unknown format version returns an `io::Error` and does not panic.
3. The fuzz target `recording_parse` runs 60 s per change and 1 hour nightly with zero panics (C-95, jobs from 020-25); a seed corpus of at least 3 valid recordings is committed and the corpus is kept.
4. The resampler query exists on the trait, and `FileCapture` reports the value stored in the recording header.
5. All tests run unprivileged without audio hardware.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The analysis code runs in CI on recorded capture files (no audio hardware needed)
- [ ] Every new factual claim carries an evidence tag
- [ ] Recording format documented in the module docs

## Out of scope
- Real device backends (Linux: 020-10; macOS and Windows: epics 021, 022).
- The refusal logic itself in the runner (020-10 and 020-14 use the query).
- The CI fuzz jobs (020-25).
- Synthetic capture generation with known errors (020-08).
