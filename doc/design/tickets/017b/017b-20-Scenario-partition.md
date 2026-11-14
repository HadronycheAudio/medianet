# 017b-20 — Scenario `partition`
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017b-04, 017a-09, 017b-09, epic 010

## Context
- AC-36: after an injected partition heals, **one holder within 1.4 s** (election window 400 ms, C-33, + hello 1 s, C-30); the **losing side receives a timeline epoch change** and **restarts streams at an announced position** (announcement lead 64 slices, C-43; re-issue every 16, C-44).
- **Session number (settled, C-19):** bumped by the source on restart and on timeline epoch change; every consumer of a stream sees the same value.
- An input on another epoch is a gap and resets the interrupted count (C-102) during the switch.
- Injection: `Injector::partition(groups)` (017b-04); a single group heals.

## Work
1. Scenario `partition`: run sender/echo/mixer plus enough modules for two sides; partition into two groups for 10 s (each side elects its own holder); heal.
2. Assertions: within 1.4 s of heal exactly one holder exists; the losing side gets an epoch change; its streams restart at an announced position with bumped session numbers that every consumer agrees on.

## Acceptance criteria
1. `hadronyche-l0 partition --duration 30` exits 0.
2. Time from heal to single holder ≤ 1.4 s.
3. Every module on the losing side logs an epoch change; restarted streams begin at the announced position, announced ≥ 64 slices ahead.
4. Each restarted stream's session number is bumped and identical at the source and every consumer (C-19).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Foreign-epoch input handling (017b-16).
