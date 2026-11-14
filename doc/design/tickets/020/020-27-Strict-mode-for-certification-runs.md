# 020-27 — Strict mode for certification runs
**Epic:** 020-L2-certification-harness · **Effort:** S · **Depends on:** 020-13, 020-14

## Context
Any run whose numbers enter the constants table as design constants runs in strict mode; L0 does not, and neither do POC runs (C-147). The Linux L2 runs of 020-22 and 020-23 write back C-68, C-70, C-74, C-75, C-76, C-77 and C-81, so they must run in strict mode. Strict mode is hosted-only (C-147), which covers every platform this harness certifies.

What strict mode requires (C-127): real-time scheduling granted on the data and clock threads, and memory locked. Restricting CPU idle states stays best-effort and is stamped. Strict mode refuses an unverified capability: unverified is treated as absent (C-128). The regime stamp reports each capability in one of four states: present, absent, unverified, not requested (C-129).

Strict mode is set by operator policy in the commissioning configuration (C-165). Tests that need a privilege run in a separate privileged nightly job; the "unprivileged" definition-of-done line applies to per-change CI (C-146).

## Work
- In `run.rs`, a `--strict` mode (the default for certification scenarios) that starts the sink and the reference holder with strict mode set in their configuration (C-165).
- Before measuring, read the capability states from the sink's regime stamp (C-129); refuse to start if real-time scheduling on the data or clock thread, or memory lock, is absent or unverified (C-127, C-128), with a non-zero exit naming the capability.
- Copy the strict-mode block (strict on or off, each capability's state, the CPU idle-state restriction as stamped) into the certification record (020-13).
- A run with `--no-strict` is allowed for development; its record is marked as not eligible for the constants table, and 020-24 refuses to write back from it.
- CI tests run the runner on recorded files without strict mode, unprivileged.

## Acceptance criteria
1. On a simulated stamp with memory lock reported unverified, a strict run is refused with a message naming memory lock (C-128).
2. On a simulated stamp with all strict-mode capabilities present and CPU idle-state restriction absent, the run starts and the record shows the idle-state restriction as absent (C-127).
3. A `--no-strict` record is marked not eligible for the constants table.
4. The CI tests pass unprivileged without audio hardware.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Regime stamp fields touched are complete and documented — not applicable (C-125): this ticket reads capability states from the sink's stamp and adds no regime stamp field

## Out of scope
- Obtaining real-time scheduling and memory lock on Linux (epics 016, 018).
- The stock versus real-time kernel question (020-22).
