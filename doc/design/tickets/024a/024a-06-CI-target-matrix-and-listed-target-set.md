# 024a-06 — CI target matrix and listed-target set
**Epic:** 024a-Runtime-ABI-and-arithmetic-kit · **Effort:** S · **Depends on:** 024a-05

## Context
Determinism claims are made per target. The epic requires kit results to be identical "on every CI target (x86-64, AArch64, one 32-bit device target)" (024a/AC-2), and part 024b requires stack bounds to hold "on the hosted target and on one device target" (024b/AC-2). exact-fixed programs must be bit-identical on every listed target; exact-float programs are bit-identical only where the fingerprint matches (024b/AC-3, contributes to AC-32). The machine pair for cross-machine bit-exactness is x86-64 and 64-bit ARM (C-151), so both are in the matrix. Exactness between a host and a small device is promised only for exact-fixed `[decision]`. Edge hardware at 48 kHz converts 2:1 through the kit's resampler node; the wire stays at 96 kHz (C-01, C-80).

The 32-bit device target is an open sub-question of the epic owned by this ticket and checked with task 026. Falsifier: the target cannot be emulated unprivileged in CI, or task 026's reference device differs from it in integer or float behaviour.

## Work
- Add CI jobs for: x86-64 Linux (hosted), AArch64 (hosted, native or emulated), and one 32-bit device target (choose, e.g. a `thumbv7em-none-eabihf` / `thumbv8m.main-none-eabihf` class target, run under an emulator).
- Each job builds `hadronyche-kit` (and later generated modules) and runs a shared test runner that prints result vectors to a file for cross-target comparison (024a-14, 024b-18 consume it).
- Record the target list as the listed-target set for exact-float as a new row of the constants table `doc/design/02-constants.md`, with the chosen device target, its emulator and the reason for the choice in a specification under `doc/design/spec/` (C-130), tagged `[decision]`.
- Confirm the choice with the owner of task 026 and record the open item (owner, falsifier above) if 026's reference device is not yet known.
- All jobs run unprivileged.

## Acceptance criteria
1. CI has three jobs (x86-64, AArch64, 32-bit device) that build `hadronyche-kit` and run its test runner; all green.
2. The 32-bit device job runs tests under an emulator without root privileges.
3. Each job uploads a result-vector artefact with a stable file name per target.
4. The constants-table row and its specification name exactly the CI targets and states that exact-float exactness is claimed only on these targets.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The CI target list is recorded as the listed-target set for exact-float
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The cross-target comparison tests themselves (024a-14, 024b-18).
- The reference embedded device of epic 026.
