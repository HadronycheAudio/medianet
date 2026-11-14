# 025b-12 — Calibration factor in the regime stamp
**Epic:** 025b-Replay-cross-target-fingerprint-and-cost-calibration · **Effort:** XS · **Depends on:** 025b-10, epic 013, epic 015
## Context
The compiler's cost bound is a count of abstract operations per slice, identical on every machine. Each target has a calibration factor (time per abstract operation), measured once and **recorded in the regime stamp**; admission converts the bound to time with it (C-106). Ticket 025b-10 measures the factors; this ticket makes sure the running target's factor reaches the stamp.

Regime stamp transport (C-18): a dedicated control message split into sections that each fit the control message bound; multicast on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch. The stamp message is defined by epic 013 and sent by the hosted backend, epic 015 (C-144). The factor comes from the reference-workload procedure of C-176 (025b-10).

Still open, owned by the regime stamp epic 013: the name and encoding of the calibration-factor stamp field. If 013 has not defined it, file the need against 013 and use a provisional name marked as such.

An uncalibrated target (emulated only, 025b-10) has no factor; the stamp must say so explicitly rather than carry a made-up value.
## Work
1. Make the per-target factors recorded by 025b-10 available to the runtime as a build-time constant per target.
2. Fill the stamp's calibration-factor field from it at start; "not calibrated" where the target has none.
3. A test that fetches the stamp by unicast request on the host and checks the factor equals the recorded one.
4. Document the field (name, unit, meaning, the "not calibrated" value) in `doc/design/spec/` beside the epic 013 stamp layout (C-130).
## Acceptance criteria
1. On the host target, the stamp's calibration factor equals the value recorded in `doc/design/reports/`.
2. A build for a target with no factor stamps "not calibrated".
3. Changing the recorded factor and rebuilding changes the stamped value.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Measuring the factor (025b-10); admission's use of it (the admission epic); the stamp message itself (epic 013) and its sending (epic 015, C-144).
