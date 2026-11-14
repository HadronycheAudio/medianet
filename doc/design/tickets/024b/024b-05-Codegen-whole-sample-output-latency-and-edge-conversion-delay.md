# 024b-05 — Codegen: whole-sample output latency and edge conversion delay
**Epic:** 024b-DSL-code-generator · **Effort:** S · **Depends on:** 024a-12, 024b-04, epic 023

## Context
Position invariant `[decision]`: the module cannot set positions. If the module has structural latency L, the runtime stamps the output p − L; generated code simply produces the delayed content and exposes L as a constant per output (`OutputInfo.latency: Latency`). L is always a whole number of domain-rate samples, in the manifest and the ABI (C-105). Where paths of different latency meet at a join, the compiler pads the shorter one and the module's L is the longest path (C-104); padding itself is 024b-06.

The DSL resampler node is fixed-ratio, offered only at 2:1 and 1:2 (C-105, C-189), and may appear only inside an edge module; the compiler rejects it anywhere else (C-134). An edge's conversion delay is part of its plumbing delay D (driver buffer, resampler and converter, C-170), not of any output's L (C-134). An edge's resampler delay is counted only in D and never in L; L is the module's structural delay in the domain-rate timeline, and the manifest reports the conversion delay separately, on the device-facing port (C-207). The kit exposes that delay as the whole-sample constant `group_delay` (024a-12). An even-tap linear-phase FIR is rejected by the compiler in v1, with a diagnostic suggesting an odd-tap design, so that L stays exact (C-183). The front end (epic 023) implements both rejections, the code generator only asserts that it never receives such a program, and the conformance corpus (epic 025a) tests them (C-218); this ticket therefore no longer re-implements them. The edge's rate actuator is a separate, steerable resampler whose ratio varies by parts per million around its nominal value to track the hardware crystal; its delay is part of D (C-189). It is not a DSL node and this generator neither emits it nor records its delay.

Epic acceptance criterion 024b/AC-9: each output's L is a whole number of domain-rate samples equal to the probe-measured delay (contributes to AC-33: compiler L equals the probe-measured value); an edge module's conversion delay is reported as a component of D, on the device-facing port, and is not part of any L (C-207); a resampler outside an edge module, or at another ratio, and an even-tap linear-phase FIR (C-183) are rejected by the front end, and the code generator asserts it never receives them (C-218).

## Work
- In codegen, compute each output's `latency` as the longest path latency from any input, summing the whole-sample structural latencies of the nodes along each path (integer arithmetic only).
- Expose per-path latencies to 024b-06 (join padding) and 024b-10 (degraded delay).
- For an edge module, take the resampler's delay from `hadronyche_kit::resample::group_delay` (single source of truth) and record it as the conversion delay of the edge module's device-facing port for the manifest (written by 024b-12), as a component of D, separate from any output's L; do not add it to any output's L (C-207).
- Assert that the IR contains no resampler node outside an edge module or at a ratio other than 2:1 and 1:2, and no even-tap linear-phase FIR; a violation is an internal compiler error naming the rule, not a user diagnostic (C-183, C-218).
- Probe test: generate edge-module programs with one resampler at 2:1 (capture) and at 1:2 (playback), run an impulse through each, and compare the measured resampler delay with the recorded conversion delay; generate non-edge programs with delay-bearing paths and compare the measured delay with `INFO.outputs[k].latency`.

## Acceptance criteria
1. For the 2:1 and 1:2 edge programs, the conversion delay is recorded on the device-facing port, is a whole number of domain-rate samples equal to the probe-measured resampler delay, and no output's L includes it (C-134, C-207).
2. An output with two paths of different whole-sample latency reports the longer one (C-104), and the probe-measured delay equals it exactly.
3. An output not downstream of any latency-bearing node reports latency 0.
4. A hand-built IR that bypasses the front end with a misplaced resampler, or with an even-tap linear-phase FIR, trips the assertion as an internal compiler error naming the rule; the user-facing rejections and their diagnostics are not tested here (epic 023, epic 025a; C-218).
5. An odd-tap linear-phase FIR compiles with a whole-sample L (C-183).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Join padding delay lines and inserted-delay reporting (024b-06).
- Runtime stamping of p − L (epic 009); how the edge assembles and certifies D, including the rate actuator's delay (C-189) (other epics, L2).
- The "frequency-dependent delay" property on outputs (024b-21).
- The front-end rejections (misplaced resampler, even-tap linear-phase FIR) and the DSL syntax of an edge module (epic 023, C-218); testing those rejections (conformance corpus, epic 025a).
- The project-level AC-33 probe in the conformance suite (epic 025).
