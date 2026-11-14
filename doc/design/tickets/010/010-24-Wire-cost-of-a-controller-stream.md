# 010-24 — Wire cost of a controller stream
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** XS · **Depends on:** 010-04, 010-05, 010-18
## Context
- C-219: the wire cost of a controller stream is computed like an audio stream: one packet per slice of (headers up to the 100-byte cap + controller bytes per slice + 66 bytes of Ethernet, IP and UDP overhead), at 2 000 packets/s. Admission and the commissioning tool use this same figure.
- The 66 bytes are the 28 bytes of IPv4 + UDP (C-100) plus the 38 bytes of Ethernet overhead that 010-05 counts; the Ethernet terms keep their `[standard]` citations there (C-132).
- Header bytes are the epic 004 header budget h, at most `HEADER_CAP_BYTES` (100, C-137). Controller bytes per slice are recorded on the controller member (010-04).
- Controller members occur in a bundle source (always, C-205) and in a single-stream source that puts its controllers in a second stream (C-181, 010-18). In the shared case of a single-stream source the controllers are part of the audio packet, whose real payload 010-05 already counts.
- 010-05 sums `packets_per_slice` × packet wire size over members; a controller member has 0 audio channels, so it needs its own term.

## Work
- In `wire_bits_per_second` and `packets_per_second` (010-05), cost a controller member as one packet per slice of h + controller bytes per slice + 66 bytes, × 8 × (96 000 / frames) bits per second, and one packet per slice in the packet rate. Cite C-219 in the doc comment.
- In the shared case of 010-18, count the controller bytes in the audio packet's payload.
- Both stay `const fn`, `no_std`, no allocation.

## Acceptance criteria
1. A controller member with controller bytes per slice c at 48 frames costs (h + c + 66) × 8 × 2 000 bits per second and 2 000 packets per second (C-219, epic AC-16).
2. A 96-channel bundle with a controller member costs the 96-channel result of 010-05 plus exactly that controller-member figure, and 34 000 packets/s.
3. A single-stream source with a second controller stream (010-18) costs one audio member plus one controller member; in the shared case it costs one audio packet per slice whose payload includes the controller bytes, at 2 000 packets/s.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Admission logic and the commissioning tool (epic 012 and the tool's epic); they consume this figure.
- Controller message formats and the value of h (epic 004).
