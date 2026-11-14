# 015a-16 — Housekeeping supervisor (supervisor.rs)
**Epic:** 015a-Hosted-runtime-core · **Effort:** S · **Depends on:** 015a-15
## Context
- File: `hadronyche-std/src/supervisor.rs`: restarts housekeeping after a panic; sets the control-degraded mark.
- **L-7:** a panic in a data or clock thread terminates the process (fail-stop). A panic in housekeeping does **not**: housekeeping is supervised and restarted, live streams continue, and the module is marked *control-degraded*.
- **L-8:** no lock shared between housekeeping and data/clock threads; the restart must not require data or clock threads to release anything.
- Restart policy (C-139): exponential backoff from 100 ms to 10 s; after 5 panics within 60 s restarts stop. The module then stays control-degraded with an alarm, its live streams continue, and it accepts no new connection until an operator restarts it. A connection alarm is raised on control-degraded (C-173).
- Refusal while control-degraded (C-185): once restarts have stopped, an admission request is answered with the refusal reason ControlDegraded by the housekeeping supervisor, which keeps a minimal refusal path; that path is 015a-22. This ticket stops restarts and signals 015a-22 to take over.
- The housekeeping-restart criterion AC-20 (c) belongs to the hosted backend (C-144).
- Panic injection into a data, clock or housekeeping thread is a test-only hook (C-145, extends C-120); the hook itself is 015b-14. Unit tests here use a panic inside the supervised closure.
- Still open: how peers see the control-degraded mark (a stamp field or a hello flag) is owned by the wire specification (epic 004). This ticket keeps the mark as a lock-free flag and publishes it to the counter board (015a-18).
## Work
1. Run housekeeping under `catch_unwind` (or a supervising thread joining it); on panic, rebuild its state from published data and restart it.
2. Set the control-degraded mark for every module in the process (lock-free flag), and publish it to the counter board.
3. Install a panic policy so that a panic in a data or clock thread aborts the process (e.g. per-thread hook calling `std::process::abort`).
4. Count restarts and implement the C-139 policy: backoff doubling from 100 ms up to 10 s; at the 5th panic within a 60 s window, stop restarting, keep the control-degraded mark, raise the alarm through the counter board, and leave data and clock threads running. With housekeeping stopped no admission runs, so no new connection is accepted until an operator restarts the process; hand over to the minimal refusal path (015a-22, C-185).
## Acceptance criteria
1. A panic in housekeeping (test build): a new housekeeping thread runs after the 100 ms initial backoff (within one loop period of it), the restart counter is 1, and the control-degraded mark is set.
2. Successive panics are restarted after backoffs of 100, 200, 400 ms … capped at 10 s (C-139).
3. A 5th panic within 60 s stops restarts: no new housekeeping thread, the mark stays set, the alarm is published, live streams continue, no new connection is admitted, and the minimal refusal path (015a-22) is started.
4. A panic in a data thread or the clock thread terminates the process (abort) in a subprocess test.
5. Data threads keep sending during the housekeeping restart (no missed slice in a loopback run).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — the mark's field is not yet specified (epic 004); "Design documents updated" — no behaviour differs; "Every new factual claim carries an evidence tag" — no new claim; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — no receive path.
## Out of scope
- The panic-injection hook (015b-14).
- Full AC-20(c) and AC-21 scenarios (015b-04, 015b-05).
- The ControlDegraded refusal path itself (015a-22).
