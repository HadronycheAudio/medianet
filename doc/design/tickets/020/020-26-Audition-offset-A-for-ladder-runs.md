# 020-26 — Audition offset A for ladder runs
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-01, epic 017, epic 018

## Context
A sink emits sample p at `emit(p) = T(p) + A`, where A is the presentation group's audition offset; A has no default (C-55). For ladder runs (L1, L2, L3) the harness computes A with the commissioning rule, the largest over sinks of maximum lateness + D + headroom, and records it with the results (C-148). The L2 rig has one sink under test, so the maximum is taken over that one sink.

Inputs, all settled:
- Maximum lateness: the sink publishes lateness percentiles in its stamp, 50th, 99th, 99.9th and the maximum (C-160); the commissioning rule uses the maximum observed (C-51). The stamp's percentiles cover a rolling 60 s window, with the maximum since the connection went live reported beside them (C-182); A uses the maximum since the connection went live, which covers the whole observation phase.
- D: the sink's declared plumbing delay, its total delay from hand-off to emission (C-170).
- Headroom: the largest of 25 % of the observed maximum, one slice period, twice the published clock bound (C-52).

If maximum lateness + D + headroom exceeds A in each of 10 consecutive 1 s windows the sink raises the "cannot meet A" alarm (C-174; evaluated on the rolling 60 s window, C-182), which is one of the connection alarms (C-173). A run in which that alarm fires is not a valid certification run.

## Work
- In `run.rs`, before the measurement phase of every scenario, run an observation phase of documented length with the sink following the reference holder; read the maximum lateness since the connection went live from the sink's stamp (C-160, C-182) and its declared D (C-170); compute the headroom (C-52) and A = maximum lateness + D + headroom (C-148).
- Configure the presentation group of the run with that A before the sink is admitted for measurement.
- Record A and its inputs (maximum lateness, D, the three headroom candidates and the one chosen, clock bound used) in the recording side channel (020-02) and the certification record (020-13).
- Watch for the connection alarms of C-173 during the run; a "cannot meet A" alarm (C-174) marks the run invalid in the record.
- A development override `--audition-offset <A>` is accepted; a record built with it is marked as not computed by C-148 and cannot feed the constants table.

## Acceptance criteria
1. On a simulated sink with known maximum lateness, D and clock bound, the computed A equals the hand-computed value for each of the three headroom cases of C-52 (25 % of the maximum, one slice period, twice the clock bound).
2. The record and the recording carry A and every input used to compute it.
3. A simulated run in which the sink raises "cannot meet A" (C-174) produces a record marked invalid.
4. A run started with `--audition-offset` is marked as not computed by C-148.
5. On a simulated stamp whose rolling 60 s maximum is below its maximum since the connection went live, A is computed from the latter (C-182).
6. Tests run in CI unprivileged without audio hardware.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Regime stamp fields touched are complete and documented — not applicable (C-125): this ticket reads the lateness percentiles, D and alarms from the sink's stamp and adds no regime stamp field

## Out of scope
- The commissioning tool's own computation of A (epic 028).
- The sink's lateness measurement and its stamp fields (epic 018; field layout epic 004).
- Ratcheting of the offset after admission (alignment epics).
