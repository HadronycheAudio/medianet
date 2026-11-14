# 012-22 — Operator and design documentation
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** 012-18, 012-19, 012-23, 012-25, 012-26, 012-27, 012-28, 012-29, 012-30

## Context
Epic 012 DoD requires that refusal reasons and connection states are documented for operators, and that design documents are updated where behaviour differs, constants table first.

Location (C-130, C-171): task outputs live in the design set's repository. The admission specification goes in `doc/design/spec/`; the operator reference (refusal reasons, connection states, alarms) goes in the operator guide in `doc/operations/`; architecture changes go in `doc/design/01-architecture.md`; constants go in `doc/design/02-constants.md`. Nothing is written into the frozen POC tree.

Evidence tags available: `[code]`, `[measured]`, `[kernel-doc]`, `[memory]`, `[standard]` (a cited published standard with its clause, C-132) and `[decision]`, as defined in `00-context.md`.

Decisions and provisional choices made in tickets that must be reflected, each marked settled (with its C-id) or provisional (with owner and falsifier):
- settled: bundle subset buffer (012-04, C-96), required E with unknown E and numeric E for commissioned sinks only (012-07, C-103, C-140), sessions in the request (012-12, C-19), Pending with no timeout and cancel (012-14, 012-24, C-45), Demoted (012-11, C-109), stamp record and its counters (012-23, C-18, C-47, C-159), table full (012-12, 012-13, epic rule), `BadMemberList` (012-05, C-173), `ForeignDomain` and domain 0 (012-06, C-173), `PathMtuTooSmall` (012-09, C-173), alarm triggers (012-15, 012-27, C-173, C-174), satisfied = Live only (012-18, C-154), inherited A outside groups (012-26, C-142), control-degraded gate (012-27, C-139), the 18 refusal reasons (C-185), `VersionMismatch`, `NameTaken` and `ModuleCap` raised at discovery, not at admission (012-06, C-185), `NoSkewBudget` for a fan-in with no budget (012-26, C-185), re-alignment of a fan-in when its inherited A changes (012-26, C-186), a changed A carried upstream in the receiver report, one hop per report (012-28, C-198), refusal with `ControlDegraded` by the housekeeping supervisor (012-27, C-185), the controller member counted in the 32 (012-05, C-209), required E judged on the sink's own figure and unchanged by an unlocked clock (012-07, C-180, C-194), a changed A beyond a fan-in's buffer memory capped with the cannot-meet-A alarm, held until a fitting change, with the given A still reported upstream (012-29, C-211, C-216), a sink's own changed A re-budgeted and switched at its own position (012-30, C-214), either controller layout of a single-stream source accepted (012-05, C-181), a controller member's wire cost (012-03, C-219);
- provisional: probe timeout (012-09), re-probe thresholds (012-10), producer state after `ProducerStopped` (012-15).

Constants to list (ids kept): C-01, C-04, C-06 1 280 bytes, C-08 32 members (controller member included, C-209), C-16 2 × admitted packets per slice, C-17 1 252 bytes, C-19, C-20 to C-27 rates, C-23 70 %, C-34 64 modules, C-35 64 followers, C-38 8 missed slices (4 ms), C-39 2 s, C-40 100 ms, C-41 2 s, C-45, C-47, C-50 4 s, C-53 one slice period (ceiling A), C-54 10 s, C-96, C-100 to C-103, C-106, C-109, C-111, C-116, C-135, C-137, C-139, C-140, C-142, C-154, C-156, C-159, C-164, C-168, C-173, C-174, C-185, C-186, C-180, C-194, C-181, C-198, C-208, C-209, C-211, C-214, C-216, C-219, plus the new pending-measurement proposals (probe timeout, re-probe thresholds).

## Work
- Operator reference in the operator guide in `doc/operations/` (C-171), with the admission specification in `doc/design/spec/`: one entry per `RefuseReason` (18, C-185) with meaning, likely cause and operator remedy, noting that `VersionMismatch`, `NameTaken` and `ModuleCap` are raised at discovery and stop a module joining; one entry per `ConnState` (11, including Demoted) with meaning, noting that only Live counts as satisfied (C-154); one entry per alarm trigger (refusal, interrupted, dead, demoted, cannot-meet-A, control-degraded, C-173), noting that cannot-meet-A is also raised when a changed A exceeds a fan-in's buffer memory, whose streams keep running, and stays raised until a later change of A that fits (C-211, C-216); note that refusal is terminal (no retry), Pending is not a refusal and has no timeout, a Pending request can be cancelled, Demoted is cleared only by re-admission, and a control-degraded module accepts no new connection until an operator restarts it and refuses requests with `ControlDegraded` meanwhile (C-139, C-185).
- Document the per-connection stamp record fields (path MTU, the C-159 counters, DSCP).
- Update the design documents: constants table first (new proposals with ids, status pending-measurement, owner and falsifier), then every behaviour that differs from the epic text.
- Ensure every new open item has an owner and a falsifier.

## Acceptance criteria
1. All 18 reasons, 11 states and 6 alarm triggers documented in `doc/operations/`; a check (script or test) fails if an enum variant is missing from the doc.
2. The constants table contains every new proposal introduced by tickets 012-09 and 012-10 with status, owner and falsifier.
3. Every item listed in Context is reflected in the design documents, marked settled, provisional or open.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable: documentation only
- [ ] Refusal reasons and connection states are documented for operators

## Out of scope
- Commissioning-tool UI text (epic 028).
