# 016-14 — Unprivileged acceptance tests
**Epic:** 016-Real-time-setup-backend · **Effort:** S · **Depends on:** 016-05, 016-13

## Context
- Running unprivileged is what makes the runtime demonstrable and testable in CI; nothing is fatal by default (best-effort mode) `[decision]`. Strict mode refuses to start, with the reason named, unless real-time scheduling is granted on the data and clock threads and memory is locked (C-127); *unverified* counts as absent (C-128). The definition-of-done line "unprivileged" applies to per-change CI (C-146).
- This ticket turns epic acceptance criteria 1, 2, 4, 6 and 8 into end-to-end tests in the unprivileged CI job:
  - AC 1 Unprivileged start: best-effort starts, the privileged capability bits (`rt_scheduling`, `memory_locked`, `cpu_idle_restricted`) read *absent* with a reason, the same text appears in the stamp (contributes to AC-31). `cpu_pinned` and `rt_time_limit_lifted` need no privilege and may read *present*.
  - AC 2 Strict refusal: no privileges plus a real-time request → strict mode returns `RtRefusal` naming the missing capability before any thread is spawned; a start-up binary following the documented order exits non-zero with that text.
  - AC 4 Memlock guard: finite `RLIMIT_MEMLOCK` and no `CAP_IPC_LOCK` → `mlockall` not attempted, later allocations succeed.
  - AC 6 Seqlock layout: `validate_layout` rejects a data thread with higher priority than the clock thread pinned to the same core, in both modes.
  - AC 8 No Wi-Fi code in the crate.
- The library returns `Err(RtRefusal)`; the production exit path is the hosted backend's (epic 015), as epic AC 2 states. This ticket adds a small test binary that mirrors the documented start-up order: `validate_layout` → `setup_process` → `setup_thread` on the main thread → spawn threads.

## Work
1. Test binary `rt-probe` in `hadronyche-os-linux` (under `tests/` or `examples/`): flags `--strict`, `--fifo <p>`, `--cpu <n>`, `--spin` (busy loop for a given time, used by 016-15), `--duration <s>`; prints the stamp section; exits 0 on success, non-zero with the refusal text on `Err`; writes a marker when it would spawn a thread.
2. Integration tests (run as an ordinary CI user):
   - AC 1: `rt-probe --fifo 70` exits 0; `rt_scheduling`, `memory_locked`, `cpu_idle_restricted` are *absent* with reasons identical in report and stamp.
   - AC 2: `rt-probe --strict --fifo 70` exits non-zero, stderr names `memory_locked` (the first strict requirement checked, C-127), and the spawn marker is absent.
   - AC 4: run with `prlimit --memlock=65536` and assert *absent* with the "not attempted" reason, `VmLck: 0 kB`, and a 64 MiB allocation succeeds.
   - AC 6: `validate_layout` with clock 70 / data 80 on the same core fails when the probe is run with and without `--strict`.
   - AC 8: the Wi-Fi check from 016-02 runs in this job.

## Acceptance criteria
1. The 5 tests above exist and pass in the unprivileged CI job.
2. The AC 2 test asserts a non-zero exit code and that no thread was spawned (marker absent).
3. Each test's name references its epic AC number (e.g. `ac2_strict_refusal`).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Privileged criteria 3 and 5 (016-15).
- Production start-up code (epic 015).
