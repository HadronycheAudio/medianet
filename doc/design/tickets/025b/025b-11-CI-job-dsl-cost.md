# 025b-11 — CI job `dsl-cost`
**Epic:** 025b-Replay-cross-target-fingerprint-and-cost-calibration · **Effort:** S · **Depends on:** 025b-10
## Context
CI job `dsl-cost` measures real time per slice against the bound on each target. The cost bound is a count of abstract operations per slice, identical on every machine; each target has a calibration factor (time per abstract operation), measured once (C-106, ticket 025b-10). **AC-33, cost (time half):** measured time per slice never exceeds bound × factor, on every listed target (x86-64, AArch64, one 32-bit device target), for every corpus program. The operation-count half runs in `dsl-conformance` (025a-11), because the count is the same on every machine.

The 32-bit target is provisional and may be emulated-only (open, epic 026); an emulated-only target has no factor and is skipped by this job with a visible notice, not silently.
## Work
1. Add CI job `dsl-cost` running the 025b-10 harness on every calibrated listed target, applying the recorded factor.
2. Fail if any program's measured time exceeds bound × factor on any target; publish the per-target table as an artefact.
3. Note measurement noise handling (repeat count, percentile) in the job description; list uncalibrated targets in the job output.
## Acceptance criteria
1. `dsl-cost` is green on every calibrated listed target for all positive corpus programs.
2. A planted program whose bound is too low turns the job red on at least the host target.
3. An uncalibrated target appears in the job output as skipped, with the reason.
4. The job runs unprivileged.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Deriving the calibration (025b-10); operation counting (025a-11).
