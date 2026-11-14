# 029-20 — Read-only verification in baseline scenarios
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-03, 029-10, epic 017b, epic 019

## Context
- `hadronyche-monitor` (epic 029) is strictly read-only: it never sends anything except the IGMP membership traffic that joining the IPv4 multicast groups requires (C-100), never sends the unicast stamp request (C-18), and can never affect a stream. `[decision]`
- Epic criterion 029-AC-1: **A capture on the medianet interface shows the monitor sends no control, sync, media or measurement packets and no stamp request. Starting and stopping it during the L0 and L1 baseline scenarios changes no counter in any module.**
- The L1 baseline scenarios belong to task 019 (C-94); the L0 baseline scenario is defined by epic 017b (C-192).
- Results and reports go to `doc/design/reports/` (C-130).

## Work
1. Scenario test using the L0 and L1 baseline scenarios: run each twice, once without the monitor and once with the monitor started and stopped at least 3 times mid-run.
2. Capture all packets on the medianet interface from the monitor host; classify by source process / port and assert none are control, sync, media, measurement or stamp-request packets (IGMP membership reports allowed).
3. Compare every counter of every module's regime stamp (including per-connection summaries C-47 and Drops group) between the runs.

## Acceptance criteria
1. Zero control, sync, media, measurement or stamp-request packets from the monitor in both scenarios.
2. Per-module counters with and without the monitor are equal (and in particular, starting/stopping the monitor causes no step in any counter).
3. The test is automated; its result is attached to CI and recorded in `doc/design/reports/` (C-130).
4. If epic 017b has not yet delivered the L0 scenario, the L1 part passes and the L0 part is recorded as blocked on epic 017b (C-192), not skipped silently.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Defining the L0/L1 scenarios (task 019 for L1; epic 017b for L0, C-192).
- Packet capture privileges: if capture needs privileges, the capture side runs in the scenario harness in the separate privileged nightly job (C-146), not in the monitor, so the monitor itself stays unprivileged and per-change CI stays unprivileged.
