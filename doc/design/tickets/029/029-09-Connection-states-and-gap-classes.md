# 029-09 — Connection states and gap classes
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-06

## Context
- `hadronyche-monitor` (epic 029) must use the runtime's vocabulary correctly.
- **Connection states:** Requested, Refused (with reason, for example RequiredEExceeded, C-103; 18 reasons, including BadMemberList, C-173, and ControlDegraded and NoSkewBudget, C-185; VersionMismatch, NameTaken and ModuleCap are raised at discovery and stop a module joining, not at admission, C-185), Pending (target not present), Admitted, Locking, Aligning, Live, Interrupted, Dead, Ended. Pending has no timeout: it persists until the target appears or the operator cancels (C-45).
- **Satisfied** (C-154): only Live. Pending, Locking, Aligning, Interrupted and Demoted are shown by name as unsatisfied.
- **Health state Demoted** (C-109): an input persistently late (at least one late slice in each of 10 consecutive 1 s windows, C-101) with its offset already at A; treated as gapped, counted separately, cleared only by re-admission. Rounds spent demoted are counted per connection (demoted rounds, C-159).
- **Four gap classes** at a consumer: *marked gap* (upstream alive, loss further up), *late* (arrived after the deadline; the remedy is budget), *missing* (unknown), *dead* (classified).
  - Interrupted after **8 consecutive missed slices** (C-38); only slices absent at the deadline count, and a marked gap or a slice on another timeline epoch resets the count (C-102).
  - Dead after **2 s** (C-39), counted from the last slice received with content or a marked gap (C-111).
  - A slice on another timeline epoch restarts the dead clock, does not count as a miss, and has its own counter, other-epoch (C-135).
  - Interrupted and dead describe streams a peer receives; the liveness of a sink-only device is judged by the producer timeout (C-41, C-168).
  - Late versus missing (C-108): counted missing at the deadline, reclassified late (missing −1, late +1) if it arrives while still within the ring span.
- **Degraded:** a slice computed from a gapped input; the flag follows the content, so on an output with structural latency L (a whole number of samples, C-105) it appears L after the gapped input (C-122).
- Per-connection summary counters in each endpoint's stamp: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159), plus the observed DSCP (C-47). These map one to one onto the panel's columns.
- **Loss versus the floor:** expected about one loss per stream per link about every 13 hours (C-90) `[memory]`; loss above the alarm threshold (C-91; proposed as 10 × the clean-wire loss measured at L1, C-167) is a cable-health alarm.
- Open question in the epic, owned by epic 013: where the connection state and the Demoted health state are published. Working assumption: in the stamp; `NotPublished` otherwise.

## Work
1. `enum ConnState` with the ten states above (Refused carries its reason), a separate `health: Option<Health>` with `Demoted`, and `enum GapClass { Marked, Late, Missing, Dead }`.
2. `fn connection_view(&self, conn) -> ConnectionView`: state, `satisfied: bool` (true only for Live with no Demoted health, C-154), Pending with the time since it was first seen Pending, health, offset, the eleven C-159 counters, observed DSCP, degraded count if published, both endpoints' profiles, session number.
3. The monitor does not reclassify: states and gap classes come from the stamp as published. Thresholds C-38 (8 slices), C-39 (2 s) and C-101 (10 windows) appear only in help text and labels, not in monitor-side logic.
4. Loss floor annotation: show the missing count next to the expected floor (about one per stream per link per 13 h, C-90); flag `cable_health_alarm` when the module's alarm field or stamp says the C-91 threshold is exceeded. The C-91 value is never hard-coded.

## Acceptance criteria
1. Each of the ten states round-trips from a stamp to `ConnState` with the exact names above; a Refused stamp with reason RequiredEExceeded keeps that reason.
2. A stamp with late = 5, missing = 2, marked = 3, interrupted = 1, dead = 0 and the other C-159 counters at 0 gives a view with those numbers in the matching columns and no other column non-zero.
3. A connection in Dead state is shown as dead regardless of counters.
4. A connection Pending for 10 minutes is still Pending and shown unsatisfied (no timeout, C-45).
5. A Demoted input is shown with health Demoted, distinct from its connection state.
6. `satisfied` is true for Live only; Pending, Locking, Aligning, Interrupted and a Live-but-Demoted input each give false and keep their name (C-154).
7. A stamp with other-epoch = 4, too-early = 1, duplicates = 2, incomplete reassemblies = 1, demoted rounds = 3 shows each in its own column (C-159).
8. No code path in the monitor compares elapsed time against 2 s, counts 8 missed slices or counts late windows to derive a state (code review checklist item).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- Not applicable (C-125): "Regime stamp fields touched are complete and documented" — this ticket adds or changes no stamp field; it reads the C-47 / C-159 summary fields.

## Out of scope
- Rendering (029-13); the end-to-end injected-fault test (029-21); connection alarms and refusal-reason names (029-25).
