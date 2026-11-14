# 014-06 — Report timer
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-01
## Context
The consumer sends exactly one receiver report per bundle every 100 ms (C-40), within ±1 slice period (slice = 48 frames at 96 kHz = 0.5 ms, C-01, C-02). Reports must be sent by the consumer's **data thread**, not by housekeeping `[decision]`: the producer stops sending after 2 s without reports (C-41), so a starved housekeeping thread would otherwise tear down a healthy stream (invariant L-1: an admitted, healthy connection becomes dead only from data-plane evidence). The timer is therefore polled from the data thread's per-cycle loop.

Signature from the epic:
```rust
pub struct ReportTimer { /* every 100 ms, driven from the data thread's cycle */ }
impl ReportTimer { pub fn new(now_ns: i64) -> Self; pub fn due(&mut self, now_ns: i64) -> bool; }
```

## Work
1. Implement `ReportTimer` with the constructor `new(now_ns: i64)`, one instance per bundle.
2. `due` returns `true` at most once per call when `now_ns` has reached the next deadline; the next deadline advances by exactly `100_000_000` ns from the previous deadline (no cumulative drift).
3. If the data thread was stalled for more than one interval, `due` returns `true` once and re-anchors to the next future multiple; it never returns `true` on consecutive calls to "catch up".
4. Handle a non-monotonic `now_ns` (earlier than the previous call) without panicking and without emitting.
5. Constant time, no allocation, no locking.

## Acceptance criteria
1. Driven at one call per slice (every 500 000 ns) for 10 s simulated, `due` returns `true` exactly 100 times, and every true call is within 500 000 ns (±1 slice period) of its nominal 100 ms deadline (epic AC-2).
2. A 16-member bundle uses one timer and yields 10 reports per second, not 160 (epic AC-2): over 10 s simulated, 100 reports for the bundle.
3. After a simulated 350 ms stall, exactly one `true` is returned, then the next at the following 100 ms grid point.
4. A `now_ns` moving backwards returns `false` and does not panic.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Report generation takes bounded time on the data thread and is shown not to allocate or lock
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Wiring the timer into the data thread and sending (epic 015).
- Whole-path rate test against simulated traffic (014-18).
