# 008b-10 — Case 09: wall-clock step
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** XS · **Depends on:** 008a-08, 008a-15
## Context
Simulation case 9: a wall-clock step during a run does not reach the estimate.

POC: receive time came from a `CLOCK_REALTIME` kernel stamp `[kernel-doc]`; a wall-clock step between stamp and read corrupted the sample, clamped to look like zero delay `[code]`. Rule `[decision]`: such a step is detected and the sample discarded by the platform backend that builds exchanges, so the servo only receives exchanges that passed (C-121). The simulated backend and the reference detection are in 008a-08.

## Work
1. Named test `case_09_wall_clock_step`: after lock, wall-clock steps of ±1 s, ±1 h and ±10 µs, some landing between a packet's kernel stamp and its read; compare with the same seed without steps.

## Acceptance criteria
1. Each step landing between stamp and read discards exactly that sample in the simulated backend (counter increments by 1) and no `Exchange` is built from it.
2. Published `offset_ns`, `rate_ppb`, `state` and `epoch` differ from the step-free run by at most the effect of the discarded samples: |true error difference| ≤ 1 µs at every sample, and no state or epoch change.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The detection function and wall-clock model (008a-08); the production backend (epic 015).
