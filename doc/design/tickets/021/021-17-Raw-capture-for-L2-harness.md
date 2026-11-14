# 021-17 — Raw capture for the L2 harness
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-10, 021-11, epic 020
## Context
The L2 harness (epic 020) measures emission against `T(p) + A` by capturing the sink output with a raw capture path. On macOS it needs `impl RawCapture for CoreAudioRawCapture { open, read }` (trait from epic 020).
## Work
- `CoreAudioRawCapture::open(device, channels, rate)` and `read(buf) -> (frames, sample_time, host_time_ns)` delivering unprocessed input samples with their callback timestamps in the H basis, no resampling.
- Bounded ring buffer between the callback and `read`; report overruns explicitly.
## Acceptance criteria
1. The epic 020 harness runs against `CoreAudioRawCapture` on a reference Mac and captures a test marker.
2. Timestamps returned are in the H basis and strictly increasing; overruns are counted, never silent.
3. Unit test with a fake callback source.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
## Out of scope
The L2 harness itself (epic 020). L2 runs (021-23, 021-24).
