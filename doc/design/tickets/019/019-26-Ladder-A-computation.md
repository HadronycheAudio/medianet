# 019-26 — Ladder A computation
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-03, 019-04, 019-05

## Context
Every aligned L1 scenario is judged "at the commissioned A" (AC-13, AC-14). A is the presentation group's audition offset: no default, one per presentation group, proposed by commissioning (C-55); alignment rings are sized to it (C-96). The commissioning tool (task 028) comes after L1, so C-148 settles the value for ladder runs: **the harness computes A with the commissioning rule — the largest over sinks of maximum lateness + D + headroom — and records it with the results.**

Terms of the rule:
- Maximum lateness: the maximum origin-to-arrival lateness the sink observed, as published in its stamp (C-51, C-160): the maximum since the connection went live, not the rolling 60 s figure, which would cover only the last minute of the calibration phase (C-182).
- Headroom: the largest of 25 % of the observed maximum, one slice period, and twice the published clock bound (C-52).
- D: the sink's total plumbing delay from hand-off to emission (C-170), as the sink publishes it.

Runs whose numbers enter the constants table are strict-mode runs (C-147); the calibration run that yields A is run in the same mode as the measured run it serves.

## Work
- Module `hadronyche_tools::a_rule`: given the stamps of every sink in a run, compute per sink maximum lateness + D + headroom (C-52) and take the largest (C-148). A missing term is a named error, never a default.
- Subcommand `hadronyche-l1 compute-a <scenario>`: run the scenario's traffic for a calibration phase of 10 minutes (short-scenario duration, C-167), read each sink's stamp at the end (maximum lateness since live, C-182), compute A, and write `<out>/a.json` with A and every term per sink.
- Scenario integration: each aligned scenario takes A from `a.json` (passed as `--a`); the report (019-04) records A and links its `a.json`, and `a_used` in `StreamStats` equals it.
- In dry-run, `compute-a` runs on loopback with the dry-run duration.

## Acceptance criteria
1. A unit test with three synthetic sink stamps returns the largest of (maximum lateness + D + headroom), with headroom computed by C-52, and lists each sink's terms; a stamp whose maximum since live exceeds its rolling 60 s maximum yields A from the maximum since live (C-182).
2. A sink stamp lacking the maximum lateness since live (C-182) or D makes `compute-a` fail with a named error naming the sink and the missing term.
3. `hadronyche-l1 compute-a baseline --dry-run` writes `a.json` in CI, and a following `run baseline --dry-run` records the same A in its report.
4. Every L1 report of an aligned scenario carries A and its C-148 terms; a report without them fails generation with a named error.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The commissioning tool's own A proposal (task 028).
- Measuring D for platform sinks (L2, task 020).
- Changing C-52 or C-148.
