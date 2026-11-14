# 018-10 — ALSA status pairs and DeviceClock
**Epic:** 018-Linux-ALSA-edge · **Effort:** S · **Depends on:** 018-01, epic 015

## Context
- Loop 2 (S → H) regresses over (device frame count, host time) pairs supplied by the audio API. **On ALSA these come from the PCM status: hardware pointer, delay, and its timestamp** `[decision]`. Host clock H is `CLOCK_MONOTONIC`.
- Core types (018-01): `DevicePair { frames: u64, host_ns: i64 }`; trait `DeviceClock { fn nominal_rate(&self) -> u32; fn declared_jitter_ns(&self) -> u32; }`.
- Crate `hadronyche-os-linux`, module `alsa/`, file `status.rs`. Hosted runtime from epic 015.
- The device may run at 96 kHz or at 48 kHz; the wire is always at the 96 kHz domain rate (C-01) and a 48 kHz device is converted 2:1 at the edge (C-80). `nominal_rate` is the device rate, not the domain rate.
- Fuzz budget for every parser: 60 s on every change and 1 hour nightly, corpus kept (C-95).

## Work
1. `status.rs`: open-handle helper that configures the PCM timestamp mode to `CLOCK_MONOTONIC` (`tstamp_type` monotonic, timestamp mode enabled) and reads `snd_pcm_status` (hw pointer, delay, `htstamp`, and `audio_tstamp` if available).
2. Convert to `DevicePair`: frames = monotonic 64-bit frame count extended from the wrapping hw pointer (handle boundary wrap); host_ns from the status timestamp. Document whether frames refer to the converter (hw_ptr + delay) or the DMA pointer, and keep it consistent with D (018-11).
3. `AlsaDeviceClock` implementing `DeviceClock`: `nominal_rate` = configured device rate; `declared_jitter_ns` = configured or measured pair jitter.
4. Abstract the status source behind a trait so the converter logic is unit-testable without hardware; add a fake source.

## Acceptance criteria
1. Unit test: a fake status source with the hw pointer wrapping at the ALSA boundary yields monotonically increasing `frames` over 10⁹ frames.
2. Timestamps not in `CLOCK_MONOTONIC` are rejected with a named error.
3. On a machine with `snd-aloop` or `snd-dummy` (optional CI lane, skipped when absent), 60 s of pairs fed to `DeviceLine` give a slope within 200 ppm of nominal.
4. Runs unprivileged.
5. The status-to-pair fuzz target runs 60 s in the per-change CI job and 1 hour in the nightly job, with its corpus committed (C-95).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed (the status-to-pair conversion is fuzzed with arbitrary status records)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- D computation (018-11), PCM write path (018-12), capture (018-16).
