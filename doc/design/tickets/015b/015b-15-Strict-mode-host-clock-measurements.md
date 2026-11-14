# 015b-15 — Strict-mode host clock measurements (C-147)
**Epic:** 015b-Hosted-runtime-verification-and-strict-mode · **Effort:** S · **Depends on:** 015a-03, 015b-13, epic 016
## Context
- Two values of this epic are open as measurements: the value of `declared_jitter_ns()` for `CLOCK_MONOTONIC`, and the tolerance of the wall-clock step detector (how far realtime-minus-monotonic may move across one receive batch without counting as a step; NTP slewing moves it slowly without a step). Falsifiers: measured read jitter exceeds the declared value; a slewed but unstepped clock discards samples, or a 1 ms step passes undetected.
- Any run whose numbers enter the constants table runs in strict mode (C-147): real-time scheduling granted on the data and clock threads and memory locked (C-127), with no capability unverified (C-128). Strict mode is 015b-13; the real-time setup is task 016.
- The probes are 015a-03's. Results go to `doc/design/reports/` (C-130); the values reach the constants table through 015b-11; `declared_jitter_ns()` is confirmed at L2.
## Work
1. Run the read-jitter probe in strict mode on the development machine: 10^6 back-to-back `CLOCK_MONOTONIC` reads, p99.9 and maximum; propose `declared_jitter_ns()`.
2. Run the slew probe in strict mode with NTP slewing: movement of realtime-minus-monotonic across one receive batch; propose the detector tolerance, below 1 ms.
3. Write the report with machine description, kernel, `CONFIG_HZ` and the strict-mode stamp, each figure tagged `[measured]`.
## Acceptance criteria
1. The report under `doc/design/reports/` gives p99.9 and maximum read jitter and the proposed `declared_jitter_ns()`, tagged `[measured]`.
2. The report gives the measured slew movement and a proposed tolerance that discards a 1 ms step and keeps every slewed sample of the run.
3. The report shows that both runs were in strict mode (C-147).
## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Tests green in CI, unprivileged" — a measurement run, no new code (the probes run in CI under 015a-03); "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — none; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — measured in 015b-10.
## Out of scope
- The probes and the detector (015a-03); confirmation at L2.
