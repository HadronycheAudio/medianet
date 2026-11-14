# 015a-17 — Runtime process assembly (process.rs)
**Epic:** 015a-Hosted-runtime-core · **Effort:** M · **Depends on:** 015a-08, 015a-10, 015a-13, 015a-14, 015a-16, 015a-18, 015a-19
## Context
- File: `hadronyche-std/src/process.rs`: `Runtime` owns the housekeeping thread, the clock thread, the file writer; registers modules.
- Thread model `[decision]`: one housekeeping thread and one clock thread per process; one data thread per module. The POC's three threads per module are retired.
- A file has one writer, owned by the process, never by a module (015a-08).
- Real-time thread setup is task 016, now a declared dependency of the epic. `Runtime` calls it through a hook whose default is a no-op, so the stub backend and the macOS and Windows backends compile without it; the Linux build binds the hook to task 016's implementation when it lands. The hook returns what it obtained; strict-mode enforcement on that result (C-127, C-128) is 015b-13. One check applies in every mode, strict or not, and is built here: if a thread layout's seqlock safety depends on a CPU pin and that pin fails, the runtime refuses to start (C-177). A pin that cannot be verified, including one the no-op default hook leaves unconfirmed, is treated as failed (C-201). Default priorities (clock above data) never depend on pinning (C-177).
- **L-7** fail-stop for data/clock threads, supervised restart for housekeeping (015a-16). **L-8** no shared locks.
- The C-120 test-only hooks (force module id and nonce, force a stream's timeline epoch, raw packet send) attach to `Runtime` in 015b-12.
## Work
1. `Runtime::new(config, os: impl OsNet, clock: impl HostClock)` starting the file service, clock thread, counter board and supervised housekeeping.
2. `Runtime::register_module(...)` spawning one data thread per module and wiring its lock-free queues to housekeeping, the counter board and the clock seqlock view.
3. Call the real-time setup hook for data and clock threads (no-op default until task 016 lands) and keep its per-thread result for 015b-13. If the result reports a failed or unverified pin on which the layout's seqlock safety depends (C-201: unverified counts as failed), stop the threads already started and return a named error: the runtime does not start, whatever the mode (C-177). A failed or unverified pin never changes the default priorities (clock above data).
4. Orderly shutdown: stop data threads, then clock and housekeeping, then flush files.
5. Example binary: one process, two modules, loopback.
## Acceptance criteria
1. A process with N registered modules has exactly N + 2 runtime threads plus the file writer thread (+ supervisor if separate), checked via `/proc/self/task`.
2. Two modules in one process exchange media over loopback for 60 s with zero missed deadlines.
3. Shutdown completes within 1 s and the metrics file is flushed.
4. The runtime is generic over `OsNet`/`HostClock` and compiles with the stub backend of 015a-01.
5. The real-time hook is called once per data and clock thread (test with a counting hook).
6. With a hook that reports a failed pin on which seqlock safety depends, or that cannot verify such a pin (C-201), the runtime refuses to start with the named error, in non-strict mode as in strict mode; with a hook that reports a failed pin on which seqlock safety does not depend, it starts with clock priority above data (C-177).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The trait surface compiles with a stub backend (proves macOS and Windows can implement it)
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — none added here; "Design documents updated" — no behaviour differs; "Every new factual claim carries an evidence tag" — no new claim; "Per-packet receive cost measured" — 015b-10.
## Out of scope
- Real-time priorities, memory locking and affinity (task 016); strict mode (015b-13).
- Reference modules and the L0 harness (epic 017).
