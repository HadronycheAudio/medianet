# 010-05 — Wire rate of a bundle
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** S · **Depends on:** 010-03, 010-04
## Context
`wire_bits_per_second` is the admission unit handed to epic 012 (admission handshake), together with packets per second and the member list.

Derived constants (all derived from the `[decision]` constants):
- C-20: wire size of one default packet is about 1 320 bytes with all Ethernet overhead.
- C-21: wire rate per stream is about 21.1 Mbit/s, about 3.5 Mbit/s per channel (1 320 × 8 × 2 000 slices/s).
- C-22: per source — 24 ch: 4 streams, 84 Mbit/s, 8 000 packets/s; 48 ch: 8 streams, 169 Mbit/s, 16 000 packets/s; 96 ch: 16 streams, 338 Mbit/s, 32 000 packets/s.
- C-92 (consistency check only): two 96-channel bundles are 32 streams, 64 000 packets/s, 676 Mbit/s.
- C-01: every stream is at the domain rate, 96 000 Hz; there is no other rate on the wire (edges convert, C-80). C-02: 48 frames per slice, so 2 000 slices/s.
- C-100: IPv4 only; IPv4 + UDP overhead is 28 bytes.
- Ethernet per-frame overhead: preamble and start delimiter 8 bytes, header 14, FCS 4, inter-frame gap 12, total 38 bytes `[memory]`. Which terms C-20 counts (in particular a VLAN tag, 4 bytes) is an open item owned by epic 004; either choice moves the per-packet size by less than 0.5 %, inside C-20's 5 % falsifier. Count the 38 bytes and no VLAN tag, and state the terms in the doc comment. Check each term against IEEE 802.3, cite the clause, and change the tag from `[memory]` to `[standard]` (C-132).

Signature (the rate parameter of the original shape is dropped, since the rate is always C-01):
```rust
pub const fn wire_bits_per_second(layout: &BundleLayout<..>, frames: u16) -> u64;
pub const fn packets_per_second(layout: &BundleLayout<..>, frames: u16) -> u32;
```

## Work
- Implement `wire_bits_per_second` in `hadronyche-core`: for each member, `packets_per_slice` × (UDP payload bytes of each packet + 28 + Ethernet overhead) × 8 × (96 000 / frames), summed over members. Partial last packets count their real size.
- Add `packets_per_second(layout, frames) -> u32` for epic 012.
- Both `const fn`, `no_std`, no allocation.

## Acceptance criteria
1. 24-channel layout (4 members, 6 ch each) at 48 frames: result within 5 % of 84 Mbit/s; `packets_per_second` = 8 000 (epic AC-3).
2. 48 channels: within 5 % of 169 Mbit/s; 16 000 packets/s (epic AC-3).
3. 96 channels: within 5 % of 338 Mbit/s; 32 000 packets/s (epic AC-3).
4. One 6-channel member: within 5 % of 21.1 Mbit/s (C-21), and the per-packet wire size used is within 5 % of 1 320 bytes (C-20).
5. Twice the 96-channel result is within 5 % of 676 Mbit/s and 64 000 packets/s (C-92).
6. If any of 1–5 fails because the real header budget of epic 004 differs, the constants table (C-20, C-21, C-22) is updated in the design document in the same change, as the epic allows.
7. Each Ethernet overhead term in the doc comment carries `[standard]` with its IEEE 802.3 clause (C-132).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Admission logic and link-capacity checks (epic 012).
- The wire cost of a controller member (C-219): 010-24.
