# 008a-07 — Wired jitter profile
**Epic:** 008a-Servo-core-and-simulation-harness · **Effort:** S · **Depends on:** 008a-06
## Context
Epic criterion A1 (verified in 008b, system AC-2): bound at most 25 µs (C-62) within 10 s (C-63) under the wired jitter profile and drift up to ±100 ppm.

The profile is settled provisionally by C-94 `[decision]`: base one-way delay 100 µs; exponential jitter with 20 µs mean; 1 % of packets in bursts delayed by 1 ms. It is synthetic. Owner of the replacement: L1 (task 019), which measures the real profile; falsifier: the measured profile differs enough to change a simulation verdict. C-94 names no loss, duplication, reordering, asymmetry or interrupt coalescing, so the preset has none; cases that need them add them on top.

Case 1 also asks for convergence "across jitter distributions", which needs further presets used only as a sensitivity check, never for A1.

## Work
1. Define a named `LinkConfig` preset `WIRED_PROFILE` in the `sim` module of `hadronyche-tools` equal to C-94 in both directions, built on the 008a-06 link model, with every other link feature off.
2. Create `doc/design/spec/servo.md` with a first section "Wired jitter profile" stating each C-94 parameter with value, unit and source "C-94, provisional synthetic profile, `[decision]`", and the statement "assumed now, replaced by the profile measured at L1 (task 019)", with the falsifier from C-94.
3. Define at least two sensitivity presets for case 1 (for example a log-normal and an empirical-histogram jitter with the same mean), labelled as sensitivity presets in code and in `servo.md`.

## Acceptance criteria
1. `WIRED_PROFILE` is a public constant/function in the `sim` module, is the only profile referenced by the A1 test (008b-01), and its parameters equal C-94 (a test checks base delay, jitter mean, burst fraction and burst delay).
2. `servo.md` lists every `WIRED_PROFILE` parameter with value, unit, source and evidence tag; the numbers equal those in code and in C-94.
3. At least 2 sensitivity presets exist and are labelled as such.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The wired jitter profile used for AC-2 is written down with its source (measured at L1 later, assumed now)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The rest of `servo.md` (algorithm, parameters, results table: 008b-18).
- L1 measurement (task 019).
