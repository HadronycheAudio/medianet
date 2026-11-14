# 001-04 — macOS don't-fragment test
**Epic:** 001-Receive-stamp-probes · **Effort:** S · **Depends on:** 001-01, 001-02

## Context
IP fragmentation is forbidden in the Medianet runtime: every media packet is sent with the IPv4 don't-fragment bit set; the network is IPv4 only in v1 (C-100). At admission the two ends run a probe pair (a padded don't-fragment packet at the intended media packet size, plus an acknowledgement; silence means the path is smaller). The probe pair is sent at the intended media packet size and is exempt from the control message bound C-17 (C-116). On failure the connection falls back to a path MTU of 1 280 bytes (C-06), 1 252 bytes of UDP payload (C-100); PathMtuTooSmall applies only when the probe at the fallback also fails (C-173). The default media packet carries 1 252 bytes of UDP payload (C-137), about 1 320 bytes on the wire including all Ethernet overhead (C-20).

This run puts no number in the constants table, so it needs no strict mode (C-147). It is made on the current macOS release; the oldest obtainable release is 001-16 (C-138).

Linux uses `IP_MTU_DISCOVER`. The claim to verify: macOS has an IPv4 don't-fragment socket option that differs from Linux's `IP_MTU_DISCOVER` `[memory]` (candidate from SDK headers: `IP_DONTFRAG`, also `[memory]` until confirmed). IPv6 is out of v1 and is not tested.

## Work
- Add a `df` mode to `probes/macos-rxstamp/` that:
  - sets the macOS don't-fragment option on an IPv4 UDP socket and logs option name, numeric value and `setsockopt` result;
  - sends to the Linux peer one datagram of the default media size (C-20), one sized to fit the interface MTU exactly, and one larger than the interface MTU (MTU read from the interface);
  - logs the return value / errno of each send.
- On the Linux peer, capture with the command from `probes/linux-peer/CAPTURE.md` (001-01) during the test; commit the `.pcap` and the capture summary under `doc/design/reports/` (C-130).

## Acceptance criteria
1. Output names the option and its numeric value, and shows `setsockopt` succeeded.
2. The oversized send either fails locally (errno logged, e.g. `EMSGSIZE`) or is dropped; it is never fragmented.
3. The committed peer capture shows 0 IP fragments from the macOS host for the oversized send, and shows the default-size and the MTU-sized datagrams arriving whole with DF set (epic 001-AC4).

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Probe sources committed so the measurements can be repeated
- [ ] Packet capture committed with the result

Not applicable (C-125): "Tests green in CI, unprivileged" — this ticket adds a hardware measurement mode with no testable pure logic; "Every parser touched is fuzzed" — no parser added; "Regime stamp fields touched" — none (001-12); "Design documents updated" — 001-14; "No new open item" — opens none; "The frozen POC is not modified" — not touched; "`REPORT-0a.md` delivered before task 004" — 001-13.

## Out of scope
- The admission probe pair protocol itself (wire specification, task 004).
- IPv6 (excluded by C-100).
- Windows (001-09).
- The run on the oldest obtainable macOS release (001-16).
