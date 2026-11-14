# 016 — Real-time setup backend

**Phase:** P4 · **Scope:** L (16 tickets, about 16 days) · **Depends on:** 013 · **Blocks:** 015, 017, 018, 019, 020, 021, 022

## Goal

Provide the Linux real-time environment for the runtime's data and clock threads: scheduling class, CPU pinning, memory locking, CPU idle-state control. Keep the POC's *best-effort* behaviour (the runtime starts unprivileged and says what it could not get), add a **strict mode** that refuses to start instead, and report every degradation through the regime stamp's capability bits.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules exchange *slices* addressed by *position* on a domain-wide timeline. Linux hosts are the backbone (clock holder, elections, conformance, commissioning); macOS and Windows machines are edges; embedded devices are followers. Each process runs one housekeeping thread, one clock thread, and one data thread per module. Data and clock threads are real-time; housekeeping is not.

**Why best-effort is kept `[decision]`.** Running unprivileged is what makes the runtime demonstrable and testable in CI. Nothing in real-time setup is fatal by default. But a degraded run must never look like a good one: the POC's first measured run showed that a run where real-time scheduling silently failed is indistinguishable from a good one `[measured]`. Therefore:

- every requested-versus-granted difference is recorded in the **regime stamp** as a capability bit with four states: *present*, *absent*, *unverified*, *not requested*; *not requested* marks a deliberate configuration choice, as distinct from a denial (C-129);
- **strict mode** refuses to start, with the reason named, unless real-time scheduling is granted on the data and clock threads and memory is locked (so a data or clock thread with no real-time request is refused in strict mode; pinning and the lifted real-time time limit are stamped, not required); restricting CPU idle states stays best-effort and is stamped (C-127). An *unverified* capability counts as absent (C-128). Strict mode is hosted-only (C-147), is set by operator policy in the commissioning configuration (C-165), and every run whose numbers enter the constants table runs in it; L0 does not (C-147);
- the **self-test** that precedes any reported number checks that real-time scheduling was actually granted and measures wake latency. Numbers produced before the self-test passes are labelled *unqualified*.

**What to port from the POC (`src/rt.rs`, 125 lines) `[code]`.** These are hard-won and must survive the rewrite with their reasons:

| POC item | Location | Reason to keep |
|---|---|---|
| Warnings collected once, deduplicated | `rt.rs:9` `warn`, `:16` `warnings` | Becomes the capability report |
| Memory locking only when the limit does not apply | `rt.rs:36` `lock_memory` | `mlockall(MCL_CURRENT \| MCL_FUTURE)` with a finite `RLIMIT_MEMLOCK` makes later allocations fail. Attempt only if the limit is unlimited or the process has `CAP_IPC_LOCK` |
| PM QoS request held open | `rt.rs:51` `keep_cpus_awake` | Writing a 0 latency request to `/dev/cpu_dma_latency` and keeping the file open for the process lifetime keeps cores out of slow idle states. Without it a core that sleeps between packets can take 0.1 ms or more to react |
| Real-time CPU time limit | `rt.rs:68` `rt_spin_allowed` | The kernel kills a `SCHED_FIFO` thread that runs longer than `RLIMIT_RTTIME` without blocking, which a spinning thread always does. Lift the limit, or do not give a spinning thread real-time scheduling |
| Thread setup | `rt.rs:106` `setup_thread` | `SCHED_FIFO` at a configured priority, optional pinning to one core |

**New constraints from the runtime design `[decision]`.**

- **Seqlock invariant.** The clock thread publishes the time view through a seqlock that data threads read. The writer must never be preempted mid-write by a higher-priority reader on the same core, or the reader spins forever. Enforce at setup: clock-thread priority ≥ every data-thread priority, **or** the two are pinned to different cores. Refuse a configuration that violates this (in any mode). Two consequences of the rule as written: a thread with no real-time request (`fifo_priority: None`) runs under `SCHED_OTHER`, which ranks below every `SCHED_FIFO` priority `[kernel-doc]`; and two threads count as "pinned to different cores" only when both have a `cpu` and the values differ — an unpinned thread may share a core with anything. A layout that passes only because two threads are pinned to different cores depends on those pins: if either pin fails, the module refuses to start in every mode, strict or not (C-177). The default priorities (clock above data) never depend on pinning (C-177).
- **Housekeeping is not real-time** and is supervised; it gets no `SCHED_FIFO` and no pinning by default. "By default" means the default `RtRequest` for the housekeeping role has no priority and no core; `setup_thread` honours an explicit request like any other.
- **E is a percentile on hosted platforms.** Without a real-time kernel, Linux scheduling latency has no hard bound; the edge's emission-error figure E (50 µs target for the Linux reference sink, C-70, pending measurement at L2) is the 99.9th percentile of absolute emission error over a one-hour certification run, with the maximum reported beside it (C-73). The stamp records the kernel's preemption model so a result can be interpreted.
- **Wi-Fi is out of scope.** Do not port `net::check_wifi_power_save` or any Wi-Fi branch.
- **Not in this task.** Wall-clock step detection lives in the platform backend that builds each clock exchange from receive stamps (C-121), not in real-time setup.

**Capability bits this task owns** (in the stamp defined by task 013): `rt_scheduling` (granted class and priority), `memory_locked`, `cpu_idle_restricted`, `cpu_pinned` (per thread), `rt_time_limit_lifted`, plus plain fields: kernel release, preemption model, measured wake latency (mean, 99.9th percentile, maximum). `rt_scheduling`, `cpu_pinned` and `rt_time_limit_lifted` are reported per thread. The stamp travels in a dedicated control message split into sections that each fit 1 252 bytes of UDP payload (C-18, C-17); field encoding and section layout are task 013's.

**Preemption model values.** The field records the kernel's own model name: `none`, `voluntary`, `full`, `rt`, `dynamic(<active mode>)` for a `PREEMPT_DYNAMIC` kernel (`dynamic(unknown)` when the active mode cannot be read), and `unknown` when nothing can be determined `[kernel-doc]`. The detection order is in ticket 016-04.

**Parsers and fuzzing.** This task parses `/proc/self/status` and the kernel version and preemption strings. Each parser is fuzzed 60 s on every change and 1 hour nightly, and its corpus is kept (C-95).

## Deliverables

Crate `hadronyche-std`, module `rt.rs`: the OS-neutral types and trait. The types live here, not in the Linux crate, because the trait needs them and the macOS backend (time-constraint policy and workgroups, task 021) and the Windows backend (MMCSS, task 022) must fit the same report.

```rust
pub enum ThreadRole { Data, Clock, Housekeeping }

/// What the caller asks for.
pub struct RtRequest {
    pub fifo_priority: Option<u8>,     // None = do not request real-time scheduling
    pub cpu: Option<usize>,
    pub spins: bool,                   // thread never blocks
    pub role: ThreadRole,
}

pub enum Grant { Present, Absent { reason: &'static str }, Unverified, NotRequested }

/// What was obtained for one thread. Feeds the regime stamp.
pub struct RtReport {
    pub rt_scheduling: Grant,
    pub granted_class: String,          // scheduling class read back after setup
    pub granted_priority: Option<i32>,  // priority read back; None when the class has none
    pub cpu_pinned: Grant,
    pub rt_time_limit_lifted: Grant,
}

pub struct ProcessRtReport {
    pub memory_locked: Grant,
    pub cpu_idle_restricted: Grant,
    pub kernel_release: String,
    pub preempt_model: String,
}

pub enum Mode { BestEffort, Strict }

/// Why setup refused: the capability missing or the rule violated, and a reason.
/// Strict mode refuses only for the capabilities C-127 requires; the seqlock rule applies in both modes.
pub struct RtRefusal { pub kind: RefusalKind, pub reason: String }
pub enum RefusalKind { RtScheduling, MemoryLocked, SeqlockLayout }

/// Wake-latency summary for the self-test and the stamp.
pub struct WakeLatency {
    pub mean: core::time::Duration,
    pub p999: core::time::Duration,
    pub max: core::time::Duration,
    pub samples: u64,
}

pub trait OsRealtime {
    fn setup_process(&self, mode: Mode) -> Result<ProcessRtReport, RtRefusal>;
    fn setup_thread(&self, req: &RtRequest, mode: Mode) -> Result<RtReport, RtRefusal>;
    fn measure_wake_latency(&self, d: core::time::Duration) -> WakeLatency;
}
```

Crate `hadronyche-os-linux`, module `rt.rs`: re-exports the types above and implements them for Linux. Either crate is created by this task if it does not yet exist.

```rust
/// Once per process, before threads start.
pub fn setup_process(mode: Mode) -> Result<ProcessRtReport, RtRefusal>;

/// On the calling thread.
pub fn setup_thread(req: &RtRequest, mode: Mode) -> Result<RtReport, RtRefusal>;

/// Rejects priority/pinning combinations that break the seqlock invariant.
pub fn validate_layout(clock: &RtRequest, data: &[RtRequest]) -> Result<(), RtRefusal>;

/// After `setup_thread` on the clock and data threads, refuses a layout whose
/// seqlock safety depends on a pin that failed, in every mode (C-177).
pub fn check_layout_pins(clock: (&RtRequest, &RtReport), data: &[(&RtRequest, &RtReport)]) -> Result<(), RtRefusal>;

/// Samples wake latency for `duration`; part of the self-test.
pub fn measure_wake_latency(duration: core::time::Duration) -> WakeLatency;

/// Implements `OsRealtime` by delegating to the functions above.
pub struct LinuxRealtime;
```

(Shapes; names may be refined.)

## Dependencies

- **Needs:** the capability-bit and field definitions of the regime stamp (013).
- **Provides:** thread and process setup to the hosted backend (015, which calls it), the reference modules and L0 harness (017), the ALSA edge (018), and the wake-latency figure used by L1 and L2 (019, 020). Tasks 021 and 022 implement `OsRealtime` for their platforms.

## Acceptance criteria

1. **Unprivileged start.** With no privileges, best-effort mode starts, every privileged capability bit (`rt_scheduling`, `memory_locked`, `cpu_idle_restricted`) reads *absent* with a reason, and the same text appears in the stamp. Bits that need no privilege (`cpu_pinned`, and `rt_time_limit_lifted` when the hard limit allows it) may read *present*. Contributes to AC-31 (every record set carries a complete stamp; missing privileges labelled).
2. **Strict refusal.** With no privileges and a real-time request, strict mode returns `RtRefusal` naming the missing capability (`memory_locked` or `rt_scheduling`, the two strict mode requires, C-127) before any thread is spawned; a start-up binary that follows the documented order (`validate_layout`, `setup_process`, `setup_thread` on the main thread, then spawn) exits non-zero with that text. The production exit path belongs to the hosted backend (015).
3. **Privileged grant.** With `CAP_SYS_NICE` (or an `rtprio` limit), `CAP_IPC_LOCK` and write access to `/dev/cpu_dma_latency`, all bits read *present*, verified by reading back `sched_getscheduler`, `/proc/self/status` (`VmLck`) and the open file descriptor.
4. **Memlock guard.** With a finite `RLIMIT_MEMLOCK` and no `CAP_IPC_LOCK`, `mlockall` is not attempted and later allocations succeed.
5. **Spin guard.** A spinning thread with a finite, unliftable `RLIMIT_RTTIME` runs without real-time scheduling (bit *absent*, reason named) and is not killed by the kernel during a 60 s run.
6. **Seqlock layout.** `validate_layout` rejects a data thread with higher priority than the clock thread pinned to the same core, in both modes. A layout that passes only through different-core pinning, where one of those pins reads *absent* or *unverified*, is refused by `check_layout_pins` in both modes (C-177, C-201).
7. **Self-test hook.** `measure_wake_latency` over 1 s returns mean, 99.9th percentile and maximum; the values are in the stamp. Contributes to AC-8 (trustworthy loopback run).
8. **No Wi-Fi code** exists in the crate.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Each ported POC behaviour carries a comment with its reason (table above)
- [ ] A privileged test job exists (may be manual) for criteria 3 and 5

**Not applicable in this epic (C-125):** none. Every line has a subject here: parsers (`/proc/self/status`, kernel version and preemption strings), stamp fields (the capability bits and plain fields above), and ported POC behaviour (the table above).

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Should a real-time kernel (`PREEMPT_RT`) be a recommended or required configuration for the Linux reference sink's 50 µs E target (C-70; the target, not the measured E, C-149)? | Task 020 (L2 Linux) | The target is met on a stock kernel over a one-hour run |
| Default priorities for clock and data threads (the POC used 70 for both) `[code]`. Proposal: data 70, clock 71, so the seqlock invariant holds even when unpinned | This task (ticket 016-06) | The seqlock invariant or L2 results require them to differ |
| Wake-latency probe method and period. Proposal: `clock_nanosleep` absolute on `CLOCK_MONOTONIC` every 100 µs, about 10 000 samples in 1 s | This task (ticket 016-12), confirmed at L1 (task 019) | Latency measured at 100 µs differs materially from latency measured at the slice period (0.5 ms, C-02), or a 1 s run yields fewer than 1 000 samples |

Settled since the last refinement: the process capabilities strict mode requires (memory locked; CPU idle restriction stays best-effort and stamped, C-127), strict mode and *unverified* (refused, C-128), the representation of a capability that was not requested (a fourth state, `Grant::NotRequested`, C-129), and a failed pin that a layout's seqlock safety depends on (the module refuses to start in every mode, strict or not; default priorities never depend on pinning, C-177; ticket 016-16), and an *unverified* pin that a layout's seqlock safety depends on (treated as failed for C-177: the module refuses to start in every mode, C-201; ticket 016-16).
