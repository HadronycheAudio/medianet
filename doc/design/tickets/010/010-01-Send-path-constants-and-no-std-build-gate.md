# 010-01 — Send-path constants and no_std build gate
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** S · **Depends on:** epic 004
## Context
Epic 010 implements the send side of the data plane in the crate `hadronyche-core`: slice and packet encoding, position stamping, channel bundles, the multi-packet regime, intra-slice pacing, gap slices and in-band announcements. All of it must be pure `no_std` code with no sockets and no allocation, and it must compile for a `no_std` target without an allocator (epic acceptance criterion 10).

This first ticket puts the guard rails in place before any send-path code lands: one module holding the epic's constants, and a CI job that fails as soon as anything in the send path needs `std` or `alloc`.

Constants to carry, all `[decision]` unless marked (the constants table of the design set is canonical):

| Id | Constant | Value |
|---|---|---|
| C-01 | Domain sample rate | 96 000 Hz, the only rate on the wire |
| C-02 | Slice length (default) | 48 frames = 0.5 ms = 2 000 slices/s (inherited) |
| C-03 | Sample format | 4 bytes per sample, f32 or Q1.31 |
| C-04 | Channels per stream at defaults | 6: 192 bytes per channel per slice (48 × 4, no per-channel id byte), 1 152 bytes of audio, about 320 bytes left in a 1 472-byte payload |
| C-06 | Path-MTU fallback | 1 280 bytes |
| C-07 | Copy index | 2 bits, reserved; v1 always sends copy 0 |
| C-08 | Members per bundle | at most 32, the controller member included (C-209) |
| C-09 | Streams per source | at most 256 |
| C-10 | Stream number / session / timeline epoch | 16 / 32 / 16 bits |
| C-14 | Intra-slice egress spread | at most 25 % of the slice period (125 µs), plus a per-source phase offset derived from the module id |
| C-43 | Announcement lead | 64 slices (32 ms at defaults), flat |
| C-44 | Announcement re-issue | every 16 slices until the named position passes, in every bundle member |
| C-100 | IP version | IPv4 only: IPv4 + UDP headers are 28 bytes; 1 472 bytes of UDP payload at MTU 1 500, 1 252 at the 1 280 fallback |

## Work
- In `hadronyche-core`, add a send-path module (suggested name `send::consts`) with `pub const` items for C-01, C-02, C-03, C-06, C-07, C-08, C-09, C-14 (as `DEFAULT_SPREAD_PERMILLE = 250`, which is also the maximum), C-43 (`ANNOUNCE_LEAD_SLICES = 64`), C-44 (`ANNOUNCE_REISSUE_SLICES = 16`) and C-100 (`IPV4_UDP_HEADER_BYTES = 28`). Each item's doc comment gives its C-nn id.
- Add compile-time assertions that tie the derived values together: 96 000 / 48 = 2 000 slices/s; 48 × 4 = 192 bytes per channel; 6 × 192 = 1 152; 1 500 − 28 = 1 472; 1 280 − 28 = 1 252.
- Add a CI job that builds `hadronyche-core` (or at least its send-path modules) for a bare-metal `no_std` target without an allocator, with `#![no_std]` and no `extern crate alloc`. The target is an open item owned by epic 026 (device), which reuses this code. Use the target epic 026 names; until it names one, use a bare-metal target and record it as provisional.
- The job runs unprivileged.

## Acceptance criteria
1. The constants module compiles and every constant listed above is present with the value in the table.
2. The compile-time assertions hold: 96 000 / 48 = 2 000; 48 × 4 = 192; 6 × 192 = 1 152; 1 500 − 28 = 1 472; 1 280 − 28 = 1 252.
3. CI builds the crate for one `no_std` target with no global allocator, and the job fails if a test commit adds `extern crate alloc` or a `Vec` to a send-path module (show this once on a throwaway branch).
4. The CI job needs no root and no special capabilities.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] The CI target name, the reason for choosing it, and whether it is still provisional are written in the crate README

## Out of scope
- Any encoding or sizing logic (010-03 onwards).
- Packet layout offsets and the header budget: they come from the wire specification (epic 004).
- The lock-free audit of the finished send path (010-20).
