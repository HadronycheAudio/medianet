# 016-15 — Privileged test job
**Epic:** 016-Real-time-setup-backend · **Effort:** S · **Depends on:** 016-14

## Context
- Epic definition of done: a privileged test job exists (may be manual) for criteria 3 and 5. Tests that need a privilege run in a separate privileged nightly job; the "unprivileged" line applies to per-change CI (C-146).
- Epic AC 3 Privileged grant: with `CAP_SYS_NICE` (or an `rtprio` limit), `CAP_IPC_LOCK` and write access to `/dev/cpu_dma_latency`, all bits read *present*, verified by reading back `sched_getscheduler`, `/proc/self/status` (`VmLck`) and the open file descriptor.
- Epic AC 5 Spin guard: a spinning thread with a finite, unliftable `RLIMIT_RTTIME` runs without real-time scheduling (bit *absent*, reason named) and is not killed by the kernel during a 60 s run. Reason: the kernel kills a `SCHED_FIFO` thread that runs longer than `RLIMIT_RTTIME` without blocking (POC `rt.rs:68`) `[code]`.
- Uses the `rt-probe` binary from 016-14.

## Work
1. A privileged nightly CI job (C-146; a manual trigger is also allowed) on a Linux runner where the probe runs with `CAP_SYS_NICE`, `CAP_IPC_LOCK` and write access to `/dev/cpu_dma_latency` (e.g. `setcap` on the binary or a container with those capabilities and the device).
2. AC 3 test: `rt-probe --strict --fifo 70 --cpu 0` exits 0; all bits *present*; the test reads back `sched_getscheduler == SCHED_FIFO` priority 70, `VmLck` > 0 in `/proc/<pid>/status`, an open fd to `/dev/cpu_dma_latency` in `/proc/<pid>/fd`.
3. AC 5 test: with `CAP_SYS_NICE` but `RLIMIT_RTTIME` hard limit set to 1 s and `CAP_SYS_RESOURCE` dropped, a spinning probe (`--spin --fifo 70`, best-effort) runs 60 s; assert `rt_scheduling` *absent* with a reason naming `RLIMIT_RTTIME`, policy `SCHED_OTHER`, and exit status 0 (not killed by `SIGXCPU`/`SIGKILL`).
4. Strict-mode scope test (C-127): with `CAP_SYS_NICE` and `CAP_IPC_LOCK` but no write access to `/dev/cpu_dma_latency`, `rt-probe --strict --fifo 70` exits 0 with `cpu_idle_restricted` *absent* and a reason.
5. Document how to run the job in the crate README.

## Acceptance criteria
1. The job exists, runs nightly, and its run log shows AC 3 passing with the three read-backs.
2. The AC 5 run lasts ≥ 60 s and the process ends with exit status 0.
3. The job's run instructions are documented and reference epic ACs 3 and 5.
4. The strict-mode scope test of Work 4 passes.

## Definition of done
- [ ] A privileged test job exists (may be manual) for criteria 3 and 5
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Unprivileged tests (016-14).
- Latency targets and L1/L2 runs (tasks 019, 020).
