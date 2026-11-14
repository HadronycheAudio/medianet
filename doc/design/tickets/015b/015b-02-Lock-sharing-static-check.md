# 015b-02 — Static check: no lock shared across thread classes (AC-20(e))
**Epic:** 015b-Hosted-runtime-verification-and-strict-mode · **Effort:** S · **Depends on:** 015a-17
## Context
- **L-8** `[decision]`: no lock is shared between housekeeping and the data or clock threads; they communicate by messages or lock-free publication only.
- AC-20(e) (015b-A2(e)): a static check or lint shows no lock type shared between housekeeping and data or clock threads.
- The paths now crossing thread classes are: housekeeping → data/clock queues, the clock seqlock, the clock thread's last-reply time (C-37), the counter board read by housekeeping when it builds the stamp (C-18, C-47), and the file service queues.
- POC defect `[code]`: `src/module.rs:1518` — the clock thread took `inner.target.lock()`, a mutex shared with housekeeping (`:1484`), every iteration.
## Work
1. Implement a check (choose one and document it): a `clippy` `disallowed-types` configuration banning `Mutex`, `RwLock`, `Condvar`, `parking_lot` locks etc. in modules reachable from data and clock threads, or a type-level marker on all values passed between housekeeping and the other threads, or a source scan in CI.
2. Run it in CI on `hadronyche-std` and `hadronyche-os-linux`.
3. Add a negative test: a deliberately introduced shared `Mutex` makes the check fail.
## Acceptance criteria
1. CI runs the check and it passes on the current code.
2. The negative test (a shared `Mutex` between housekeeping and a data thread) fails the check.
3. The check covers every cross-thread path listed in Context; its scope and limits are documented.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — none; "Every new factual claim carries an evidence tag" — no new claim; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — not a runtime test.
## Out of scope
- Runtime behaviour tests of AC-20 (015b-04).
