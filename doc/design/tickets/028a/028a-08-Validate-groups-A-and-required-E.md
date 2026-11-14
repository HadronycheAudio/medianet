# 028a-08 — `validate`: presentation groups, A and required E
**Epic:** 028a-Commissioning-configuration-and-offline-validation · **Effort:** S · **Depends on:** 028a-03, 028a-04

## Context
A presentation group is a set of sinks that emit in step. Every sink in a group emits sample p at `emit(p) = T(p) + A` and submits it to hardware at `emit(p) − D` (D = that sink's total plumbing delay from hand-off to emission: driver buffer, resampler and converter, C-170).
- A (audition offset) is operator configuration, one per group. **A has no default value** (C-55). The default group (the whole domain) is the `[[group]]` named `default` with no `members` list; it holds every sink not listed in another group, and its A is required like any other (028a-03, epic 028a Background).
- A group may carry a **required E**; the runtime refuses at admission a sink whose E exceeds it. E = 99.9th percentile of |actual emission − (T(p) + A)| over one hour (C-73).
- Only the commissioned state gives a numeric E; uncertified and self-monitored sinks declare E as unknown (C-140). So a declared `e_us` stands for a commissioned E (from `certify-sink`, 028b-12).
- A sink with unknown E is refused with reason RequiredEExceeded from any group that sets a required E, and admitted to groups that set none (C-103).
- Targets: Linux reference sink 50 µs (C-70); macOS 100 µs (C-71); Windows WASAPI exclusive 1 ms (C-72).
- Typical D: Linux about 1 ms; macOS a few ms; Windows 6 to 20 ms (C-81). A group containing a Windows edge needs an A of the order of 10 to 25 ms.

## Work
- Check `group.no_a`: refuse a group with no A (C-55), including the `default` group whenever it has at least one sink.
- Check `group.member_unknown`: refuse a member that is not a declared sink module.
- Check `group.duplicate_member`: refuse a sink in more than one named group.
- Check `group.e_exceeds`: refuse a member whose declared E > the group's required E, naming sink, declared E and required E (reason RequiredEExceeded).
- Check `group.e_unknown`: refuse a member with unknown E in a group that sets a required E (C-103), with the same reason.
- Flag `group.a_below_d`: warn if A is below a member's declared D (when D is declared), since emission at `emit(p) − D` would precede T(p).
- Tests for each.

## Acceptance criteria
1. A group with required E 100 µs containing a Windows sink declaring 1 ms is refused by `validate` before deployment (epic AC-5), naming the sink.
2. The same group with a macOS sink declaring 100 µs passes.
3. A sink with no declared E is refused from a group with a required E, and passes in a group with none (C-103).
4. A group without `a_ms` is refused; sinks left in the `default` group with no `a_ms` are refused.
5. Unknown and duplicate members are refused.
6. All tests run with no network.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] `validate` runs with no network and has tests for every refusal

## Out of scope
- Fan-in budgets and operator policy (028a-09). The runtime-side refusal at admission (runtime AC-30, verified by the runtime epics). Proposing A (028a-11, 028b-11). Measuring E (028b-12, 028b-14).
