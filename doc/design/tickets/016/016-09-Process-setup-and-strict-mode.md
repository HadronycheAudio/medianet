# 016-09 — Process setup and strict mode
**Epic:** 016-Real-time-setup-backend · **Effort:** S · **Depends on:** 016-04, 016-07, 016-08

## Context
- Signature: `pub fn setup_process(mode: Mode) -> Result<ProcessRtReport, RtRefusal>;` — once per process, before threads start.
- `ProcessRtReport { memory_locked: Grant, cpu_idle_restricted: Grant, kernel_release: String, preempt_model: String }`.
- Best-effort mode (default, kept from the POC `[decision]`): nothing is fatal; every degradation is an *absent* bit with a reason. Running unprivileged is what makes the runtime demonstrable and testable in CI.
- Strict mode, at process level, requires `memory_locked` only; restricting CPU idle states stays best-effort and is stamped (C-127). An *unverified* `memory_locked` is refused like an absent one (C-128). `setup_process` always attempts both, so neither process bit is ever *not requested*.
- Strict mode is hosted-only and set by operator policy in the commissioning configuration (C-147, C-165); runs whose numbers enter the constants table use it (C-147). This ticket only receives `Mode`.
- Epic AC 1: unprivileged best-effort start → privileged bits *absent* with a reason. Epic AC 2: unprivileged strict mode with a real-time request returns `RtRefusal` naming the missing capability before any thread is spawned; the exit code belongs to the caller.

## Work
1. Implement `setup_process` in `hadronyche-os-linux::rt`: call `lock_memory()` (016-07), `keep_cpus_awake()` (016-08), `kernel_release()` and `preempt_model()` (016-04); record every `Absent` in the reason collector (016-02).
2. Strict mode: a pure `fn strict_process_check(&ProcessRtReport) -> Result<(), RtRefusal>` returns `Err(RtRefusal { kind: MemoryLocked, reason })` when `memory_locked` is not `Present` (*absent* or *unverified*, C-127, C-128) and ignores `cpu_idle_restricted`. Do **not** release an obtained resource on refusal unless needed for a clean exit (the process is about to exit).
3. Guard: a second call returns the cached report without re-doing the work.
4. Document the strict-mode rule for process capabilities (C-127, C-128) in the doc comment of `setup_process`.

## Acceptance criteria
1. Unprivileged, `setup_process(BestEffort)` returns `Ok` with `memory_locked` and `cpu_idle_restricted` both `Absent` with non-empty reasons, and non-empty `kernel_release` and `preempt_model`.
2. Unprivileged, `setup_process(Strict)` returns `Err`, and the error's `Display` names `memory_locked`.
3. Calling `setup_process` twice performs `mlockall` and opens `/dev/cpu_dma_latency` at most once each (unit test with a call counter or fd count).
4. In the privileged job (016-15), `setup_process(Strict)` returns `Ok` with both bits `Present`, and returns `Ok` with `cpu_idle_restricted` *absent* when write access to `/dev/cpu_dma_latency` is withheld.
5. `strict_process_check` unit tests: `memory_locked` *present* with `cpu_idle_restricted` *absent* → `Ok`; `memory_locked` *unverified* → `Err(MemoryLocked)`; `memory_locked` *absent* → `Err(MemoryLocked)`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Thread-level setup (016-11).
- The process exit code and "before any thread starts" (caller, epic 015; tested in 016-14).
