# 010-03 — Packet sizing functions
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** S · **Depends on:** 010-01, epic 004
## Context
- C-02: slice length 48 frames (0.5 ms). C-03: 4 bytes per sample.
- C-04: at defaults a stream has 6 channels; 192 bytes per channel per slice, which is exactly 48 × 4 bytes of samples: there is no per-channel id byte, the channel map is in the manifest; 1 152 bytes of audio; about 320 bytes left in a 1 472-byte UDP payload for headers and in-band content. Falsifier: headers plus in-band content exceed those ~320 bytes.
- C-100: IPv4 only. IPv4 + UDP headers are 28 bytes, so the UDP payload is 1 472 bytes at MTU 1 500 and 1 252 bytes at the 1 280-byte fallback (C-06).
- C-05: the default slice fits one packet at the path MTU; a slice larger than the path MTU uses the explicit, stamped multi-packet regime. IP fragmentation is forbidden; every packet is sent with don't-fragment set.
- The header budget `header_bytes` (packet header + slice header + reserve for in-band content) is fixed by the wire specification of epic 004; its exact value is still open there, but C-137 caps all headers and the checksum at 100 bytes so that a default slice fits 1 252 bytes of UDP payload.
- Formula settled by C-04 and C-100: channels per packet = ⌊(path_mtu − 28 − header_bytes) / (frames × 4)⌋. Because of the C-137 cap, a default 6-channel slice fits one packet also at the fallback (1 152 + at most 100 = 1 252); the multi-packet regime is for slices wider than `channels_per_packet`. Enforcing the cap is 010-23.
- C-116: the path-MTU probe pair is sent at the intended media packet size; the backend needs that size from this code.

## Work
In `hadronyche-core`, `no_std`, no allocation:
- `pub const fn channels_per_packet(path_mtu: u16, frames: u16, header_bytes: u16) -> u8` — the formula above, saturating at 255.
- `pub const fn packets_per_slice(channels: u8, frames: u16, path_mtu: u16, header_bytes: u16) -> u8` — the number of packets needed for one slice of `channels` channels; 1 when the slice fits (C-05). Account for packet 0 carrying transport state, controllers and announcements in the multi-packet regime if the wire specification reserves room for them.
- `pub const fn media_packet_bytes(channels: u8, frames: u16, path_mtu: u16, header_bytes: u16) -> u16` — UDP payload size of the largest packet of such a slice (packet 0), for the C-116 probe.
- Document in the doc comments that 28 bytes of IPv4 + UDP are subtracted (C-100).
- Expose a constant `DEFAULT_HEADER_BYTES` taken from the epic 004 layout.

## Acceptance criteria
1. `channels_per_packet(1500, 48, DEFAULT_HEADER_BYTES) >= 6` (epic AC-2); a lower value is reported as the C-04 falsifier firing, not hidden.
2. `channels_per_packet(1280, 48, DEFAULT_HEADER_BYTES) == (1252 - DEFAULT_HEADER_BYTES) / 192`, which is at least 6 (C-137); the test comment shows the arithmetic (epic AC-2, fallback part).
3. `packets_per_slice(6, 48, 1500, DEFAULT_HEADER_BYTES) == 1`.
4. `packets_per_slice(48, 48, 1500, DEFAULT_HEADER_BYTES) >= 8` and for every channel count 1..=255 at MTU 1 500 and 1 280, `packets_per_slice` × per-packet capacity covers the channels (property test).
5. `media_packet_bytes(6, 48, 1500, DEFAULT_HEADER_BYTES) <= 1472` and equals the length of the packet 010-06 encodes for that slice; at 1 280 it is ≤ 1 252.
6. All functions can be evaluated in a `const` context.
7. No function panics for `path_mtu` smaller than 28 + `header_bytes`; the documented result (0 channels / error value) is tested.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Splitting a slice into packets (010-09).
- Enforcing the C-137 header cap and omitting in-band content on a fallback path (010-23).
- Bundle layouts and wire rate (010-04, 010-05).
- Path-MTU discovery and sending the probe (backend, epic 015).
