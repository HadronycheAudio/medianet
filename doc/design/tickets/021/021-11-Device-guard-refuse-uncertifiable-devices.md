# 021-11 — Device guard: refuse aggregate, Bluetooth and network devices
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-10
## Context
`coreaudio/guard.rs`. Scope rulings [decision]:

- **Direct devices only.** Aggregate devices are excluded: their drift correction makes D drift, so they cannot be certified.
- Bluetooth and network (AirPlay-class) outputs have large variable latency and are excluded [memory].
- **Capture is physical device inputs only.** Capturing an application's output (a virtual audio device) is out of v1; no virtual device is installed.
- The kit's resampler node is offered only for the 2:1 and 1:2 ratios; other ratios are out of v1 (C-105). A device that cannot run at the domain rate of 96 kHz (C-01) or at a 2:1 or 1:2 ratio to it cannot be used.
- Epic criterion 021-AC5 (part): selecting an aggregate, Bluetooth or network device is refused with a named reason. 021-AC8: only physical inputs are offered.
## Work
- Classify each device by transport type and class: aggregate, Bluetooth, network/AirPlay, virtual, built-in, bus-attached (USB, Thunderbolt, PCIe).
- `fn admit_device(id) -> Result<(), DeviceRefused>` with named reasons `Aggregate`, `Bluetooth`, `Network`, `Virtual`, `UnsupportedRate` (no available nominal rate at the domain rate or a 2:1 or 1:2 ratio to it, C-105).
- Device listing for sink and capture offers only admissible devices; for capture, only devices with physical input streams.
- Unit tests over a table of fake device descriptions.
## Acceptance criteria
1. Each of aggregate, Bluetooth, network, virtual and unsupported-rate devices is refused with its named reason (unit tests; one manual hardware check each for aggregate and Bluetooth).
2. The capture listing contains no virtual device and no device without physical inputs.
3. The crate installs no audio driver or virtual device (checked: no driver bundle in the build artefacts).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
## Out of scope
Sample-rate change, device loss and held-interface detection (021-12).
