# 015b-09 — No allocation on the data thread after go-live
**Epic:** 015b-Hosted-runtime-verification-and-strict-mode · **Effort:** S · **Depends on:** 015a-17
## Context
- Epic criterion 015b-A6: no allocation on the data thread after go-live, checked with an asserting allocator in a loopback scenario of this epic; the L0 harness (epic 017) reuses the same allocator.
- POC defect `[code]`: `src/module.rs:1639` — `routes = r.clone()` allocated on the data thread.
## Work
1. Provide an asserting global allocator (in `hadronyche-std` test support) that panics or records when a thread flagged as "live data thread" allocates.
2. Flag each data thread at go-live in `data_thread.rs`.
3. Run a loopback scenario covering admission changes, liveness transitions (interrupted, dead, marked gaps), receiver reports and counter publication for ≥ 60 s.
4. Export the allocator for the L0 harness (epic 017) to reuse.
## Acceptance criteria
1. 0 allocations recorded on any data thread after go-live in the 60 s scenario.
2. A deliberately inserted allocation in the data loop is detected (negative test).
3. The allocator is usable from another crate (a doc test or example shows it).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — none; "Design documents updated" — no behaviour differs; "No new open item" — none raised; "Every new factual claim carries an evidence tag" — no new claim; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — 015b-10.
## Out of scope
- The L0 harness itself (epic 017).
