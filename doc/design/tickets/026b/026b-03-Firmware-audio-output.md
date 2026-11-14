# 026b-03 — Firmware audio output
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** M · **Depends on:** 026b-02, 026a-10

## Context

The reference device receives one whole 6-channel stream — the unit of subscription; the wire has no per-channel id and the channel map is in the manifest (C-04) — and plays the channels its codec has: 2, or 6 if the codec allows. This ticket implements `AudioOut` on the board's codec (chosen in 026b-01).

```rust
pub trait AudioOut {
    fn rate(&self) -> u32;                   // 48_000 in v1
    fn write(&mut self, frames: &[i32]) -> usize;
    fn position(&self) -> DevicePair;        // (frame count, timer-captured device time)
    fn fixed_delay_ns(&self) -> u32;         // converter and interface part of D
}
```

Facts: the wire is at 96 kHz only (C-01); the device clock is 48 kHz and the edge converts 2:1 (C-80); 4-byte samples (C-03). Loop 2 regresses (sample count, timer-captured time) pairs; on a device, a **timer capture of the audio DMA position**. The audio edge (026a-10) computes `submit(p) = T(p) + A − D`.

## Work

1. Codec init (I2S/SAI or equivalent) at 48 kHz, 32-bit slots, circular DMA.
2. `write` copies into the DMA ring without blocking; returns frames accepted.
3. `position`: a hardware timer captures the device clock at a DMA half/complete event (or reads the DMA counter atomically with the timer); returns (frame count, time).
4. `fixed_delay_ns`: from codec datasheet plus interface latency; tagged `[memory]` until measured in 026b-14.
5. Wire into `Device` in `examples/reference-device/`; channel selection from the manifest channel map; 2-channel output (6 if the codec allows).
6. Rebuild image, symbol check still passes.

## Acceptance criteria

1. With a host producer streaming a 6-channel stream carrying a 1 kHz sine on the played channels, the board outputs a clean 1 kHz tone on each played channel for 10 minutes with zero ring underruns reported.
2. `position()` pairs regress to a rate within ±100 ppm of 48 000 Hz vs device clock, with residual jitter recorded `[measured]` in `doc/design/reports/` (C-130).
3. Loop 2 converges: rate-actuator ratio settles within 60 s and the reported loop-2 error stays below 1 frame (20.8 µs).
4. Firmware image passes the no-alloc / no-libm check.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] The crate builds for at least one device target and for the host (for tests) in CI
- [ ] Every new factual claim carries an evidence tag
- [ ] The reference board, its timestamp source and its oscillator are named in the docs

## Out of scope

- Capture source (not a deliverable of this epic).
- D and E measurement (026b-14); memory and CPU measurements (026b-04, 026b-05).
