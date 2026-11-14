# 021-16 — Core Audio edge capture
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** M · **Depends on:** 021-05, 021-12, 021-13, 021-14, epic 018, epic 024a
## Context
`coreaudio/capture.rs`: the physical-input capture module on the device I/O callback, using epic 018's edge module structure.

- Contract [decision]: capture stamps a sample with `T⁻¹(capture_time − D_capture)`, i.e. the position whose network time equals the capture host time minus the input-direction D.
- **Capture is physical device inputs only** [decision]; no virtual device is installed (epic criterion 021-AC8). Device admission by 021-11.
- Wire format: 96 kHz only (C-01); a 48 kHz device converts 2:1 through the kit's resampler node (C-80) and steerable rate actuator, both built by epic 024a; this module builds none (C-197). The fixed node is used only when the device is not at 96 kHz; the rate actuator tracks the device crystal at every device rate and its delay is also part of D_capture (C-189), through the kit's rate-actuator trait (C-210). The conversion delay, a whole number of domain-rate samples (C-105), is part of D_capture (C-134, 021-14) and never of the module's L; L is the module's structural delay in the domain-rate timeline, and the manifest reports the conversion delay separately, on the device-facing port (C-207). Slices of 48 frames (C-02), 4-byte samples (C-03), at most 6 channels per stream, 192 bytes per channel per slice, 1 152 bytes of audio at 6 channels, no per-channel id byte: the channel map is in the manifest (C-04). More than 6 inputs means more than one stream.
- Session numbers [decision]: the source assigns one session number per stream and bumps it on restart and on timeline epoch change (C-19). The capture edge is the source; core's producer path assigns the number, this module must restart the stream through that path, never reuse a stream across a clean end.
- Liveness: producer stops after 2 s without receiver reports (C-41). DSCP media 46 (C-13); don't-fragment on every packet, path-MTU fallback 1 280 bytes (C-06); IPv4 only, multicast TTL 1 (C-100).
## Work
- In the input callback, take the input host time and sample time, map through loop 2 and loop 1 to positions with `T⁻¹(capture_time − D_capture)`, resample to the domain rate through the kit's steerable rate actuator and, when the device is not at 96 kHz, its fixed resampler node (C-189, C-197), and hand slices to the producer path for sending.
- Group device inputs into streams of at most 6 channels and describe the channel map in the manifest.
- In the module manifest, report the conversion delay on the device-facing port, and keep it out of the module's L (C-207).
- No allocation, locking or syscalls in the callback.
- Clean end on guard events (021-12) and on 2 s without reports (C-41); every restart opens new streams with new session numbers.
## Acceptance criteria
1. Manual hardware test: capturing a physical input for 10 minutes into a Linux sink yields continuous slices with zero gaps on a quiet network.
2. Unit test with fake clocks: a sample captured at host time h is stamped with position `T⁻¹(h − D_capture)` to within one sample period.
3. With reports withheld, the producer stops after 2 s (±100 ms).
4. Only devices admitted by 021-11 with physical inputs can be opened.
5. Unit test: 8 device inputs produce two streams (6 + 2 channels) with a matching channel map; every packet carries 192 bytes per channel per slice.
6. After a clean end and restart, each stream's session number differs from the previous one (C-19).
7. Unit test on a 48 kHz fake device: the manifest reports the resampler's conversion delay on the device-facing port, the module's L excludes it, and the delay appears in D_capture (C-134, C-207); no resampler code exists in this crate (C-197).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
## Out of scope
Raw capture for the L2 harness (021-17). The resampler node and rate actuator (epic 024a, C-197). Certification measurements (021-23, 021-24). Session number assignment and packet layout (core and epic 004). Emitting the gap slice when the module misses its own deadline and raising the processing-overrun event (the hosted backend, epic 015, C-144).
