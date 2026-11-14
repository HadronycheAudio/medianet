# 024a-11 — Resampler filter length versus device cost, and coefficient table
**Epic:** 024a-Runtime-ABI-and-arithmetic-kit · **Effort:** M · **Depends on:** 024a-05, 024a-06, 024a-07

## Context
The kit's DSL resampler node is fixed-ratio and is used for edges whose hardware runs at 48 kHz, which convert 2:1 through it while the wire stays at the domain rate of 96 kHz (C-01, C-80). The edge's rate actuator is a separate, steerable resampler (C-189, 024a-13), not this node. The node may appear only inside an edge module (C-134). It is offered only at 2:1 and 1:2, whose delay is a whole number of samples; other ratios are out of v1 (C-105). Its coefficients are a shipped table, and every shipped table has a checksum test and a note on how it was generated `[decision]`. No libm at run time.

Structural latency is always a whole number of domain-rate samples (C-105), so the chosen filter length must give each ratio a group delay that is a whole number of domain-rate samples. Even-tap linear-phase FIRs are rejected by the compiler in v1 so that L stays exact (C-183); the resampler filters are therefore odd-tap linear-phase designs. That delay is the edge's conversion delay, counted only in its plumbing delay D and never in L (C-134, C-170, C-207). The kit is the single builder of both resamplers; every edge (Linux, macOS, Windows, embedded) uses these tables and builds none of its own (C-197). The cost bound counts abstract operations per slice, identical on every machine; time follows from a per-target calibration factor (C-106).

This ticket answers the epic's open sub-question "Resampler filter length versus cost on a device" (owner: this task, then task 026; falsifier: the filter does not fit the reference device). Slice: 48 frames = 0.5 ms at 96 kHz (C-01, C-02); 6 channels (C-04).

## Work
- Choose the filter design (an odd-tap linear-phase FIR, e.g. windowed-sinc, in polyphase form; C-183), its tap count per phase and phase count for each ratio (2:1 and 1:2), constrained so that the group delay is a whole number of domain-rate samples, and the coefficient format (Q1.31 for the fixed path; f32 for the float path if one is kept).
- Measure on the 32-bit device CI target (024a-06), for candidate lengths, the counted abstract operations and the time per 48-frame, 6-channel slice; record results `[measured]` with the target and method.
- Write the decision (length, passband and stopband figures, cost) as a report in `doc/design/reports/` (C-130) and add an open item for task 026 with owner "task 026" and falsifier "does not fit the reference device".
- Add the coefficient tables to `hadronyche-kit/src/tables.rs` as constants, each with a checksum constant, checksum test and generation note (generator: the `gen-tables` binary in the `hadronyche-tools` crate, C-133; exact command recorded).

## Acceptance criteria
1. The report records the chosen tap count and phase count per ratio (2:1 and 1:2), the measured operation count and time per slice on the device target, and the frequency-response figures, all tagged `[measured]`.
2. Each coefficient table has a checksum test that passes; changing one entry by 1 makes it fail.
3. Re-running the generator reproduces each table byte for byte.
4. The group delay implied by the chosen length is stated for each ratio as a whole number of domain-rate samples (C-105) (consumed by 024a-12); the report states each prototype filter's total tap count, which is odd (C-183).
5. The open item for task 026 is recorded with owner and falsifier.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every shipped table has a checksum test and a note on how it was generated
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The resampler node implementation (024a-12) and variable-ratio mode (024a-13).
- Fitting the reference device of epic 026 (task 026 follows up).
