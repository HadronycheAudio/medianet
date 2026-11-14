# 019-16 — Serving scenario
**Epic:** 019-L1-two-machine-harness · **Effort:** M · **Depends on:** 019-04, 019-05, 019-06, 019-23

## Context
A clock holder serves sync requests from followers. Settled facts:
- At most 64 followers per holder (C-35, decided; owner L1, task 019; falsifier: holder serving lateness grows with followers below 64); the 65th is refused with a named event.
- At most 64 active modules per medianet (C-34), so a holder has at most 63 real followers. The 64-follower load and the refusal of a 65th (AC-25) are tested with synthetic followers (C-114), provided by 019-23 through the test-only hooks of C-120.
- Followers send 50 sync requests/s by default (C-36); a holder accepts 10 to 200 per second per follower (C-112). 64 followers at 50/s → 3 200 requests/s at the holder.
- Sync packets use DSCP CS6 (48) (C-13).
- The holder answers sync requests on its clock thread, never on housekeeping (C-188), so serving lateness measures the clock thread. Requests beyond 200 per second from one follower are dropped before any work and counted per follower in the holder's stamp (C-204); at 50 per second no request should be dropped, and a dropped request would leave serving lateness unmeasured for it.
- Follower servo: lock bound at most 25 µs (C-62), slew at most 10 ppm (C-60), rate change at most 1 ppm/s (C-61).

Scenario `serving`: 1, 16, 32 and 63 real follower processes against one holder, then 64 and 65 followers using synthetic followers. C-35 is measured by "serving lateness against follower count".

**AC-25.** 64 followers at 50 requests/s each with no growth in holder serving lateness; the 65th refused with a named event.

Holder serving lateness is reply send time minus request receive time at the 99.9th percentile; "no growth" means within 20 % from 1 to 64 followers (C-167). Every measured step is a strict-mode run (C-147).

## Work
- Scenario `serving`: holder on host `a`; followers on host `b` (all on one host to share the wire), each at 50 requests/s.
  - N ∈ {1, 16, 32, 63}: real follower processes.
  - N = 64: 63 real followers plus 1 synthetic follower (019-23), or 64 synthetic followers; record which.
  - N = 65: the 64 above, then one more synthetic follower; record the holder's refusal event name and the 65th follower's observed outcome.
  - Each step runs 10 minutes (C-167).
- Check that the total module count stays within C-34 for real modules and that every synthetic follower appears as an injection in the holder's stamp (C-120).
- Report section: holder serving lateness p50/p99/p99.9/max per N, request rate actually served per N, the holder's per-follower over-rate drop counter per N (C-204), follower lock (offset p99 vs 25 µs bound) per real follower per N, refusal event for N = 65.
- Plot or table of serving lateness against N.
- Record in `doc/design/reports/l1-results.md` under C-35: whether 64 holds, lateness numbers, hardware, stamp, tag `[measured]`.

## Acceptance criteria
1. Dry-run passes in CI with reduced N (e.g. 1, 2, 3 with limit lowered to 2, the extra follower synthetic) and shows the refusal of the extra follower.
2. On the hardware pair, with 64 followers at 50 requests/s each (3 200 requests/s), holder serving lateness at the 99.9th percentile is within 20 % of its value at N = 1 (AC-25, C-167).
3. The 65th follower is refused; the report names the event (AC-25).
4. Real followers keep their offset p99 within the 25 µs lock bound (C-62) at every N.
5. The holder's per-follower over-rate drop counter (C-204) is 0 for every follower at every N; a non-zero count fails the step.
6. `l1-results.md` contains a table of serving lateness p50/p99/p99.9/max for N = 1, 16, 32, 63, 64, marking which are real and which synthetic, and C-35 has a value tagged `[measured]` or a reason why not.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Each pending-measurement constant in the table above has a measured value tagged `[measured]`, or a stated reason why not
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The synthetic follower generator itself (019-23).
- Implementing holder admission and refusal (epic 015).
- Writing C-35 to the constants table (019-22).
