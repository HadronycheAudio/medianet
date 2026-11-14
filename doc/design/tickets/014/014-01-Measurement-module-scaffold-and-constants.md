# 014-01 — Measurement module scaffold and constants
**Epic:** 014-Measurement-path · **Effort:** XS · **Depends on:** epic 004
## Context
Epic 014 implements, in the crate `hadronyche-core`, the reverse unicast measurement path (receiver reports, one-way probes), the lineage extension carried in slices and the per-connection summaries published in the regime stamp. Everything in this epic is `no_std` and does not allocate. The epic names the module `measure`. This ticket creates the empty module that later tickets fill, with the constants they all use, and a CI build that proves the module compiles without `std` and without an allocator.

Constants (from the constants table, which is canonical):

| Id | Constant | Value |
|---|---|---|
| C-01 | Domain sample rate | 96 000 Hz, the only rate on the wire |
| C-02 | Slice length | 48 frames = 0.5 ms = 2 000 slices/s |
| C-06 | Path-MTU fallback | 1 280 bytes |
| C-08 | Members per bundle | at most 32, the controller member included (C-209) |
| C-13 | DSCP classes | sync 48; media and probes 46; control 40 |
| C-17 | Control message size bound | at most 1 252 bytes of UDP payload |
| C-38 | Interrupted | 8 consecutive missed slices |
| C-39 | Dead | 2 s without data |
| C-40 | Receiver report | one per bundle every 100 ms |
| C-41 | Producer timeout | 2 s without receiver reports |
| C-100 | IP version | IPv4 only in v1; 1 280 − 20 (IPv4) − 8 (UDP) = 1 252 bytes of UDP payload |

DSCP of the two message kinds `[decision]`: receiver report CS5 (40); probe request and reply EF (46), because probes measure the media path.

## Work
1. Add the module `measure` to `hadronyche-core`. No `std`, no `alloc` imports.
2. Add the constants above as `pub const` items, in nanoseconds where they are durations: `DOMAIN_RATE_HZ = 96_000` (C-01), `SLICE_FRAMES = 48`, `SLICE_NS = 500_000` (C-02), `REPORT_INTERVAL_NS = 100_000_000` (C-40), `PRODUCER_TIMEOUT_NS = 2_000_000_000` (C-41), `PATH_MTU_FALLBACK = 1_280` (C-06), `MAX_UDP_PAYLOAD = 1_252` (C-17, C-100), `MAX_BUNDLE_MEMBERS = 32` (C-08), `DSCP_SYNC = 48`, `DSCP_MEDIA = 46`, `DSCP_PROBE = 46`, `DSCP_CONTROL = 40`, `DSCP_REPORT = 40` (C-13). Reuse epic 004's definitions where they already exist instead of duplicating them.
3. Add a CI job that builds `hadronyche-core` (with this module) for a `no_std` target that has no allocator (for example `thumbv7em-none-eabihf`), with no global allocator defined.

## Acceptance criteria
1. `hadronyche-core` builds for a `no_std` target with no allocator in CI (epic AC-11); the job fails if anything in the module pulls in `std` or `alloc`.
2. Each constant above exists once with the stated value, and a unit test asserts `REPORT_INTERVAL_NS / SLICE_NS == 200`, `PRODUCER_TIMEOUT_NS / REPORT_INTERVAL_NS == 20`, `SLICE_FRAMES * 1_000_000_000 / DOMAIN_RATE_HZ == SLICE_NS` and `PATH_MTU_FALLBACK - 20 - 8 == MAX_UDP_PAYLOAD`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] The `no_std`, no-allocator build job runs on every change to `hadronyche-core`

## Out of scope
- Message layouts (epic 004) and any encoder or parser (014-03, 014-04, 014-08).
- Sending on sockets (epic 015).
