# Epic 001 — Receive stamp probes: tickets

## Summary
Epic 001 (phase P0, pre-implementation check **0a**) replaces five from-memory platform claims with measured or cited facts before the wire specification (task 004) is drafted: whether macOS and Windows offer a kernel UDP receive timestamp and on which clock basis, how the IPv4 don't-fragment bit is set on each OS (IPv4 only, C-100), whether a per-socket drop counter exists, and what the published Ethernet bit-error figure is. It delivers a Linux peer sender (`probes/linux-peer/`), two standalone probes (`probes/macos-rxstamp/`, `probes/windows-rxstamp/`) in the design set's repository (C-133), a report `doc/design/reports/REPORT-0a.md` with its raw results beside it (C-130), a `[standard]`-tagged loss-floor statement confirming or correcting C-90 (and C-118 with it) (C-132), and capability definitions (receive-stamp source values of C-131, four states of C-129) for the regime stamp (task 013). Probe runs use the default media packet, 1 252 bytes of UDP payload (C-137), about 1 320 bytes on the wire (C-20), on the current and the oldest obtainable release of each OS (C-138). Its strict-mode (C-147) stamp-to-read 99.9th percentiles are set against the clock shares of C-150 (macOS 40 µs, Windows 100 µs) behind the E targets C-71 (macOS 100 µs) and C-72 (Windows 1 ms) and system criteria AC-27 and AC-28; E is a 99.9th percentile (C-73).

**Scope:** L (17 tickets, about 17 days), per C-126.

## Tickets
| Ticket | Title | Effort | Depends on |
|---|---|---|---|
| 001-01 | Linux peer traffic generator | S | none |
| 001-02 | macOS receive timestamp probe | M | 001-01 |
| 001-03 | macOS stamp-to-read distribution | S | 001-01, 001-02, 001-17 |
| 001-04 | macOS don't-fragment test | S | 001-01, 001-02 |
| 001-05 | macOS drop counter check | XS | 001-01, 001-02 |
| 001-06 | Windows receive timestamp probe | M | 001-01 |
| 001-07 | Windows stamp-to-read distribution | S | 001-01, 001-06, 001-17 |
| 001-08 | Windows minimum build and fallback | S | 001-06 |
| 001-09 | Windows don't-fragment test | S | 001-01, 001-06 |
| 001-10 | Windows drop counter check | XS | 001-01, 001-06 |
| 001-11 | Ethernet bit-error figure and loss floor | S | none |
| 001-12 | Capability-bit definitions | S | 001-02, 001-04, 001-05, 001-06, 001-08, 001-09, 001-10, 001-16 |
| 001-13 | REPORT-0a assembly | S | 001-03, 001-04, 001-05, 001-07, 001-08, 001-09, 001-10, 001-11, 001-12, 001-16, 001-17 |
| 001-14 | Constants and design document updates | XS | 001-13 |
| 001-15 | Probe fuzz targets in CI (new, from C-95) | XS | 001-01, 001-02, 001-06 |
| 001-16 | macOS oldest-release runs and minimum version (new, from C-138) | S | 001-02, 001-03, 001-04, 001-05, 001-17 |
| 001-17 | Strict-mode run conditions for the probes (new, from C-147, C-127, C-128) | S | 001-02, 001-06 |

No ticket was deleted: no ruling removed work from this epic. The revised C-104 and C-105 (whole-sample L, withdrawn rational-L rule), C-134 (resampler only inside an edge module) and the change of C-35's owner touch nothing here: no statement or ticket of this epic depended on them. 001-05 and 001-10 depend on 001-01, whose sender they use for the forced-drop run. 001-03 and 001-07 now depend on 001-17 for the strict-mode setup.

## Effort tally
- XS: 4 (about 1 day)
- S: 11 (about 11 days)
- M: 2 (about 5 days)
- Total: 17 tickets, about 17 days → scope L (C-126: XS 0.25 day, S 1 day, M 2.5 days; above 15 and up to 40 days = L)

## Coverage
| Epic item | Tickets |
|---|---|
| Deliverable 1 — `probes/macos-rxstamp/` (timestamp options, per-packet print, all candidate clocks) | 001-02 (peer: 001-01) |
| Deliverable 1 — distribution over ≥ 100 000 default-size packets at 2 000 packets/s, strict mode (C-147) | 001-03 (setup: 001-17) |
| Deliverable 1 — don't-fragment test with peer capture (default size, MTU, oversized) | 001-04 |
| Deliverable 1 — unprivileged drop counter check | 001-05 |
| Deliverable 1 — current and oldest obtainable macOS release, minimum version (C-138) | 001-02 to 001-05 (current), 001-16 (oldest) |
| Deliverable 2 — `probes/windows-rxstamp/` (same as 1) | 001-06, 001-07, 001-09, 001-10 (peer: 001-01; strict mode: 001-17) |
| Deliverable 2 — minimum OS build and behaviour on older build | 001-08 |
| Deliverable 3 — `doc/design/reports/REPORT-0a.md` (C-130), with versions tested (C-138) | 001-13 |
| Deliverable 4 — sourced BER for 1000BASE-T / 10GBASE-T, loss floor at 1 320 bytes, 2 000 packets/s, C-118 prediction, `[standard]` tag (C-132) | 001-11 |
| Deliverable 5 — capability definitions table (C-129, C-131) | 001-12 |
| Deliverable 6 — `probes/linux-peer/` sender and capture command | 001-01 |
| Provides: C-90 (and C-118), evidence for C-150, C-71, C-72, input to C-138 | 001-14 (values from 001-11, 001-13) |
| Provides: options and step-detection clock basis to 015, 021, 022 | 001-13 |
| 001-AC1 macOS yes/no, option, clock basis, minimum version, two releases | 001-02, 001-16, 001-13 |
| 001-AC2 Windows yes/no, control, minimum build, fallback | 001-06, 001-08, 001-13 |
| 001-AC3 distribution over ≥ 100 000 packets per OS, 99.9th percentile first, strict mode | 001-03, 001-07, 001-16, 001-17, 001-13 |
| 001-AC4 capture shows no fragmentation, default size whole with DF, per OS | 001-04, 001-09, 001-13 |
| 001-AC5 BER cited, C-90 confirmed or corrected | 001-11, 001-13, 001-14 |
| 001-AC6 every `[memory]` claim re-tagged, cited or struck | 001-13, 001-14 |
| 001-AC7 contributes to AC-27 / AC-28 against the C-150 clock share | 001-03, 001-07, 001-13, 001-14 |
| 001-AC8 Linux peer rate and contiguous sequence numbers | 001-01 |
| Open sub-question: macOS monotonic stamp (this task) | 001-02, 001-13 |
| Open sub-question: macOS minimum version (this task) | 001-16, 001-13 |
| Open sub-question: Windows minimum build (this task) | 001-08, 001-13 |
| Open sub-question: fit to the C-150 clock share (this task, then task 020) | 001-03, 001-07, 001-13, 001-14 |
| Open sub-question: C-90 figure (this task, then task 019) | 001-11, 001-14 |
| Open sub-question: interrupt coalescing (task 019) | noted in 001-02, 001-13, 001-14 |
| DoD: tests green in CI, unprivileged (per-change CI, C-146) | 001-01, 001-02, 001-03, 001-06, 001-07, 001-15, 001-17 |
| DoD: every parser touched is fuzzed (C-95) | 001-01, 001-02, 001-06, 001-15 |
| DoD: regime stamp fields complete and documented | 001-12 |
| DoD: design documents updated, constants table first | 001-14 |
| DoD: no new open item without owner and falsifier | 001-08, 001-11, 001-12, 001-13, 001-14, 001-16 |
| DoD: frozen POC not modified | 001-01, 001-02, 001-06, 001-14, 001-15, 001-17 |
| DoD: every new factual claim carries an evidence tag | all |
| DoD: `REPORT-0a.md` before task 004 freezes option assumptions | 001-13 |
| DoD: probe sources committed | 001-01 to 001-10, 001-13, 001-15, 001-16, 001-17 |
| DoD: lines with no subject marked not applicable (C-125) | every ticket |

## Remaining gaps
1. **Wire encoding and width of the capability states** (four states, C-129, need two bits per field). Transport is settled by C-18; encoding is not. Out of scope for 001-12. Answer: regime stamp specification, task 013.
2. **Candidate option names** (`SO_TIMESTAMP_MONOTONIC`, `IP_DONTFRAG`, `SIO_TIMESTAMPING`, `IP_DONTFRAGMENT`) are `[memory]` until confirmed. Affects 001-02, 001-04, 001-06, 001-09. Answer: measurement, by those tickets.
3. **Fit of the stamp-to-read 99.9th percentile to the C-150 clock share** (macOS 40 µs, Windows 100 µs; the split is itself a proposal). Affects 001-03, 001-07, 001-16, 001-13, 001-14. Answer: measurement, by those tickets; closed by L2 (task 020).
4. **NIC interrupt-coalescing bias of the stamps.** Falsifier: offset shifts when coalescing is toggled. Answer: L1, task 019.
5. **Clean-network loss and the loss alarm threshold C-91.** Answer: L1, task 019.
6. **Exact header layout of the default packet.** The probe size is settled at 1 252 bytes of UDP payload (C-137); the layout inside it does not affect this epic. Answer: wire specification, task 004.

None needs the design owner.

## Settled since the split
- Former gap 1, no peer sender listed: the epic now lists `probes/linux-peer/` as deliverable 6 with criterion 001-AC8 (epic change).
- Former gap 3, probe packet size for the distribution runs: the default media packet, 1 252 bytes of UDP payload (C-137), about 1 320 bytes on the wire (C-20), stated in the epic Background.
- Former gap 4, "wire packet" undefined and 1 320 vs 1 500 bytes: C-20 defines the wire size as including all Ethernet overhead, and C-90 is now stated per default packet of about 1 320 bytes.
- Former gap 5, comparing stamp and read across clock bases: the probes read every candidate clock and compare against the one matching the stamp's basis (epic change); sleep behaviour of the macOS clocks does not matter because sleep is a departure (C-124).
- Former gap 7, CI versus hardware probes: the CI line covers unit tests of parsers and statistics only; hardware runs are measurements made repeatable by committed sources (epic change, with the C-125 note).
- Former gap 8, scope of fuzzing for probes: the parsers are the two control-message parsers and the peer payload decoder, fuzzed 60 s per change and 1 hour nightly with a kept corpus (C-95); new ticket 001-15 wires it into CI.
- Former gap 12, where `[memory]` claims are re-tagged: in `doc/design/reports/REPORT-0a.md` (C-130), then the constants table first, then design documents including the epic Background (epic change).
- Former gap 13, location of the constants table and design documents: `doc/design/02-constants.md` and `doc/design/` (epic change).
- Former gap 9, in part: regime stamp transport is settled (C-18); encoding remains with task 013 (remaining gap 1); variant meanings are now settled by C-131 (below).
- Former gap 10, in part: the statistic compared with E is the 99.9th percentile (C-73); the threshold is now settled by C-150 (below).
- Former gap 11, in part: 001-AC6 now accepts a citation in place of `[measured]` (epic change); the tag is now settled by C-132 (below).
- Also corrected in the epic: the Goal now names all five claims (drop counter was missing); acceptance criteria carry ids 001-AC1 to 001-AC8; the scope label is recomputed from the tally (C-126); the definition of done carries a C-125 applicability note; the wall-clock step detection is placed in the platform backend (C-121); the don't-fragment background states IPv4 only (C-100) and the probe pair exemption (C-116).
- Former remaining gap 1, clock-bound share of the macOS and Windows E budgets: C-150 splits macOS as 40 + 40 + 20 µs and Windows as 100 + 100 + 800 µs (proposals, pending L2); the stamp-to-read 99.9th percentile is compared with the clock share of the target E, never a measured E (C-149). Updated in the epic, 001-03, 001-07, 001-13, 001-14.
- Former remaining gap 2, supported macOS and Windows versions and older images: the supported versions are set after this task reports, and the probes test the current release and the oldest obtainable one (C-138). New ticket 001-16 covers the oldest macOS release; 001-08, 001-07, 001-09, 001-10 use the Windows 11 (current) and Windows 10 (oldest obtainable) machines; 001-13 adds a supported-versions input section and 001-14 proposes a range in C-138.
- Former remaining gap 3, meaning of `MacHardware` and `Interrupt`: settled by C-131 (source values kernel, userspace, mac-hardware, interrupt, each with a capability state). The epic's shape replaces `KernelMonotonic` / `KernelWallClock` by one `kernel` value, with the clock basis reported separately for step detection (C-121); `Tri` becomes the four-state `CapabilityState` (C-129). Updated in the epic, 001-02, 001-05, 001-06, 001-10, 001-12.
- Former remaining gap 4, evidence tag for a cited standard: `[standard]` (C-132). Updated in the epic, 001-08, 001-11, 001-13, 001-14.
- Former remaining gap 5, location of `probes/`: the design set's repository, `probes/` (C-133); the report and all results go in `doc/design/reports/` (C-130), not under `probes/`. Updated in the epic and every ticket that named a result path.
- Former remaining gap 7, exact UDP payload length: the probe uses 1 252 bytes of UDP payload, the C-137 cap (1 152 bytes of audio plus at most 100 bytes of headers and checksum), which matches the C-20 wire size; the header layout stays with task 004 (remaining gap 6). Updated in the epic and 001-01, 001-04, 001-09.
- New from C-147, C-127, C-128: runs whose numbers enter the constants table run in strict mode; new ticket 001-17 adds the strict-mode setup to both probes, and 001-03, 001-07, 001-16 run with it.
- Also applied: C-146 ("unprivileged" applies to per-change CI) in the epic's C-125 note and 001-01, 001-15, 001-17; C-173 (PathMtuTooSmall only when the fallback probe also fails) in the epic, 001-04, 001-09; C-171 (`doc/design/01-architecture.md` among the documents to update) in the epic and 001-14.
- New from C-179: loss above 10 × the C-90 floor (about 580 losses in the C-118 soak) reopens the redundancy decision; between 1 × and 10 × it is investigated as a cabling or host problem. The count is derived from C-90, so 001-11 recomputes it beside the C-118 prediction and 001-14 updates it if C-90 changes. Updated in the epic Background, Provides and settled list, 001-11 and 001-14. No new ticket.
- Checked with no effect on this epic: the revised C-104 and C-105 and the new C-134 (no latency, resampler or delay-line statement here), C-35's owner, C-144 (duties for epic 015 only), and the other rules from C-135 to C-176. From C-177 to C-212 and the late revisions of C-08, C-134, C-141, C-153, C-169, C-181, C-184: no subject here apart from C-179. The probes are standalone programs, not modules, so the seqlock and pinning rules (C-177, C-201) do not apply; they report stamp-to-read spreads, not one-way figures, so C-187 does not apply; the probe output formats and draft file names are this epic's own choice (C-206); the epic depends on no other epic (C-196).
