# 002-11 — Servo phase step measurement
**Epic:** 002-POC-archaeology · **Effort:** S · **Depends on:** 002-01
## Context
Part of epic 002 (POC archaeology, check 0b), deliverable **(e) Run confirmations**, item 2.

Claim read from code, never run `[code]`: `Servo::update` (in `clock.rs`, function `update`) sets `theta0 = pred + alpha·w·err` at each window, which is a **step in published time** of a fraction of the error every **200 ms**.

The 200 ms window is inherited by the design as C-66 (a window uses its minimum-round-trip exchange). The POC declares lock when the smoothed error is below max(20 µs, path delay / 4) after at least 8 windows (P-04) `[code]`; "locked" in this ticket means that condition. The runtime servo is bounded by a slew of at most 10 ppm (C-60) and a rate of change of at most 1 ppm per second (C-61), both pending simulation; the step distribution measured here is an input to that simulation, not a test of those limits.

Diagnostic logging must live in the scratch copy of the POC (002-01), never in the frozen tree.
## Work
1. In the scratch copy, log the published `theta0` immediately before and after each `Servo::update`, with the lock state, `err`, `alpha`, `w` and a monotonic timestamp.
2. Run `hadropoc loopback` from the scratch copy for at least 60 s, unprivileged; discard updates before lock.
3. Compute the step `Δ = theta0_after − theta0_before` for every locked update and report: count, min, median, p99, max of |Δ| (in ns), the update interval (median, ms), and a histogram.
4. Write the result and the verdict line in section (e) of `doc/design/reports/archaeology/REPORT-0b.md`, tagged `[measured]`: "phase step per update: confirmed" or "refuted".
## Acceptance criteria
1. At least 200 locked updates are measured (≈ 40 s at one per 200 ms).
2. Count, min, median, p99, max of |Δ| in ns and the median update interval in ms are reported.
3. The verdict line names the mechanism with a `clock.rs` `file:line` pointer (epic criteria 002-AC-1 and 002-AC-6 for (e) 2).
4. The frozen-tree fingerprint from 002-01 is unchanged.
5. The report states which lock condition was used to discard pre-lock updates and whether it matches P-04.
## Definition of done
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Diagnostic logging used for (b) and (e) lives in a scratch copy of the POC, not in the frozen tree
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
## Out of scope
- Changing the servo (runtime design, other epics).
- Flood test (002-10) and sync rate (002-12, 002-14).
- Checking the step sizes against C-60 and C-61 (servo simulation).
