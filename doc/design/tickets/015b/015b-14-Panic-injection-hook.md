# 015b-14 — Panic-injection hook (C-145)
**Epic:** 015b-Hosted-runtime-verification-and-strict-mode · **Effort:** XS · **Depends on:** 015a-16, 015b-12
## Context
- C-145: an additional test-only hook injects a panic into a data, clock or housekeeping thread; it extends the C-120 list. Like the C-120 hooks, each use is stamped as an injection.
- Users: the housekeeping restart test AC-20 (c) (015b-04) and the fail-stop test AC-21 (015b-05).
- Expected outcomes: a panic in a data or clock thread terminates the process (L-7); a panic in housekeeping is restarted under the C-139 policy (015a-16).
- The hook shares the feature gate and injection record of 015b-12, and does not exist in a release build without the test feature.
## Work
1. Add `inject_panic(thread)` to the test-only hook set, with `thread` one of: a module's data thread, the clock thread, the housekeeping thread.
2. Deliver the request lock-free (a flag each thread checks once per loop), so the panic happens on the target thread itself.
3. Record the injection on the counter board before the panic is raised, so it reaches the stamp when the process survives (housekeeping case).
## Acceptance criteria
1. Injecting into a data thread or the clock thread aborts the process (subprocess test).
2. Injecting into housekeeping restarts it (015a-16) and the next stamp records the injection.
3. A release build without the test feature has no hook symbol (build test).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Design documents updated" — follows the epic; "Every new factual claim carries an evidence tag" — no new claim; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — not a receive-cost test.
## Out of scope
- The scenarios that use the hook (015b-04, 015b-05).
