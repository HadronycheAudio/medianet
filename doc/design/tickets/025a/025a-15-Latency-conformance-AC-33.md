# 025a-15 — Latency conformance (AC-33)
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** S · **Depends on:** 025a-04, 025a-06, 025a-10, 025a-13
## Context
**AC-33, latency:** for every corpus program, the compiler's structural L equals the probe-measured value **exactly**, per output, as a whole number of domain-rate samples. Programs flagged non-constant-delay are reported as such by the probe (delay varies with marker spectrum).

Settled: L is always a whole number of domain-rate samples and equality is integer equality (C-105). The earlier rational-L rule is withdrawn, so no fractional measurement method is needed and none is built. The resampler node exists only at 2:1 and 1:2 and only inside edge modules (C-105, C-134); the corpus chains are 96→48, 48→96 and the 96→48→96 round trip (025a-04). Where paths of different latency meet, the compiler pads the shorter path with a whole-sample delay line and L is the longest path; a path with frequency-dependent delay is padded on its structural part only and the join is flagged in the manifest (C-104). The join programs of 025a-06 are in scope.

Settled (C-207): an edge's resampler delay is counted only in D and never in L; L is the module's structural delay in the domain-rate timeline, and the manifest reports the conversion delay separately, on the device-facing port. For a resampler path the probe therefore measures L plus that conversion delay.
## Work
1. A test in the `dsl-conformance` job that runs the probe (025a-12/025a-13) on every positive corpus program and compares per output with the compiler's L: integer equality for constant delays; for outputs the compiler flags non-constant, the probe must report `NonConstant`.
2. For the resampler programs of 025a-04, compare the probe with the compiler's L plus the conversion delay the manifest reports on the device-facing port (C-207), and report both terms on a mismatch.
3. For the join programs of 025a-06, compare the probe with L = the longest path; on the flagged recursive-path join, check that the probe reports `NonConstant` on the affected output.
4. On a mismatch, report program, output, compiler value and probe value; file the defect against epic 023/024, never edit the expectation to match.
## Acceptance criteria
1. Every positive corpus program (at least 40) passes: L equal exactly per output, or both sides non-constant.
2. The 96→48, 48→96 and round-trip programs pass, float and fixed: the probe equals L plus the reported conversion delay, and L does not include the resampler delay (C-207).
3. Every join program of 025a-06 passes with L equal to its longest path.
4. Changing one program's delay by 1 sample without updating the compiler output makes the test fail.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Cost and memory checks (025b-10, 025a-11, 025a-17); cross-target runs (025b-06); the inserted-delay values themselves (025a-06).
