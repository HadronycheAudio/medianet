# 008b-02 — Case 02: step response
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** S · **Depends on:** 008a-13, 008a-15
## Context
Simulation case 2: a stepped holder causes an epoch change and re-lock within C-63.

Rule `[decision]`: a step larger than 2 ms (C-67, inherited, to be confirmed here) is never slewed; it is a timeline epoch change. Locked requires bound ≤ 25 µs (C-62). After an epoch change the estimator restarts, so re-lock is an acquisition: within 10 s (C-63) from the first exchange of the new epoch.

## Work
1. Named test `case_02_step_response`: holder clock steps by +5 ms, −5 ms, +2.5 ms at a scheduled time together with an epoch change; also a +1.5 ms step without epoch change (below C-67).
2. Measure re-lock time (from first exchange of the new epoch to `Locked`) over at least 20 seeds under `WIRED_PROFILE` (C-94).
3. Assert re-lock within C-63 (or its replacement from 008b-01); record C-67 confirmed or replaced, with justification (noting that C-136 reuses it as the holder stall threshold), in `doc/design/reports/008-simulation-results.md` (consumed by 008b-18).

## Acceptance criteria
1. For each >2 ms step: the published epoch changes, no view of the old epoch shows a phase movement toward the step, and re-lock occurs within 10 s (C-63, or its replacement) for all seeds.
2. For the 1.5 ms step: no epoch change; the servo absorbs it by frequency only with the correction term within C-60/C-61 (outside acquisition) — or the test documents that it drops out of Locked honestly.
3. Results file has a C-67 line: "confirmed" or replacement with justification.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Deciding when the network bumps the epoch (epic 006).
