# 018-16 — AlsaCapture read path and stamping
**Epic:** 018-Linux-ALSA-edge · **Effort:** M · **Depends on:** 018-03, 018-10, 018-11, epic 004, epic 008, epic 015, epic 024a

## Context
- `AlsaCapture` (`hadronyche-os-linux/src/alsa/capture.rs`): PCM read → resampler → slices stamped with `T⁻¹(capture_time − D_capture)`. Capture is the mirror of the sink: a captured sample is stamped with the position of its acoustic instant `[decision]`.
- `position = (network_time − epoch_origin) × rate`; the wire carries only the 96 kHz domain rate (C-01), so a 48 kHz device is converted 2:1 at the edge (C-80); slices 48 frames = 0.5 ms (C-02); samples f32 or Q1.31 (C-03); a stream carries at most 6 channels, 192 bytes per channel per slice, no per-channel id byte, channel map in the manifest (C-04).
- Capture is a source: it assigns **one session number per stream**, bumped on restart and on timeline-epoch change; every consumer sees the same value (C-19).
- Capture direction ratio: device samples in, domain samples out (inverse of the sink); a 48 kHz device is converted 1:2, one of the two conversion ratios of v1 (C-105), by the kit's fixed-ratio resampler node; the ppm correction is applied by the separate steerable rate actuator around its nominal 1.0 (C-189). Both are the kit's, built by epic 024a; this epic builds no resampler (C-197). D_capture runs from acoustic instant to the sample being available and includes the conversion delay (C-134, C-170), which is never counted in L (C-207).
- When the capture misses its own deadline, the hosted backend (epic 015) emits the gap slice and raises the processing-overrun event (C-144).
- If the device cannot be opened because another process has it open, the module reports "the interface is held" (C-175). The same `DeviceLine` and `EdgeTimebase` from `edge/` are used.
- Epic criterion 018-A5: a marker played by an external source at a known network time is captured with a position within E of `T⁻¹` of that time minus D_capture (checked at L2, epic 020).
- AC-26 contribution: round-trip marker offset constant within ±2 samples (C-75).

## Work
1. Hosted-runtime module `AlsaCapture`: open capture PCM (an `EBUSY` open is reported as "the interface is held", C-175), hw/sw params, monotonic timestamps, read loop.
2. Pairs from status (018-10) → `DeviceLine`; ratio from `EdgeTimebase` (inverted for capture), with the fixed node's conversion factor removed, → `set_ratio` of the kit's rate-actuator trait (defined by epic 024a with its implementation, ratio range ±1 000 ppm around nominal, C-197, C-210), so the actuator receives only the ppm deviation around 1.0 (C-189); on a 48 kHz device the fixed 1:2 node completes the conversion.
3. Compute `D_capture` with `DeclaredDelay` (018-11) for the capture direction; stamp each output slice with its position via `T⁻¹(capture_time − D_capture)`, aligned to 48-frame slice boundaries.
4. Session number per stream (C-19): assign at start, bump on restart and on timeline-epoch change; on an epoch change restart stamping at the announced position.
5. Publish the channel map in the manifest (C-04); emit slices to the runtime (≤ 6 channels; wider handled in 018-17).

## Acceptance criteria
1. With a fake capture backend and fake clock, a marker at known network time t lands at position `T⁻¹(t)` ± 1 sample after accounting for D_capture.
2. With a +100 ppm fake crystal, the marker position stays within ±1 sample over 10 min of simulated time.
3. Output slices are exactly 48 frames on position boundaries, 192 bytes per channel.
4. A module restart and a timeline-epoch change each bump the stream's session number; nothing else does.
5. An L2 test procedure for epic criterion 018-A5 (marker within the target E, C-149; run in strict mode, C-147) is written and handed to epic 020.
6. With a fake PCM backend returning `EBUSY` on open, the capture reports "the interface is held" (C-175) and does not start.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Bundles for more than 6 channels (018-17).
- Producer health counters in the stamp (018-23).
- Running the L2 marker check (epic 020).
- Emitting the gap slice on a missed deadline and the processing-overrun event (epic 015, C-144).
- Strict mode (018-25).
