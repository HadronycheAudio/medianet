# 007b-05 — Admission scenario vectors
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** M · **Depends on:** 007a-04, epic 004, epic 005, epic 006

## Context
- Scenario vectors are ordered lists of `(time, input) → expected (outputs, events, resulting state)`, stored in `vectors/control/*.json`, driven through `UnderTest::control_step` (007a-04). Every transition row must be exercised (epic criterion 007-AC3).
- Admission is two-sided: both producer and receiver must accept `[decision]`. Each refusal reason must be exercised. There are 18 refusal reasons (C-185); those named in the constants table are RequiredEExceeded (C-103), PathMtuTooSmall, ForeignDomain and BadMemberList (C-173), ControlDegraded, NoSkewBudget, VersionMismatch, NameTaken and ModuleCap (C-185). VersionMismatch, NameTaken and ModuleCap are raised at discovery, not at admission (C-185; 007b-01). The remaining names are still owned by epic 005; this ticket takes them from the admission state table, which epic 005 supplies (C-193).
- Relevant connection states: Requested, Pending, Refused, Admitted `[decision]`.
- Settled admission rules:
  - Pending has no timeout: it persists until the target appears or the operator cancels, and is shown as unsatisfied (C-45); only Live counts as satisfied (C-154);
  - only a commissioned sink has a numeric E; uncertified and self-monitored sinks declare E as unknown (C-140). A sink with E unknown is refused with reason RequiredEExceeded from any presentation group that sets a required E, and admitted to groups that set none (C-103);
  - the admission ceiling is 70 % of the link rate (C-23); declared trunk capacity is per direction (C-117). The name of the capacity refusal reason comes from epic 005;
  - limits: at most 32 members per bundle, the controller member included (C-08, C-209), 256 streams per source (C-09), 64 modules in the map (C-34).
- Sources pinned at admission are the only ones whose packets are parsed beyond the filter `[decision]`.

## Work
1. Write scenarios in `vectors/control/admission-*.json`:
   - success: Requested → Pending → Admitted, both sides accept;
   - Pending persistence: the target is absent for at least 1 hour of scenario time; the state stays Pending, shown as unsatisfied, with no timeout transition; the target then appears and admission proceeds;
   - Pending cancel: the operator cancels a Pending connection (resulting state from the transition table);
   - one scenario per refusal reason raised at admission, from each side where it applies, ending in Refused;
   - sink with unknown E, once for an uncertified and once for a self-monitored sink (C-140): refused with RequiredEExceeded by a group with a required E; admitted by a group with none;
   - capacity per direction: a request that keeps one direction at or below 70 % of its declared capacity is admitted even though the opposite direction is already near its ceiling; a request that would take one direction above 70 % is refused;
   - the 257th stream from one source, if that is a refusal reason in the tables. The 33rd bundle member is BadMemberList, covered with the other C-173 cases in 007b-06.
2. Each Admitted scenario records the pinned source in the expected state.

## Acceptance criteria
1. Every refusal reason in the tables, 18 per C-185, has at least 1 scenario here, in 007b-06 or, for VersionMismatch, NameTaken and ModuleCap, in 007b-01 (checked by a test against the enumeration).
2. The success scenario passes through Requested, Pending and Admitted in that order.
3. The Pending persistence scenario contains no transition out of Pending before the target appears.
4. The unknown-E scenarios expect RequiredEExceeded for the group with a required E and Admitted for the group without, for both an uncertified and a self-monitored sink.
5. The capacity scenarios show that only the load in the requested direction is compared with 70 % of that direction's declared capacity.
6. Every admission transition row is covered (007b-13 report).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Vector files carry the specification version they were generated from

## Out of scope
- States after Admitted (Locking onwards) (007b-07).
- Packet filtering of unpinned sources (007a-13).
- Follower refusal at the clock holder (007b-03).
- PathMtuTooSmall, ForeignDomain, BadMemberList, ControlDegraded and the fan-in budget rule with NoSkewBudget (007b-06); VersionMismatch, NameTaken and ModuleCap (007b-01); connection alarms (007b-09).
