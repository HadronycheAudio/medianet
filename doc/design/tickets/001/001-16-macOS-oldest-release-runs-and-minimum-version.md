# 001-16 — macOS oldest-release runs and minimum version
**Epic:** 001-Receive-stamp-probes · **Effort:** S · **Depends on:** 001-02, 001-03, 001-04, 001-05, 001-17

## Context
The supported OS versions are set after epic 001 reports, and the probes test the current release of each OS and the oldest obtainable one (C-138). On Windows this is ticket 001-08 with the Windows 10 and 11 machines. On macOS, 001-02 to 001-05 run on the current release only; no ticket runs the macOS probe on the oldest obtainable release or reports the minimum macOS version on which the kernel receive stamp works (epic 001-AC1). This ticket does.

Claim to verify on that release, as on the current one: a kernel receive timestamp exists for UDP, possibly on a monotonic (mach) basis through a Darwin-specific socket option, in addition to the BSD wall-clock `SO_TIMESTAMP` `[memory]`. Whatever the basis, a hosted OS kernel stamp is the source value `kernel` (C-131); the basis is reported for the macOS backend's step detection (task 021, C-121).

The stamp-to-read run feeds the clock share of the macOS budget, 40 µs (C-150), so it runs in strict mode (C-147) with the setup of 001-17; unverified conditions count as absent (C-128). The don't-fragment and drop-counter runs put no number in the table and need no strict mode.

Open sub-question owned by this ticket: "From which macOS version is the kernel receive stamp available?" Falsifier: not available on the current release; the kernel source is absent on macOS and the C-150 clock share must hold with userspace stamps.

Outputs go under `doc/design/reports/` (C-130); the probe is `probes/macos-rxstamp/` (C-133).

## Work
- Obtain the oldest macOS release that can be installed on available hardware (a second machine, partition or image) and wire it to the Linux peer (001-01).
- Run on it, unchanged, the probe modes of 001-02 (options and per-packet stamps), 001-03 (strict-mode distribution over ≥ 100 000 default-size packets at 2 000 packets/s), 001-04 (don't-fragment with peer capture) and 001-05 (drop counters).
- Establish the minimum macOS version for the kernel receive-stamp option: from Apple SDK headers or documentation (cite it) and confirmed by the runs on the two releases. Tag documentary claims with their citation and observed ones `[measured]`.
- Save every output, summary and capture under `doc/design/reports/` with the macOS version in each header.
- List the macOS versions tested and the minimum version found per facility (kernel stamp, don't-fragment option, drop counter), as input to C-138.

## Acceptance criteria
1. Committed outputs from the oldest obtainable macOS release for all four modes, each with its macOS version in the header.
2. The distribution result from that release gives 99.9th percentile, median and maximum and shows strict mode met, or states that the run is not usable for a table value (C-128).
3. A stated minimum macOS version for the kernel receive stamp, with its source, and the falsifier outcome of the open sub-question.
4. A list of versions tested and minimum version per facility, ready for 001-13's supported-versions section (C-138).

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Probe sources committed so the measurements can be repeated
- [ ] No new open item without an owner and a falsifier

Not applicable (C-125): "Tests green in CI, unprivileged" and "Every parser touched is fuzzed" — runs only, no new code; "Regime stamp fields touched" — none (001-12); "Design documents updated" — 001-14; "The frozen POC is not modified" — not touched; "`REPORT-0a.md` delivered before task 004" — 001-13.

## Out of scope
- Writing the probe modes (001-02 to 001-05) and the strict-mode setup (001-17).
- Releases older than the oldest obtainable one (C-138).
- Setting the supported versions in C-138 (001-14 proposes from the report).
- Windows (001-08).
