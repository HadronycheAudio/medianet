# 021-10 — Core Audio device layer and DeviceClock
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** M · **Depends on:** 021-02, epic 018
## Context
The edge modules run on the Core Audio device I/O callback. The shared two-loop timebase (epic 018) needs a device clock S:

- Loop 2 maps the device sample counter S to host clock H by regression over (sample count, host time) pairs; the ratio of the kit's steerable rate actuator (epic 024a, C-197) is the quotient of the slopes; rate-loop bandwidth well below 1 Hz (C-68).
- Mechanism [memory]: the device I/O callback supplies sample time and host time for input and output on every cycle, plus a rate scalar. Check: regression residual.
- Trait: `impl DeviceClock for CoreAudioDevice { nominal_rate, declared_jitter_ns }`.
- Host times from Core Audio are mach absolute time; convert with the same timebase ratio as `MachClock` (021-02).
- The domain rate is 96 kHz and it is the only rate on the wire (C-01). Laptop devices often run at 48 kHz; the edge then converts 2:1 through the kit's resampler node (C-80). Whatever the device rate, the kit's steerable rate actuator tracks the device crystal, its ratio steered by loop 2 by parts per million around its nominal value (C-189), within the range of ±1 000 ppm that the kit's rate-actuator trait declares (C-210); both are built by epic 024a and this epic builds none (C-197). `nominal_rate` must therefore report the device's own rate, not the domain rate.
- No design value exists for `declared_jitter_ns`; it is set from the measurement made in this ticket.
## Work
- `coreaudio/mod.rs`: enumerate devices, open a device by id, install an I/O proc, start/stop.
- `CoreAudioDevice`: on every callback, push a (sample time, host time ns) pair per direction into a lock-free queue consumed by epic 018's loop 2; expose the rate scalar for diagnostics.
- `impl DeviceClock for CoreAudioDevice`: `nominal_rate` from the device's nominal sample rate property; `declared_jitter_ns` from a measured regression residual (record value `[measured]`).
- Hardware test (manual job): 10-minute run, compute regression residual of the pairs.
## Acceptance criteria
1. Opening the built-in output starts the callback; pairs arrive every cycle with strictly increasing sample time (no gaps over a 10-minute run except reported overloads).
2. Host times in pairs are in the H basis (agree with `MachClock::now_ns` read in the callback to within 1 ms).
3. The regression residual over 10 minutes is reported; `declared_jitter_ns` is set from it with a `[measured]` tag.
4. Unit test with a fake callback source feeds pairs to the loop-2 consumer without allocation in the callback path.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
- [ ] Every new factual claim carries an evidence tag
## Out of scope
Device guards (021-11, 021-12). D (021-14). Sink and capture modules (021-15, 021-16). The loop-2 regression itself (epic 018).
