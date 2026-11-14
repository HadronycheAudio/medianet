# 001-01 — Linux peer traffic generator
**Epic:** 001-Receive-stamp-probes · **Effort:** S · **Depends on:** none

## Context
Epic 001 (pre-implementation check **0a**) replaces from-memory platform claims about macOS and Windows with measured facts. Both the macOS probe (`probes/macos-rxstamp/`) and the Windows probe (`probes/windows-rxstamp/`) receive UDP packets from a Linux peer wired to the machine under test. This ticket is epic deliverable 6, `probes/linux-peer/`: the sender and the capture procedure on that peer. The peer is a throwaway probe, so it lives under `probes/` in the design set's repository, not in the `hadronyche-tools` crate (C-133).

Facts the peer must support:
- The network is IPv4 only in v1 (C-100); the peer sends IPv4 UDP only.
- Stamp-distribution runs need at least 100 000 packets at 2 000 packets/s (about 50 s per run); 2 000 packets/s is one stream at the default slice of 0.5 ms (C-02).
- Packets are of the default media size. A default slice must fit 1 252 bytes of UDP payload: 1 152 bytes of audio (C-04) plus at most 100 bytes for all headers and the checksum (C-137). The peer sends a UDP payload of 1 252 bytes, the cap (a 1 280-byte IPv4 datagram), which is about 1 320 bytes on the wire including all Ethernet overhead (preamble, inter-frame gap and FCS included) (C-20). The header layout belongs to the wire specification (task 004) and does not change this size; the peer prints the payload length and the wire size it used.
- The don't-fragment tests send, from the macOS / Windows host, a default-size datagram and one larger than the interface MTU; the peer must capture what arrives and show that no IP fragments arrive.
- IP fragmentation is forbidden in the runtime; the fallback path MTU is 1 280 bytes (C-06), 1 252 bytes of UDP payload (C-100).
- On Linux, `SO_TIMESTAMPNS` gives a `CLOCK_REALTIME` stamp in a control message `[kernel-doc]` (socket(7)); this ticket does not need it, but the peer may log its own send times for cross-checks.

The payload decoder is a parser and is fuzzed: 60 s on every change and 1 hour nightly, corpus kept (C-95). The CI wiring of that budget is ticket 001-15.

## Work
- Create `probes/linux-peer/` (C or Rust) with a sender that:
  - sends IPv4 UDP datagrams to a given address/port at a configurable rate (default 2 000 packets/s) for a configurable count (default 100 000);
  - puts a 64-bit sequence number and a send time in each payload; UDP payload length defaults to 1 252 bytes (C-137) and is configurable;
  - prints achieved rate, count, UDP payload length and assumed wire size at the end.
- Add `probes/linux-peer/CAPTURE.md` with the exact capture command used on the peer during don't-fragment tests (for example a `tcpdump` filter for the probe port and for IPv4 fragments: `ip[6:2] & 0x3fff != 0`) and how to save the `.pcap` with the results under `doc/design/reports/` (C-130). Capture may need privilege on the peer; it is a measurement step, not a CI test ("unprivileged" applies to per-change CI, C-146).
- Unit tests for payload encode/decode and for the rate pacer (pure logic, no network), runnable unprivileged in CI.
- A fuzz target for the payload decoder, runnable for a fixed duration from the command line.

## Acceptance criteria
1. A run with defaults sends exactly 100 000 datagrams in 50 s ± 1 s (2 000 packets/s ± 2 %) over loopback, measured by the program's own summary (epic 001-AC8).
2. With defaults, the printed UDP payload length is 1 252 bytes (C-137) and the printed wire size is the C-20 figure (about 1 320 bytes).
3. Payload sequence numbers are contiguous from 0 to count − 1; the decoder used by the macOS and Windows probes can detect gaps.
4. `CAPTURE.md` gives one command that, when run on the peer, lists any IP fragment received from the probe host; on an unfragmented run it lists 0 fragments.
5. Unit tests pass in CI without root or capabilities; the payload-decoder fuzz target completes a 60 s run without a crash (C-95).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed (the payload decoder)
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Probe sources committed so the measurements can be repeated

Not applicable (C-125): "Regime stamp fields touched" — no stamp field here; "Design documents updated" — no design behaviour changes here (001-14); "No new open item" — this ticket opens none; "`REPORT-0a.md` delivered before task 004" — the report is 001-13.

## Out of scope
- Receiving and timestamping on macOS / Windows (001-02, 001-06).
- Statistics on stamp-to-read deltas (001-03, 001-07).
- The don't-fragment send itself (001-04, 001-09); this ticket only provides the capture side.
- CI fuzz scheduling and corpus storage (001-15).
- The media packet header layout (task 004).
