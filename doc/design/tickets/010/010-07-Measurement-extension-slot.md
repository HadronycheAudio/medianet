# 010-07 — Measurement extension slot
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** XS · **Depends on:** 010-06, epic 004
## Context
- The packet layout has an optional **measurement extension** (hop count, accumulated residence, accumulated declared delay), placed after the optional announcements and before the content messages. Exact offsets come from the wire specification (epic 004).
- Epic 010 must tell epic 014 *where* the measurement extension is written; epic 014 owns the values.
- Forward packets carry no per-receiver state: the extension holds per-path accumulations, never a destination.
- Open item owned by epic 004: where the extension goes in a multi-packet slice. Until epic 004 rules, it goes in packet 0 alongside transport state.
- C-137: on a fallback path the extension fits inside the 100-byte header cap or is omitted from the slice; that omission is implemented in 010-23.

## Work
- In the `Packetiser`/`SliceWriter` of 010-06, add an option to reserve the measurement extension in a slice (flag set, zeroed fields at the epic 004 offset), and expose the byte offset of the extension inside each encoded packet through `Packets<'b>` (e.g. `measurement_offset(i) -> Option<usize>`).
- For a multi-packet slice, follow epic 004 if it has ruled; otherwise use packet 0 and document it as provisional, naming the epic 004 open item.

## Acceptance criteria
1. With the extension requested, the encoded packet has the extension flag set and the three fields at the offset reported by `Packets`, initialised as epic 004 prescribes.
2. Without it, the flag is clear, no bytes are reserved, and output is identical to 010-06.
3. Epic 007 conformance vectors that include a measurement extension encode byte for byte (epic AC-1, partial).
4. The reported offset plus the extension size stays inside the packet, and the packet's UDP payload stays at most path MTU − 28 bytes (C-100).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Writing and accumulating hop count, residence and declared delay (epic 014).
- Omitting the extension when it does not fit the C-137 cap on a fallback path (010-23).
