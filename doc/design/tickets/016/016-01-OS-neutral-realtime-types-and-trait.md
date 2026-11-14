# 016-01 — OS-neutral real-time types and trait
**Epic:** 016-Real-time-setup-backend · **Effort:** S · **Depends on:** none

## Context
- Epic 016 provides the Linux real-time environment (scheduling class, CPU pinning, memory locking, CPU idle-state control) for the runtime's data and clock threads. macOS (task 021, time-constraint policy and workgroups) and Windows (task 022, MMCSS) must fit the same report, so the types and the trait live in the OS-neutral crate `hadronyche-std`.
- Each process runs one housekeeping thread, one clock thread and one data thread per module. Data and clock threads are real-time; housekeeping is not `[decision]`.
- Every requested-versus-granted difference is recorded as a capability bit with four states: *present*, *absent* (with a reason), *unverified*, *not requested*; *not requested* marks a deliberate configuration choice, as distinct from a denial (C-129).
- Two modes: best-effort (the POC behaviour: start unprivileged, say what could not be obtained) `[decision]` and strict, which refuses to start, with the reason named, unless real-time scheduling is granted on the data and clock threads and memory is locked; CPU idle restriction stays best-effort and is stamped (C-127). *Unverified* counts as absent in strict mode (C-128).
- The epic fixes these shapes ("names may be refined"). The types live in `hadronyche-std` because the trait needs them; `hadronyche-os-linux` re-exports them.

```rust
pub enum ThreadRole { Data, Clock, Housekeeping }
pub struct RtRequest {
    pub fifo_priority: Option<u8>,     // None = do not request real-time scheduling
    pub cpu: Option<usize>,
    pub spins: bool,                   // thread never blocks
    pub role: ThreadRole,
}
pub enum Grant { Present, Absent { reason: &'static str }, Unverified, NotRequested }
pub struct RtReport {
    pub rt_scheduling: Grant,
    pub granted_class: String,          // scheduling class read back after setup
    pub granted_priority: Option<i32>,  // priority read back; None when the class has none
    pub cpu_pinned: Grant,
    pub rt_time_limit_lifted: Grant,
}
pub struct ProcessRtReport {
    pub memory_locked: Grant, pub cpu_idle_restricted: Grant,
    pub kernel_release: String, pub preempt_model: String,
}
pub enum Mode { BestEffort, Strict }
pub struct RtRefusal { pub kind: RefusalKind, pub reason: String }
pub enum RefusalKind { RtScheduling, MemoryLocked, SeqlockLayout }
pub struct WakeLatency {
    pub mean: Duration, pub p999: Duration, pub max: Duration, pub samples: u64,
}

pub trait OsRealtime {
    fn setup_process(&self, mode: Mode) -> Result<ProcessRtReport, RtRefusal>;
    fn setup_thread(&self, req: &RtRequest, mode: Mode) -> Result<RtReport, RtRefusal>;
    fn measure_wake_latency(&self, d: core::time::Duration) -> WakeLatency;
}
```

- `granted_class` and `granted_priority` exist because the stamp's `rt_scheduling` bit records the granted class and priority (epic, capability bits). The class is a plain string so that macOS and Windows can report their own classes.
- `RefusalKind` has three variants: the two capabilities strict mode requires (C-127) and the seqlock rule, which applies in both modes. `cpu_pinned`, `rt_time_limit_lifted` and `cpu_idle_restricted` are stamped but never refused.

## Work
1. In crate `hadronyche-std`, module `rt.rs` (create the crate/module if it does not exist yet): define `ThreadRole`, `RtRequest`, `Grant`, `RtReport`, `ProcessRtReport`, `Mode`, and the trait `OsRealtime` exactly as above.
2. Define `RtRefusal` and `RefusalKind` as above. Implement `Display` for `RtRefusal` so the text names the capability in stamp form (`rt_scheduling`, `memory_locked`, `seqlock_layout`) followed by the reason.
3. Define `WakeLatency` as above.
4. Add a `Grant::reason(&self) -> Option<&str>` helper and doc comments for every type and field, stating the four-state meaning (C-129): *present* = requested and verified by read-back; *absent* = requested but not obtained (or not attempted), with reason; *unverified* = the call succeeded but could not be read back, treated as absent by strict mode (C-128); *not requested* = deliberately not asked for, never counted as a degradation. Document on `Mode::Strict` which capabilities it requires (C-127).
5. Unit tests for `Display` of `RtRefusal` and for `Grant::reason`.

## Acceptance criteria
1. `hadronyche-std` builds with `rt.rs` exporting the 10 public items listed in Work 1–3 (`ThreadRole`, `RtRequest`, `Grant`, `RtReport`, `ProcessRtReport`, `Mode`, `RtRefusal`, `RefusalKind`, `WakeLatency`, `OsRealtime`).
2. `RtRefusal`'s `Display` output contains the capability name (e.g. `rt_scheduling`) for each of the 3 `RefusalKind` variants; a unit test checks all 3.
3. `Grant::Absent { reason }` returns `Some(reason)` from `Grant::reason`; `Present`, `Unverified` and `NotRequested` return `None` (tested).
4. The module compiles with no platform-specific dependency (no `libc`, no `cfg(target_os)`).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] Public items documented, including the four-state meaning of `Grant` (C-129)

## Out of scope
- Any Linux implementation (016-02 onward).
- macOS and Windows implementations (tasks 021 and 022).
- Mapping to regime-stamp bits (016-13; stamp definitions are epic 013).
