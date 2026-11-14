# 004-13 — Slice checksum algorithm, width and placement
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-08
## Context
Rule 6 `[decision]`: a slice checksum covers the reassembled slice. Slices fit one packet by default; multi-packet slices are an explicit, stamped regime (C-05). The checksum bytes count against the 100-byte cap for all headers and the checksum beside 1 152 bytes of audio (C-137; 1 152 bytes = 6 channels × 48 frames × 4 bytes, no per-channel id byte, C-04).

Open sub-question owned by epic 004:

| Question | Owner | Falsifier |
|---|---|---|
| Checksum algorithm and width | This task | Chosen algorithm too slow on the reference embedded target (task 026) |

Rejection reason: `BadChecksum`.

## Work
1. Choose the algorithm and width (e.g. a CRC-32 variant), stating the polynomial/parameters exactly, the byte range covered (which headers are included, which mutable fields such as the measurement extension and copy index are excluded), and its position in the packet (in the last packet of a multi-packet slice, or per packet; decide).
2. Justify against the embedded-target falsifier: estimate cost per default slice (about 1 152 audio bytes at 2 000 slices/s) and record the measurement owed to task 026 as an open item with owner and falsifier.
3. Write the field table and the validity rule (`BadChecksum`, drop and count).
4. Provide one worked example (input bytes and expected checksum) for task 007.
5. Add to the machine-readable file.

## Acceptance criteria
1. Algorithm parameters are fully specified (width, polynomial/initial value/reflection/final XOR, or equivalent).
2. Covered byte range is exact, including treatment of mutable fields.
3. A worked example with expected value is included.
4. The task 026 performance check is recorded as an open item with owner and falsifier.
5. Passes the 004-02 checker.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Benchmarking on the embedded target (task 026).
- Integrity of control messages (authentication reservation, 004-05).
