# 016-11 — Thread scheduling and pinning
**Epic:** 016-Real-time-setup-backend · **Effort:** M · **Depends on:** 016-03, 016-06, 016-10

## Context
- Signature: `pub fn setup_thread(req: &RtRequest, mode: Mode) -> Result<RtReport, RtRefusal>;` — on the calling thread.
- `RtRequest { fifo_priority: Option<u8> /* None = no RT request */, cpu: Option<usize>, spins: bool, role: ThreadRole }`; `RtReport { rt_scheduling: Grant, granted_class: String, granted_priority: Option<i32>, cpu_pinned: Grant, rt_time_limit_lifted: Grant }` (016-01).
- Ported from POC `rt.rs:106` `setup_thread` `[code]`: `SCHED_FIFO` at a configured priority, optional pinning to one core.
- Spin guard (POC `rt.rs:68`, 016-10): a spinning thread whose `RLIMIT_RTTIME` cannot be lifted must not get `SCHED_FIFO`, because the kernel kills a `SCHED_FIFO` thread that runs longer than `RLIMIT_RTTIME` without blocking.
- Housekeeping is not real-time and is supervised; it gets no `SCHED_FIFO` and no pinning by default `[decision]` (defaults in 016-06). An explicit request for a housekeeping thread is honoured like any other (epic).
- Real-time scheduling needs `CAP_SYS_NICE` or an `rtprio` limit (epic AC 3). Verification: read back with `sched_getscheduler` / `sched_getparam`, and `sched_getaffinity` for pinning.
- Best-effort: never fails for a missing capability. Strict: requires real-time scheduling granted on data and clock threads (C-127): `rt_scheduling` not `Present` on a `Data` or `Clock` thread → `Err(RtRefusal)` naming it (epic AC 2), and *unverified* counts as absent (C-128). A `Data` or `Clock` request with `fifo_priority: None` is refused in strict mode, because C-127 requires the grant. `cpu_pinned` and `rt_time_limit_lifted` are stamped, never refused by `setup_thread` (C-127); housekeeping is never refused. A failed pin that the layout's seqlock safety depends on refuses start in every mode (C-177), but that needs the whole layout and is checked across threads by `check_layout_pins` (016-16), not here.
- A capability not asked for is reported as `Grant::NotRequested` (C-129) and is not recorded as a degradation.

## Work
1. Implement `setup_thread` in `hadronyche-os-linux::rt`:
   - if `req.spins` → `lift_rt_time_limit()`; if not lifted, skip `SCHED_FIFO` and set `rt_scheduling = Absent { reason: "spinning thread and RLIMIT_RTTIME cannot be lifted" }`;
   - if `fifo_priority = Some(p)` → `sched_setscheduler(0, SCHED_FIFO, p)`; on `EPERM` → `Absent { reason: "no CAP_SYS_NICE and rtprio limit below priority" }`; on success read back policy and priority: both match → `Present`; read-back fails → `Unverified`;
   - if `cpu = Some(c)` → `sched_setaffinity` to `{c}`; read back; exactly `{c}` → `Present`; invalid cpu → `Absent` with reason;
   - a capability not asked for (`fifo_priority: None`, `cpu: None`, or `spins: false` for `rt_time_limit_lifted`) → `NotRequested` (C-129);
   - record every `Absent` in the reason collector (016-02).
2. Strict mode: for a `Data` or `Clock` thread, return `Err(RtRefusal { kind: RtScheduling, .. })` when `rt_scheduling` is not `Present` (C-127, C-128), with the reason "strict mode requires real-time scheduling on data and clock threads" when it was not requested. No other per-thread bit is refused.
3. Comment ported behaviour with its reason.
4. Fill `granted_class` (`SCHED_FIFO`, `SCHED_OTHER`, ...) and `granted_priority` from a `sched_getscheduler` / `sched_getparam` read-back after setup, whatever the outcome; the stamp's `rt_scheduling` bit records them (epic).

## Acceptance criteria
1. Unprivileged, `setup_thread(&{fifo 70, cpu None, spins false, Data}, BestEffort)` → `Ok` with `rt_scheduling = Absent` and a reason, `granted_class == "SCHED_OTHER"`; `sched_getscheduler` still returns `SCHED_OTHER`.
2. Unprivileged, the same request in `Strict` → `Err` whose `Display` contains `rt_scheduling`.
3. Unprivileged, `cpu Some(0)` → `cpu_pinned = Present` and `sched_getaffinity` returns exactly CPU 0.
4. A Housekeeping request built with `default_for(Housekeeping)` yields all three bits `NotRequested` and `Ok` in both modes (C-129).
5. A spinning request with an unliftable `RLIMIT_RTTIME` never calls `sched_setscheduler` (unit test with the pure decision helper).
6. In the privileged job (016-15), `fifo 70` → `Present` with `granted_class == "SCHED_FIFO"` and `granted_priority == Some(70)`, verified by `sched_getscheduler == SCHED_FIFO` and priority 70 (epic AC 3).
7. In `Strict`, a `Data` request with `fifo_priority: None` → `Err` naming `rt_scheduling`; in the privileged job (016-15), a `Strict` `fifo 70` request with an invalid `cpu` → `Ok` with `cpu_pinned` *absent* (C-127).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Each ported POC behaviour carries a comment with its reason (table above)

## Out of scope
- Layout validation across threads (016-05) and the failed-pin layout check (C-177, 016-16).
- Process-level setup (016-09).
- Spawning threads; callers (epic 015) call this on each new thread.
