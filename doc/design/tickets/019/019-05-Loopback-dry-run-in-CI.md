# 019-05 — Loopback dry-run in CI
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-02, 019-03, 019-04

## Context
Epic 019's definition of done requires: "The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback", and "Tests green in CI, unprivileged". Real L1 runs need two hosts, a switch and, for some scenarios, root (for example `ethtool -C`, MTU changes, `rmem` sysctls); CI has none of these. The epic settles the conflict: CI runs the harness unprivileged on loopback; privileged steps run only on the hardware pair and are recorded as skipped in dry-run.

Fuzzing is settled by C-95: each parser is fuzzed 60 s on every change and 1 hour nightly, and the corpus is kept.

The harness (binary `hadronyche-l1`, crate `hadronyche-tools`) already has a `--dry-run` flag (019-01), a `LoopbackTransport` (019-02), stamps (019-03) and a report (019-04). This ticket wires them into CI and defines what "dry-run" means for every scenario: same plan, same orchestration, same collection and report code, but both "hosts" are local processes on `lo`, durations are shortened, and any step needing privilege or hardware is replaced by a recorded `Skipped(reason)` step.

## Work
- Define a `DryRun` policy in `hadronyche_tools::scenario`: each scenario declares its real duration and its dry-run duration (default 5 s), and lists privileged steps; in dry-run those steps are logged as `Skipped(reason)` and appear in the report.
- CI job `l1-dry-run`: builds `hadronyche-l1`, runs `hadronyche-l1 run <s> --dry-run` for every registered scenario, and checks that each produces `report.json`, `report.md`, `stamp.json`, exits 0, and has no orphan processes.
- The job runs as an unprivileged user with no network beyond loopback.
- A test that enumerates the registry ensures every registered scenario is covered by the job (adding a scenario without dry-run support fails CI).
- Fuzzing (C-95): every fuzz target in the crate (from 019-01, 019-03, 019-04, 019-13, 019-14 and any later ticket) runs 60 s on every change in the same pipeline, and 1 hour in a nightly job; each target's corpus is stored in the repository (or the CI cache promoted to the repository) and reused by both jobs. The registry test also fails if a fuzz target is not listed in both jobs.

## Acceptance criteria
1. CI job `l1-dry-run` passes on the main branch, running as a non-root user.
2. Removing dry-run support from any registered scenario makes the job fail with the scenario's name in the error.
3. The total wall-clock time of the per-change job (dry-runs plus 60 s per fuzz target) is under 15 minutes with all scenarios and fuzz targets registered at the time of merge.
4. Each dry-run report lists its skipped privileged steps (0 or more) with a reason.
5. A nightly job runs every fuzz target for 1 hour and keeps the grown corpus; a fuzz target missing from either job fails CI (C-95).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Real-hardware runs (each scenario ticket).
- Scenario logic; each scenario ticket (019-07 onward) adds its own dry-run case, which this job then picks up automatically.
