# 016-10 — RT time limit and spin guard
**Epic:** 016-Real-time-setup-backend · **Effort:** S · **Depends on:** 016-02

## Context
- Ported from POC `rt.rs:68` `rt_spin_allowed` `[code]`. The kernel kills a `SCHED_FIFO` thread that runs longer than `RLIMIT_RTTIME` without blocking, which a spinning thread always does. Lift the limit, or do not give a spinning thread real-time scheduling.
- `RtRequest.spins: bool` marks a thread that never blocks. Capability bit `rt_time_limit_lifted` (`Grant`) in `RtReport`.
- Epic AC 5: a spinning thread with a finite, unliftable `RLIMIT_RTTIME` runs without real-time scheduling (bit *absent*, reason named) and is not killed during a 60 s run.

## Work
1. `fn lift_rt_time_limit() -> Grant` in `hadronyche-os-linux::rt`: if `RLIMIT_RTTIME` soft is already `RLIM_INFINITY` → `Present`; else try `setrlimit` to infinity (soft and hard); verify with `getrlimit`: infinite → `Present`, else `Absent { reason: "RLIMIT_RTTIME finite and cannot be lifted" }`.
2. `fn rt_spin_allowed(spins: bool) -> (bool, Grant)`: non-spinning threads are always allowed (the limit does not threaten a thread that blocks; the limit is then not touched and the bit is reported as `Grant::NotRequested`, C-129); spinning threads are allowed only if the limit is lifted.
3. Comment with the reason above.
4. Pure helper `fn spin_decision(spins: bool, limit_infinite: bool) -> bool` for unit tests.

## Acceptance criteria
1. `spin_decision` truth table: (false,_) → true; (true,true) → true; (true,false) → false (unit test).
2. In an unprivileged test that first sets `RLIMIT_RTTIME` hard limit to 1 s, `lift_rt_time_limit()` returns `Absent` with a reason containing `RLIMIT_RTTIME`.
3. With hard limit infinite and soft finite, unprivileged `lift_rt_time_limit()` returns `Present` (raising soft to hard needs no privilege).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Each ported POC behaviour carries a comment with its reason (table above)

## Out of scope
- Applying the decision to `SCHED_FIFO` (016-11).
- The 60 s privileged run of epic AC 5 (016-15).
