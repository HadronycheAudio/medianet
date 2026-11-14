# 015b-06 — Wall-clock step test
**Epic:** 015b-Hosted-runtime-verification-and-strict-mode · **Effort:** S · **Depends on:** 015a-10
## Context
- Epic criterion 015b-A7: stepping `CLOCK_REALTIME` by ±1 s during a run changes no servo estimate; discarded samples are counted.
- `SO_TIMESTAMPNS` delivers `CLOCK_REALTIME` stamps `[kernel-doc]`; the step detector in the backend compares realtime-minus-monotonic before and after and discards the sample on change (015a-03). The servo only ever receives exchanges that passed it (C-121).
- POC defect `[code]`: `src/module.rs:1557` and `:1671` derived receive time as `real_ns() − kernel_stamp`, with `.max(0)` hiding steps.
- A real `CLOCK_REALTIME` step needs `CAP_SYS_TIME`, and `CLOCK_REALTIME` is not virtualised by time namespaces `[kernel-doc]`, so the real step cannot run in unprivileged CI. Tests that need a privilege run in a separate privileged nightly job; the definition-of-done line "unprivileged" applies to per-change CI (C-146). The real step runs in that job; the per-change variant uses the injectable realtime source of 015a-03.
## Work
1. Unprivileged variant: run the clock thread against a responder with the injectable realtime source; inject +1 s and then −1 s steps.
2. Privileged variant: the same scenario with real `clock_settime` steps, registered in the privileged nightly job (C-146).
3. Compare servo offset and rate estimates before and after each step.
## Acceptance criteria
1. Servo offset estimate changes by less than its normal jitter (no 1 s jump) across each step.
2. The discarded-sample counter increases by ≥ 1 per step.
3. The unprivileged variant runs in per-change CI; the privileged variant runs in the privileged nightly job and meets criteria 1 and 2 (C-146).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — none; "Design documents updated" — no behaviour differs; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — not a receive-cost test.
## Out of scope
- Detector tolerance value (measured in strict mode in 015b-15, C-147).
