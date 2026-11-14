# 015b-01 — Achieved sync rate verification (AC-7)
**Epic:** 015b-Hosted-runtime-verification-and-strict-mode · **Effort:** S · **Depends on:** 015a-10, 015b-12, 015b-13, epic 016
## Context
- Epic criterion 015b-A4 (AC-7): achieved sync rate within 2 % of the configured rate over the whole accepted range, and reported.
- The accepted range is 10 to 200 sync requests per second, and AC-7 is checked over this range (C-112). Default 50/s (C-36).
- POC baseline: 37/s achieved at `--sync-hz 50` must not reproduce `[measured]`; the POC ran on a kernel with `CONFIG_HZ=250` `[measured]`. The POC figure at 1 000/s is outside the accepted range and is no longer a test point.
- Sync requests are paced by deadline (`ppoll` or equivalent), not by receive timeouts.
- The holder answers sync requests on its clock thread, never on housekeeping (C-188), so the local responder is a runtime holder's clock thread. Requests beyond 200 per second from one follower are dropped before any work is done and counted per follower in the holder's stamp (C-204).
- Results go to `doc/design/reports/` (C-130). Any run whose numbers enter the constants table runs in strict mode (C-147): the recorded run uses strict mode (015b-13) with the task 016 real-time setup bound; the per-change CI test runs outside strict mode.
## Work
1. Integration test: clock thread against a local responder at configured rates 10, 50, 100 and 200 per second.
2. Measure completed exchanges/s over ≥ 30 s per point and compare to the reported metric.
3. Over-rate case: a scripted follower (raw-packet hook, 015b-12) sends 300 requests per second to the holder for ≥ 30 s (C-204).
4. Record results from a strict-mode run on the development machine (with `CONFIG_HZ`) in a report under `doc/design/reports/` with `[measured]` tags (C-147).
## Acceptance criteria
1. At 10/s, achieved rate ∈ [9.8, 10.2] per second.
2. At 50/s, achieved rate ∈ [49, 51] per second.
3. At 100/s, achieved rate ∈ [98, 102] per second.
4. At 200/s, achieved rate ∈ [196, 204] per second.
5. The reported metric matches the measured rate within 1 %.
6. The recorded run's stamp shows strict mode on (C-147).
7. In the over-rate case the holder answers at most 200 requests per second from that follower, and its stamp counts the dropped requests against that follower; replies to other followers keep their rate (C-204).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — read only (strict mode, per-follower dropped requests); "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — not a receive-cost test.
## Out of scope
- Configured follower rates outside 10 to 200 per second (refused at configuration, 015a-10); the holder side of an over-rate follower is in scope (C-204).
