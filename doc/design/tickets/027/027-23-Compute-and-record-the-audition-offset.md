# 027-23 — Compute and record the audition offset A
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-07, 027-22

## Context
- Every sink emits sample p at `emit(p) = T(p) + A`, A being the presentation group's audition offset: one number per group, no default, proposed by commissioning (C-55).
- For ladder runs (L1, L2, L3) the harness computes A with the commissioning rule — the largest over sinks of maximum lateness + D + headroom — using the maximum lateness since the connection went live, never the rolling 60 s figure of C-182, and records it with the results (C-148).
- Headroom (C-52): the largest of 25 % of the observed maximum lateness, one slice period (0.5 ms at defaults, C-02), and twice the published clock bound.
- Inputs come from each sink's regime stamp (027-22): the maximum lateness, D (the sink's total plumbing delay, C-170) and the published clock bound. The stamp's lateness figures use a rolling 60 s window, with the maximum since the connection went live reported beside them (C-182). `compute_a` therefore uses the since-live maximum, as C-148 now states (settled; it replaces the earlier derivation from C-148 and C-51), and records the rolling-window maximum beside it for information only; the "cannot meet A" alarm is judged on the rolling 60 s window (C-182).
- After A is applied, a sink whose maximum lateness + D + headroom exceeds A in each of 10 consecutive 1 s windows raises the "cannot meet A" alarm (C-174).
- No value of A is invented here: it is computed from the stamps of the run in which it is used.

## Work
1. In `run.rs` of `hadronyche-l3`: `pub fn compute_a(sinks: &[SinkStamp], slice_period_ns: i64) -> AComputation` implementing C-148 with the C-52 headroom; `AComputation` holds A and, per sink, the since-live maximum lateness used (with the rolling 60 s maximum beside it, C-182), D, clock bound, the headroom term chosen and the resulting sum.
2. Runner step (called by 027-11 before capture): with both sinks playing and aligned, fetch both stamps, compute A, apply it to the group by the configuration mechanism documented in the rig procedure (027-09), wait for both sinks to re-align, then start the scenario.
3. Record `a_ns` and the `AComputation` in the scenario report (027-07 field) and in the run log.
4. During the run, record any "cannot meet A" alarm (C-174) visible in the stamps and mark the run as failed with that reason.
5. Unit tests with synthetic stamps covering each of the three headroom terms being the largest.

## Acceptance criteria
1. For synthetic stamps where 25 % of maximum lateness, one slice period, and twice the clock bound are in turn the largest term, `compute_a` returns the matching headroom and A = max over sinks of (maximum lateness + D + headroom).
2. The sink with the largest sum determines A; the report names it.
3. Every scenario report contains `a_ns` and its per-sink inputs (C-148).
4. A run during which a "cannot meet A" alarm (C-174) appears is reported as failed with that reason.
5. For a synthetic stamp whose rolling 60 s maximum lateness is below its since-live maximum, `compute_a` uses the since-live maximum and never the rolling figure (C-148).
6. Tests run in CI without network or privileges, on synthetic stamps.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented (lateness percentiles, D, clock bound, as read)
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier (the clock-bound field is the owned epic sub-question on stamp fields, epic 004)
- Not applicable (C-125): "Every parser touched is fuzzed" — reads `SinkStamp` values parsed by 027-07 and 027-22, adds no parser; "Design documents updated" — implements C-148 and C-52 as written; "Every new factual claim carries an evidence tag" — no factual claim (A values are recorded by 027-20); "The skew analysis runs in CI on recorded two-channel files" — no skew analysis; "The rig is photographed or diagrammed" — no rig change; "`l3-results.md` is committed" — 027-20.

## Out of scope
- The operator-facing commissioning computation of A (epic 028).
- Changing the headroom rule C-52 or the alarm rule C-174.
- Defining the stamp layout (epic 004).
