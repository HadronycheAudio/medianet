# 001-07 — Windows stamp-to-read distribution
**Epic:** 001-Receive-stamp-probes · **Effort:** S · **Depends on:** 001-01, 001-06, 001-17

## Context
Epic 001 (check **0a**) must measure, on Windows, how far a userspace QueryPerformanceCounter read after the receive call lags the kernel receive stamp. This spread feeds the clock bound and the Windows emission error target E = 1 ms (C-72, WASAPI exclusive). System criterion AC-28 (Windows E at most 1 ms, restart spread at most 0.5 ms, C-74) depends on this clock-bound assumption. The clock share of the Windows budget is 100 µs (C-150: 100 clock + 100 tracking + 800 uncertainty of D, a proposal pending L2); the run's 99.9th percentile is compared with that share of the target E, never with an E measured in the same run (C-149).

E is the 99.9th percentile of absolute emission error, with the maximum beside it (C-73); the headline figure of this run is therefore the 99.9th percentile, with median and maximum beside it.

Required run: at least 100 000 packets of the default size (about 1 320 bytes on the wire, C-20) at 2 000 packets/s from the Linux peer (001-01) to `probes/windows-rxstamp/` (001-06), on the clock matching the stamp's observed basis. The figures enter the constants table as evidence on C-150 and C-72, so each run is made in strict mode (C-147) with the probe's strict-mode setup (001-17); a run where real-time scheduling or memory lock is unverified counts as one where it is absent (C-128) and is not used for a table value. The probes test the current release and the oldest obtainable build (C-138): the Windows 11 and Windows 10 machines.

## Work
- Add a summary mode to `probes/windows-rxstamp/`: count, 99.9th percentile, median, maximum of (userspace read − kernel stamp) converted to ns using `QueryPerformanceFrequency`; lost sequence numbers counted; wall-clock-step samples discarded and counted if the basis is the wall clock.
- Statistics as a pure function with unit tests.
- Run in strict mode on the Windows 11 machine (current release) and on the Windows 10 machine (oldest obtainable build, C-138), each wired to the Linux peer. Save raw output and summary under `doc/design/reports/` (C-130) with date, build number, NIC, link speed, packet size and the strict-mode conditions recorded by 001-17.
- State in each summary whether the 99.9th percentile is within the 100 µs clock share (C-150).
- On a machine where no kernel stamp exists, report the receive-to-read figures that are measurable in fallback mode and say so.

## Acceptance criteria
1. For each machine with a kernel stamp, a committed result shows ≥ 100 000 received default-size packets at 2 000 packets/s (± 2 %) (epic 001-AC3).
2. The summary gives 99.9th percentile, median and maximum in µs with at least 0.1 µs resolution, and names the clock used.
3. Lost packets and discarded step samples are counted and reported.
4. Each result file shows real-time scheduling and memory lock granted (strict mode, C-147, C-127), or states that the run is not usable for a table value (C-128).
5. Each summary states the 99.9th percentile against the 100 µs clock share (C-150).
6. Unit tests pass in CI, unprivileged.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Probe sources committed so the measurements can be repeated
- [ ] Raw output committed alongside the summary

Not applicable (C-125): "Every parser touched is fuzzed" — no parser added; "Regime stamp fields touched" — none; "Design documents updated" — 001-14; "No new open item" — opens none; "The frozen POC is not modified" — not touched; "`REPORT-0a.md` delivered before task 004" — 001-13.

## Out of scope
- Minimum-build determination (001-08).
- Report writing and any C-150 or C-72 revision proposal (001-13, 001-14).
- Interrupt-coalescing bias (task 019).
