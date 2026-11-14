# 016-02 — Linux crate skeleton and reason collector
**Epic:** 016-Real-time-setup-backend · **Effort:** XS · **Depends on:** 016-01

## Context
- Epic 016 implements real-time setup for Linux in crate `hadronyche-os-linux`, module `rt.rs`, behind the OS-neutral types of `hadronyche-std::rt` (016-01).
- The POC (`src/rt.rs`, 125 lines) collects warnings once and deduplicates them (`rt.rs:9` `warn`, `rt.rs:16` `warnings`) `[code]`. In the new runtime these warnings become the capability report: each degradation is an *absent* bit with a reason.
- Reason: a degraded run must never look like a good one; the POC's first measured run showed that a run where real-time scheduling silently failed is indistinguishable from a good one `[measured]`.
- Wi-Fi is out of scope: `net::check_wifi_power_save` and any Wi-Fi branch must not be ported `[decision]` (epic AC 8).

## Work
1. Create (if absent) crate `hadronyche-os-linux` with module `rt.rs`, depending on `hadronyche-std` and `libc`; re-export the `hadronyche-std::rt` types.
2. Add an internal, thread-safe reason collector ported from POC `rt.rs:9`/`:16`: `fn note(cap: Capability, reason: &'static str)` records once per (capability, reason) pair; `fn reasons() -> Vec<(Capability, &'static str)>` returns them in first-seen order.
3. Comment on the collector giving its reason: "Ported from POC rt.rs:9/16. Becomes the capability report; a degraded run must not look like a good one."
4. Add a CI check (script or test) that fails if the string `wifi` (case-insensitive) or `check_wifi_power_save` appears in the crate sources.

## Acceptance criteria
1. `cargo build -p hadronyche-os-linux` succeeds on Linux.
2. Calling `note` 3 times with the same (capability, reason) and once with a different one yields exactly 2 entries from `reasons()` (unit test).
3. Concurrent `note` calls from 8 threads produce no duplicate and no lost entry (unit test).
4. The Wi-Fi check passes on the crate and fails when a file containing `wifi` is added (demonstrated once in the PR description or a test fixture).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Each ported POC behaviour carries a comment with its reason (table above)

## Out of scope
- Any setup function (016-05 to 016-12).
- Stamp wiring (016-13).
