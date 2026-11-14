# 019-11 — Coalescing scenario
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-07

## Context
Receive timestamps are kernel software stamps (`SO_TIMESTAMPNS`, `CLOCK_REALTIME` basis `[kernel-doc]`); the regime stamp reports the source as `kernel` with its capability state (C-131, C-129), and each run checks it. NIC interrupt coalescing delays them, and delays only one direction of a sync exchange, which biases the servo offset. The bias is currently **unknown**; epic 019 must measure it ("Coalescing bias | unknown | `coalescing`") and AC-24 requires "coalescing bias measured and recorded".

Scenario `coalescing` "repeats `baseline` with NIC interrupt coalescing on and off (`ethtool -C`), records servo offset and one-way figures". `baseline` = one 6-channel stream each way (019-07).

Clock facts: follower servo lock bound at most 25 µs (C-62), slew at most 10 ppm (C-60), rate change at most 1 ppm/s (C-61); followers send 50 sync requests/s (C-36). One-way latency figures are accurate to half the path asymmetry; the worst-case asymmetry bound is published with every one-way figure.

## Work
- Scenario `coalescing`: for each configuration, set coalescing on the receiving NIC(s) with `ethtool -C <if> ...` (privileged step, run via the host's sudo), verify with `ethtool -c`, then run `baseline`.
  - Configurations: coalescing off (`rx-usecs 0`, adaptive off), and on (driver default, recorded verbatim from `ethtool -c`). Run each with coalescing changed on host `a` only, host `b` only, and both, to separate the per-direction effect (6 runs).
  - Duration: 10 minutes per run (C-167).
  - A is computed by the harness with the commissioning rule and recorded (C-148, 019-26), passed through to `baseline`. Every run is a strict-mode run (C-147).
- Restore the original coalescing settings at the end, even on abort.
- Report: servo offset p50/p99/max, one-way figures (with asymmetry bound) per direction, coalescing state from the stamp.
- Compute coalescing bias as the difference in mean servo offset between on and off configurations, per direction.
- Record in `l1-results.md` under "Coalescing bias" with hardware, stamp and tag `[measured]`.
- In dry-run, `ethtool -C` is a `Skipped("privileged")` step (the epic settles that privileged steps run only on the hardware pair).
- Keep the records with coalescing on: 019-25 uses them for the bounded-drain check (C-16), since coalescing delivers packets in bursts.

## Acceptance criteria
1. Dry-run passes in CI unprivileged, listing the `ethtool -C` steps as skipped.
2. Each hardware run's stamp shows the coalescing state that was in force, and the state matches what the scenario set.
3. After the scenario, `ethtool -c` on both hosts equals the state before it.
4. `l1-results.md` states the coalescing bias in µs per direction, tagged `[measured]`, and whether servo offset stays within the 25 µs lock bound (C-62) with coalescing on.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Each pending-measurement constant in the table above has a measured value tagged `[measured]`, or a stated reason why not
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Correcting the bias in the servo (epic 015/016 or a follow-up).
- The asymmetry check against an external reference (019-12, 019-13).
