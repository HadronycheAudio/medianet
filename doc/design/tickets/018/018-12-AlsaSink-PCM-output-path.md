# 018-12 — AlsaSink PCM output path
**Epic:** 018-Linux-ALSA-edge · **Effort:** M · **Depends on:** 018-10, 018-11, epic 015, epic 016, epic 024a

## Context
- `AlsaSink` (file `hadronyche-os-linux/src/alsa/sink.rs`) is the Linux reference sink: position ring → resampler → PCM write; `submit(p)` scheduling; self-monitoring of E; alarms. This ticket builds the module shell and the resampler → PCM write path only.
- Rates: the domain rate is 96 kHz and is the only rate on the wire (C-01); 48-frame slices = 0.5 ms (C-02); 4-byte samples f32 or Q1.31 (C-03). A 48 kHz device is converted 2:1 at the edge by the kit's fixed-ratio resampler node (C-80); the ppm correction is applied by the separate steerable rate actuator (C-189). Both are the kit's, built by epic 024a; this epic builds no resampler (C-197).
- **Epic criterion 018-A8:** the sink runs unprivileged in best-effort mode and stamps what it lacks. Real-time setup comes from epic 016; hosted runtime/module model from epic 015. Strict mode and the capability states are ticket 018-25.
- **C-175:** if the device cannot be opened in the required mode because another process has it open, the module reports "the interface is held".

## Work
1. `AlsaSink` as a hosted-runtime module (epic 015): configuration = device name, device rate, sample format, channel count, buffer and period sizes.
2. Open PCM (an open refused because another process holds the device, `EBUSY`, is reported with the named reason "the interface is held", C-175), set hw params (format f32 or S32 for Q1.31, rate, channels, buffer, period), sw params (timestamps monotonic), start threshold.
3. Process loop: per period, pull frames through the fixed 2:1 node (48 kHz device only) and the kit's rate actuator (epic 024a, C-197), and write with `snd_pcm_writei` / mmap; xrun recovery with a counter.
4. Real-time setup via epic 016 when permitted; when not (unprivileged, strict mode not set), continue in best-effort mode and record each capability (real-time scheduling, memory lock, CPU idle restriction) with its state, present, absent, unverified or not requested (C-129), in a capability set for the stamp (018-18). Strict-mode refusal, and the refusal in every mode when seqlock safety depends on a failed or unverified CPU pin (C-177, C-201), are 018-25.
5. Test with a fake PCM backend (trait) and an optional `snd-aloop` lane.

## Acceptance criteria
1. With a fake PCM backend, 10 s of a 1 kHz sine at 96 kHz arrives at a 48 kHz backend at the right frequency (within 1 ppm, through the fixed 2:1 node with the actuator ratio fixed at 1.0, C-189).
2. **Epic criterion 018-A8 (part):** started as a non-root user without RT permissions, the sink runs, logs best-effort mode, and the capability set shows every missing RT feature as absent (C-129).
3. An injected xrun is recovered and counted; the stream continues.
4. Both f32 and Q1.31 formats are accepted; an unsupported device format is refused with a named error.
5. With a fake PCM backend returning `EBUSY` on open, the sink reports "the interface is held" (C-175) and does not start.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Position ring reading and `submit(p)` timing (018-13).
- Rate loop and E self-monitoring (018-14).
- Stamp fields (018-18).
- Strict mode (018-25).
