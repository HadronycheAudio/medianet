# 010-08 — Gap slice encoding
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** S · **Depends on:** 010-06
## Context
- A **gap slice** is header-only and marks positions the sender did not produce or receive `[decision]`. Being header-only, it carries no content and no announcements (settled in the epic).
- If a module misses its own deadline a gap slice is emitted for it. C-144 assigns that duty to the hosted backend (epic 015), which detects the missed deadline and calls this encoder.
- Why gap slices matter downstream: consumers count marked gaps as their own class (C-47); a marked gap proves the upstream alive and resets the interrupted count (C-102); the dead threshold runs from the last slice with content or a marked gap (C-111).
- Members of a bundle are independent on the wire: gaps are per member.
- The packet header has a gap flag. Exact layout from the wire specification (epic 004).
- Settled in the epic: `gap_slice` takes a send timestamp like any slice and returns an error when `out` is too small.
- Open item owned by epic 004: what happens to an announcement re-issue that falls on a gap slice (010-13 handles the answer).

Signature (names may be refined):
```rust
pub fn gap_slice(&mut self, position: u64, frames: u16, send_ns: i64, out: &mut [u8]) -> Result<usize, OutTooSmall>;
```

## Work
- Implement `Packetiser::gap_slice` in `hadronyche-core`: writes one header-only packet (packet header with gap flag and the stream's session and epoch, slice header with position, frames and `send_ns`, packet index 0, count 1, slice checksum per epic 004) into `out` and returns its length.
- Return `Err(OutTooSmall)` without panic when `out` is shorter than the header-only size.

## Acceptance criteria
1. A gap slice has the gap flag set, the given position, frames and send timestamp, and no content messages and no announcements, even when announcements are pending.
2. Its length is the header-only size defined by epic 004 and is the same for any `frames` value.
3. Epic 007 conformance vectors for gap slices encode byte for byte (epic AC-1, partial).
4. A gap slice for one member of a bundle does not alter the next slice of any other member (per-member gaps).
5. An `out` one byte shorter than the header-only size returns `Err(OutTooSmall)` without panic.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Detecting a missed deadline and emitting its gap slice (hosted backend, epic 015, C-144).
- Belated gap slices from arrival-driven modules (010-17).
- Counting gap slices for the regime stamp (010-21).
