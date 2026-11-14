# 022a-12 — Media deadline clock from the audio callback
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** S · **Depends on:** 022a-11, epic 018

## Context
- Part of epic 022a (Windows edge backend), `deadline.rs`: the core deadline-clock trait implemented from the audio callback plus QPC regression.
- The media deadline clock exists in core only as an abstract trait; this backend provides it from the audio callback plus a QPC regression `[decision]`.
- **No spinning ticker on Windows in v1** `[decision]`: sink edges do not pace, they consume; capture edges pace emission off the audio callback. A 0.5 ms slice clock (C-02) cannot be paced by sleeping (default timer 15.6 ms, best waitable timer about 0.5 ms `[memory]`).
- Sub-millisecond pacing is report-only on Windows: stamped *pacing: batch per engine period*.
- Deadline overruns are diagnostics only (bounded real-time: *absent*).

## Work
1. `deadline.rs`: `WasapiDeadline` implementing the core deadline-clock trait (epic 018). Ticks are delivered on each WASAPI event callback; the deadline time for a position is computed from the loop-2 regression (022a-11) mapped to H (QPC).
2. Between callbacks, `now`/deadline queries are answered from the regression, not from a timer.
3. Overrun detection: when a callback arrives later than its expected time by more than one period, increment a diagnostic counter; never treat it as fatal.
4. Tests: synthetic callback sequence at 10 ms period with jitter → deadlines monotonic and consistent with the regression.

## Acceptance criteria
1. `WasapiDeadline` implements the core trait; no thread, timer or loop in this module wakes more often than the device callback (code review + test).
2. With synthetic callbacks at 10 ms ± 1 ms, deadline positions are monotonic and within 1 sample of the regression prediction.
3. A late callback (> 1 period) increments the overrun diagnostic and does not stop the stream.

## Definition of done
- [ ] Tests green in CI, unprivileged

## Out of scope
- Using the deadline clock in render and capture (022a-13, 022a-14).
- Emitting the gap slice when a module misses its own deadline, and raising the processing-overrun event: both belong to the hosted backend (epic 015, C-144); this ticket only supplies the late-callback diagnostic.
