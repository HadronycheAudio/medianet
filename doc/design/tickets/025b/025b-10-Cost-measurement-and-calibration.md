# 025b-10 — Cost measurement and per-target calibration
**Epic:** 025b-Replay-cross-target-fingerprint-and-cost-calibration · **Effort:** M · **Depends on:** 025a-10, 025b-06
## Context
Settled unit (C-106): the compiler's **cost bound** is a count of **abstract operations per slice**, identical on every machine. Each target has a **calibration factor** (time per abstract operation), **measured once** and **recorded in the regime stamp**; admission converts the bound to time with it.

**AC-33, cost (time half):** on every listed target, measured time per slice never exceeds the bound multiplied by that target's recorded calibration factor, for every corpus program. The operation-count half (executed operations ≤ bound) is 025a-11.

Definition-of-done item: **the cost-bound calibration (operations to time, per target) is recorded.** The factor values are a deliverable of this ticket.

Procedure (C-176): the DSL conformance suite runs a **reference workload** once per listed target and records time per operation; the value is carried in the regime stamp. This ticket chooses and documents the reference workload. C-147 requires strict mode only for runs whose numbers enter the constants table; the factors go to `doc/design/reports/` and the stamp, and the report records whether the run was in strict mode (C-127).

Listed targets: x86-64, AArch64, one 32-bit device target (provisional, chosen in 025b-06). Still open, owned by the embedded device epic 026: whether the 32-bit target is available on hardware. Time measured under emulation is meaningless, so an emulated-only target gets no calibration factor; record it as "not calibrated: emulated" rather than inventing one. Slice = 48 frames = 0.5 ms at 96 kHz (C-01, C-02).
## Work
1. A cost measurement harness: per corpus program, run `process` over many slices with worst-case inputs and measure time per slice (maximum and a high percentile), unprivileged.
2. Define the reference workload (for example a fixed set of generated programs whose executed operation counts are known from 025a-11) and run it once per listed target; derive that target's calibration factor (time per abstract operation) from it with a stated safety margin (C-176). The factor is per target, not per program. Then check bound × factor ≥ measured time for every corpus program.
3. Record the calibration table (target, factor, margin, reference workload, measurement method, strict mode or not, hardware or emulated, date) in `doc/design/reports/` (C-130) with `[measured]` tags, and the per-target factors where the runtime reads them for the stamp (025b-12).
4. Expose the comparison as a test usable by the `dsl-cost` CI job (025b-11).
## Acceptance criteria
1. Harness runs every positive corpus program on the host and reports measured time per slice and bound.
2. A calibration factor, derived from one run of the reference workload (C-176), exists for every listed target measured on hardware; an emulated-only target is reported as not calibrated.
3. With the recorded factor, measured time ≤ bound × factor for every program on every calibrated target.
4. A program with an artificially halved bound is flagged.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The cost-bound calibration (operations to time, per target) is recorded
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- The CI job wiring (025b-11); executed operation count (025a-11); the stamp field (025b-12); the cost-bound computation in the compiler (epic 023/024).
