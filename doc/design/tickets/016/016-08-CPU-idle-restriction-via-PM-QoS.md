# 016-08 — CPU idle restriction via PM QoS
**Epic:** 016-Real-time-setup-backend · **Effort:** S · **Depends on:** 016-02

## Context
- Ported from POC `rt.rs:51` `keep_cpus_awake` `[code]`. Writing a 0 latency request to `/dev/cpu_dma_latency` and keeping the file open for the process lifetime keeps cores out of slow idle states. Without it a core that sleeps between packets can take 0.1 ms or more to react.
- Feeds capability bit `cpu_idle_restricted` (`Grant`) in `ProcessRtReport`. It is best-effort in both modes: strict mode does not require it, and the result is only stamped (C-127).
- Verification (epic AC 3): via the open file descriptor — the descriptor is held and reading the device returns the active request.

## Work
1. `fn keep_cpus_awake() -> Grant` in `hadronyche-os-linux::rt`:
   - open `/dev/cpu_dma_latency` read-write; on failure → `Absent { reason }` naming the errno class (e.g. "no write access to /dev/cpu_dma_latency");
   - write a 32-bit 0; on failure → `Absent`;
   - store the `File` in a process-global (`OnceLock<File>`) so it stays open for the process lifetime; calling twice does not open a second descriptor;
   - read back the current value from the device: 0 → `Present`; unreadable → `Unverified`.
2. Comment with the reason above.
3. `fn cpu_idle_fd_held() -> bool` for tests.

## Acceptance criteria
1. In unprivileged CI (no write access), `keep_cpus_awake()` returns `Absent` with a reason containing `/dev/cpu_dma_latency`, and does not panic.
2. Calling `keep_cpus_awake()` twice yields the same `Grant` and at most 1 open descriptor to the device (checked by counting `/proc/self/fd` links in the privileged job).
3. In the privileged job (016-15), the result is `Present`, `cpu_idle_fd_held()` is true, and reading the device returns 0 (epic AC 3).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Each ported POC behaviour carries a comment with its reason (table above)

## Out of scope
- Strict-mode refusal: there is none for this bit (C-127); the strict check lives in 016-09.
- Per-core idle control or cpufreq tuning (not in the epic).
