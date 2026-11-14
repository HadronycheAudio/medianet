# 015a-05 — Non-blocking send_to with counted drops
**Epic:** 015a-Hosted-runtime-core · **Effort:** XS · **Depends on:** 015a-04
## Context
- File: `hadronyche-os-linux/src/net.rs`. Trait method: `fn send_to(&mut self, packet: &[u8], to: core::net::SocketAddrV4) -> Result<(), SendError>;` — "Never blocks; a full queue is a counted drop at the sender."
- `SendError` (epic): `QueueFull`, `MessageTooBig`, `Other(errno)`.
- IP fragmentation is forbidden; sockets are opened with don't-fragment (015a-04). A packet larger than the path MTU (fallback 1 280 bytes, 1 252 bytes of UDP payload, C-06, C-100) must fail with `MessageTooBig`, not fragment.
- The path-MTU probe pair is sent on the media socket at the intended media packet size and is exempt from the control message bound (C-116). A local `MessageTooBig` on a probe is how this backend reports that the size cannot leave the interface; deciding the fallback is core's (epics 012, 014).
- Where the sender-side drop count appears in the stamp is not decided (wire specification, epic 004; README remaining gaps). This ticket exposes the counter only.
## Work
1. Implement `send_to` with `sendto(..., MSG_DONTWAIT)`.
2. Map `EAGAIN`/`EWOULDBLOCK`/`ENOBUFS` to `SendError::QueueFull` and increment a per-socket send-drop counter; map `EMSGSIZE` to `SendError::MessageTooBig`.
3. Expose the send-drop counter for lock-free publication (015a-18).
## Acceptance criteria
1. `send_to` never blocks: with a full send queue (tiny `SO_SNDBUF`, burst of 10 000 packets), each call returns within 1 ms and the drop counter equals the number of `QueueFull` results.
2. Sending a datagram larger than the interface MTU on loopback with a forced small MTU (in an unprivileged user and network namespace) returns `MessageTooBig` and nothing is fragmented.
3. A normal send to a loopback receiver succeeds and the packet arrives intact.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — the field is not specified yet (epic 004); "Design documents updated" — no behaviour differs; "Every new factual claim carries an evidence tag" — no new claim; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — send path only.
## Out of scope
- Pacing of sends (015a-09).
- The path-MTU decision procedure (core, epics 012 and 014).
- The stamp field for send drops (epic 004).
