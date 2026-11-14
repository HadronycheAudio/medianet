# 021-12 — Device guard: rate change, device loss, held interface
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-10, 021-11
## Context
`coreaudio/guard.rs`, continued.

- Porting trap [memory]: other applications can change the device's sample rate under the edge.
- Epic criterion 021-AC5 (part): a sample-rate change by another application ends the stream cleanly with a named event.
- Scope ruling [decision]: if the musician's own software holds the interface so that the edge cannot use it as required, the edge **reports it and certifies nothing** (`InterfaceHeld`, certification state *uncertified*, C-140).
- "The interface is held" [decision, C-175]: the audio device cannot be opened in the required mode because another process has it open.
- The wire carries 96 kHz only (C-01) and a 48 kHz device converts through the kit's resampler node (C-80), so a device rate change is not a wire-format change: it changes D and the resampler ratio, which is why the stream must end.
- Device loss (unplug) must also end the stream cleanly.
- After a clean end, a capture edge's next stream carries a new session number (C-19); core assigns it.
## Work
- Register property listeners for nominal sample rate, device alive, and device hog/exclusive ownership.
- On rate change: stop the I/O proc, end the stream with named event `DeviceRateChanged { from, to }`.
- On device loss: end with named event `DeviceLost`.
- At open and while running, evaluate the predicate `interface_held()`: true when opening the device in the required mode fails because another process has it open (C-175), for example another process holds it in hog/exclusive mode. A failure with no other process holding the device is not "held"; it is reported as an open failure. When held, emit `InterfaceHeld` and set the certification state to *uncertified*, so E is *unknown* (C-140; consumed by 021-20).
- Events are delivered to core through the edge module's clean-end path (epic 018 structure).
## Acceptance criteria
1. Manual hardware test: changing the sample rate in Audio MIDI Setup during a live stream ends it within 1 s with `DeviceRateChanged` carrying old and new rates.
2. Manual hardware test: unplugging a USB interface ends the stream with `DeviceLost`; the process does not crash.
3. Manual hardware test: another app holding the device exclusively yields `InterfaceHeld` and the stamp's certification state is *uncertified* with E *unknown* (C-140).
4. Unit tests drive each listener and each branch of `interface_held()` through a fake property source, including an open failure with no other process holding the device, which is not `InterfaceHeld` (C-175).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
- [ ] Regime stamp fields touched are complete and documented
## Out of scope
Static device-class refusal (021-11). Re-admission after the stream ends (core / epic 018).
