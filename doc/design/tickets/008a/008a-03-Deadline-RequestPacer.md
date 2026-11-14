# 008a-03 — Deadline RequestPacer
**Epic:** 008a-Servo-core-and-simulation-harness · **Effort:** S · **Depends on:** 008a-01
## Context
Followers send sync requests at a configured rate: default 50 per second (C-36), accepted range 10 to 200 per second (C-112) `[decision]`. In the POC, requests were paced by a 5 ms socket receive timeout; on a `CONFIG_HZ=250` kernel this achieved 37 exchanges/s at a configured 50, and 77 at a configured 1 000 `[measured]` (P-02).

Requirement `[decision]`: requests are paced by deadline; the achieved rate equals the configured rate within 2 % over the accepted range, and is reported (epic criterion A6, verified in simulation by 008b, system AC-7; the unit part here is criterion A15 of 008a).

Must compile for the `no_std`, no-allocator target chosen in 008a-01.

## Work
In `hadronyche-core`:

```rust
pub struct RequestPacer { /* deadline-based */ }
impl RequestPacer { pub fn next_deadline(&mut self, now_ns: i64) -> i64; }
```

1. Constructor taking the configured rate (requests per second). Declare the accepted range as constants `MIN_SYNC_RATE = 10` and `MAX_SYNC_RATE = 200` (names may be refined) citing C-112; reject rates outside it at construction.
2. Deadlines advance by a fixed period from the previous deadline, not from `now_ns`, so lateness of the caller does not lower the rate. If the caller is late by more than one period, skip missed deadlines rather than bursting (document the policy).
3. Expose the achieved rate (requests issued per second over a sliding or cumulative window) for reporting.
4. Unit tests with a synthetic clock: on-time caller, caller late by up to 0.9 period every call, caller with random lateness.

## Acceptance criteria
1. With an on-time caller at 50/s for 60 s of synthetic time, exactly 3 000 deadlines (±1) are produced.
2. With a caller whose wake-up lateness is uniformly random in [0, 0.9 period), the achieved rate is within 2 % of configured at 10/s, 50/s and 200/s.
3. Rates 9/s and 201/s are rejected at construction.
4. The reported achieved rate matches the count of issued deadlines within 0.1 %.
5. Builds for the 008a-01 no_std target.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Accepted rate range declared as constants that cite C-112

## Out of scope
- Simulation case 8 over the full scenario harness (008b-09).
- Wiring to real sockets and timers (epic 015).
