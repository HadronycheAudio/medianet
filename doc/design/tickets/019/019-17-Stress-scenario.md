# 019-17 — Stress scenario
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-07

## Context
Scenario `stress`: "`baseline` under each load of `tools/stress.sh` on either host". `baseline` is one 6-channel stream each way (019-07), 2 000 packets/s per stream (C-21).

Background `[code]`: the frozen POC's experiments ran sender and echo on two hosts under CPU, I/O, memory and network stress. Their one design-relevant finding — machines under I/O pressure should not hold the clock — rests on metrics files whose integrity task 002 must confirm. `tools/stress.sh` is a usable load generator and may be reused as an external tool. The frozen POC must not be modified (task 003 is the only exception), so the script is invoked as-is (copied to the hosts unchanged, or run from a pinned path).

Clock facts for evaluation: lock bound at most 25 µs (C-62), slew at most 10 ppm (C-60), rate change at most 1 ppm/s (C-61). Late slices are counted separately from missing slices; an absent slice that arrives within the ring span after its deadline is reclassified as late (C-108). A is computed by the harness with the commissioning rule and recorded (C-148, 019-26), and passed to `baseline`.

## Work
- Scenario `stress`: enumerate the loads `tools/stress.sh` offers (CPU, I/O, memory, network per the POC description; take the exact list from the script's own usage text) and, for each load × {host `a`, host `b`} (holder side and follower side recorded), run `baseline` with that load active. 10 minutes per run (C-167). Record which host holds the clock in each run.
- Deploy `tools/stress.sh` unchanged with the build; record its content hash in the stamp/run log.
- Report section: per load × host, late and missing slices, servo offset p99/max, slew and rate change max, consumer lateness p99.
- Keep the records: 019-25 uses them for the bounded-drain check (C-16).
- Record in `l1-results.md`, including whether the POC finding "machines under I/O pressure should not hold the clock" is reproduced; tag `[measured]`, and note that the POC data's integrity is task 002's concern.
- In dry-run, the stress load is replaced by a no-op and marked skipped.

## Acceptance criteria
1. Dry-run passes in CI.
2. `l1-results.md` contains one row per (load, host) pair with the clock holder, late slices, missing slices and servo offset p99.
3. The deployed `tools/stress.sh` hash equals the hash of the frozen POC's file (no modification).
4. A clear yes/no statement on the I/O-pressure clock-holder finding, tagged `[measured]`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Verifying the POC metrics files (task 002).
- Changing clock-holder election rules.
