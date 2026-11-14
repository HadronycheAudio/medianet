# 014-17 — Producer liveness scenario (AC-21)
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-03, 014-04, 014-06, 014-07, epic 012
## Context
System criterion AC-21 (producer side): with receiver reports withheld, the producer stops at 2 s (C-41); with reports flowing and gossip fully suppressed, it never stops. Invariant L-1 `[decision]`: an admitted, healthy connection becomes dead only from data-plane evidence; the receiver report (one per bundle every 100 ms, C-40, sent from the consumer's data thread, unicast only, C-47) is the producer's only evidence of its consumer, and any valid report counts. `ProducerWatch::silent_for_2s` feeds epic 012's connection machine through the `ReportsSilentFor2s` evidence hook. The start of the window before the first report is open (epic open sub-question 5, owner task 005); this scenario starts the watch at admission and checks only windows that follow a delivered report.

## Work
1. Build a deterministic, simulated-time test (no sockets, unprivileged) connecting: a consumer side driving `ReportTimer` once per slice (0.5 ms, C-02) and `encode_report`; a producer side parsing with `ReceiverReport::parse`, calling `ProducerWatch::on_report`, and feeding `silent_for_2s` into epic 012's connection machine.
2. Scenario A: reports withheld from time W.
3. Scenario B: reports flowing, gossip fully suppressed for 60 s simulated.
4. Scenario C: consumer housekeeping thread starved (simulated: housekeeping never runs) while the data thread keeps running.
5. Scenario D: reports flowing but every member record carries a stale session (C-19); the reports still count as liveness.

## Acceptance criteria
1. Scenario A: the producer's connection machine reaches its stop state at W' + 2 s, where W' is the time of the last delivered report, and not before (± one slice period, 500 000 ns).
2. Scenario B: the producer never stops over 60 s.
3. Scenario C: the producer never stops over 60 s.
4. Scenario D: the producer never stops over 60 s.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Consumer-side liveness (Interrupted C-38 with C-102, Dead C-39 with C-111; epics 009, 012).
- The same scenario over real sockets (epic 015), including a housekeeping panic injected with the C-145 hook under the restart policy of C-139 (live streams continue).
- Liveness of a sink-only device is the same producer timeout (C-168) and needs no extra scenario here.
