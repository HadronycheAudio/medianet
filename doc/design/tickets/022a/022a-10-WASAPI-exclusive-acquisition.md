# 022a-10 — WASAPI exclusive-mode acquisition and "interface held" reporting
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** S · **Depends on:** 022a-01

## Context
- Part of epic 022a (Windows edge backend), `wasapi/device.rs`: exclusive-mode acquisition with "interface held, nothing certified" reporting.
- **WASAPI exclusive mode is mandatory for certification** `[decision]`. Shared mode passes through the audio engine with opaque latency and possible resampling.
- If the musician's own software holds the interface (exclusive mode cannot be obtained), the edge **reports it and certifies nothing** `[decision]`.
- 022-AC-3: if exclusive mode cannot be obtained, the edge reports "interface held" and its stamp says *uncertified*; it **never silently falls back to shared mode**.
- Exclusive-mode hand-off is quantised to the device period, about 3 to 10 ms `[memory]`.
- The wire carries one rate only, 96 kHz (C-01). A device at 48 kHz is fine: the edge converts 2:1 through the kit's resampler node (C-80) in 022a-13 and 022a-14. The resampler node may appear only inside an edge module (C-134), and it is offered only for ratios whose delay is a whole number of samples, 2:1 and 1:2; other ratios are out of v1 (C-105). The device must therefore run at 96 kHz or 48 kHz; a device that offers neither cannot join in v1. Streams carry 6 channels (C-04).
- "The interface is held" means the audio device cannot be opened in the required mode (here exclusive) because another process has it open (C-175).
- Stamp fields use the four capability states of C-129 where they describe a capability.

## Work
1. `wasapi/device.rs`: `WasapiDevice::open_exclusive(endpoint_id, direction) -> Result<WasapiDevice, AcquireError>`:
   - enumerate endpoints; select by id;
   - negotiate an exclusive-mode format (96 kHz, the domain rate, when the device supports it; otherwise 48 kHz, which the edge converts 2:1, C-80; no other rate is accepted, C-105) and the device period (minimum supported, recorded);
   - initialise in exclusive, event-driven mode.
2. `AcquireError::InterfaceHeld { endpoint }` when exclusive access is refused because another process has the device open (C-175); `AcquireError::FormatUnsupported` for format failure, including a device that supports neither 96 kHz nor 48 kHz in exclusive mode (C-105).
3. No code path opens the device in shared mode (enforced by a single initialisation function that only takes the exclusive flag).
4. Record device name, driver identity/version, negotiated rate, channel count and period for the stamp (022a-17).
5. Certification state: `CertState::Uncertified { reason: InterfaceHeld }` on `InterfaceHeld`.

## Acceptance criteria
1. **(022-AC-3.)** With another application holding the device in exclusive mode, `open_exclusive` returns `InterfaceHeld` within 2 s, the edge reports "interface held", and the certification state is *uncertified* (manual test on reference hardware, 3 runs).
2. A code search / test shows no shared-mode initialisation path exists in the crate.
3. On reference hardware without contention, acquisition succeeds and reports rate, channels and period (period within 3 to 10 ms recorded `[measured]`).
4. Device, driver and format metadata are exposed for the stamp.
5. A (mocked) device that offers only another rate, for example 44.1 kHz, is refused with `FormatUnsupported`; no other conversion ratio is ever configured (C-105).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] The device-period fact is confirmed `[measured]` or corrected, for 022b-08

## Out of scope
- Device clock regression (022a-11).
- Render and capture loops (022a-13, 022a-14).
