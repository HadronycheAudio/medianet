# 016-06 — Default thread priorities
**Epic:** 016-Real-time-setup-backend · **Effort:** XS · **Depends on:** 016-05

## Context
- Open sub-question owned by this task (epic, open sub-questions): default priorities for clock and data threads. The POC used `SCHED_FIFO` 70 for both `[code]`. The epic records the proposal data 70, clock 71. Falsifier: the seqlock invariant or L2 results require them to differ.
- Seqlock invariant: clock-thread priority ≥ every data-thread priority, or the two are pinned to different cores; violations are refused in both modes (`validate_layout`, 016-05).
- Housekeeping is not real-time and is supervised; it gets no `SCHED_FIFO` and no pinning by default `[decision]`. The epic defines "by default" as the default `RtRequest` for the housekeeping role (no priority, no core); an explicit request is honoured by `setup_thread`.

## Work
1. Add constants in `hadronyche-os-linux::rt` (or the config defaults of the runtime): `DEFAULT_CLOCK_PRIORITY`, `DEFAULT_DATA_PRIORITY`, and a `RtRequest::default_for(role: ThreadRole) -> RtRequest` returning: Clock and Data with their default priority and `cpu: None`; Housekeeping with `fifo_priority: None`, `cpu: None`, `spins: false`.
2. Use data 70 (the POC value) and clock 71, the epic's proposal, so the invariant holds even when unpinned `[decision]`.
3. Add the constants to the constants table (`doc/design/02-constants.md`, C-130), with the open item: owner = this task, falsifier = "L2 results (task 020) require a different ordering or gap".

## Acceptance criteria
1. `validate_layout(&default_for(Clock), &[default_for(Data); 8])` returns `Ok` (unit test).
2. `default_for(Housekeeping)` has `fifo_priority == None` and `cpu == None` (unit test).
3. The constants table in the design document lists both values with the `[decision]` tag and the falsifier.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Measuring whether the defaults meet L2 targets (task 020).
- Configuration parsing (epic 015 or the config epic).
