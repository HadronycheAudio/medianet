# 010-06 — Single-packet slice encoder
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** M · **Depends on:** 010-02, 010-03, epic 004, epic 007
## Context
The default slice is one packet (C-05). The receiving handler gets a slice whole or not at all. Packets are sent with don't-fragment set. IP is IPv4 only (C-100): the UDP payload is at most the path MTU − 28 bytes.

Packet layout `[decision]`, exact offsets from the wire specification (epic 004):
- packet header: magic, version, domain, stream number (16 bits), session (32 bits), timeline epoch (16 bits), copy index (2 bits, C-07: reserved, v1 always sends copy 0), flags including gap and degraded;
- slice header: 64-bit position, frames, packet index and count, send timestamp;
- optional announcements;
- optional measurement extension (hop count, accumulated residence, accumulated declared delay);
- content messages (audio is 192 bytes per channel at defaults, no per-channel id byte, C-04);
- slice checksum.

Rules:
- **Forward packets carry no per-receiver state**, so the same bytes can go to several consumers. Streams are identified by source and stream number, never by destination.
- C-19: the session number is assigned by the source, one per stream; every consumer of the stream sees the same value. The packetiser receives it at construction; it does not invent one.
- The **degraded flag** marks a slice computed from a gapped input; it follows the content (C-122): the output stamped p − L is degraded if the input at p − L was gapped. The caller decides it and passes it in `SliceFlags`.
- Settled in the epic: `finish` returns an error when `out` is too small, instead of a partial result.
- `[code]` POC behaviour retired: the send timestamp was taken once before a per-destination `sendto` loop (`sender.rs:290`, `module.rs:979-986`); events and audio were already sent as two datagrams with the same header.

Signatures (names may be refined):
```rust
pub struct Packetiser { /* per stream: session, epoch, announcements pending */ }
impl Packetiser {
    pub fn new(stream: StreamId, session: u32, epoch: u16 /* , domain, ... */) -> Self;
    pub fn begin_slice(&mut self, position: u64, frames: u16, flags: SliceFlags) -> SliceWriter<'_>;
}
impl SliceWriter<'_> {
    pub fn content(&mut self, msg: ContentMessage<'_>) -> Result<(), Full>;
    pub fn finish<'b>(self, send_ns: impl FnMut(u8) -> i64, out: &'b mut [u8]) -> Result<Packets<'b>, OutTooSmall>; // one stamp per packet
}
```

## Work
- In `hadronyche-core`, `no_std`, no allocation: `Packetiser` (per-stream state: stream number, session, epoch, domain), `SliceFlags`, `SliceWriter`, `Full`, `OutTooSmall`, `Packets<'b>` (iterator or view over the encoded packets inside `out`).
- `begin_slice` + `content` + `finish` for the single-packet case: `finish` calls `send_ns(0)` once and writes that value into the slice header; packet index 0, count 1; copy index 0; slice checksum per epic 004.
- `content` returns `Err(Full)` when the message does not fit the slice's budget.
- `finish` returns `Err(OutTooSmall)` when `out` cannot hold the packet, and does not panic.
- Leave the announcement and measurement-extension areas empty in this ticket; their flags are clear.
- Conformance-vector test harness that loads the epic 007 media-packet vectors and compares bytes.

## Acceptance criteria
1. Every single-packet media-packet conformance vector from epic 007 that has no announcements and no measurement extension encodes byte for byte (epic AC-1, system AC-1, partial).
2. A 6-channel, 48-frame slice at MTU 1 500 produces exactly one packet no longer than 1 472 bytes of UDP payload.
3. `send_ns` is called exactly once for a single-packet slice and its return value appears in the slice header.
4. Copy index is 0 in every packet.
5. Encoding the same slice twice and comparing the buffers yields identical bytes; the public API takes no destination, address or receiver argument (epic AC-5).
6. Degraded flag set in `SliceFlags` appears in the packet flags; unset otherwise.
7. `content` returns `Err(Full)` for a message one byte larger than the remaining budget, and `Ok` for one that fits exactly.
8. The session given to `new` appears unchanged in every packet header.
9. `finish` with an `out` one byte too short returns `Err(OutTooSmall)` without panic.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Measurement extension slot (010-07), gap slices (010-08), multi-packet regime (010-09), announcements (010-13 to 010-15), epoch switch (010-16), send counters (010-21).
- Choosing the session after a restart: the source supplies it (C-19).
- Socket send and don't-fragment socket option (epic 015).
- Fuzzing (010-19).
