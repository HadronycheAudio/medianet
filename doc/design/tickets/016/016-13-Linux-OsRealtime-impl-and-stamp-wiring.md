# 016-13 — Linux OsRealtime impl and stamp wiring
**Epic:** 016-Real-time-setup-backend · **Effort:** S · **Depends on:** 016-09, 016-11, 016-12, epic 013

## Context
- `hadronyche-std::rt::OsRealtime` (016-01) is the OS-neutral trait with `setup_process`, `setup_thread`, `measure_wake_latency`. The Linux functions (016-09, 016-11, 016-12) must implement it so that the hosted backend (epic 015) calls them through the trait, and macOS (021) and Windows (022) fit the same report.
- Every requested-versus-granted difference is recorded in the regime stamp (defined by epic 013) as a capability bit with four states: *present*, *absent*, *unverified*, *not requested* (C-129). A degraded run must never look like a good one `[measured]`.
- Capability bits this task owns: `rt_scheduling` (granted class and priority), `memory_locked`, `cpu_idle_restricted`, `cpu_pinned` (per thread), `rt_time_limit_lifted`; plain fields: kernel release, preemption model, measured wake latency (mean, 99.9th percentile, maximum).
- The stamp is carried by a dedicated control message split into sections that each fit 1 252 bytes of UDP payload (C-18, C-17). Field names, types, encoding, per-thread keying and section layout belong to epic 013; this ticket uses them as given.
- Epic AC 1: in unprivileged best-effort mode, the same reason text that appears in the report appears in the stamp. Contributes to AC-31 (every record set carries a complete stamp; missing privileges labelled). Epic AC 7: wake-latency values are in the stamp.

## Work
1. `pub struct LinuxRealtime;` in `hadronyche-os-linux::rt` implementing `OsRealtime` by delegating to the free functions.
2. Conversion functions into the epic 013 stamp types: `ProcessRtReport` → process bits and fields; `RtReport` + thread identity (role, index) → per-thread bits (`cpu_pinned` is per thread; apply the same per-thread keying to `rt_scheduling` and `rt_time_limit_lifted`); `rt_scheduling` includes granted class and priority (from `RtReport.granted_class` and `granted_priority`, 016-11); `WakeLatency` → mean / p99.9 / max fields.
3. Reason text is copied verbatim from `Grant::Absent { reason }` to the stamp. `Grant::NotRequested` maps to the stamp's *not requested* state, never to *absent* (C-129).
4. Document every stamp field touched (name, type, unit, four-state meaning) in the regime-stamp specification under `doc/design/spec/` (C-130).

## Acceptance criteria
1. A unit test converts a `ProcessRtReport` with both bits `Absent` and checks the stamp has both bits *absent* with byte-identical reasons.
2. A unit test converts an `RtReport` for 2 data threads and 1 clock thread and finds 3 distinct per-thread `cpu_pinned` entries, each `rt_scheduling` entry carrying its class and priority.
3. A `WakeLatency { mean: 10 µs, p999: 40 µs, max: 80 µs }` appears in the stamp with those three values and their units.
4. `LinuxRealtime` is usable as `&dyn OsRealtime` (compile test).
5. A housekeeping `RtReport` with all three bits `NotRequested` converts to three *not requested* stamp entries, none *absent* (unit test, C-129).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Defining the stamp format (epic 013).
- macOS / Windows implementations (tasks 021, 022).
- Calling the backend from process start-up (epic 015).
