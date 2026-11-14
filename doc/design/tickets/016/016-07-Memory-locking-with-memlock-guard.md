# 016-07 — Memory locking with memlock guard
**Epic:** 016-Real-time-setup-backend · **Effort:** S · **Depends on:** 016-03

## Context
- Ported from POC `rt.rs:36` `lock_memory` `[code]`. `mlockall(MCL_CURRENT | MCL_FUTURE)` with a finite `RLIMIT_MEMLOCK` makes later allocations fail. Therefore attempt it **only** if the limit is unlimited or the process has `CAP_IPC_LOCK`.
- The result feeds the capability bit `memory_locked` (`Grant`: *present* / *absent* with reason / *unverified*) in `ProcessRtReport`.
- Verification (epic AC 3): read back `VmLck` from `/proc/self/status` (parser from 016-03): non-zero → *present*.
- Epic AC 4: with a finite `RLIMIT_MEMLOCK` and no `CAP_IPC_LOCK`, `mlockall` is not attempted and later allocations succeed.

## Work
1. `fn lock_memory() -> Grant` in `hadronyche-os-linux::rt`:
   - read `RLIMIT_MEMLOCK` via `getrlimit`; read `CapEff` via `read_self_status()` (016-03);
   - if limit finite and no `CAP_IPC_LOCK` → do not call `mlockall`; return `Absent { reason: "RLIMIT_MEMLOCK finite and no CAP_IPC_LOCK; mlockall not attempted" }`;
   - else call `mlockall(MCL_CURRENT | MCL_FUTURE)`; on error → `Absent` with the errno class in the reason;
   - on success read `VmLck`: > 0 → `Present`; unreadable → `Unverified`; 0 → `Absent { reason: "mlockall succeeded but VmLck is 0" }`.
2. Comment with the reason from the POC table (finite limit makes later allocations fail).
3. Factor the decision into a pure `fn should_lock(limit_infinite: bool, has_ipc_lock: bool) -> bool` for unit testing.

## Acceptance criteria
1. `should_lock` truth table: (false,false) → false; (true,false), (false,true), (true,true) → true (unit test).
2. In an unprivileged CI test process with `RLIMIT_MEMLOCK` set to 64 KiB, `lock_memory()` returns `Absent` with the reason above, `VmLck` stays 0, and a subsequent allocation and touch of 64 MiB succeeds (epic AC 4).
3. In the privileged job (016-15), `lock_memory()` returns `Present` and `VmLck` > 0 (epic AC 3).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Each ported POC behaviour carries a comment with its reason (table above)

## Out of scope
- Strict-mode refusal (016-09).
- The privileged CI job itself (016-15).
