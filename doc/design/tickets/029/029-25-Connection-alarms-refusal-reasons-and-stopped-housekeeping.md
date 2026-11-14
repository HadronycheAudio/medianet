# 029-25 — Connection alarms, refusal reasons and stopped housekeeping
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-09, 029-11, 029-13

## Context
- `hadronyche-monitor` (epic 029) shows alarms as the runtime publishes them and never derives them itself. `[decision]`
- **Connection alarms** (C-173): a connection alarm is raised on refusal, interrupted, dead, demoted, cannot-meet-A and control-degraded.
- **Cannot meet A** (C-174): maximum lateness + D + headroom exceeds A in each of 10 consecutive 1 s windows, the lateness being over the rolling 60 s window (C-182). The runtime raises it; the monitor only names it.
- **Refusal reasons** (C-173, completed by C-185): there are 18, including RequiredEExceeded (C-103), BadMemberList (an unknown or duplicate bundle member, or more than 32 members), ControlDegraded (the target's housekeeping has stopped restarting, C-139; answered by the housekeeping supervisor, which keeps a minimal refusal path) and NoSkewBudget (a fan-in with no downstream group and no configured budget, C-142). VersionMismatch, NameTaken and ModuleCap are raised at discovery, where they stop a module joining, not at admission (C-185). PathMtuTooSmall applies only when the probe at the 1 280-byte fallback also fails; ForeignDomain is for a different non-zero domain. A hello with domain 0 is ignored and counted, not refused.
- **Stopped housekeeping** (C-139): housekeeping restarts back off from 100 ms to 10 s; after 5 panics within 60 s restarts stop. The module stays control-degraded with an alarm, its live streams continue, and it accepts no new connection until an operator restarts it. This differs from an ordinary control-degraded mark (one restart, streams unaffected) and the operator must be able to tell the two apart. The C-145 test hook injects a thread panic, and that injection is stamped.
- 029-09 (connection view), 029-11 (Medianet panel) and 029-13 (Connections panel) did not cover alarm kinds, the full set of refusal reasons or the stopped-restart state.

## Work
1. In the model (029-09), `enum ConnAlarm { Refused, Interrupted, Dead, Demoted, CannotMeetA, ControlDegraded }` (C-173), read from the published alarm field. Refusal reasons are carried as the published name, so all 18 render without a monitor-side list going stale; unknown names render verbatim. VersionMismatch, NameTaken and ModuleCap are not connection refusals (C-185): they are shown against the module that failed to join on the Medianet panel (029-11) if published, otherwise recorded as `NotPublished` under the epic's "where published" open question (epic 013).
2. Per module, a `control: ControlHealth` with `Normal`, `Degraded` (restarted, C-139 backoff in progress) and `Stopped` (restarts stopped, accepts no new connection, C-139), if the stamp or hello distinguishes them; otherwise `Degraded` only, and the missing distinction is recorded as `NotPublished` and reported under the epic's "where published" open question (epic 013).
3. Connections panel (029-13): an alarm column naming each active alarm; cannot-meet-A in error style with the C-174 rule and its 60 s window (C-182) in its help text.
4. Medianet panel (029-11): `Stopped` shown as "control stopped — no new connections, live streams continue" in error style, distinct from the neutral control-degraded note.
5. Headless line (029-18): count of active connection alarms by kind and a `CTRL-STOPPED` flag.
6. Text captures for the documentation.

## Acceptance criteria
1. A fixture with one connection per C-173 alarm kind shows six rows, each naming its alarm.
2. A Refused connection with reason BadMemberList renders that name, and so do ControlDegraded and NoSkewBudget (C-185); a reason name not known to the monitor renders verbatim, not as an error.
3. A module in `Stopped` renders the stopped mark in error style; a module with one restart renders the neutral control-degraded note; its live connections are not marked faulty in either case.
4. A fixture driven by the C-145 panic hook carries the injection label (029-08) as well as the control-degraded mark.
5. No monitor code computes cannot-meet-A or counts panics (code review checklist item).
6. A fixture with a module stopped at discovery by VersionMismatch shows it on the Medianet panel against that module, not as a connection row in the Connections panel (C-185).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] A screenshot or text capture of each panel is in the tool's documentation
- [ ] Every new factual claim carries an evidence tag
- Not applicable (C-125): "Regime stamp fields touched are complete and documented" — this ticket adds or changes no stamp field; it reads the published alarm field.

## Out of scope
- Raising alarms, the restart policy and refusal itself (runtime epics; the housekeeping restart criterion belongs to epic 015, C-144).
- Defining where the stopped state is published (epic 013).
