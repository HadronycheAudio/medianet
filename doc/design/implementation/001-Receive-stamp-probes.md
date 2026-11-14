# 001 — Receive stamp probes

**Phase:** P0 · **Scope:** L (17 tickets, about 17 days) · **Depends on:** none · **Blocks:** 004, 013, 015, 021, 022

## Goal

Replace five from-memory platform claims with measured or cited facts before the wire specification is drafted: whether macOS and Windows offer a kernel receive timestamp for UDP and on which clock basis, how don't-fragment is set on each OS, whether either OS has a per-socket drop counter, and what the published Ethernet bit-error figure is. This is pre-implementation check **0a**. It does not block the specification, but its report is an input to it.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules (sources, processors, sinks) discover each other, agree on one clock holder and exchange *slices*: the content of one stream for one stretch of media time, addressed by *position* on a domain-wide timeline. The wire carries one sample rate only, 96 kHz (C-01); an edge whose hardware runs at 48 kHz converts at the edge (C-80). The default slice is 48 frames, 0.5 ms, so a stream sends 2 000 packets/s (C-02). The network is IPv4 only in v1, multicast with TTL 1 (C-100). Linux hosts are the backbone (clock holders, elections, conformance, commissioning). macOS and Windows machines join as edges only (sinks and physical-input capture). Embedded devices are followers.

**Evidence tags used in this epic.** `[code]` read in the POC source · `[measured]` observed in a run · `[kernel-doc]` documented kernel behaviour · `[memory]` recalled, not yet verified · `[decision]` a design choice · `[standard]` a figure cited from a named, published standard, with the clause; it replaces `[memory]` once the citation is checked (C-132). The tag set is defined in `00-context.md`.

**Default packet.** A default media packet carries 6 channels × 192 bytes = 1 152 bytes of audio, with no per-channel id byte (the channel map is in the manifest) (C-04). A default slice must fit 1 252 bytes of UDP payload: the 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum (C-137). The probes therefore use a UDP payload of 1 252 bytes, the cap, which is a 1 280-byte IPv4 datagram; the exact header layout belongs to the wire specification (task 004) and does not change the probe size. Its wire size is about 1 320 bytes including all Ethernet overhead (C-20), that is about 21.1 Mbit/s per stream (C-21). "Wire packet" in this epic means exactly that: the C-20 size, preamble, inter-frame gap and FCS included. The probe runs use this packet size (deliverables 1, 2 and 6).

**Why receive timestamps matter.** Every node estimates its offset to the clock holder from request/reply exchanges (50 per second per follower, C-36; accepted range 10 to 200 per second, C-112). The quality of the receive timestamp sets the clock bound, and the clock bound is one of three terms of every edge's emission error E:

| Platform | E target | Budget split (clock + tracking + uncertainty of D) | Constants |
|---|---|---|---|
| Linux reference sink | 50 µs | 20 + 20 + 10 µs | C-70 |
| macOS, built-in or bus-attached device | 100 µs | 40 + 40 + 20 µs (proposal, pending L2) | C-71, C-150 |
| Windows, WASAPI exclusive | 1 ms | 100 + 100 + 800 µs (proposal, pending L2) | C-72, C-150 |

E is the 99.9th percentile of absolute emission error over a one-hour run, with the maximum reported beside it (C-73). The stamp-to-read figure this epic compares against the E budgets is therefore the 99.9th percentile; median and maximum are reported beside it. The threshold it is compared with is the clock share of the platform's target E: 40 µs on macOS and 100 µs on Windows (C-150), always the target, never an E measured in the same run (C-149). A 99.9th percentile above the clock share leads to a proposed revision of C-150 (and of C-71 or C-72 if the total cannot hold). The lock bound required of the servo is at most E/2, that is 25 µs on Linux (C-62, pending simulation). C-70, C-71, C-72 and the C-150 split are pending measurement and owned by L2 (task 020); this epic supplies evidence on their clock-bound term and proposes a revision when that evidence warrants one, it does not close them.

**Run conditions.** Any run whose numbers enter the constants table runs in strict mode (C-147). The stamp-to-read runs feed C-150, C-71 and C-72, so they run in strict mode: real-time scheduling granted on the receive thread (the probe's data and clock thread) and memory locked; restricting CPU idle states is best-effort and recorded (C-127). A run where either required condition is unverified is treated as one where it is absent (C-128) and its figures are not proposed for the table. The don't-fragment and drop-counter runs put no number in the table and need no strict mode.

**Tested OS versions.** The supported OS versions are set after this task reports; the probes test the current release of each OS and the oldest obtainable one (C-138).

**What is known on Linux.** `SO_TIMESTAMPNS` delivers a `CLOCK_REALTIME` stamp in a control message `[kernel-doc]` (socket(7)). The POC uses it (`net.rs:33`, parsed at `net.rs:122-130`) `[code]`. Because the stamp is on the wall clock, a wall-clock step between the stamp and the read must be detected and the sample discarded. That detection lives in the platform backend that builds each exchange from receive stamps; the servo only ever receives exchanges that passed it (C-121). `SO_RXQ_OVFL` (since Linux 2.6.33) provides a cumulative per-socket drop counter in a control message `[kernel-doc]`.

**What is claimed, unverified, on the other platforms.**

- macOS: a kernel receive timestamp exists for UDP, possibly on a monotonic (mach) basis through a Darwin-specific socket option, in addition to the BSD wall-clock `SO_TIMESTAMP` `[memory]`.
- Windows: recent Windows 10 and 11 builds offer a Winsock timestamping control for UDP receive, stamped in QueryPerformanceCounter units `[memory]`. If absent, the starting point is a userspace read of the clock immediately after the receive call.
- macOS has an IPv4 don't-fragment socket option that differs from Linux's `IP_MTU_DISCOVER`; Windows has its own option `[memory]`.
- Neither macOS nor Windows has a per-socket drop counter equivalent to `SO_RXQ_OVFL` `[memory]`.

Candidate option names (`SO_TIMESTAMP_MONOTONIC`, `IP_DONTFRAG` on macOS; `SIO_TIMESTAMPING`, `IP_DONTFRAGMENT` on Windows) are equally `[memory]` until the probes confirm them from SDK headers and runs.

**Which clock the probe compares against.** The (userspace read − kernel stamp) difference is only meaningful when both are on the same basis. Each probe therefore reads, immediately after the receive call, every candidate clock (macOS: `mach_absolute_time`, `mach_continuous_time` and the wall clock; Windows: QueryPerformanceCounter and the wall clock) and computes the difference against the clock that matches the stamp's observed basis `[decision]`. Whether a clock advances during sleep does not matter for the runtime: a laptop edge that sleeps is treated as a departure, both timebase loops are reset and it returns through normal admission (C-124).

**Why don't-fragment matters.** IP fragmentation is forbidden in the runtime: every media packet is sent with the IPv4 don't-fragment bit set (C-100). At admission the two ends run a probe pair (a padded don't-fragment packet at the intended media packet size, plus an acknowledgement; silence means the path is smaller). The probe pair is exempt from the control message bound C-17 because it is sent at the intended media packet size (C-116). On failure the connection falls back to a path MTU of 1 280 bytes (C-06), which gives 1 252 bytes of UDP payload (C-100, C-17); the refusal PathMtuTooSmall applies only when the probe at the 1 280-byte fallback also fails (C-173).

**Why the bit-error figure matters.** The design publishes an expected loss floor (C-90): at a bit error rate of 1e-12, about 1.1e-8 per default packet (about 1 320 bytes on the wire, C-20), one loss per stream per link about every 13 hours at 2 000 packets/s `[memory]`. A looser 1e-10 figure for gigabit copper was also recalled `[memory]`. The 24-hour loss soak at the two-bundle load (64 000 packets/s, about 5.5e9 packets) predicts about 58 losses from the same figure (C-118, C-92). The same figure sets when measured loss reopens the redundancy decision: clean-medianet loss above 10 × the floor, about 580 losses in the soak, reopens it; between 1 × and 10 × it is recorded and investigated as a cabling or host problem (C-179). Measured loss above the floor raises a cable-health alarm whose threshold L1 sets (C-91), so the figure must be sourced.

**Capability bits.** The regime stamp (a wire object every node publishes) carries capabilities with four states: present, absent, unverified, not requested; "not requested" marks a deliberate configuration choice, as distinct from a denial (C-129). The receive-timestamp source values are kernel (hosted OS kernel stamp), userspace (clock read after receive), mac-hardware (Ethernet MAC timestamp register, embedded) and interrupt (stamped in the receive interrupt, embedded), each reported with one of the four states (C-131). The stamp's clock basis (monotonic or wall clock) is not a source value; the report gives it per platform for the backends' step detection (C-121). The stamp travels in a dedicated control message split into sections that each fit C-17, multicast on any regime change and refreshed every 10 s; the hello carries only a stamp version number (C-18). This task defines the bits for receive timestamp source, drop accounting and don't-fragment; their wire encoding and width belong to the regime stamp specification (task 013).

**Where code and results go.** Everything lives in the design set's repository. The probes, the Linux peer included, are throwaway probes under `probes/` (C-133). The report and the raw results (outputs, summaries, packet captures) go in `doc/design/reports/` (C-130). `[memory]` claims are re-tagged or struck in `doc/design/reports/REPORT-0a.md`, then in the constants table (`doc/design/02-constants.md`, edited first) and in any design document under `doc/design/` that repeats them, including `doc/design/01-architecture.md` (C-171) and this epic's Background `[decision]`. Nothing is written into the frozen POC tree (C-130).

**Fuzzing and CI.** Each parser is fuzzed for 60 s on every change and for 1 hour nightly, and its corpus is kept (C-95). In this epic the parsers are the control-message parsers of both probes and the payload decoder of the Linux peer. The CI line of the definition of done covers unit tests of those parsers and of the statistics code. The hardware runs (macOS and Windows machines wired to a Linux peer, packet capture on the peer) are measurements, not CI tests; they are made repeatable by committing sources, commands and raw output `[decision]`. "Unprivileged" applies to per-change CI (C-146); the privilege that strict-mode runs or a packet capture may need belongs to the measurement runs, not to CI.

## Deliverables

1. `probes/macos-rxstamp/` — one small standalone program (any language; C or Rust) that:
   - opens an IPv4 UDP socket, enables every candidate receive-timestamp option, receives packets from the Linux peer (deliverable 6) and prints, per packet: the kernel stamp, its clock basis, and a read of each candidate clock taken immediately after the receive call (see Background, "Which clock the probe compares against");
   - reports the distribution of (userspace read − kernel stamp) on the matching clock over at least 100 000 packets of the default size (C-20) at 2 000 packets/s, from a strict-mode run (C-147, C-127);
   - tests the don't-fragment option: a send at the default media packet size arrives whole with DF set, and a send larger than the interface MTU fails or is dropped, never fragmented (confirmed with a packet capture on the peer);
   - checks whether any per-socket or per-interface drop counter is readable without privilege;
   - is run on the current macOS release and on the oldest obtainable one (C-138), reporting the minimum version on which the kernel stamp option works.
2. `probes/windows-rxstamp/` — the same, for Windows, run on the current release and the oldest obtainable build (C-138), additionally reporting the minimum OS build on which the timestamp control works, and the behaviour on a build where it does not.
3. `doc/design/reports/REPORT-0a.md` (C-130) — one page per OS with: option names and values, clock basis, required OS version, the versions tested (input to C-138), measured stamp-to-read distribution (median, 99.9th percentile, maximum) and whether its run met strict mode, don't-fragment option and observed behaviour, drop-counter availability. Each claim tagged `[measured]`.
4. A sourced statement of the Ethernet bit-error requirement for 1000BASE-T and for 10GBASE-T (standard, edition and clause cited, tagged `[standard]`, C-132), with the recomputed loss floor for a 1 320-byte wire packet (C-20) at 2 000 packets/s, and the recomputed loss-soak prediction (C-118).
5. Capability definitions, as a table to be copied into the regime stamp specification (task 013), using the source values of C-131 and the four states of C-129:

```rust
// shape, names may be refined
pub enum CapabilityState { Present, Absent, Unverified, NotRequested } // C-129
pub struct RxStampCapabilities {          // one state per source value, C-131
    pub kernel: CapabilityState,
    pub userspace: CapabilityState,
    pub mac_hardware: CapabilityState,     // embedded only
    pub interrupt: CapabilityState,        // embedded only
}
pub struct StampCapabilities {
    pub rx_stamp: RxStampCapabilities,
    pub drop_accounting: CapabilityState,
    pub dont_fragment: CapabilityState,
}
```

6. `probes/linux-peer/` (C-133) — the sender used by deliverables 1 and 2: IPv4 UDP at a configurable rate (default 2 000 packets/s) and count (default 100 000), packets of the default size (1 252 bytes of UDP payload, C-137; about 1 320 bytes on the wire, C-20) carrying a sequence number and send time, plus the documented capture command used on the peer for the don't-fragment tests.

## Dependencies

- Needs: no other task. Hardware, per C-138: a macOS machine on the current release and one on the oldest obtainable release (a second machine, partition or image), a Windows 11 machine on the current release and a Windows 10 machine (or an older image) for the oldest obtainable build, each wired to a Linux peer.
- Provides to 004 (wire specification): confirmed option set, so the specification does not assume Linux-only facilities.
- Provides to 013 (regime stamp): the capability-bit table.
- Provides to 015, 021, 022 (hosted, macOS and Windows backends): the exact socket options to use, and the clock basis on which each backend runs the wall-clock step detection of C-121.
- Provides to the constants table: a confirmed or corrected C-90 (and C-118 and the C-179 reopen count if C-90 changes), re-tagged `[standard]` (C-132); evidence for the clock share of C-150 and so for C-71 and C-72, with a proposed revision if the 99.9th percentile exceeds the clock share or kernel stamps are absent; and the tested versions and minimum versions from which the supported OS versions of C-138 are set.
- Hands open items to 019 (L1): interrupt-coalescing bias and the measured clean-network loss.

## Acceptance criteria

1. **001-AC1.** For macOS: the report states yes or no to "a kernel UDP receive timestamp exists", names the option, states its clock basis and the minimum macOS version on which it works, each backed by program output from the current release and the oldest obtainable release (C-138).
2. **001-AC2.** For Windows: the report states yes or no, names the control, gives the minimum build, and shows the fallback behaviour on an older build or states why no older build could be run.
3. **001-AC3.** For each OS the stamp-to-read distribution is reported over at least 100 000 packets of the default size, with the 99.9th percentile (C-73), median and maximum, from a run that met strict mode (C-147); a run that did not is reported as such and not used for a table value.
4. **001-AC4.** For each OS a packet capture shows that an oversized don't-fragment send is not fragmented on the wire, and that a default-size send arrives whole with DF set.
5. **001-AC5.** The bit-error figure is cited from the standard (edition and clause), and C-90 is confirmed or a corrected value is proposed.
6. **001-AC6.** Every `[memory]` claim listed in Background is re-tagged `[measured]`, replaced by a citation (standard, edition, clause), or struck.
7. **001-AC7.** Contributes to system criteria AC-27 (macOS E at most 100 µs, restart spread at most 50 µs) and AC-28 (Windows E at most 1 ms, restart spread at most 0.5 ms) by comparing the stamp-to-read 99.9th percentile with the clock share of C-150 (macOS 40 µs, Windows 100 µs) and confirming it or proposing a revision.
8. **001-AC8.** The Linux peer (deliverable 6) sends 100 000 default-size packets at 2 000 packets/s ± 2 % with contiguous sequence numbers.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] `REPORT-0a.md` delivered before task 004 freezes its option assumptions
- [ ] Probe sources committed so the measurements can be repeated

**Not applicable in this epic (C-125).** No line is wholly without a subject. Two apply in part: "Tests green in CI, unprivileged" covers the unit tests of parsers and statistics only, not the hardware runs, which need a macOS or Windows machine wired to a Linux peer and a packet capture, and may need privilege for strict mode or capture ("unprivileged" applies to per-change CI, C-146); "Regime stamp fields touched are complete and documented" covers the capability-bit table this epic defines, not the stamp's wire encoding, which belongs to task 013. "The frozen POC is not modified" applies as a constraint: the probes live outside the POC and only cite it.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Does macOS offer a monotonic-basis kernel receive stamp? | This task (ticket 001-02) | Probe shows wall-clock only; then the backend step detection (C-121) applies on macOS as on Linux |
| From which macOS version is the kernel receive stamp available? | This task (ticket 001-16) | Not available on the current release; the kernel source is absent on macOS and the C-150 clock share must hold with userspace stamps |
| From which Windows build is kernel receive stamping available? | This task (ticket 001-08) | Not available on the current release; clock bound and the C-150 / C-72 budget rise |
| Does the stamp-to-read 99.9th percentile fit the C-150 clock share (macOS 40 µs, Windows 100 µs)? | This task (tickets 001-03, 001-07, 001-13), then L2 (task 020) | The strict-mode 99.9th percentile exceeds the clock share |
| Is the loss floor C-90 computed from the right bit-error figure? | This task (ticket 001-11), then L1 (task 019) | Standard cites a different figure; clean-network loss well above the floor |
| Does NIC interrupt coalescing bias the stamps measured here? | L1 (task 019) | Offset shifts when coalescing is toggled |

Settled since the ticket split: the clock share of the macOS and Windows budgets (C-150); which OS versions are tested (current and oldest obtainable, C-138); the meaning of the receive-timestamp source values (C-131) and the fourth capability state (C-129); the evidence tag for a cited standard (C-132); the location of `probes/` (C-133) and of the report and results (C-130); the probe payload size (C-137); the loss level that reopens the redundancy decision, 10 × the floor (C-179).
