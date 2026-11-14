# 007b-06 — Admission details scenario vectors
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** M · **Depends on:** 007b-05

## Context
- Scenario vectors are ordered lists of `(time, input) → expected (outputs, events, resulting state)`, stored in `vectors/control/*.json`, driven through `UnderTest::control_step` (007a-04). Every transition row must be exercised (epic criterion 007-AC3).
- Admission details settled by C-173 `[decision]`:
  - PathMtuTooSmall applies only when the path-MTU probe at the 1 280-byte fallback (C-06) also fails; the probe pair is sent at the intended media packet size (C-116);
  - a hello with domain 0 is ignored and counted, not refused; ForeignDomain is the refusal for a different non-zero domain (domain 0 = unconfigured, C-11);
  - BadMemberList, the 16th refusal reason, covers an unknown bundle member, a duplicate bundle member, or more than 32 members (C-08), the controller member included: a source with controllers has at most 31 audio members, because its controllers always ride in their own member stream (C-205, C-209).
- Fan-in outside any presentation group (C-142) `[decision]`: it inherits the A of the downstream presentation group it feeds, the largest if several, learned at admission. With no downstream group it needs an explicit budget in configuration, or its aligned inputs are refused.
- The refusal of an aligned input of a fan-in with no downstream group and no configured budget is NoSkewBudget (C-185).
- ControlDegraded (C-185): a target whose housekeeping has stopped restarting (after 5 panics within 60 s, C-139) refuses a new connection with ControlDegraded; the refusal is answered by the housekeeping supervisor, which keeps a minimal refusal path, and the target's live streams continue (C-139).
- On a real implementation, probe failures and raw hellos are produced with the test-only hook that sends a raw packet on a media or control socket (C-120), and the control-degraded state with the panic-injection hook (C-145).

## Work
1. Write scenarios in `vectors/control/admission-detail-*.json`:
   - probe at the intended size fails, probe at 1 280 bytes succeeds: not refused with PathMtuTooSmall (admission proceeds as the tables state for a fallback path); both probes fail: refused with PathMtuTooSmall;
   - a request from a different non-zero domain: refused with ForeignDomain; a hello with domain 0 at admission time: ignored and counted, no refusal;
   - BadMemberList: one request naming an unknown member, one naming a member twice, one with 33 audio members, one with 32 audio members plus the controller member (33 in all, C-209); and, as the boundary, 31 audio members plus the controller member (32 in all): not refused with BadMemberList;
   - a fan-in outside any presentation group feeding one group: inherits that group's A; feeding two groups with different A: inherits the larger; feeding none, with an explicit budget: admitted with that budget; feeding none, without a budget: its aligned inputs are refused with NoSkewBudget (C-185);
   - ControlDegraded: a connection request to a control-degraded target is refused with ControlDegraded, and an existing live stream of that target continues (C-139, C-185).
2. Each scenario records the A or budget learned at admission in the expected state where one applies.

## Acceptance criteria
1. PathMtuTooSmall appears only in the scenario where the 1 280-byte probe also fails.
2. ForeignDomain is expected for the non-zero foreign domain only; the domain-0 hello produces a counted drop and no refusal.
3. Each of the 4 BadMemberList cases has its own scenario ending in Refused with BadMemberList, and the 31-plus-controller boundary scenario is not refused with BadMemberList (C-209).
4. The fan-in scenarios expect the inherited A (the largest of several), the configured budget, and the refusal of aligned inputs with NoSkewBudget when neither exists.
5. Every row these rules add to the admission tables is covered (007b-13 report).
6. All files carry `spec_version`.
7. The ControlDegraded scenario expects the refusal with ControlDegraded and no change to the target's live streams.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- The other refusal reasons and the main admission flow (007b-05).
- Encoding of the probe pair (007a-08) and of over-size inputs (007a-11).
- The alarm raised on refusal (007b-09).
- The housekeeping restart policy itself and the control-degraded alarm (epic 015, C-139, C-144).
- A change of A while inputs are live (007b-24).
