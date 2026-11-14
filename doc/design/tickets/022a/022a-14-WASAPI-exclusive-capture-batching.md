# 022a-14 — WASAPI exclusive-mode capture with batched emission
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** M · **Depends on:** 022a-06, 022a-07, 022a-10, 022a-11, 022a-12, epic 018, epic 024a

## Context
- Part of epic 022a (Windows edge backend), `wasapi/capture.rs`: exclusive-mode capture; per-callback batch of slices, each stamped from the device line; batch size recorded.
- **Capture is physical device inputs only** `[decision]`; application-output capture (virtual audio device = signed kernel driver) is out of v1.
- Capture stamps a sample with `T⁻¹(capture_time − D_capture)` `[decision]`. D_capture is a commissioned measured value read from the edge configuration; who measures it in the field is an open sub-question owned by epic 028.
- **Capture edges pace emission off the audio callback, batching several slices per engine period; this is stamped** `[decision]`: *pacing: batch per engine period* with the batch size. The format of the batch-size field is an open sub-question owned by epic 004 (stamp layout); this ticket records minimum, typical and maximum so any format can be filled.
- Exclusive-mode hand-off is quantised to the device period, about 3 to 10 ms `[memory]`; a capture edge therefore receives 6 to 20 slices (48 frames, 0.5 ms each, C-02) per callback and emits them as a batch. Positions stay exact, but lateness at consumers includes the period, so the group's A must cover it (C-50, C-51, C-52).
- One wire rate, 96 kHz (C-01): a 48 kHz device is converted to 96 kHz through the kit's fixed resampler node (C-80), one of the two ratios offered in v1 (C-105); the node exists only inside an edge module and its delay is part of D_capture, which mirrors D: from acoustic instant to the sample being available (C-134, C-170). The device crystal is tracked by the kit's steerable rate actuator at the slope-quotient ratio (C-189). Both come from the arithmetic kit, epic 024a; this ticket builds no resampler of its own (C-197). The rate actuator is driven through the kit's rate-actuator trait, with a ratio range of ±1 000 ppm around the nominal ratio (C-210).
- The conversion delay is never counted in L: L is the module's structural delay in the domain-rate timeline, and the manifest reports the conversion delay separately, on the device-facing port (C-207).
- Packet layout at defaults: one packet per slice (C-05), 6 channels of 192 bytes = 1 152 bytes of audio, no per-channel id byte; the channel map is in the manifest (C-04); all headers and the checksum take at most 100 bytes, so the slice fits 1 252 bytes of UDP payload (C-137). Packing is core; this ticket hands slices to it.
- The source assigns one session number per stream; it is bumped on restart (including the return from sleep, C-124) and on a timeline epoch change, and every consumer sees the same value (C-19).
- DSCP media 46 (C-13). Producer stops after 2 s without receiver reports (C-41). As a producer, the edge publishes per-connection summary counters in its own regime stamp (C-47, with the counter list of C-159), from the epic-018 edge structure.

## Work
1. `wasapi/capture.rs`: `WasapiCapture` on the epic-018 edge structure.
2. Capture thread (MMCSS, 022a-07): on each WASAPI event, read the device buffer, take (device position, QPC) from 022a-11, compute each sample's domain position as `T⁻¹(capture_time − D_capture)`, convert to 96 kHz through the kit's rate actuator and, for a 48 kHz device, its fixed node (C-80, C-189, C-197), driving the actuator through the kit's trait (C-210), cut into 0.5 ms slices, and send all of them back-to-back as one batch through `WinNet` media sockets.
3. Record batch size per callback (minimum / typical / maximum) for the stamp.
4. Session number per stream: assigned at stream start, bumped on restart and on a timeline epoch change (C-19).
5. Stop producing after 2 s without receiver reports (C-41).
6. Report the conversion delay of the kit resamplers in the capture module's manifest on its device-facing port, and keep it out of the module's L (C-207).
7. Tests with a mock device: positions exact across batches, no gap or overlap between consecutive batches.

## Acceptance criteria
1. With a mock device at a 10 ms period and 48 kHz, each callback yields 20 slices of 48 frames at 96 kHz, each 1 152 bytes of audio for 6 channels; positions are contiguous across 1 000 callbacks (no gap, no overlap).
2. Each slice's position equals `T⁻¹(capture_time − D_capture)` of its first sample within 1 sample (synthetic timebase).
3. Batch size statistics are exposed for the stamp.
4. A restart of the stream and a forced timeline epoch change each produce a new session number for that stream (C-19).
5. With receiver reports withheld, the producer stops after 2 s ± 100 ms (C-41).
6. Only physical capture endpoints are offered (loopback / render endpoints are rejected).
7. Conversion uses the epic-024a kit's node and rate actuator; the crate contains no resampler implementation (C-197).
8. For a 48 kHz mock device, the manifest reports the conversion delay on the device-facing port, the module's L does not include it, and the same delay is inside D_capture (C-207, C-170).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented

## Out of scope
- The one-hour consumer gap run at the commissioned A (022b-04).
- `RawCapture` for the L2 harness (022a-15).
- Packet header layout (epic 004).
