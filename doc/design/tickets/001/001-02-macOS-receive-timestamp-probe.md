# 001-02 — macOS receive timestamp probe
**Epic:** 001-Receive-stamp-probes · **Effort:** M · **Depends on:** 001-01

## Context
Epic 001 (check **0a**) must establish whether macOS offers a kernel receive timestamp for UDP and on which clock basis. macOS machines join Medianets as edges only (sinks and physical-input capture). Each node estimates its clock offset from request/reply exchanges (50 per second per follower, C-36); the receive-stamp quality sets the clock bound, one of three terms of the emission error E. The macOS E target is 100 µs (C-71); the Linux reference is 50 µs (C-70), with lock bound at most E/2 = 25 µs on Linux (C-62, pending simulation). The network is IPv4 only (C-100).

Claims to verify on macOS:
- A kernel receive timestamp exists for UDP, possibly on a monotonic (mach) basis through a Darwin-specific socket option, in addition to the BSD wall-clock `SO_TIMESTAMP` `[memory]`. The candidate name `SO_TIMESTAMP_MONOTONIC` is also `[memory]` until confirmed from SDK headers.

Reference behaviour on Linux: `SO_TIMESTAMPNS` delivers a `CLOCK_REALTIME` stamp in a control message `[kernel-doc]`. Because it is on the wall clock, a wall-clock step between stamp and read must be detected and the sample discarded; in the runtime that detection lives in the platform backend that builds exchanges from receive stamps (C-121). The probe only flags steps so the measurement can exclude them.

Settled for this probe (epic Background): the probe reads every candidate clock right after the receive call (`mach_absolute_time`, `mach_continuous_time`, wall clock) and compares the stamp against the clock matching its observed basis. Whether a clock advances during sleep does not matter: a sleeping laptop edge is a departure and returns through normal admission (C-124).

Capability reporting (for 001-12): whatever the clock basis, a hosted OS kernel stamp is the source value `kernel`; a clock read after receive is `userspace` (C-131). The clock basis is reported separately, because the macOS backend (task 021) needs it for step detection (C-121).

The probe lives in `probes/macos-rxstamp/` in the design set's repository (C-133); its run outputs go under `doc/design/reports/` (C-130). The strict-mode run conditions (real-time scheduling, memory lock) are added by 001-17; runs on the oldest obtainable macOS release are 001-16 (C-138).

Parsers are fuzzed 60 s on every change and 1 hour nightly, corpus kept (C-95); CI wiring is 001-15.

Open sub-question owned by this ticket: "Does macOS offer a monotonic-basis kernel receive stamp?" Falsifier: the probe shows wall-clock only; then the backend step detection (C-121) applies on macOS as on Linux.

## Work
- Create `probes/macos-rxstamp/` (C or Rust), a standalone program that:
  - opens an IPv4 UDP socket and enables every candidate receive-timestamp socket option available in the macOS SDK headers (at least `SO_TIMESTAMP` and any Darwin-specific monotonic option found); logs for each option its name, numeric value and whether `setsockopt` succeeded;
  - receives packets from the Linux peer (001-01) with `recvmsg`, parses every control message, and prints per packet: the payload sequence number, the kernel stamp, its clock basis (identified by control-message type), and reads of `mach_absolute_time`, `mach_continuous_time` and the wall clock taken immediately after the receive call;
  - flags samples where the wall clock stepped between stamp and read when the basis is the wall clock;
  - records the macOS version (`sw_vers`), hardware, NIC and link speed in the output header.
- Control-message parsing is a separate function with unit tests on captured byte buffers and a fuzz target. Keep it free of macOS-only APIs so it can be fuzzed on the Linux CI runners (001-15).

## Acceptance criteria
1. Output states, for each candidate option, its name, numeric value and enabled yes/no, and for each received packet which control-message type(s) carried a stamp.
2. Output answers "a kernel UDP receive timestamp exists" yes or no, names the option, and states its clock basis (monotonic/mach, continuous, or wall clock), from observed control messages (epic 001-AC1).
3. Per-packet lines contain: sequence number, kernel stamp, basis, and the three clock reads; wall-clock steps are flagged.
4. The control-message parser has unit tests passing in CI, unprivileged, and its fuzz target completes a 60 s run without a crash (C-95).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Probe sources committed so the measurements can be repeated

Not applicable (C-125): "Regime stamp fields touched" — the capability table is 001-12; "Design documents updated" — done in 001-14; "No new open item" — this ticket answers an open item and opens none; "`REPORT-0a.md` delivered before task 004" — the report is 001-13.

## Out of scope
- The 100 000-packet distribution run and statistics (001-03).
- Strict-mode run conditions (001-17) and runs on the oldest obtainable macOS release (001-16).
- Don't-fragment test (001-04) and drop counter check (001-05).
- Whether NIC interrupt coalescing biases stamps: owned by L1 (task 019), not here.
- The runtime's step detection (C-121) and the macOS backend (task 021).
