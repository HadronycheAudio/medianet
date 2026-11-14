# 001-09 — Windows don't-fragment test
**Epic:** 001-Receive-stamp-probes · **Effort:** S · **Depends on:** 001-01, 001-06

## Context
IP fragmentation is forbidden in the Medianet runtime: every media packet is sent with the IPv4 don't-fragment bit set; the network is IPv4 only in v1 (C-100). At admission the two ends run a probe pair (a padded don't-fragment packet at the intended media packet size, plus an acknowledgement; silence means the path is smaller). The probe pair is sent at the intended media packet size and is exempt from the control message bound C-17 (C-116). On failure the connection falls back to a path MTU of 1 280 bytes (C-06), 1 252 bytes of UDP payload (C-100); PathMtuTooSmall applies only when the probe at the fallback also fails (C-173). The default media packet carries 1 252 bytes of UDP payload (C-137), about 1 320 bytes on the wire including all Ethernet overhead (C-20).

This run puts no number in the constants table, so it needs no strict mode (C-147). It runs on the current release and the oldest obtainable build (C-138).

Linux uses `IP_MTU_DISCOVER`. Claim to verify: Windows has its own IPv4 don't-fragment option `[memory]` (candidate: `IP_DONTFRAGMENT`, also `[memory]` until confirmed). IPv6 is out of v1 and is not tested.

## Work
- Add a `df` mode to `probes/windows-rxstamp/` that:
  - sets the Windows don't-fragment option on an IPv4 UDP socket and logs option name, numeric value and result;
  - sends to the Linux peer one datagram of the default media size (C-20), one exactly at interface MTU and one larger than interface MTU, logging each send's return / WSA error.
- On the Linux peer, capture with the command from `probes/linux-peer/CAPTURE.md` (001-01); commit `.pcap` and summary under `doc/design/reports/` (C-130).
- Run on both the Windows 11 machine (current release) and the Windows 10 machine (oldest obtainable build, C-138).

## Acceptance criteria
1. Output names the option and its numeric value, and shows it was set successfully, on both machines.
2. The oversized send fails locally (error logged, e.g. `WSAEMSGSIZE`) or is dropped; it is never fragmented.
3. Committed peer captures show 0 IP fragments from the Windows host for the oversized send, and the default-size and MTU-sized datagrams arriving whole with DF set (epic 001-AC4).

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Probe sources committed so the measurements can be repeated
- [ ] Packet capture committed with the result

Not applicable (C-125): "Tests green in CI, unprivileged" — hardware measurement mode with no testable pure logic; "Every parser touched is fuzzed" — no parser added; "Regime stamp fields touched" — none (001-12); "Design documents updated" — 001-14; "No new open item" — opens none; "The frozen POC is not modified" — not touched; "`REPORT-0a.md` delivered before task 004" — 001-13.

## Out of scope
- Admission probe pair protocol (task 004).
- IPv6 (excluded by C-100).
- macOS (001-04).
