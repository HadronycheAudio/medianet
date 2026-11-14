# 025b-09 — Resampler fixed-point exactness across targets
**Epic:** 025b-Replay-cross-target-fingerprint-and-cost-calibration · **Effort:** XS · **Depends on:** 025a-04, 025b-06
## Context
Open sub-question owned by this epic: **is the resampler node exact-fixed in its fixed-point form?** Falsifier: bits differ across targets.

The kit's resampler node is offered only at 2:1 and 1:2 (C-105) and only inside an edge module (C-134); an edge with 48 kHz hardware converts 2:1 through it (C-80). The embedded device (epic 026) relies on host-to-device exactness. exact-fixed means bit-exact everywhere, including between a host and a device. Fixed point is Q1.31 / Q0.32. The former long-run vectors for the 147/320 ratio are dropped with the withdrawn rational-L rule: no such ratio exists in v1 (C-105).

Only the fixed-ratio node is under test. An edge's steerable rate actuator is a separate resampler whose ratio varies by parts per million and whose delay is part of D (C-189); it and its trait are built and tested by the kit epic 024a (C-197, C-210). The node's conversion delay is never counted in L; the manifest reports it separately on the device-facing port (C-134, C-207), so the vectors' output positions use the module's structural L only.

Corpus resampler programs (025a-04): 96→48, 48→96 and the 96→48→96 round trip, fixed variants. The 32-bit device target is provisional until epic 026 names it (025b-06).
## Work
1. Check that every fixed-variant resampler program has exact vectors (025a-09) and is run by `dsl-cross-target` (025b-06) on x86-64, AArch64 and the 32-bit device target.
2. Record the answer in `doc/design/reports/` (C-130) with `[measured]` evidence: exact-fixed confirmed, or the failure and the correction filed against epic 024. Since the DSL has no non-deterministic node and the compiler never assigns bounded (C-123), a fixed resampler that is not bit-exact is a compiler or kit defect, not a class change.
## Acceptance criteria
1. All fixed resampler vectors are bit-identical on all three listed targets, or the failure is documented and filed against epic 024.
2. The open sub-question is closed in the design docs with evidence, or kept open with owner and falsifier.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Resampler node and rate actuator implementation (kit epic 024a, C-197); exactness or tracking of the steerable rate actuator (024a, C-189, C-210); delay of resampler paths (025a-15).
