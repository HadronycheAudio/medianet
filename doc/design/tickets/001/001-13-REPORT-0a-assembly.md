# 001-13 — REPORT-0a assembly
**Epic:** 001-Receive-stamp-probes · **Effort:** S · **Depends on:** 001-03, 001-04, 001-05, 001-07, 001-08, 001-09, 001-10, 001-11, 001-12, 001-16, 001-17

## Context
`doc/design/reports/REPORT-0a.md` (C-130) is the deliverable of pre-implementation check **0a**. It is an input to the wire specification (task 004) and must be delivered before task 004 freezes its option assumptions. It also provides the exact socket options to the hosted, macOS and Windows backends (tasks 015, 021, 022), including the clock basis on which each backend runs its wall-clock step detection (C-121), and the capability-bit table to the regime stamp (task 013).

E targets: Linux 50 µs (C-70), macOS 100 µs (C-71), Windows WASAPI exclusive 1 ms (C-72); lock bound at most E/2, 25 µs on Linux (C-62, pending simulation). E is the 99.9th percentile of absolute emission error, maximum beside it (C-73), so the stamp-to-read 99.9th percentile is the figure set against the budgets. System criteria: AC-27 (macOS E ≤ 100 µs, restart spread ≤ 50 µs), AC-28 (Windows E ≤ 1 ms, restart spread ≤ 0.5 ms). C-71 and C-72 are pending measurement, owned by L2 (task 020). The clock share of the budgets is settled as a proposal pending L2: macOS 40 µs (of 40 + 40 + 20 µs), Windows 100 µs (of 100 + 100 + 800 µs) (C-150). The report sets each strict-mode stamp-to-read 99.9th percentile against that share of the target E, never an E measured in the same run (C-149), and gives a verdict. Only figures from runs that met strict mode (C-147) are proposed for the table; unverified conditions count as absent (C-128).

The supported OS versions are set after this report; the probes tested the current release and the oldest obtainable one of each OS (C-138), and the report gives the input.

`[memory]` claims that must each be re-tagged `[measured]`, replaced by a citation, or struck (epic 001-AC6):
1. macOS: a kernel receive timestamp exists for UDP, possibly on a monotonic (mach) basis through a Darwin-specific socket option, in addition to the BSD wall-clock `SO_TIMESTAMP`.
2. Windows: recent Windows 10 and 11 builds offer a Winsock timestamping control for UDP receive, stamped in QueryPerformanceCounter units.
3. macOS has an IPv4 don't-fragment socket option that differs from Linux's `IP_MTU_DISCOVER`; Windows has its own option.
4. Neither macOS nor Windows has a per-socket drop counter equivalent to `SO_RXQ_OVFL`.
5. C-90 figure: BER 1e-12, about 1.1e-8 per default packet (about 1 320 bytes on the wire, C-20), one loss per stream per link about every 13 hours at 2 000 packets/s; and the looser 1e-10 for gigabit copper.
6. Candidate option names `SO_TIMESTAMP_MONOTONIC`, `IP_DONTFRAG`, `SIO_TIMESTAMPING`, `IP_DONTFRAGMENT`.

A claim replaced by a citation of a published standard is tagged `[standard]` with its clause (C-132).

## Work
- Write `doc/design/reports/REPORT-0a.md`: one page per OS (macOS, Windows) with: option names and values, clock basis, required OS version, the versions tested (current and oldest obtainable, C-138), measured stamp-to-read distribution (99.9th percentile, median, maximum), the clock it was taken on and whether the run met strict mode, don't-fragment option and observed behaviour, drop-counter availability. Each claim tagged `[measured]` and linked to its result file.
- Include the Ethernet bit-error section (001-11) and the capability-bit table (001-12).
- Add a "Claims" table listing the six `[memory]` claims above, each re-tagged `[measured]` (with the result), cited (`[standard]` with standard, edition, clause, C-132, or a vendor document), or struck (with the reason).
- Add an "E targets" section: for macOS and Windows, set the strict-mode stamp-to-read 99.9th percentile against the clock share of C-150 (40 µs, 100 µs), and so against C-71 / C-72 and AC-27 / AC-28, and either state that it fits or propose a revised C-150 split (and C-71 / C-72 if the total cannot hold) with the reasoning.
- Add a "Supported versions input" section: per OS, the versions tested and the minimum version at which each facility (kernel stamp, don't-fragment option, drop counter) works, for setting the supported versions (C-138).
- Answer the open sub-questions owned by this epic (macOS monotonic stamp; macOS minimum version; Windows minimum build; fit to the C-150 clock share; C-90 figure) with their falsifier outcome. Record that NIC interrupt-coalescing bias stays open, owned by L1 (task 019), falsifier: offset shifts when coalescing is toggled.

## Acceptance criteria
1. macOS page: yes/no to "a kernel UDP receive timestamp exists", option name, clock basis and minimum macOS version, each linked to program output from the current and the oldest obtainable release (epic 001-AC1).
2. Windows page: yes/no, control name, minimum build, fallback behaviour on an older build or the reason none could be run, each linked to output (epic 001-AC2).
3. Each OS page reports a stamp-to-read distribution over ≥ 100 000 default-size packets (99.9th percentile, median, maximum) from a run that met strict mode, or states that none did (epic 001-AC3).
4. Each OS page references a packet capture showing an oversized don't-fragment send was not fragmented and a default-size send arrived whole with DF set (epic 001-AC4).
5. BER figure cited from the standard; C-90 confirmed or corrected value proposed (epic 001-AC5).
6. All six `[memory]` claims re-tagged, cited or struck; no untagged factual claim remains (epic 001-AC6).
7. The C-150 clock shares, and so C-71 and C-72, each explicitly supported by the evidence or a revision proposed, with reference to AC-27 / AC-28 (epic 001-AC7).
8. A supported-versions input section lists, per OS, the versions tested and the minimum version per facility (C-138).

## Definition of done
- [ ] `REPORT-0a.md` delivered before task 004 freezes its option assumptions
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] Probe sources committed so the measurements can be repeated

Not applicable (C-125): "Tests green in CI, unprivileged" and "Every parser touched is fuzzed" — no code; "Regime stamp fields touched" — the table comes complete from 001-12; "Design documents updated" — 001-14; "The frozen POC is not modified" — not touched.

## Out of scope
- Editing the constants table and other design documents (001-14).
- New measurements; this ticket only assembles results.
