# 025b-06 — CI job `dsl-cross-target`: exact vectors and fingerprint gating
**Epic:** 025b-Replay-cross-target-fingerprint-and-cost-calibration · **Effort:** M · **Depends on:** 025a-09, 025a-10, epic 024
## Context
CI job `dsl-cross-target` runs the exact vectors and the fingerprint on **every listed target: x86-64, AArch64, and one 32-bit device target under emulation or on hardware**. The machine pair for cross-machine bit-exactness is x86-64 and 64-bit ARM (C-151).

Determinism classes:
- **exact-fixed** — bit-exact everywhere, including between a host and a device;
- **exact-float** — bit-exact only where the startup **fingerprint** matches;
- **bounded** — a runtime outcome only: an exact-float module whose fingerprint mismatched, or a hand-written module that does not claim exactness. The DSL has no non-deterministic node in v1, so every positive corpus program is exact-fixed or exact-float and has vectors (C-123).

Fingerprint: a compile-time vector (including subnormal and NaN paths) evaluated at start **on the processing thread itself**, because floating-point modes are per thread and audio stacks change them. It is a compiler promise implemented by epic 024; this epic tests it. Match ⇒ exact-float eligible. Mismatch ⇒ the module runs bounded-class and the regime stamp says so. **The listed-target set is whatever CI verified; nothing else claims exactness.**

Acceptance criteria covered:
- **AC-32, exact-fixed:** every exact-fixed program is bit-identical on every listed target.
- **AC-32, exact-float:** every exact-float program is bit-identical on every target where the fingerprint matches.
- From criterion 6: **a target that passes the fingerprint and differs on any vector is a failed suite.**

Vectors: `hadronyche-dsl/conformance/vectors/` (input slices and expected output hashes per positive program, 025a-09).

Still open, owned by the embedded device epic 026: **which 32-bit device target is listed, and whether it is available on hardware in CI.** Vectors and the fingerprint can run under emulation; cost timing cannot (C-106 needs a measured factor, 025b-10). Until 026 names it, this ticket picks a 32-bit target matching the device architecture as far as known, runs it under emulation, and records the choice as provisional.
## Work
1. Build the vector runner (025a-09) and the fingerprint evaluation for x86-64, AArch64 and the chosen 32-bit device target; run the latter under emulation (e.g. QEMU user mode) unless hardware is available in CI.
2. Run the fingerprint on the processing thread of the runner before vectors.
3. Rules: exact-fixed vectors must match on every target; exact-float vectors must match on every target whose fingerprint matches; a fingerprint match followed by any vector difference fails the job with "failed suite".
4. Also run the 025a-17 memory checks and the 025a-20 equivalence test per target where feasible.
5. Emit a per-target report (fingerprint result, vectors passed/failed, emulated or hardware) as a CI artefact for 025b-13.
## Acceptance criteria
1. `dsl-cross-target` is green on all three targets for every positive corpus program.
2. Every exact-fixed vector is bit-identical on all three targets.
3. Every exact-float vector is bit-identical on every target where the fingerprint matches; the report lists which targets matched.
4. A planted vector difference on a fingerprint-matching target fails the job with "failed suite".
5. The job runs unprivileged.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] The chosen 32-bit device target, whether it is provisional, and the emulation method are recorded in the CI job description
## Out of scope
- Fallback with altered FP mode (025b-07); fingerprint mutation coverage (025b-08); cost measurement (025b-10, 025b-11, 025a-11); publishing the listed-target set (025b-13).
