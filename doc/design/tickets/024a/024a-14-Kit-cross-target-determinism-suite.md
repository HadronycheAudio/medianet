# 024a-14 — Kit cross-target determinism suite
**Epic:** 024a-Runtime-ABI-and-arithmetic-kit · **Effort:** S · **Depends on:** 024a-06, 024a-07, 024a-08, 024a-09, 024a-10, 024a-11, 024a-12

## Context
Epic acceptance criterion 024a/AC-2: "Table checksums match on every CI target. `sine`, the fixed-point operations, the ramps and the resampler produce identical bits on every CI target (x86-64, AArch64, one 32-bit device target)." x86-64 and 64-bit ARM are the machine pair for cross-machine bit-exactness (C-151). Cross-platform float differences come from fused multiply-add contraction, non-IEEE subnormal handling (flush-to-zero / denormals-are-zero, per thread), NaN payloads and 80-bit intermediates on legacy x86 `[decision]`. The kit avoids FMA by construction.

## Work
- A kit test runner (from 024a-06) that, on each CI target, writes a result file containing: every table checksum; `sine` bits on a 4096-point grid plus edge points; Q1.31/Q0.32 saturating, wrapping and `mul_high` results on a fixed pseudo-random vector (seeded, 10⁴ cases); fixed and float ramp outputs; resampler outputs for 2:1 and 1:2 (the only ratios, C-105) on a fixed input.
- A CI job that downloads the three result files and compares them byte for byte, failing with the first differing item.

## Acceptance criteria
1. Table checksums are identical on x86-64, AArch64 and the 32-bit device target.
2. `sine` bits, all fixed-point operation results, ramp outputs and resampler outputs are identical on the three targets.
3. A deliberately injected difference (test fixture altering one value on one target) makes the comparison job fail and name the item.
4. The job runs unprivileged and is required for merge.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Cross-target identity of generated programs (024b-18).
- Adding targets beyond the listed-target set (024a-06).
