# 001-08 — Windows minimum build and fallback
**Epic:** 001-Receive-stamp-probes · **Effort:** S · **Depends on:** 001-06

## Context
Epic 001 must report the minimum Windows build on which the UDP receive-timestamp control works, and the probe's behaviour on a build where it does not (epic 001-AC2). Claim to verify: recent Windows 10 and 11 builds offer a Winsock timestamping control for UDP receive in QueryPerformanceCounter units `[memory]`. If absent, the starting point is a userspace clock read immediately after the receive call.

Open sub-question owned by this ticket: "From which Windows build is kernel receive stamping available?" Falsifier: not available on the current release; then the clock bound and the Windows budget rise (C-150 clock share 100 µs, C-72 E = 1 ms).

Settled (C-138): the supported OS versions are set after this task reports, and the probes test the current release of each OS and the oldest obtainable one. Hardware per the epic: a Windows 11 machine (current release) and a Windows 10 machine or an older image (oldest obtainable build), each wired to a Linux peer. This ticket reports the builds it ran and the minimum build found, as input to C-138. Outputs go under `doc/design/reports/` (C-130). A documentary claim is tagged with its citation; a figure from a published standard would be `[standard]` (C-132), vendor documentation is cited inline.

## Work
- Run `probes/windows-rxstamp/` (001-06) on the Windows 11 (current release) and the oldest obtainable Windows build and record build number and whether the control works.
- Establish the minimum build: from Microsoft SDK / API documentation for the control (cite it) and confirm by running on at least one build at or above it; if an older build is reachable (e.g. a VM or an older Windows 10 image), run there to observe the failure. Tag documentary claims with their citation and observed ones `[measured]`.
- Capture the fallback output (error code returned, fallback mode engaged) on a build without the control.
- Save outputs under `doc/design/reports/` (C-130).
- List the builds tested and the minimum build found, as input for setting the supported versions (C-138).

## Acceptance criteria
1. A stated minimum build number, with its source, and at least one `[measured]` run on a build at or above it showing the control working.
2. A committed output from a build where the control does not work, showing the exact error and fallback-mode behaviour; or, if the oldest obtainable build already has the control, an explicit statement of that and of the oldest build run.
3. Each run's build number appears in its output header.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Probe sources committed so the measurements can be repeated
- [ ] No new open item without an owner and a falsifier

Not applicable (C-125): "Tests green in CI, unprivileged" and "Every parser touched is fuzzed" — runs only, no new code; "Regime stamp fields touched" — none; "Design documents updated" — 001-14; "The frozen POC is not modified" — not touched; "`REPORT-0a.md` delivered before task 004" — 001-13.

## Out of scope
- Distribution statistics (001-07).
- Setting the supported versions in C-138 (done after this task reports; 001-14 records the evidence).
