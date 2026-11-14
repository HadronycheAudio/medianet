# 014-11 — Probe initiation policy
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-08, epic 012
## Context
Open sub-question 4 of the epic, owned by **this task**: "Who initiates one-way probes, and how often by default?" Falsifier: probes at the default rate change an admission decision against the 70 % admission ceiling (C-23).

Settled facts:
- Probes may be sent by either end, on demand, with DSCP EF (46) because they measure the media path (C-13) `[decision]`. The reverse path also carries one receiver report per bundle every 100 ms (C-40, DSCP 40). Nothing else travels on this path in v1.
- The commissioning tool may drive test traffic, and so request probes, only inside a declared commissioning window that every module stamps; outside it the tool is read-only (C-107). The window is announced by the tool on the control group with an expiry time, stamped by modules while open, and closes at expiry or on explicit close (C-153). While a window is open, ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified; the commissioning tool's own measurements are valid commissioning results, each recorded with the window it was taken in (C-178). One-way figures from tool-driven probes are therefore valid commissioning results recorded with their window, not unqualified figures.
- Admission is expressed as a fraction of the link rate, ceiling 70 % (C-23); one default stream is about 21.1 Mbit/s (C-21). IP is IPv4 only (C-100), so per-packet header overhead is IPv4 + UDP.
- Still open: the granularity with which epic 012 accounts admitted bandwidth (README remaining gaps, owner epic 012). The falsifier test uses it once epic 012 publishes it.

## Work
1. Write a short decision record in `doc/design/spec/` (C-130): which end initiates by default (producer, consumer, or both), the default rate, and the on-demand triggers (the commissioning tool inside a commissioning window, C-107; the ladder, epics 019/020).
2. Compute the probe bandwidth at the default rate (request + reply bytes per second including IPv4 and UDP headers) and compare it with the admission accounting of epic 012.
3. Expose the default as a constant in the module `measure` (for example `PROBE_DEFAULT_INTERVAL_NS`) and add it to the constants table of the design documents with a new C-nn id.
4. Add a test that asserts the computed probe bandwidth at the default rate is below epic 012's admission granularity (the falsifier).

## Acceptance criteria
1. A decision record exists with initiator, default rate, on-demand triggers (tool-driven probes only inside a commissioning window, C-107, C-153, whose figures are valid commissioning results recorded with that window, C-178), owner and falsifier, tagged `[decision]`.
2. The constant exists and the falsifier test passes.
3. Open sub-question 4 is closed in the design documents.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Scheduling and sending probes on the socket (epic 015).
- Enforcing the commissioning window (epic 028 and the stamp, epic 013).
- The probe codec (014-08) and one-way figures (014-09).
