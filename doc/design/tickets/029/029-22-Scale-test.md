# 029-22 — Scale test
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-03, 029-10

## Context
- Epic 029 criterion 029-AC-7: **64 modules (C-34), each sending a hello every 1 s (C-30) and its stamp every 10 s (C-18), with summaries for 32 streams, plus a burst in which all 64 modules change regime within one hello period, display without dropped observations on the development machine.** The machine's specification is recorded with the result.
- Receiver reports (one per bundle every 100 ms, C-40) are unicast and not visible to the monitor (C-47), so they are not part of the load. Stamps are multicast on any regime change and refreshed every 10 s (C-18), in sections of at most 1 252 bytes (C-17).
- `LiveSource` exposes `dropped_observations` (029-03).
- Results and reports go to `doc/design/reports/` (C-130).

## Work
1. A synthetic load generator (test-only, outside the monitor binary) emitting on a loopback IPv4 multicast group (C-100): 64 modules' hellos at 1 s and their stamps every 10 s, with C-47 summaries for 32 streams spread over the modules, for 10 minutes; once per minute, all 64 modules change regime within one hello period.
2. Run the monitor in UI mode (TestBackend) and headless mode against it; record `dropped_observations`, CPU and redraw latency.

## Acceptance criteria
1. `dropped_observations == 0` over 10 minutes in both modes, including during every regime-change burst.
2. All 64 modules and 32 streams appear in the Medianet and Connections panels.
3. Measured CPU and redraw latency, and the development machine's specification, are recorded in `doc/design/reports/` with a `[measured]` tag.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Optimising the runtime's stamp publication rate (epic 013/015).
