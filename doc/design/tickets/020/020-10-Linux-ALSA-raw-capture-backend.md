# 020-10 — Linux ALSA raw capture backend
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-02, epic 018

## Context
The L2 harness certifies the Linux ALSA reference sink first. It needs a Linux implementation of the measurement-only capture trait (020-02):

```rust
pub trait RawCapture {
    fn open(device: &str, rate: u32, channels: u16) -> std::io::Result<Self> where Self: Sized;
    fn read(&mut self, out: &mut [f32]) -> std::io::Result<RawBlock>;
    /// True if any rate conversion is in the capture path; an unknown answer is true.
    fn resampler_in_use(&self) -> std::io::Result<bool>;
}
pub struct RawBlock { pub frames: usize, pub first_frame_count: u64, pub host_ns: i64 }
```

The resampler query is part of the epic's trait.

**Shared-crystal trap** `[decision]`: a loopback from a sound card's output to its own input shares one crystal; if both directions resample, the two resamplers cancel each other's error. Capture must be read raw against the hardware sample counter with no capture resampler. **Epic criterion 020-A4 (trap 2 demonstrably avoided): the harness refuses to run if the capture path reports a resampler in use.**

The domain rate is 96 kHz and it is the only rate on the wire (C-01) `[decision]`. The capture device is opened at its own hardware rate (the reference rig runs it at 96 kHz, 020-21); any rate conversion in the capture path is a refusal.

## Work
- Implement `AlsaRawCapture: RawCapture` on a direct hardware PCM (no `plug`, `dmix`, `dsnoop` or rate plugin in the path), reusing the ALSA plumbing of epic 018 where possible.
- `first_frame_count`: the hardware frame position of the first frame of each block; `host_ns`: the host time at which that counter value was valid (from the ALSA timestamp facilities, with the timestamp type documented).
- Resampler query: report `true` if the PCM chain contains any rate-converting plugin or if the opened hardware rate differs from the requested rate; unknown means `true`.
- In the runner entry point, refuse to start any scenario when the query returns `true`, with exit status non-zero and a message naming the plugin or rate mismatch.
- Detect and report overruns (gap in `first_frame_count`).

## Acceptance criteria
1. Opening a device through a rate-converting plugin (e.g. a `plug` PCM at a rate the hardware does not support) makes `hadronyche-l2 run steady` refuse to run with a message naming the resampler (epic criterion 020-A4).
2. On a direct `hw:` device at 96 kHz the query reports `false`, and consecutive `RawBlock`s have `first_frame_count[n+1] = first_frame_count[n] + frames[n]` for 10 minutes without overrun, on a Linux machine with a 96 kHz-capable sound card (the intended reference card if already chosen).
3. The regression of `first_frame_count` against `host_ns` over 10 minutes has residuals documented with their measured standard deviation `[measured]`.
4. Unit tests for the plugin-chain inspection run in CI without audio hardware (using a mocked or file-described PCM configuration).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The Linux sink itself and the two-loop core (epic 018).
- macOS and Windows backends (epics 021, 022).
- Real-time setup and wake-latency measurement (epic 016).
