# 021-15 — Core Audio edge sink
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** M · **Depends on:** 021-05, 021-12, 021-13, 021-14, epic 018, epic 024a
## Context
`coreaudio/sink.rs`: the edge sink module on the device I/O callback, using the shared edge module structure and two-loop timebase of epic 018.

- Contract [decision]: `emit(p) = T(p) + A`; `submit(p) = emit(p) − D`. A is the presentation group's audition offset (C-55); D is the declared plumbing delay (021-14). `position = (network_time − epoch_origin) × rate`.
- Loop 1 maps H to the domain line; loop 2 maps S to H; the ratio of the kit's steerable rate actuator = quotient of the slopes; rate-loop bandwidth well below 1 Hz (C-68).
- Wire format: 96 kHz only (C-01); 48-frame slices (C-02); 4-byte samples (C-03); at most 6 channels per stream, 192 bytes per channel per slice, no per-channel id byte — the channel map comes from the manifest (C-04). A 48 kHz device is fed through the kit's resampler node (C-80), which may appear only inside an edge module such as this one (C-134). The fixed 2:1 / 1:2 node and the steerable rate actuator are both the kit's (epic 024a); this module uses them and builds none (C-197). The fixed node is instantiated only when the device is not at 96 kHz; the steerable rate actuator, whose ratio varies by parts per million to track the device crystal, runs at every device rate and its delay is part of D (C-189); its trait, with a ratio range of ±1 000 ppm around nominal, is the kit's (C-210). The conversion delay is counted only in D, never in the module's L; L is the module's structural delay in the domain-rate timeline, and the manifest reports the conversion delay separately, on the device-facing port (C-207).
- The sink's alignment ring is sized to the presentation group's A (C-96). When A changes while the sink is live, it re-enters aligning with its ring and ratchet ceiling resized (C-186). It switches to the new A at a position it picks itself, its own current position plus one lead (C-43, 64 slices, 32 ms at defaults), as a fan-in does (C-208); unlike a fan-in it announces nothing, since it has nobody downstream to announce it to (C-214); its receiver reports carry its current A (C-198). A ratchet step is taken the same way, at its own position one lead ahead, unannounced (C-225); a change of A that does not fit the sink's buffer memory is clamped as for a fan-in, with cannot-meet-A held until a later change that fits and the given A carried upstream (C-222, C-211, C-216). Core and epic 018's edge structure drive all of these; this module must let the ring be resized without breaking the callback rules below.
- Liveness rules unchanged: interrupted after 8 consecutive missed slices (C-38), counting only slices absent at the deadline (C-102); dead 2 s after the last slice with content or a marked gap (C-39, C-111); receiver report per bundle every 100 ms (C-40); producer stops after 2 s without reports (C-41); 50 sync requests/s (C-36). A slice absent at the deadline counts missing and is reclassified late if it arrives while still in the ring span (C-108). Core keeps these counters and publishes them in the stamp (C-47).
- Admission by E [decision]: only the certification state *commissioned* gives a numeric E; while *uncertified* or *self-monitored* this sink declares E as **unknown** (C-140); it is refused with reason `RequiredEExceeded` from any presentation group that sets a required E and admitted to groups that set none (C-103). The refusal itself is core admission.
## Work
- Wire `MacNet` sockets (021-04/05), `MachClock`, `CoreAudioDevice` and declared D into epic 018's edge sink skeleton.
- Declare the sink's E: *unknown* by default; a numeric value only when the certification state is *commissioned* (C-140; 021-20, 021-26).
- Size the alignment ring from the group's A, and resize it on a change of A (C-186) outside the I/O callback (for example by preparing the new ring off the callback and swapping it at the switch position). The sink picks that position itself, one lead (C-43) ahead of its own current position, and announces nothing (C-214).
- Instantiate the kit's steerable rate actuator inside this module at every device rate, its ratio from loop 2; when the device is not at 96 kHz, also instantiate the kit's fixed resampler node (epic 024a, C-189, C-197, C-210).
- In the module manifest, report the conversion delay on the device-facing port, and keep it out of the module's L (C-207).
- In the I/O callback: for the output host time of the buffer, compute the positions to fill so that `submit(p) = T(p) + A − D`; pull resampled slices; write silence for missing slices and report misses to core's counters.
- No allocation, locking or syscalls in the callback.
- Clean end on guard events (021-12).
## Acceptance criteria
1. Manual hardware test: a 96 kHz, 6-channel stream from a Linux producer plays on a 48 kHz built-in output for 10 minutes without audible glitches and with zero missed slices on a quiet network.
2. Receiver reports are sent every 100 ms (±10 ms) per bundle (C-40), verified by capture.
3. Unit test with a fake device and fake clocks: the first sample written for position p appears at host time `T(p) + A − D` to within one sample period.
4. Callback path passes an allocation/lock checker in tests.
5. Unit test on a 48 kHz fake device: the manifest reports the resampler's conversion delay on the device-facing port, the module's L excludes it, and the delay appears in D (C-134, C-207); no resampler code exists in this crate (C-197).
6. Unit test with a fake device and fake core: a change of A during a live stream re-enters aligning and resumes with the ring sized to the new A, with no allocation in the callback (C-186); the switch happens at the sink's own position one lead (C-43) after the change is learned, and the sink emits no announcement (C-214).
7. Uncertified sink: joining a group with a required E is refused with `RequiredEExceeded`; joining a group with none is admitted (C-103), checked against core's admission in an integration test.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
## Out of scope
E measurement and certification (021-23, 021-24). Sleep handling (021-18). Capture (021-16). The resampler node and rate actuator (epic 024a, C-197). The admission logic and gap counters (core).
