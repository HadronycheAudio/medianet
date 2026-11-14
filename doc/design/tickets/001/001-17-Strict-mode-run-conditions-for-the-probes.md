# 001-17 — Strict-mode run conditions for the probes
**Epic:** 001-Receive-stamp-probes · **Effort:** S · **Depends on:** 001-02, 001-06

## Context
Any run whose numbers enter the constants table runs in strict mode; L0 does not; strict mode is hosted-only (C-147). The stamp-to-read runs of epic 001 (001-03, 001-07, 001-16) provide the evidence for the clock shares of C-150 and so for C-71 and C-72, so they must run in strict mode. Strict mode requires real-time scheduling granted on the data and clock threads and memory locked; restricting CPU idle states stays best-effort and is stamped (C-127). An unverified capability is refused in strict mode: unverified is treated as absent (C-128). Capability states are present, absent, unverified or not requested (C-129).

In a probe the receive thread is both the data and the clock thread. No ticket of this epic sets these conditions in the macOS and Windows probes or records them with the results.

The privilege these settings may need belongs to the measurement runs; "unprivileged" applies to per-change CI (C-146).

## Work
- In `probes/macos-rxstamp/` and `probes/windows-rxstamp/`, add a `--strict` option that, before receiving:
  - requests the platform's real-time scheduling class for the receive thread;
  - locks the process memory;
  - attempts to restrict CPU idle states where the platform offers a way, best-effort.
- Verify each request after making it (read back the thread policy, the lock state) and report each condition with one of the four states of C-129.
- Write the conditions into the output header of every run, and a one-line verdict: "strict mode met" only if real-time scheduling and memory lock are both present; otherwise "strict mode not met", with unverified counted as absent (C-128).
- Pure logic that turns the reported states into the verdict is a function with unit tests.

## Acceptance criteria
1. With `--strict`, each probe's output header lists real-time scheduling, memory lock and CPU idle restriction, each as present, absent, unverified or not requested (C-129).
2. The verdict is "strict mode met" only when real-time scheduling and memory lock are present; an unverified condition yields "not met" (C-128); the CPU idle state does not affect it (C-127).
3. Without `--strict`, the conditions are reported as not requested.
4. Verdict unit tests pass in CI, unprivileged.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Probe sources committed so the measurements can be repeated

Not applicable (C-125): "Every parser touched is fuzzed" — no parser added (the verdict function parses no input); "Regime stamp fields touched" — the probe reports these states in its own output, not in a regime stamp; "Design documents updated" — 001-14; "No new open item" — opens none; "`REPORT-0a.md` delivered before task 004" — 001-13.

## Out of scope
- The distribution runs themselves (001-03, 001-07, 001-16).
- Strict mode in the runtime and its operator policy (hosted, macOS and Windows backends: tasks 015, 021, 022).
- Strict mode for the don't-fragment and drop-counter runs, which put no number in the table (C-147).
