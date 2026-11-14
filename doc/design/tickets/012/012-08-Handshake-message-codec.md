# 012-08 — Handshake message codec
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** 012-01, 012-07, epic 004

## Context
Epic 012's handshake messages use the layouts defined by epic 004. The exchange carries `[decision]`: declared rates, the bundle member list with the source's session of each requested stream (C-19), the path-MTU probe pair, the reverse measurement port, the pinned IPv4 addresses (C-100), and the manifest (ports, content types, and for presentation groups the required E and the sink's declared E or "unknown", C-103). Refusals carry a named `RefuseReason` (`#[repr(u8)]`, 16 variants including `BadMemberList`, C-173).

Size and framing rules:
- A control message is at most 1 252 bytes of UDP payload (C-17), which is what the 1 280-byte fallback path MTU leaves over IPv4 (C-06, C-100).
- Bundles have at most 32 members, the controller member included (C-08, C-209); a valid request carries at most 32 sessions of 32 bits. A longer member list is not a decoding error: it is decoded when it fits the message and refused by the engine with `BadMemberList` (C-173, 012-05).
- Processing one bad input is linear in packet length and at most 10 µs per packet (C-164, a proposal that L1 confirms).
- The probe pair is exempt from C-17: the probe is padded to the intended media packet size (C-116).
- Every control message carries the authentication reservation: scheme 8 bits, counter 64 bits, tag 128 bits (C-12). In v1 the scheme is 0 and the tag is zero; the counter is live because it is the liveness counter (C-115).

Message kinds needed by the engine: request, accept (`Accept { pinned_source, pinned_dest, reverse_port, path_mtu, group_a_ns }`, the last field for C-142), refuse (reason), MTU probe and probe acknowledgement.

Whether epic 004 delivers a codec or only byte layouts is still open (owner epic 004); this ticket writes the codec against 004's layouts and is dropped in favour of 004's codec if 004 delivers one.

## Work
- In `hadronyche-core`, `encode_*` into a caller-provided `&mut [u8]` and `decode(&[u8]) -> Result<HandshakeMsg<'_>, DecodeError>` for each kind, following epic 004's layouts, zero-copy, no allocation.
- Encode the authentication reservation with scheme 0, zero tag and the caller's counter; on decode reject a non-zero scheme (v1 knows only scheme 0) and expose the counter.
- Decoding is total: any byte string returns `Ok` or `Err`, never panics, never reads out of bounds, and does work linear in the input length (C-164).
- Reject non-probe messages over 1 252 bytes, a session count different from the stream count, non-IPv4 address families, and unknown `RefuseReason` bytes.

## Acceptance criteria
1. Round-trip property test: `decode(encode(m)) == m` for every message kind and random valid contents.
2. Encoded request with 32 members, 32 sessions and the full manifest fits in 1 252 bytes (C-17).
3. Inputs of 1 253 bytes or more (non-probe kinds), a reason byte ≥ 18 (C-185), or a non-zero authentication scheme return `Err`. A request with 33 members that fits 1 252 bytes decodes, so the engine can refuse it with `BadMemberList` (C-173).
4. A probe at 1 472 bytes of UDP payload decodes (C-116).
5. Truncation at every byte offset of a valid message returns `Err` without panic.
6. Byte layouts match epic 004 test vectors (epic 007 format) where provided.
7. Decode time grows linearly with input length (a benchmark over sizes up to 1 500 bytes, recorded for L1 to confirm the C-164 bound of 10 µs per packet).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field in handshake messages (the hello carries only a stamp version, C-18)
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: layouts are epic 004's
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable: no transitions
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Defining the layouts (epic 004). The fuzz target itself (012-21). Engine logic (012-12, 012-13). Authentication schemes other than 0.
