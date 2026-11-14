# 016-16 — Failed-pin layout check
**Epic:** 016-Real-time-setup-backend · **Effort:** S · **Depends on:** 016-05, 016-11

## Context
- Seqlock invariant (016-05): clock-thread priority ≥ every data-thread priority, **or** the clock thread and that data thread are pinned to different cores. `validate_layout` checks the request, before any thread is set up.
- C-177: if a thread layout's seqlock safety depends on a CPU pin and that pin fails, the module refuses to start in every mode, strict or not. Default priorities (clock above data, 016-06) never depend on pinning.
- C-201: an *unverified* pin is treated as failed for C-177: the module refuses to start in every mode.
- A data thread's safety depends on pinning when its requested priority is above the clock thread's (a `None` priority is `SCHED_OTHER`, below every `SCHED_FIFO` priority `[kernel-doc]`) and the layout passed only because the two are pinned to different cores. Both pins then matter: the clock thread's and that data thread's.
- `setup_thread` (016-11) reports `cpu_pinned` per thread and never refuses for it, because it sees one thread only. The C-177 refusal therefore needs a separate check over the clock and data requests and their reports.
- Signature (epic Deliverables): `pub fn check_layout_pins(clock: (&RtRequest, &RtReport), data: &[(&RtRequest, &RtReport)]) -> Result<(), RtRefusal>;`

## Work
1. Implement `check_layout_pins` in `hadronyche-os-linux::rt` (pure, no syscalls): for each data thread whose requested priority is above the clock thread's, require `cpu_pinned` to be `Present` on both the clock thread and that data thread.
2. Otherwise return `RtRefusal` with kind `SeqlockLayout` and a reason naming the data-thread index, which pin failed (clock or data), both priorities and both requested cores. The function takes no `Mode`: the refusal applies in both modes (C-177).
3. `Grant::Absent` is a failed pin. `Grant::Unverified` is also a failed pin (C-201); doc comment says so.
4. Doc comment stating the rule, C-177, and the call point: after `setup_thread` on the clock and every data thread, before any of them enters its loop (the caller is epic 015).
5. Unit tests covering the table below, plus one unprivileged integration test that obtains a real `Absent` pin from `setup_thread` with an invalid `cpu`.

## Acceptance criteria
1. Clock 70 on cpu 1, data 80 on cpu 2, both pins `Present` → `Ok`.
2. Same layout, data pin `Absent` → `Err(SeqlockLayout)` naming the data pin; clock pin `Absent` → `Err(SeqlockLayout)` naming the clock pin.
3. Same layout with a failed pin gives the same result whichever mode the caller uses (documented; the function takes no `Mode`).
4. Default priorities from `default_for` (clock 71, data 70, unpinned, 016-06) → `Ok` whatever the pin grants (C-177).
5. Clock 70, data 70 on different cores with a failed pin → `Ok` (equal priority does not depend on pinning).
6. Same layout as 1, data pin or clock pin `Unverified` → `Err(SeqlockLayout)` naming that pin, in both modes (C-201).
7. With 4 data threads where only index 2 relies on a pin that failed, the error reason names index 2.
8. Unprivileged integration test: a data report from `setup_thread` with an out-of-range `cpu` (`cpu_pinned` *absent*), data priority above the clock's → `Err(SeqlockLayout)` in both modes.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Request-level layout validation (016-05).
- Per-thread setup and the `cpu_pinned` grant itself (016-11).
- Calling the check from process start-up and exiting the process (epic 015).
