# 014-22 — Per-connection stamp summary (C-47)
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-02, 014-04, 014-07, epic 012, epic 013
## Context
Receiver reports are unicast from consumer to producer only: they are never multicast and there is no query message (C-47). Third parties learn stream health from the regime stamp instead: producer and consumer each publish **per-connection summary counters** in their own regime stamp: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events and dead (C-159, completing C-47), plus the observed DSCP (C-47) `[decision]`. The consumer builds its summary from the counters it reports; the producer builds its summary from the reports it receives. Both use the same type so a third party can compare the two views.

Facts used:
- Each member record carries its stream number (16 bits) and the stream's session (32 bits, C-10); the session is assigned by the source, one per stream, bumped on restart and on timeline epoch change, identical for every consumer (C-19). The producer ignores member records whose session is not the stream's current one.
- Missing can decrease after a late reclassification (missing −1, late +1, C-108); the summary shows the latest values.
- Dead on the consumer side: 2 s without data (C-39) counted from the last slice with content or a marked gap (C-111), or on another timeline epoch (C-135).
- The member record carries every C-159 count except dead (014-03), so the producer can fill the same fields.
- The stamp message is sent by epic 015 (C-144); lateness percentiles (C-160, over the rolling 60 s window of C-182, with the maximum since the connection went live beside them) are separate stamp fields, not part of this summary. Dead on the producer side: `ProducerWatch::silent_for_2s` (C-41, 014-07).
- Summaries published while a commissioning window is open are ordinary production loss figures and therefore unqualified (C-178); the open window is itself stamped by the module (C-153, epic 013), which is how a reader tells them apart. The summary type needs no field of its own for it.
- The stamp travels on its own control message (C-18). Epic 013 places the fields in the stamp; observed DSCP is shown with its capability state (C-129).
- What "connection" covers (one stream, or one bundle) is defined by epic 012 (README remaining gaps); the summary type works for either, by aggregating member records.

## Work
1. Define `ConnectionSummary { received, late, missing, marked_gap, other_epoch, too_early, duplicates, incomplete_reassemblies, demoted_rounds, interrupted, dead: bool, observed_dscp: Option<u8> }` (C-159, C-47) in the module `measure`, with widths from 014-02.
2. Producer side: `fn update_from_report(&mut self, report: &ReceiverReport, current_session: impl Fn(StreamId) -> Option<u32>)`, which ignores member records of a stale or unknown session, and `fn set_dead(&mut self, dead: bool)` fed from `ProducerWatch`.
3. Consumer side: `fn from_counters(members: &[MemberCounters], dscp: Option<u8>, dead: bool) -> ConnectionSummary`, using the same counters that go into the report.
4. Hand the summary to epic 013's stamp writer (call site or adapter). No allocation, no locking.

## Acceptance criteria
1. From the same simulated traffic (16 members, faults on three members), the producer's summary after the last delivered report equals the consumer's summary built at that report, field by field over every C-159 counter (epic AC-12).
2. A member record whose session differs from the stream's current one leaves the producer's summary unchanged (C-19).
3. A reclassification between two reports lowers `missing` by one and raises `late` by one in both summaries.
4. With reports withheld for 2 s, the producer's summary shows `dead`.
5. The stamp fields written from the summary are documented with epic 013.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The stamp layout and its control message (epic 013, C-18); sending that message (epic 015, C-144).
- Display of the summaries by tools (epics 017, 028).
