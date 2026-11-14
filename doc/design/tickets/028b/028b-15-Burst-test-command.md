# 028b-15 — `burst-test` command
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** M · **Depends on:** 028a-02, 028b-01, 028b-08, epic 019

## Context
Sources fire once per 0.5 ms slice (C-02). Two 96-channel bundles arriving at one port are **32 packets (about 42 KB) per slice** (16 streams each per C-22, 64 000 packets/s together; about 1 320 bytes per packet on the wire, C-20). The runtime spreads packets over at most **25 % of the slice** (C-14), i.e. 125 µs, with a per-source phase offset derived from the module id. Switch buffers must absorb this. Burst behaviour was measured at L1 (epic 019). The tool runs a burst test and reports drops.

Settled (C-107, C-153): the tool may drive test traffic only inside a declared commissioning window that every module stamps; outside it, it is read-only. The window is announced with an expiry and closes at expiry or on explicit close.

Pacing fit (C-155): phase offset plus spread stay inside the slice period, so the phase offset is in [0, 75 %) of the period. The test traffic is IPv4, multicast TTL 1 where multicast is used (C-100), marked EF (46) like media (C-13).

## Work
- `burst-test --target <port/node> --sources <n> --channels <c> [--duration <d>]` (default 10 minutes, the L1 short-scenario length, C-167): check the window with the guard (028b-08), including that it will not expire before the run ends (C-153), then generate test traffic matching the runtime's pacing (at most 25 % of the slice, per-source phase offset within [0, 75 %) of the period, C-14, C-155) into one destination port, and count drops at the receiver per slice.
- Report: packets sent, received, dropped, per-source; worst slice.
- Stop and report if the window closes during the run.

## Acceptance criteria
1. Two 96-channel sources into one GbE port (32 packets ≈ 42 KB per slice) run for a configurable duration and drops are reported per source.
2. On the L1 rig, results agree with epic 019's burst measurements in drop count order of magnitude.
3. Outside a commissioning window the command refuses to start, naming the modules not stamping it (epic AC-10).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Switch QoS recommendations (028b-19). The window guard itself (028b-08).
