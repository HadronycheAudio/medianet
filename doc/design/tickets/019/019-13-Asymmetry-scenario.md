# 019-13 — Asymmetry scenario
**Epic:** 019-L1-two-machine-harness · **Effort:** M · **Depends on:** 019-07, 019-11, 019-12, epic 008a

## Context
Scenario `asymmetry` "compares in-band one-way figures with an external reference". The method was chosen in 019-12 (one of: hardware-timestamping NIC pair, shared pulse, swapped-cable method).

Facts:
- One-way latency figures are accurate to half the path asymmetry, which no in-band method can observe; the worst-case asymmetry bound is published with every one-way figure. The lock state and bound are published by the servo core (epic 008a) through the published time view (C-187).
- Receive timestamps are kernel software stamps (`SO_TIMESTAMPNS`, `CLOCK_REALTIME` basis `[kernel-doc]`); interrupt coalescing biases one direction (bias measured by 019-11).
- One-way probes are sent with DSCP EF (46) (C-13). Followers send 50 sync requests/s (C-36). Servo lock bound at most 25 µs (C-62).

AC-24 (part): **the asymmetry bound holds against the external check**, i.e. for every one-way figure `x ± bound` the runtime reports, the externally measured one-way latency lies within `[x − bound, x + bound]`. Each figure is tested against its own published bound; no separate fixed value is needed.

Fuzzing is settled by C-95: each parser 60 s on every change and 1 hour nightly; the corpus is kept.

## Work
- Scenario `asymmetry`: runs the `baseline` traffic (one 6-channel stream each way) and, in parallel, the external reference chosen in 019-12; time-aligns the two series.
- Reference adapter in `hadronyche_tools::asymmetry` that imports the external measurement (e.g. hardware timestamp log, pulse timestamps, or the two half-runs of the swapped-cable method) through a parser with a fuzz target `asymmetry_ref`.
- Report section: per direction, in-band one-way figure, published bound, external one-way value, residual, and the fraction of samples inside the bound.
- Run with coalescing off and with coalescing on (settings from 019-11), 10 minutes each (C-167). A is computed by the harness with the commissioning rule and recorded (C-148, 019-26), and passed to the `baseline` traffic.
- Record results in `l1-results.md` under "Asymmetry", tagged `[measured]`.
- In dry-run, a synthetic reference file is used.

## Acceptance criteria
1. Dry-run passes in CI with a synthetic reference; a synthetic reference placed outside the bound makes the verdict fail.
2. On the hardware pair, 100 % of in-band one-way figures lie within their published bound of the external value, in both directions, with coalescing on and off (AC-24); any exception is listed with its time and magnitude.
3. The `asymmetry_ref` parser fuzz target runs 60 s on every change and 1 hour nightly without a crash, with its corpus kept (C-95).
4. Both hosts' complete stamps and the reference method's identity are in the result files.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Choosing the reference method (019-12).
- Measuring coalescing bias itself (019-11).
