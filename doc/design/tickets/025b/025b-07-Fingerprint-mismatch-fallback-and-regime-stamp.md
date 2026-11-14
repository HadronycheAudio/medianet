# 025b-07 — Fingerprint mismatch fallback and regime stamp
**Epic:** 025b-Replay-cross-target-fingerprint-and-cost-calibration · **Effort:** S · **Depends on:** 025b-06, epic 013, epic 015
## Context
**AC-32, fallback:** with a deliberately altered floating-point mode on the processing thread, the fingerprint mismatches, the module runs bounded-class, and the regime stamp records it.

Fingerprint: a compile-time vector (including subnormal and NaN paths) evaluated at start **on the processing thread itself**, because floating-point modes are per thread and audio stacks change them. Match ⇒ exact-float eligible. Mismatch ⇒ the module runs **bounded**-class and the **regime stamp** says so. A fingerprint mismatch is one of only two sources of the bounded class; the other is a hand-written module that does not claim exactness (C-123).

Regime stamp transport (C-18): a dedicated control message split into sections that each fit the control message bound (C-17); multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch; the hello carries only a stamp version number. The stamp message is defined by epic 013 and sent by the hosted backend, epic 015 (C-144).

Still open, owned by the regime stamp epic 013: the names and encoding of the stamp fields for fingerprint result and effective determinism class. This ticket asserts on whatever 013 defines; if 013 has not defined them, file the need against 013 and use provisional names marked as such.
## Work
1. A test that starts an exact-float corpus module on a processing thread whose FP mode was changed before module start (e.g. flush-to-zero set in MXCSR on x86-64, FPCR.FZ on AArch64).
2. Assert: fingerprint mismatches; the module's effective class is bounded; the stamp, fetched by unicast request (C-18), shows the fingerprint mismatch and the bounded class.
3. Control: the same module on an unaltered thread runs exact-float and the stamp says so.
4. Check that the fingerprint is evaluated on the processing thread (altering FP mode on a different thread must not cause a mismatch).
5. Document the stamp fields involved (name, values, meaning) in `doc/design/spec/` beside the epic 013 stamp layout (C-130).
## Acceptance criteria
1. Altered FP mode ⇒ fingerprint mismatch, effective class bounded, stamp shows bounded/mismatch — on x86-64 and AArch64.
2. Unaltered ⇒ match, exact-float, stamp shows exact-float/match.
3. Altering FP mode only on another thread ⇒ match.
4. The stamp read by unicast request and the next multicast refresh agree (C-18).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Mutation coverage of other divergence mechanisms (025b-08); implementing the fingerprint evaluation (epic 024); the stamp message itself (epic 013) and its sending (epic 015, C-144); the calibration-factor stamp field (025b-12).
