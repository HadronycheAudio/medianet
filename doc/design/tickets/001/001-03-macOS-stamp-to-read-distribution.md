# 001-03 — macOS stamp-to-read distribution
**Epic:** 001-Receive-stamp-probes · **Effort:** S · **Depends on:** 001-01, 001-02, 001-17

## Context
Epic 001 (check **0a**) must measure, on macOS, how far a userspace clock read after the receive call lags the kernel receive stamp. This spread feeds the clock bound and so the macOS emission error target E = 100 µs (C-71). System criterion AC-27 (macOS E at most 100 µs, restart spread at most 50 µs, C-74) depends on this clock-bound assumption. The clock share of the macOS budget is 40 µs (C-150: 40 clock + 40 tracking + 20 uncertainty of D, a proposal pending L2); the run's 99.9th percentile is compared with that share of the target E, never with an E measured in the same run (C-149).

E is the 99.9th percentile of absolute emission error, with the maximum beside it (C-73); the headline figure of this run is therefore the 99.9th percentile, with median and maximum reported beside it.

Required run: at least 100 000 packets of the default size (about 1 320 bytes on the wire, C-20) at 2 000 packets/s, from the Linux peer (001-01) to `probes/macos-rxstamp/` (001-02). The difference is taken against the clock that matches the stamp's observed basis (001-02). Samples with a wall-clock step are discarded, as the runtime's backend does (C-121). The figures enter the constants table as evidence on C-150 and C-71, so the run is made in strict mode (C-147) with the probe's strict-mode setup (001-17); a run where real-time scheduling or memory lock is unverified counts as one where it is absent (C-128) and is not used for a table value. This ticket runs on the current macOS release; the oldest obtainable release is 001-16 (C-138).

## Work
- Add a summary mode to `probes/macos-rxstamp/` that computes, over all received packets, the distribution of (userspace read − kernel stamp) in nanoseconds on the matching clock: count, 99.9th percentile, median, maximum; also the count of lost sequence numbers and of samples discarded for a wall-clock step (if the basis is the wall clock).
- Statistics code as a pure function with unit tests (known inputs → known percentiles).
- Run once in strict mode on the macOS machine (current release) wired to the Linux peer; save raw output and summary under `doc/design/reports/` (C-130) with date, macOS version, NIC, link speed, packet size and the strict-mode conditions recorded by 001-17.
- State in the summary whether the 99.9th percentile is within the 40 µs clock share (C-150).
- If no kernel stamp exists, run the same measurement on the userspace-only path and report that instead, stating it.

## Acceptance criteria
1. A committed result file shows a run of ≥ 100 000 received default-size packets at 2 000 packets/s (± 2 %) (epic 001-AC3).
2. The summary gives 99.9th percentile, median and maximum in µs with at least 0.1 µs resolution, and names the clock used.
3. Lost packets and discarded (step) samples are counted and reported.
4. The result file shows real-time scheduling and memory lock granted (strict mode, C-147, C-127), or states that the run is not usable for a table value (C-128).
5. The summary states the 99.9th percentile against the 40 µs clock share (C-150).
6. Percentile function unit tests pass in CI, unprivileged.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Probe sources committed so the measurements can be repeated
- [ ] Raw output committed alongside the summary

Not applicable (C-125): "Every parser touched is fuzzed" — no parser added (the statistics code parses nothing); "Regime stamp fields touched" — none; "Design documents updated" — 001-14; "No new open item" — opens none; "The frozen POC is not modified" — the POC is not touched; "`REPORT-0a.md` delivered before task 004" — 001-13.

## Out of scope
- Writing the report page (001-13).
- Proposing a revision of C-150 or C-71 (001-13, 001-14); this ticket reports the figures and the comparison with the clock share.
- The run on the oldest obtainable macOS release (001-16).
- Interrupt-coalescing bias (task 019).
