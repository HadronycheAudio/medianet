# 016-03 — /proc/self/status parser
**Epic:** 016-Real-time-setup-backend · **Effort:** S · **Depends on:** 016-02

## Context
- Real-time setup must verify what it obtained by reading back kernel state. Epic AC 3 requires memory locking to be verified through `/proc/self/status` (`VmLck`).
- Memory locking may be attempted only if `RLIMIT_MEMLOCK` is unlimited **or** the process has `CAP_IPC_LOCK` (POC `rt.rs:36`) `[code]`. Real-time scheduling needs `CAP_SYS_NICE` or an `rtprio` limit (epic AC 3). Effective capabilities are visible in the `CapEff` line of `/proc/self/status` as a hexadecimal mask; `CAP_IPC_LOCK` is bit 14 and `CAP_SYS_NICE` is bit 23 `[kernel-doc]`.
- Definition of done of the epic: every parser touched is fuzzed. Fuzz budget (C-95): each parser 60 s on every change and 1 hour nightly; the corpus is kept.

## Work
1. In `hadronyche-os-linux`, module `rt/procfs.rs` (or inside `rt.rs`): `fn parse_status(text: &str) -> ProcStatus` with `ProcStatus { vm_lck_kib: Option<u64>, cap_eff: Option<u64> }`.
2. Helpers `fn has_cap_ipc_lock(&ProcStatus) -> bool`, `fn has_cap_sys_nice(&ProcStatus) -> bool`, and `fn read_self_status() -> io::Result<ProcStatus>`.
3. The parser never panics; malformed lines are ignored and yield `None` for that field.
4. A `cargo fuzz` target `proc_status` over `parse_status`, run 60 s on every change and 1 hour in the nightly job (C-95); commit its corpus to the repository and reuse it on every run.

## Acceptance criteria
1. On the line `VmLck:	    1024 kB` the parser returns `vm_lck_kib == Some(1024)`; on `VmLck:	       0 kB` it returns `Some(0)` (unit tests).
2. On `CapEff:	0000000000004000` `has_cap_ipc_lock` is true and `has_cap_sys_nice` is false; on `CapEff:	0000000000800000` the reverse (unit tests).
3. Missing or malformed `VmLck`/`CapEff` lines yield `None` and no panic (unit tests with at least 5 malformed inputs).
4. The fuzz target runs 60 s on every change and 1 hour nightly without a crash (C-95), starting from the kept corpus, and new corpus entries are kept.
5. `read_self_status()` succeeds in the unprivileged CI job and returns `Some` for both fields.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Deciding whether to lock memory (016-07).
- Scheduling read-back via `sched_getscheduler` (016-11).
