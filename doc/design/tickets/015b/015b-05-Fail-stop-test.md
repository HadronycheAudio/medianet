# 015b-05 — Fail-stop test (AC-21)
**Epic:** 015b-Hosted-runtime-verification-and-strict-mode · **Effort:** S · **Depends on:** 015a-17, 015b-14
## Context
- AC-21 (015b-A3): killing a data or clock thread terminates the process; peers report interrupted within 4 ms and dead at 2 s; the producer stops after 2 s without reports.
- **L-7:** a panic in a data or clock thread terminates the process (fail-stop).
- Constants: interrupted after 8 consecutive slices absent at their deadline (C-38, C-102) = 4 ms at 0.5 ms slices; dead 2 s after the last slice with content, a marked gap or another timeline epoch (C-39, C-111, C-135); receiver report every 100 ms per bundle (C-40); producer stops after 2 s without reports (C-41).
- A test kills a thread with the panic-injection hook (C-145, extends C-120; built in 015b-14), stamped like any injection.
## Work
1. Two processes on loopback (producer P, consumer C). Inject a panic (015b-14) in P's data thread; separately in P's clock thread; separately in C's data thread.
2. Measure P/C exit, consumer liveness transitions, and producer stop time.
## Acceptance criteria
1. In each case, the panicking process exits (abort) within 100 ms.
2. When the producer dies, the consumer reports interrupted at the deadline of the 8th absent slice (≤ 4 ms after the first missed deadline) and dead 2 s ± 1 slice after the last received slice.
3. When the consumer dies, the producer stops sending 2 s ± 1 slice after the last report.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — none; "Design documents updated" — no behaviour differs; "Every new factual claim carries an evidence tag" — no new claim; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — 015b-10.
## Out of scope
- Housekeeping restart (015b-04).
