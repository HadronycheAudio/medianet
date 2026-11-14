# 010-09 — Multi-packet regime
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** M · **Depends on:** 010-03, 010-06, epic 004, epic 007
## Context
- C-05: the default slice fits one packet at the path MTU; multi-packet is an explicit, stamped regime. C-06: path-MTU fallback is 1 280 bytes.
- C-100: IPv4 only; the UDP payload is the path MTU − 28 bytes: 1 472 at MTU 1 500, 1 252 at the fallback. At the fallback a default 6-channel slice (1 152 bytes of audio) still fits one packet, because C-137 caps all headers and the checksum at 100 bytes; this regime serves members wider than `channels_per_packet` (for example a single 48-channel stream).
- IP fragmentation is forbidden; every packet is sent with don't-fragment set. `[code]` The POC sent one 9.6 KB datagram per slice for a 48-channel feed (7 IP fragments at MTU 1 500, about 150 Mbit/s); this is retired.
- **Multi-packet regime** `[decision]`: a slice larger than the path MTU is split by the runtime into ordered packets, each carrying the slice identity, its index, the packet count and **its own send timestamp**; the regime is recorded in the regime stamp. Transport state, controllers and announcements go in packet 0; audio fills the rest.
- The handler on the receiving side gets a slice whole or not at all.
- Packet index and count are `u8` in the API (`packets_per_slice` returns `u8`).
- Open item owned by epic 004: placement of the slice checksum in a multi-packet slice and the content of "transport state" in packet 0.

Signature:
```rust
pub fn finish<'b>(self, send_ns: impl FnMut(u8) -> i64, out: &'b mut [u8]) -> Result<Packets<'b>, OutTooSmall>; // one stamp per packet
```

## Work
- Extend `SliceWriter::finish` in `hadronyche-core` so that, when the content exceeds one packet at the configured path MTU, it emits `packets_per_slice(...)` packets into `out`, each with packet header, slice header (same position and frames, own index, same count) and `send_ns(i)` for packet i.
- Packet 0 holds transport state, controllers and announcements; audio fills packets 0..n in order. Controllers are present only for a single-stream source in the shared case (C-181); an audio member of a bundle source never carries controller content (C-205).
- Place the slice checksum and define transport state as epic 004 prescribes; if epic 004 has not ruled, the ticket is blocked on that item and cannot merge with a guess.
- `Err(OutTooSmall)` when `out` cannot hold all packets.
- Expose the packet count of the last finished slice for the regime stamp (used by 010-12).

## Acceptance criteria
1. A 48-channel, 48-frame slice at MTU 1 500 is encoded as `packets_per_slice(48, 48, 1500, h)` packets, each with UDP payload ≤ 1 472 bytes (epic AC-4).
2. The same slice at MTU 1 280 produces packets each with UDP payload ≤ 1 252 bytes (epic AC-4, C-100).
3. `send_ns` is called once per packet, with indices 0..n−1 in order, and packet i's header holds the value returned for i; a test closure returning 1 000 × i checks all distinct values (epic AC-4).
4. Every packet carries the same position, frames, packet count and its own index 0..n−1.
5. Announcements and controller content placed in the slice appear only in packet 0.
6. Epic 007 multi-packet conformance vectors encode byte for byte (epic AC-1, partial).
7. A slice that fits one packet still produces exactly one packet (no regression of 010-06).
8. An `out` too small for all packets returns `Err(OutTooSmall)` without panic.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] Regime stamp fields touched are complete and documented
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Reassembly on the receive side, including its deadlines (C-15, C-110; epic 009) and the round-trip fuzz target (010-19).
- When packets are sent (pacing, 010-11; execution, epic 015).
