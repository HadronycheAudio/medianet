# 016-05 — Seqlock layout validation
**Epic:** 016-Real-time-setup-backend · **Effort:** S · **Depends on:** 016-01, 016-02

## Context
- The clock thread publishes the time view through a seqlock that data threads read. The writer must never be preempted mid-write by a higher-priority reader on the same core, or the reader spins forever `[decision]`.
- Rule (seqlock invariant): clock-thread priority ≥ every data-thread priority, **or** the clock thread and that data thread are pinned to different cores. A violating configuration is refused **in both modes** (best-effort and strict).
- Signature: `pub fn validate_layout(clock: &RtRequest, data: &[RtRequest]) -> Result<(), RtRefusal>;` where `RtRequest { fifo_priority: Option<u8>, cpu: Option<usize>, spins: bool, role: ThreadRole }`.
- The epic states how `None` values are treated, as consequences of the rule: a `None` priority means `SCHED_OTHER`, which ranks below every `SCHED_FIFO` priority `[kernel-doc]`; two threads count as pinned to different cores only when both have `Some(cpu)` with different values, so an unpinned thread may share a core with anything.
- `validate_layout` checks the request. A layout that passes only through different-core pinning depends on those pins; if one fails, the module refuses to start in every mode (C-177). That check runs on the grants, after `setup_thread`, and is ticket 016-16. The default priorities (clock above data, 016-06) never depend on pinning (C-177).

## Work
1. Implement `validate_layout` in `hadronyche-os-linux::rt` (pure, no syscalls) with the rule above, checked per data thread.
2. On violation return `RtRefusal` with kind `SeqlockLayout` and a reason naming the data-thread index, both priorities and both cores.
3. Doc comment stating the invariant and why.
4. Unit tests covering the table below.

## Acceptance criteria
1. Clock 70 on cpu 1, data 80 on cpu 1 → `Err(SeqlockLayout)` (epic AC 6).
2. Clock 70 on cpu 1, data 80 on cpu 2 → `Ok`.
3. Clock 70, data 70, same core → `Ok` (equal priority satisfies ≥).
4. Clock 70 unpinned, data 80 unpinned → `Err`.
5. Clock `None`, data 10 on the same core → `Err`; clock `None`, data `None` → `Ok`.
6. With 4 data threads where only index 2 violates, the error reason names index 2.
7. The function takes no `Mode` and returns the same result whatever mode the caller uses (documented; checked by the integration test in 016-14).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Default priority values (016-06).
- Refusing a layout whose relied-on pin failed (C-177, 016-16).
- Calling `validate_layout` from process start-up (callers: epic 015).
