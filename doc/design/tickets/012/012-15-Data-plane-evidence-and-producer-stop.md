# 012-15 — Data plane evidence and producer stop
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** 012-10, 012-11, 012-12, 012-13

## Context
Epic 012's `Admission::on_data_plane(&mut self, conn: ConnId, ev: DataPlaneEvidence) -> Option<ConnEvent>` feeds data-plane evidence (from epic 009) into each connection's state machine (012-11).

`DataPlaneEvidence { ClockUsable(bool), AligningDone { offset_ns: i64 }, MissedSlices(u32), Resumed, SilentFor2s, EndOfStream, Disconnect, RatchetStep, AnnouncedRealign, Demoted, CannotMeetA, LargePacketsVanished, ReportsSilentFor2s }` (012-01).

Rules `[decision]`:
- Live → Interrupted after 8 consecutive missed slices (C-38, 4 ms); Interrupted → Live immediately when slices return on the same session; Interrupted → Dead at 2 s without data (C-39) (012-A8, AC-16). Epic 009 has already applied C-102 (what counts as a miss), C-111 and C-135 (where the 2 s start).
- `RatchetStep` means persistently late (C-101); `Demoted` means persistently late with the offset already at A (C-109): the input is treated as gapped, counted separately, and the state is cleared only by re-admission.
- **L-1** reverse direction: the producer stops only after 2 s without receiver reports (C-41). Receiver reports are one per bundle every 100 ms (C-40). → `ConnEvent::ProducerStopped(conn)` (012-A9, AC-21 producer side).
- `ConnEvent::Alarm(conn)` raises an operator alarm, carried in the hello's alarm field. Settled triggers (C-173): refusal, interrupted, dead, demoted, cannot-meet-A and control-degraded. `CannotMeetA` is evidence from epic 009: maximum lateness + D + headroom above A in each of 10 consecutive 1 s windows (C-174), judged by epic 009 over the rolling 60 s window of C-182; it raises the alarm and changes no state. The control-degraded alarm is raised by 012-27.
- A sink-only device is judged live by the producer timeout only (C-168): on a producer-side slot whose consumer is a sink-only device, `ReportsSilentFor2s` is the only liveness evidence.
- An edge going to sleep departs (C-124): its `Disconnect` ends the connection.

Still open (epic open sub-question): the producer slot's state after `ProducerStopped` (owner epic 005; falsifier: the engine's table differs from 005's). This ticket uses a provisional choice, isolated in one function and recorded in the module docs as provisional: after `ProducerStopped` the producer slot is set to Dead.

## Work
- Route each evidence to `step` (012-11) for the slot; on a change emit `ConnEvent::State(conn, new)`.
- `ReportsSilentFor2s` on a producer-side slot → `ConnEvent::ProducerStopped(conn)`, then the provisional state above.
- `LargePacketsVanished` → restart the slot's `MtuProbe` while staying Live.
- Raise `ConnEvent::Alarm(conn)` on → Refused, → Interrupted, → Dead, → Demoted and on `CannotMeetA` (C-173, C-174), and on no other event; the trigger list is one `const` function.
- Evidence for an unknown or terminal `ConnId` → `None`.

## Acceptance criteria
1. AC-16: 7 then 8 `MissedSlices` → Interrupted only at 8; `Resumed` → Live; `SilentFor2s` from Interrupted → Dead.
2. AC-21: `ReportsSilentFor2s` on a producer slot → `ProducerStopped` emitted once; nothing before that evidence stops the producer.
3. Admitted + `ClockUsable(false)` → Locking; + `ClockUsable(true)` → Aligning; + `AligningDone` → Live.
4. `LargePacketsVanished` on Live emits a new probe into the outbox and the state stays Live.
5. Live + `Demoted` → Demoted with an alarm; later evidence other than a new admission leaves it Demoted (C-109).
6. Live + `Disconnect` → Ended.
7. Evidence for a terminal slot returns `None`.
8. An alarm is emitted on each of refusal, Live → Interrupted, → Dead, → Demoted and `CannotMeetA`; `CannotMeetA` leaves the state unchanged; no alarm on any other transition (Requested, Admitted, Locking, Aligning, Live, Ended) (C-173, C-174).
9. Producer slot toward a sink-only device: no evidence other than `ReportsSilentFor2s` stops it (C-168).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented — not applicable: the stamp record is 012-23
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection)
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Measuring missed slices, lateness, silence or reports (epic 009). Hello alarm field encoding (epic 011). Gossip isolation tests (012-16). Counters for the stamp (012-23).
