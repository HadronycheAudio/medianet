# 011-02 — Hello encode
**Epic:** 011-Gossip-and-discovery-core · **Effort:** M · **Depends on:** 011-01, epic 004

## Context
Modules discover each other by multicast gossip: every module sends a binary, bounded-size *hello* at join (the end of the 1.5 s listen-before-claim, 011-11) and then every 1 s (C-30) on an IPv4 multicast group with TTL 1 (C-100). This ticket writes the encoder in `hadronyche-core` (`no_std`, no allocation). The parser is 011-03.

Hello content `[decision]` — the hello carries:
- module id (64 bits), instance nonce (64 bits), domain (16 bits) — C-11;
- protocol version;
- name;
- capability bits, including **holder-eligible**;
- ports with content types (no per-stream sample rate: every stream is at the 96 kHz domain rate, C-01);
- bundles (group id, member list, channel map);
- election epoch;
- believed holder;
- timeline epoch (16 bits, C-10);
- transport role;
- an alarm field;
- the data-plane liveness counter;
- the regime stamp version number. The regime stamp itself travels in its own sectioned control message, defined by epic 013 and sent by the hosted backend, epic 015 (C-144); the hello carries only its version (C-18).

Constants:
- C-12 (reserved) and C-115 (decided): authentication reservation = scheme 8 bits + message counter 64 bits + tag 128 bits. In v1 the scheme is 0 and the tag is zero; the 64-bit counter is live because it is the liveness counter.
- C-17 (decided): a control message is at most 1 252 bytes of UDP payload (the 1 280-byte IPv4 fallback minus IP and UDP headers, C-100).

The byte layout, including the width of the stamp version number, is owned by epic 004 (hello message layout). `[code]` The retired POC sent text messages prefixed `"HADRO/1"` (`module.rs:47`) parsed with string splitting; v1 is binary.

## Work
1. Define `LocalInfo` holding every field listed above, with fixed-capacity storage (fixed-size arrays for name, ports, bundles and bundle members; no `Vec`).
2. Implement, with the signature the epic gives:
   ```rust
   impl<'a> Hello<'a> {
       pub fn encode(info: &LocalInfo, out: &mut [u8]) -> usize;   // <= 1252 bytes
   }
   ```
   following the epic 004 layout exactly. Authentication scheme byte = 0, tag = 128 zero bits, counter = liveness counter (C-115).
3. Define `pub const HELLO_MAX: usize = 1252;` and make `encode` never write more than `HELLO_MAX` bytes. If `out` is shorter than needed, behaviour is defined (return 0, documented) — never a panic, never a partial valid-looking hello.
4. Golden-byte tests against layout examples from epic 004.

## Acceptance criteria
1. For a `LocalInfo` with every field populated, `encode` produces bytes identical to the epic 004 reference example (golden test).
2. `encode` never returns more than 1 252 (C-17), checked over at least 10 000 randomly generated `LocalInfo` values within declared capacities.
3. Auth scheme byte is 0 and the 128-bit tag is all zeros in every encoded hello; the 64-bit counter field equals `LocalInfo`'s liveness counter (C-115).
4. The holder-eligible capability bit is set in the output if and only if it is set in `LocalInfo`.
5. The regime stamp version in the output equals `LocalInfo`'s stamp version (C-18).
6. With `out` shorter than the required length, `encode` does not panic and does not write past `out`.
7. Builds in the `no_std`, no-allocator CI job from 011-01.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] Design documents updated where behaviour differs, constants table first

Not applicable (C-125): "Every parser touched is fuzzed" (encoder only; the parser and its fuzz target are 011-03 and 011-05); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (the hello carries only the stamp version; stamp fields are 011-22); "No election path reachable from hello expiry" (no expiry or election code).

## Out of scope
- Parsing and rejection reasons: 011-03.
- The 32-member bundle size-budget check of AC 8: 011-04.
- Sending on the network, DSCP (C-13 CS5) and multicast TTL: the platform transport (epics 015, 026).
- The regime stamp message and its fields: 011-22 for the holder-eligible field; the message itself is defined by epic 013 and sent by epic 015 (C-144).
