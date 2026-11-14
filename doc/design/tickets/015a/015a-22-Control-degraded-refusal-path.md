# 015a-22 — Control-degraded refusal path (C-185)
**Epic:** 015a-Hosted-runtime-core · **Effort:** S · **Depends on:** 015a-15, 015a-16, epic 012
## Context
- File: `hadronyche-std/src/supervisor.rs`.
- C-139: after 5 panics within 60 s housekeeping restarts stop; the module stays control-degraded with an alarm, its live streams continue, and it accepts no new connection until an operator restarts it. A connection alarm is raised on control-degraded (C-173).
- C-185: the refusal reasons number 18; ControlDegraded is the refusal for a target whose housekeeping has stopped restarting (C-139). It is answered by the housekeeping supervisor, which keeps a minimal refusal path. The refusal encoding and the admission messages are core's (epic 012); this ticket only receives and answers.
- Control messages use DSCP CS5 (40, C-13) and fit 1 252 bytes of UDP payload (C-17, C-100).
- **L-7, L-8:** live streams continue; the refusal path shares no lock with the data or clock threads and touches no live data path (L-4).
## Work
1. Keep the control socket on which admission requests arrive reachable by the supervisor once housekeeping has stopped (owned by the supervisor and lent to housekeeping, or handed back when housekeeping stops).
2. When 015a-16 stops restarts, run the minimal refusal path on the supervisor: receive control messages, decode admission requests with the core codec (epic 012), answer each with an admission refusal carrying ControlDegraded, and ignore everything else.
3. Run nothing of this path while housekeeping is running or restarting under backoff.
## Acceptance criteria
1. After the 5th panic within 60 s (test build), an admission request from a new peer on loopback is answered with a refusal carrying ControlDegraded and sent at DSCP 40.
2. While housekeeping is running, or waiting in a restart backoff, the refusal path sends nothing.
3. Live streams continue while the refusal path runs (no missed slice in a loopback run), and no data thread receives a message from it.
4. No lock type is shared with the data or clock threads (verified by 015b-02).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — admission decoding is core's (epic 012), no parser here; "Regime stamp fields touched" — none; the control-degraded mark and alarm are 015a-16's; "Design documents updated" — follows the epic; "Every new factual claim carries an evidence tag" — no new claim; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — not a media receive path.
## Out of scope
- The restart policy and the control-degraded mark (015a-16).
- The refusal reason encoding (core, epic 012).
- The AC-20(c) scenario (015b-04).
