# 001-06 — Windows receive timestamp probe
**Epic:** 001-Receive-stamp-probes · **Effort:** M · **Depends on:** 001-01

## Context
Epic 001 (check **0a**) must establish whether Windows offers a kernel receive timestamp for UDP and on which clock basis. Windows machines join Medianets as edges only (sinks via WASAPI exclusive, and physical-input capture). Each node estimates its clock offset from request/reply exchanges (50 per second per follower, C-36); receive-stamp quality sets the clock bound, one term of the emission error E. Windows E target is 1 ms (C-72); macOS 100 µs (C-71); Linux 50 µs (C-70). The network is IPv4 only (C-100).

Claim to verify: recent Windows 10 and 11 builds offer a Winsock timestamping control for UDP receive, stamped in QueryPerformanceCounter units `[memory]` (candidate: `SIO_TIMESTAMPING` ioctl with `SO_TIMESTAMP` control messages, also `[memory]` until confirmed). If absent, the starting point is a userspace read of the clock immediately after the receive call.

Settled for this probe (epic Background): the probe reads every candidate clock right after the receive call (QueryPerformanceCounter and the wall clock) and compares the stamp against the clock matching its observed basis. If the stamp is on the wall clock, steps are flagged; in the runtime, step detection lives in the platform backend (C-121). Whether a clock advances during sleep does not matter: a sleeping laptop edge is a departure and returns through normal admission (C-124).

Capability reporting (for 001-12): a hosted OS kernel stamp is the source value `kernel` whatever its units; a clock read after receive, including fallback mode, is `userspace` (C-131). The clock basis and units are reported separately for the Windows backend's step detection (task 022, C-121).

The probe lives in `probes/windows-rxstamp/` in the design set's repository (C-133); its run outputs go under `doc/design/reports/` (C-130). The strict-mode run conditions are added by 001-17.

Parsers are fuzzed 60 s on every change and 1 hour nightly, corpus kept (C-95); CI wiring is 001-15.

## Work
- Create `probes/windows-rxstamp/` (C or Rust), a standalone program that:
  - opens an IPv4 UDP socket, attempts every candidate receive-timestamp control, and logs for each its name, numeric code and result (success / WSA error);
  - receives from the Linux peer (001-01) with `WSARecvMsg`, parses control messages, and prints per packet: sequence number, kernel stamp, its clock basis/units (QPC or other), and a `QueryPerformanceCounter` read and a wall-clock read taken immediately after the receive call;
  - when no kernel stamp is available, runs in fallback mode: prints only the userspace reads and states "fallback: userspace";
  - prints Windows version and build number (e.g. from `RtlGetVersion`), NIC and link speed in the output header.
- Control-message parsing as a separate function with unit tests and a fuzz target. Keep it free of Windows-only APIs so it can be fuzzed on the Linux CI runners (001-15).

## Acceptance criteria
1. Output answers "a kernel UDP receive timestamp exists on this build" yes or no, names the control, and states the clock basis/units (epic 001-AC2).
2. Per-packet lines contain sequence number, kernel stamp (or none), basis, userspace QPC and wall-clock reads.
3. On a build without the control, the program does not fail: it reports the error and runs in fallback mode.
4. The control-message parser has unit tests passing in CI, unprivileged, and its fuzz target completes a 60 s run without a crash (C-95).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Probe sources committed so the measurements can be repeated

Not applicable (C-125): "Regime stamp fields touched" — the capability table is 001-12; "Design documents updated" — 001-14; "No new open item" — opens none; "`REPORT-0a.md` delivered before task 004" — 001-13.

## Out of scope
- The 100 000-packet distribution run (001-07).
- Minimum-build determination (001-08).
- Don't-fragment (001-09), drop counters (001-10).
- Strict-mode run conditions (001-17).
- The runtime's step detection (C-121) and the Windows backend (task 022).
