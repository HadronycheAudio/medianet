# Epic 016 — Real-time setup backend: tickets

## Summary
Epic 016 (phase P4, scope L, depends on 013, blocks 015, 017–022) provides the Linux real-time environment for the runtime's data and clock threads: `SCHED_FIFO` scheduling, CPU pinning, memory locking (`mlockall`, guarded by `RLIMIT_MEMLOCK` / `CAP_IPC_LOCK`), CPU idle-state control (held `/dev/cpu_dma_latency` request), and the `RLIMIT_RTTIME` spin guard, ported from the POC's `src/rt.rs` with their reasons. It keeps the POC's best-effort start, adds a strict mode that refuses to start unless real-time scheduling is granted on data and clock threads and memory is locked (C-127, C-128), enforces the seqlock layout invariant (clock priority ≥ data priority or different cores) in both modes, refuses in both modes a layout whose relied-on pin failed (C-177), measures wake latency for the self-test, and reports every degradation as a four-state capability bit (present, absent, unverified, not requested; C-129) in the regime stamp. Code goes into `hadronyche-os-linux::rt` behind the OS-neutral `OsRealtime` trait in `hadronyche-std::rt`. Wi-Fi code is out of scope.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 016-01 | OS-neutral real-time types and trait | S | none |
| 016-02 | Linux crate skeleton and reason collector | XS | 016-01 |
| 016-03 | /proc/self/status parser | S | 016-02 |
| 016-04 | Kernel release and preemption model | S | 016-02 |
| 016-05 | Seqlock layout validation | S | 016-01, 016-02 |
| 016-06 | Default thread priorities | XS | 016-05 |
| 016-07 | Memory locking with memlock guard | S | 016-03 |
| 016-08 | CPU idle restriction via PM QoS | S | 016-02 |
| 016-09 | Process setup and strict mode | S | 016-04, 016-07, 016-08 |
| 016-10 | RT time limit and spin guard | S | 016-02 |
| 016-11 | Thread scheduling and pinning | M | 016-03, 016-06, 016-10 |
| 016-12 | Wake latency measurement | S | 016-01, 016-02 |
| 016-13 | Linux OsRealtime impl and stamp wiring | S | 016-09, 016-11, 016-12, epic 013 |
| 016-14 | Unprivileged acceptance tests | S | 016-05, 016-13 |
| 016-15 | Privileged test job | S | 016-14 |
| 016-16 | Failed-pin layout check | S | 016-05, 016-11 |

## Effort tally
- XS: 2
- S: 13
- M: 1
- Total: 16 tickets, about 16 days (XS 0.25 day, S 1 day, M 2.5 days); epic scope L (C-126; over 15 days)

No ticket was added or deleted after the design rulings up to C-176: those settled points changed the content of existing tickets only. C-177 added 016-16 (the failed-pin layout check), which moved the scope from M to L (see "Settled since the split"). The epic never had rational-L content, so the withdrawal of that rule (C-104, C-105, C-134) removes nothing here.

## Coverage

### Deliverables

| Epic deliverable | Tickets |
|---|---|
| `hadronyche-std::rt`: `OsRealtime` trait and shared types (`RtRequest`, `ThreadRole`, `Grant`, `RtReport` with granted class and priority, `ProcessRtReport`, `Mode`, `RtRefusal`, `RefusalKind`, `WakeLatency`) | 016-01 |
| Crate `hadronyche-os-linux`, module `rt.rs`; warnings collected once, deduplicated (POC `rt.rs:9`, `:16`) | 016-02 |
| `setup_process(mode)` | 016-09 (with 016-03, 016-04, 016-07, 016-08) |
| Memory locking guard (POC `rt.rs:36`) | 016-03, 016-07 |
| PM QoS request held open (POC `rt.rs:51`) | 016-08 |
| Kernel release and preemption model fields (value set fixed in the epic) | 016-04 |
| `setup_thread(req, mode)` (POC `rt.rs:106`) | 016-11 |
| RT CPU time limit / spin guard (POC `rt.rs:68`) | 016-10, 016-11 |
| `validate_layout(clock, data)` (seqlock invariant) | 016-05 |
| `check_layout_pins(clock, data)`: a failed pin that the layout relied on refuses start in every mode (C-177) | 016-16 |
| `measure_wake_latency(duration)` | 016-12 |
| Housekeeping not real-time by default | 016-06, 016-11 |
| Linux implementation of `OsRealtime` (`LinuxRealtime`); capability bits and fields in the regime stamp | 016-13 |
| Granted class and priority in `RtReport` | 016-01, 016-11, 016-13 |
| No Wi-Fi port | 016-02, 016-14 |
| Open sub-question: default clock/data priorities (owner: this task) | 016-06 |
| Open sub-question: `PREEMPT_RT` recommended/required (owner: task 020) | note in 016-04 |
| Strict mode requires RT scheduling on data and clock threads and memory locked; CPU idle restriction best-effort and stamped (C-127) | 016-01, 016-08, 016-09, 016-11, 016-14, 016-15 |
| Strict mode refuses *unverified* (C-128) | 016-01, 016-09, 016-11 |
| Fourth capability state *not requested* (C-129) | 016-01, 016-10, 016-11, 016-13 |
| Privileged tests in a separate nightly job (C-146) | 016-14, 016-15 |
| An *unverified* pin counts as failed under C-177, in every mode (C-201) | 016-16 |
| Open sub-question: wake-latency probe method and period (owner: this task, confirmed at L1) | 016-12 |

### Acceptance criteria

| Epic AC | Tickets |
|---|---|
| 1 Unprivileged start | 016-07, 016-08, 016-09, 016-11, 016-13, 016-14 |
| 2 Strict refusal | 016-09, 016-11, 016-14 |
| 3 Privileged grant | 016-07, 016-08, 016-11, 016-15 |
| 4 Memlock guard | 016-07, 016-14 |
| 5 Spin guard | 016-10, 016-11, 016-15 |
| 6 Seqlock layout | 016-05, 016-14, 016-16 |
| 7 Self-test hook | 016-12, 016-13 |
| 8 No Wi-Fi code | 016-02, 016-14 |

### Definition of done

| Epic DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all except 016-15 (per-change CI; privileged tests run nightly, C-146) |
| Every parser touched is fuzzed (60 s per change, 1 hour nightly, corpus kept: C-95) | 016-03, 016-04 |
| Regime stamp fields touched are complete and documented | 016-04, 016-13 |
| Design documents updated where behaviour differs, constants table first | 016-01, 016-04, 016-05, 016-06, 016-09, 016-12, 016-13, 016-16 (locations per C-130) |
| No new open item without an owner and a falsifier | 016-01, 016-06, 016-09, 016-12, 016-16 |
| The frozen POC is not modified | 016-02 |
| Every new factual claim carries an evidence tag | all |
| Each ported POC behaviour carries a comment with its reason | 016-02, 016-07, 016-08, 016-10, 016-11 |
| A privileged test job exists (may be manual) for criteria 3 and 5 | 016-15 |

No checklist line is not applicable in this epic (C-125): every line has a subject.

## Remaining gaps

None of these is answered by the epic or the constants table. R1, R2, R3, R8 and R10 were settled (see below).

- **R4** Regime-stamp field names, types, encoding, per-thread keying, and how per-thread entries for many data threads split into sections of at most 1 252 bytes (C-18, C-17). Affects 016-13. Answer: the regime stamp epic 013.
- **R5** Default clock and data priorities (proposal data 70, clock 71). Affects 016-06. Answer: this epic proposes; L2 measurement (task 020) is the falsifier.
- **R6** Wake-latency probe method and period (proposal `clock_nanosleep` absolute, 100 µs). Affects 016-12. Answer: this epic proposes; L1 measurement (task 019) confirms.
- **R7** Whether `PREEMPT_RT` is recommended or required for the 50 µs E target (C-70; the target, not the measured E, C-149). Affects 016-04 (reporting only). Answer: L2 Linux measurement (task 020).
- **R9** Which component sets the control-isolation capability (C-161), present on hosted platforms by construction of the thread model; it is not among the bits this epic owns. Affects none of 016's tickets. Answer: epic 015 (thread model) with epic 013 (stamp).

## Settled since the split

- **G1** `RtRefusal` and `WakeLatency` undefined: defined in the epic's Deliverables (`RtRefusal { kind: RefusalKind, reason }`, `WakeLatency { mean, p999, max, samples }`). Epic change.
- **G2** Shared types placed in the wrong crate, crate existence unknown: the epic puts the types in `hadronyche-std::rt`, re-exported by `hadronyche-os-linux`, and says either crate is created if absent. Epic change.
- **G4** AC 2 required a library to exit the process: AC 2 reworded (refusal before any thread is spawned; a start-up binary exits non-zero; production exit belongs to 015). Epic change.
- **G5** (AC 1 part) AC 1 claimed every bit reads *absent* unprivileged: AC 1 now names the privileged bits and allows `cpu_pinned` and `rt_time_limit_lifted` to read *present*. Epic change. The "not requested" representation is settled as R3 (C-129).
- **G6** `None` priority and unpinned threads in `validate_layout`: stated in the epic as consequences of the invariant (`None` is `SCHED_OTHER`, below every FIFO priority; unpinned may share a core). Epic change.
- **G7** Preemption model detection and value set: the epic fixes the value set from the kernel's model names; 016-04 owns the detection order. Epic change.
- **G8** (method and period undefined): recorded in the epic as an open sub-question with owner and falsifier; continues as R6.
- **G9** `rt_scheduling` must carry class and priority but `Grant` could not: `RtReport` gains `granted_class` and `granted_priority`. Epic change.
- **G11** Housekeeping override mechanism: the epic defines "by default" as the role's default `RtRequest`; an explicit request is honoured. Epic change.
- **G13** Scope marked S against 15 tickets: recomputed from the tally as M (15 tickets, about 15 days). C-126.
- **G3**, **G10**, **G12**: G3 and G10 settled as R1 and R2 (next entries); G12 continues as R4.
- **R1** Process capabilities strict mode requires: memory locked only; CPU idle restriction stays best-effort and is stamped (C-127). At thread level, real-time scheduling granted on data and clock threads is required; pinning and the lifted RT time limit are stamped, never refused. `RefusalKind` shrinks to `RtScheduling`, `MemoryLocked`, `SeqlockLayout`. Epic and 016-01, 016-08, 016-09, 016-11, 016-14, 016-15 changed.
- **R2** Strict mode and *unverified*: refused, treated as absent (C-128); the working assumption stands, no longer provisional. 016-01, 016-09, 016-11 changed.
- **R3** "Not requested": a fourth state, `Grant::NotRequested`, mapped to the stamp's *not requested* state and never counted as a degradation (C-129). Replaces the `Absent { reason: "not requested" }` assumption. Epic and 016-01, 016-10, 016-11, 016-13 changed.
- **R8** A failed pin that a layout's seqlock safety depends on: the module refuses to start in every mode, strict or not; default priorities (clock above data) never depend on pinning (C-177). Since `setup_thread` sees one thread only and `validate_layout` checks the request, the check over grants is a new ticket, 016-16 (`check_layout_pins`, refusal kind `SeqlockLayout`); epic Deliverables, AC 6, seqlock background and open sub-questions updated; 016-05 and 016-11 point to it. Scope recomputed M → L (16 tickets, about 16 days, C-126).
- **R10** Whether an *unverified* pin (affinity set but not read back) counts as a failed pin under C-177: yes, treated as failed; the module refuses to start in every mode (C-201). 016-16's provisional reading stands, no longer provisional; an acceptance criterion for an *unverified* pin added. Epic open sub-questions and 016-16 changed. No ticket added; tally unchanged.
- Also applied in the last round: privileged tests run in a separate nightly job (C-146, 016-14, 016-15); strict mode is hosted-only, set by operator policy in the commissioning configuration, and used by every run whose numbers enter the constants table (C-147, C-165; epic, 016-09, 016-12); output locations of the constants table and the stamp specification (C-130; 016-04, 016-06, 016-12, 016-13). C-144's duties belong to epic 015 and add nothing here. No other rule from C-127 to C-176 bears on this epic.
- Sweep of C-177 to C-212 (and the late revisions of C-08, C-134, C-141, C-153, C-169, C-181, C-184): epic AC 6 now names an *unverified* pin beside an *absent* one (C-201; 016-16 already tested it). No other rule in that range has a subject here: C-188 places sync answers on the clock thread, which this epic already sets up as real-time; C-206 leaves `rt-probe`'s output format (016-14) to this epic, which already decides it; the rest concern other epics. No ticket added or deleted; tally unchanged.
- Also applied: fuzz budget for 016-03 and 016-04 (C-95); E definition with the maximum reported beside the percentile (C-73); stamp transport bound (C-18, C-17) in 016-13; wall-clock step detection placed outside this epic (C-121); missing blocked tasks 015, 021, 022 added to the epic header; definition-of-done applicability checked (C-125, none not applicable).
