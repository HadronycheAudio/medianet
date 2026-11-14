# 014-07 — Producer watch
**Epic:** 014-Measurement-path · **Effort:** XS · **Depends on:** 014-01, epic 005, epic 012
## Context
Invariant L-1 `[decision]`: an admitted, healthy connection becomes dead only from data-plane evidence. The consumer judges the producer by slices (Interrupted after 8 consecutive missed slices, C-38, counting only slices absent at the deadline, C-102; Dead after 2 s, C-39, from the last slice with content or a marked gap, C-111, or on another timeline epoch, C-135). A sink-only device sends no streams, so this producer timeout is the only judgement of its liveness (C-168). The producer has no data-plane evidence of its consumer except receiver reports, so **the producer stops sending only after 2 s without receiver reports** (C-41). Any valid report counts, whatever its member records contain. The result feeds the connection machine of epic 012 through its `ReportsSilentFor2s` evidence hook. Gossip (control-plane) silence must never stop the producer.

Still open (epic open sub-question 5, owner task 005): when the 2 s window starts before the first report arrives. Falsifier: a producer stops a connection whose consumer is alive but has not yet reported, or keeps sending to a consumer that never reports. This ticket takes the start time as a constructor argument so the answer only changes the call site.

Signatures from the epic:
```rust
pub struct ProducerWatch { /* per consumer */ }
impl ProducerWatch {
    pub fn new(start_ns: i64) -> Self;
    pub fn on_report(&mut self, now_ns: i64);
    pub fn silent_for_2s(&self, now_ns: i64) -> bool;        // feeds the connection machine
}
```

## Work
1. Implement `ProducerWatch::new(start_ns)`; the window runs from `start_ns` until the first report. The call site passes the instant epic 005 defines (admission until it rules otherwise).
2. `on_report` records the latest report time (ignore times earlier than the stored one).
3. `silent_for_2s` returns `true` iff `now_ns - last >= 2_000_000_000`.
4. Connect the result to epic 012's `ReportsSilentFor2s` hook (call site or adapter).

## Acceptance criteria
1. With reports every 100 ms for 60 s simulated, `silent_for_2s` is never `true`.
2. After the last report at time T, it is `false` at T + 1 999 999 999 ns and `true` at T + 2 000 000 000 ns.
3. With no report since `start_ns` = S, it becomes `true` at S + 2 s.
4. An out-of-order `on_report` with an older time does not shorten the window.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The connection state machine itself and stopping the sender (epics 005, 012, 015).
- End-to-end AC-21 scenario with gossip suppressed (014-17).
