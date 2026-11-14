# 014-26 — Current A in the receiver report (C-198)
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-03, 014-04, 014-05, epic 004, epic 007
## Context
C-198 makes the receiver report (C-40) the carrier of a changed A upstream `[decision]`: every report carries the consumer's **current A**, its own or the one it inherited. A fan-in outside any presentation group inherits the A of the downstream presentation group it feeds, the largest if several, learned at admission (C-142). A change therefore propagates one hop per report, on the data plane, and each fan-in that inherited the old value re-aligns (C-186): it re-enters aligning at an announced position, with rings and ratchet ceiling resized, as the sinks do.

Facts used:
- One report per bundle every 100 ms (C-40), sent by the consumer's data thread, unicast to the producer only (C-47). The current A is a property of the consumer, not of a member stream, so it is carried once per report, not per member record.
- The report must still fit 1 252 bytes of UDP payload with 32 members and every optional field present (C-06, C-17, C-100); there is no split rule.
- The unit and width of A on the wire come from epic 004's report layout (README remaining gap 1).
- This epic carries the value and exposes it to the producer; it does not decide whether a module inherited its A (C-142) and does not perform the re-alignment (C-186).

## Work
1. Add the consumer's current A to the report header: a `current_a` argument to `encode_report` and an accessor `ReceiverReport::current_a()`, following epic 004's layout. The field is always present.
2. Consumer side: the caller passes the consumer's current A, its own or the inherited one (C-198); the report timer and encoder add no other condition.
3. Producer side: per consumer, keep the last current A received and report a change when a valid report carries a different value, as a hand-off to the code that applies C-142 and C-186 (call site or adapter). A member record with a stale session (C-19) does not affect the field, which belongs to the report.
4. Recompute `MAX_REPORT_LEN` (014-03) with the field included.
5. Add report vectors in the epic 007 format with distinct `current_a` values, including the maximum of its width, and extend the `receiver_report_parse` fuzz target's accessor walk and round-trip property to the field (C-95).
6. No allocation, no locking, no panics on any input.

## Acceptance criteria
1. Every encoded report carries the current A passed to the encoder, and `ReceiverReport::current_a()` returns it (vectors pass in both directions, AC-1).
2. `MAX_REPORT_LEN` with the field, 32 members and every optional field present is ≤ 1 252 bytes (C-17, C-100).
3. The producer reports a change exactly once when the current A in a report differs from the previous report's, and not when it is unchanged.
4. Simulated chain of three hops in which each inheriting module reports the A it inherited: after a change of A at the sink end, the module k hops upstream receives the new value through k successive reports, one hop per report (C-198), with no control-plane message involved.
5. The fuzz target, extended to the field, runs at the C-95 budget with zero panics and zero round-trip mismatches.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Report generation takes bounded time on the data thread and is shown not to allocate or lock
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Deciding which A a fan-in inherits (C-142) and the re-alignment itself (C-186).
- The unit and width of A on the wire (epic 004).
- Sending the report on the reverse socket (epic 015).
