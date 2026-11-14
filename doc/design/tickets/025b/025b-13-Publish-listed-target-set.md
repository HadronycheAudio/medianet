# 025b-13 — Publish the listed-target set
**Epic:** 025b-Replay-cross-target-fingerprint-and-cost-calibration · **Effort:** XS · **Depends on:** 025b-06, 025b-08, 025b-09, 025b-11, 025b-12
## Context
**The listed-target set is whatever CI verified; nothing else claims exactness.** Epic 025b provides the listed-target set for exact-float, the per-target calibration factors (C-106), and the verified compiler that the embedded device (epic 026) relies on for host-to-device exactness.

Definition-of-done item: **the listed-target set is published in the design docs, with the CI job that verifies each target.** It goes in `doc/design/spec/`; the calibration measurements it references are in `doc/design/reports/` (C-130).

Candidate targets: x86-64, AArch64, one 32-bit device target (provisional until epic 026 names it, 025b-06). CI jobs: `dsl-conformance` (host), `dsl-cross-target` (exact vectors and fingerprint), `dsl-cost` (time against bound × factor).

On an unlisted target, or wherever the fingerprint mismatches, an exact-float module runs bounded (C-123).
## Work
1. Add a "Listed targets" table to `doc/design/spec/`: target triple, exact-fixed verified, exact-float verified (fingerprint match), calibration factor and its report entry (or "not calibrated: emulated"), verifying CI job(s), hardware or emulated.
2. Include only targets green in `dsl-cross-target`, and in `dsl-cost` where calibrated.
3. State the rule that unlisted targets run exact-float modules as bounded.
## Acceptance criteria
1. The table lists exactly the targets green in the jobs above, each with its job names.
2. Each row references its 025b-10 calibration entry or states why there is none.
## Definition of done
- [ ] The listed-target set is published in the design docs, with the CI job that verifies each target
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Running the jobs (025b-06, 025b-11).
