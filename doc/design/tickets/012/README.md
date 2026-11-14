# Epic 012 — Admission handshake: tickets

## Summary
Epic 012 (phase P3, scope L: 30 tickets, about 31.5 days, C-126) implements in `hadronyche-core`, as pure `no_std` state machines with no allocation, the two-sided admission handshake between a source output (or bundle members) and a consumer input, and the per-connection life cycle from Requested to Dead or Ended. Both ends check capacity in the settled units (wire bit/s against a 70 % ceiling, packets/s, slice rate, input count, buffer memory sized to the group's A, DSL cost bound in abstract operations), refuse explicitly and terminally with a named `RefuseReason` (18, C-185), run a path-MTU probe pair with a 1 280-byte fallback (IPv4 only), pin source and destination addresses (L-5), allocate the reverse measurement port, carry the source-assigned session of each stream (C-19), enforce a presentation group's required E (C-103, C-140), resolve the skew budget of a fan-in outside any group (C-142) and re-align it when its inherited A changes (C-186), including a change carried upstream in receiver reports (C-198) and one the fan-in's buffer memory cannot hold (C-211, C-216), re-budget a sink's inputs when its own group's A changes (C-214), keep Pending without timeout and cancellable (C-45), handle Demoted (C-109), raise the C-173 alarms, close admission while control-degraded and refuse with `ControlDegraded` (C-139, C-185), publish a per-connection stamp record (C-18, C-47, C-159), and drive connection state only from data-plane evidence once Live (L-1). It also delivers scenario vectors, a fuzz target and a published graph view. It depends on epics 004, 005, 007, 009, 010, 011 and blocks 014, 015, 017, 026, 028.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 012-01 | Core admission types and no_std build | S | epic 004, epic 010, epic 011 |
| 012-02 | Port budget 70 percent ceiling | XS | 012-01 |
| 012-03 | Receiver capacity checks | S | 012-01, 012-02 |
| 012-04 | Bundle subset buffer memory rule | XS | 012-03 |
| 012-05 | Manifest and bundle member checks | S | 012-01, epic 010 |
| 012-06 | Peer identity and cap checks | XS | 012-01, epic 011 |
| 012-07 | Presentation group required E check | XS | 012-01 |
| 012-08 | Handshake message codec | S | 012-01, 012-07, epic 004 |
| 012-09 | Path MTU probe machine | S | 012-01, 012-08 |
| 012-10 | MTU re-probe heuristic | S | 012-09 |
| 012-11 | Connection state machine | M | 012-01, epic 005, epic 009 |
| 012-12 | Admission engine requester side | M | 012-02, 012-06, 012-08, 012-09, 012-11, epic 011 |
| 012-13 | Admission engine consumer side | M | 012-03, 012-04, 012-05, 012-06, 012-07, 012-08, 012-09, 012-11, 012-12 |
| 012-14 | Tick probe timeout and pending recheck | S | 012-12, 012-13 |
| 012-15 | Data plane evidence and producer stop | S | 012-10, 012-11, 012-12, 012-13 |
| 012-16 | Gossip isolation and pinned destinations | S | 012-14, 012-15 |
| 012-17 | Receive side enforcement parameters | S | 012-13 |
| 012-18 | Graph view | S | 012-14, 012-15 |
| 012-19 | Admission scenario vectors | M | 012-14, 012-17, 012-18, 012-24, epic 004, epic 005, epic 007 |
| 012-20 | RefuseReason coverage test | XS | 012-19 |
| 012-21 | Handshake message fuzz target | S | 012-08, 012-13, 012-19, 012-27 |
| 012-22 | Operator and design documentation | S | 012-18, 012-19, 012-23, 012-25, 012-26, 012-27, 012-28, 012-29, 012-30 |
| 012-23 | Per-connection stamp record (new, C-18, C-47) | S | 012-14, 012-15, epic 004, epic 009 |
| 012-24 | Cancel a Pending request (new, C-45) | XS | 012-14 |
| 012-25 | Connection life-cycle scenario vectors (new, split from 012-19; C-109, C-124) | S | 012-16, 012-19, 012-23 |
| 012-26 | Inherited skew budget outside presentation groups (new, C-142) | M | 012-04, 012-08, 012-12, 012-13, 012-17 |
| 012-27 | Control-degraded admission gate (new, C-139, C-173) | S | 012-12, 012-13, 012-14, 012-15 |
| 012-28 | Changed A carried in receiver reports (new, C-198) | S | 012-15, 012-26 |
| 012-29 | A beyond fan-in buffer memory (new, C-211) | XS | 012-03, 012-04, 012-26, 012-28 |
| 012-30 | Changed A at a sink (new, C-214) | XS | 012-26, 012-28, 012-29 |

No ticket was deleted. The withdrawn rational-L rule (C-104, C-105 revised; C-134) left nothing to remove: no ticket of this epic handled structural latency, rounding of L or fractional delay lines, since admission carries no latency figure. 012-04 shrank from S to XS earlier because C-96 fixes the rule it had to decide. In the last round the rulings C-135 to C-176 changed 012-01, 012-03 to 012-15, 012-17 to 012-23 and 012-25 in place, without changing their effort, and added 012-26 (C-142) and 012-27 (C-139). In the final round C-185 and C-186 changed 012-01, 012-06, 012-08, 012-13, 012-20, 012-21, 012-22, 012-26 and 012-27 in place and added no ticket; 012-06 shrank from S to XS because `VersionMismatch`, `NameTaken` and `ModuleCap` are now raised at discovery, leaving it `ForeignDomain` and `FollowerCap`. C-198 then added 012-28 and changed 012-22 and 012-26 in place, without changing their effort. The sweep of C-177 to C-212 added 012-29 (C-211) and changed 012-01, 012-04, 012-05, 012-07, 012-08, 012-11, 012-15, 012-19, 012-21, 012-22, 012-23, 012-25, 012-26 and 012-28 in place, without changing their effort; no ticket was deleted. The round of C-181 (revised), C-214, C-216 and C-219 added 012-30 (C-214) and changed 012-03, 012-05, 012-22, 012-26 and 012-29 in place, without changing their effort; no ticket was deleted. The round of C-220 to C-225 changed 012-29 and 012-30 in place (C-222, C-224), without changing their effort; no ticket was added or deleted.

## Effort tally
- XS: 8 (0.25 day each, 2 days)
- S: 17 (1 day each, 17 days)
- M: 5 (2.5 days each, 12.5 days)
- Total: 30 tickets, about 31.5 days (31.5) → scope L (C-126); below the 40-day split threshold of C-143

## Coverage

### Deliverables

| Epic deliverable | Tickets |
|---|---|
| `Request` (with sessions), `Declared`, `PresentationTerms`, `Accept` (with `group_a_ns`), `RefuseReason` (18, with `BadMemberList`, `ControlDegraded`, `NoSkewBudget`), `ConnState` (with Demoted), `ConnEvent`, `DataPlaneEvidence` (with `CannotMeetA`), `RequestError` types | 012-01 |
| `PortBudget` with `CEILING_PERMILLE = 700` and `can_admit` | 012-02 |
| Capacity checks in the settled units (bit/s, packets/s, slice rate, input count, buffer memory, cost bound; a controller member at the C-219 figure) | 012-02, 012-03, 012-04, 012-26 |
| Explicit refusal with a named reason (the 15 raised by this epic; `VersionMismatch`, `NameTaken`, `ModuleCap` raised at discovery, C-185) | 012-03, 012-05, 012-06, 012-07, 012-09, 012-13, 012-14, 012-20, 012-26, 012-27 |
| Handshake messages (rates, members and sessions, probe pair, reverse port, pinned addresses, manifest, E, authentication reservation) | 012-08 |
| `MtuProbe` (padded DF packet at the intended size, ack, timeout, 1 280-byte fallback probe, `PathMtuTooSmall` per C-173) and re-probe | 012-09, 012-10, 012-14 |
| Pinned destinations (L-5) | 012-12, 012-13, 012-16 |
| Reverse measurement port and source sessions | 012-13 (stored by 012-12) |
| Presentation-group required E, unknown E (C-103), numeric E for commissioned sinks only (C-140) | 012-07, 012-13 |
| Skew budget of a fan-in outside any group (C-142), `group_a_ns`, `set_configured_skew_budget`, `NoSkewBudget` (C-185), new A on a changed inherited A (C-186) | 012-26 |
| `report_a_ns`, `on_report_a`: changed A carried upstream in receiver reports, one hop per report (C-198) | 012-28 |
| Changed A beyond a fan-in's buffer memory: ceiling capped, cannot-meet-A alarm held until a fitting change, `a_clamped`, given A still reported (C-211, C-216) | 012-29 |
| `set_group_a_ns`: a sink's own changed A, announced by the commissioning tool (C-224), re-budgets its inputs, switched at its own position (C-214), clamped beyond buffer memory (C-222) | 012-30 |
| Control-degraded gate, `set_control_degraded`, `RequestError::ControlDegraded` (C-139), refusal with `ControlDegraded` and the supervisor's stateless refusal path (C-185) | 012-27, 012-21 |
| Connection life cycle Requested → Dead / Ended, Demoted | 012-11, 012-15 |
| `Admission<CONNS>::request` (with `RequestError::TableFull`) | 012-12 |
| `Admission<CONNS>::cancel` | 012-24 |
| `Admission<CONNS>::on_message` | 012-12, 012-13 |
| `Admission<CONNS>::on_peer_map` and `on_tick` | 012-14 |
| `Admission<CONNS>::on_data_plane`, `ProducerStopped`, `Alarm` (C-173, C-174 triggers) | 012-15, 012-27 |
| `on_counters`, `stamp_records`, per-connection stamp record (C-159 counters) | 012-23 |
| Receive-side enforcement set-up (source filter, C-16 drain per slice period C-156, socket buffer) | 012-17, 012-26 |
| Scenario-vector tests (epic 005 scripts, C-193; vectors crate, C-192) | 012-19, 012-25 |
| Fuzz target for handshake messages (with the C-164 linear-cost check) | 012-08, 012-21 |
| Graph view (every wanted connection, state, refusal reason; satisfied = Live only, C-154) | 012-18 |
| Open sub-questions owned by this epic: probe timeout, re-probe thresholds | 012-09, 012-10 |
| DoD: documentation for operators, design docs, constants | 012-22 (and 012-04, 012-09, 012-10, 012-18) |
| DoD: regime stamp fields | 012-09, 012-14, 012-23, 012-22 |
| DoD: no transition out of Live/Interrupted from gossip | 012-11, 012-14, 012-15, 012-16, 012-21, 012-25, 012-27 |

### Acceptance criteria

| Epic criterion | Tickets |
|---|---|
| 012-A1 (AC-1): all admission and connection scenario vectors pass | 012-19, 012-25 |
| 012-A2 (AC-12): `PortCeiling` above 70 %, graph view unsatisfied, no retry; third 96 ch bundle on GbE refused (C-92) | 012-02, 012-03, 012-12, 012-13, 012-18, 012-19 |
| 012-A3: every `RefuseReason` this epic raises produced by at least one test; none of the three discovery reasons produced by admission (C-185) | 012-03, 012-05, 012-06, 012-07, 012-13, 012-14, 012-20, 012-26, 012-27 |
| 012-A4 (AC-30): `RequiredEExceeded` on the sink's own E (C-194), never Live; unknown E (C-103) | 012-07, 012-13, 012-19 |
| 012-A5 (AC-19 core): probe fails, admit at 1 280, MTU exposed; `PathMtuTooSmall` when 1 280 also fails; re-probe | 012-09, 012-10, 012-14, 012-15, 012-19 |
| 012-A6 (AC-20 d): changed address during Live changes no pinned address | 012-16, 012-25 |
| 012-A7 (AC-20 a): 30 s of hello silence leaves Live connections Live | 012-16, 012-25 |
| 012-A8 (AC-16): Interrupted at exactly 8 missed slices; immediate resume; Dead at 2 s | 012-11, 012-15, 012-25 |
| 012-A9 (AC-21 producer): stop after 2 s without receiver reports | 012-15, 012-25 |
| 012-A10 (C-45): Pending visible and unsatisfied, no timeout, → Requested when target appears, cancellable | 012-11, 012-14, 012-18, 012-19, 012-24 |
| 012-A11: compiles for `no_std` without allocator | 012-01 (kept by every ticket) |
| 012-A12 (C-109): Demoted, cleared only by re-admission | 012-11, 012-15, 012-18, 012-25 |
| 012-A13 (C-18, C-47, C-159): per-connection stamp record with every C-159 counter on both sides | 012-23, 012-25 |
| 012-A14 (C-19): source sessions in the request, same on every consumer | 012-01, 012-08, 012-12, 012-13, 012-19 |
| 012-A15 (C-124): sleep departure and fresh admission on wake | 012-15, 012-25 |
| 012-A16 (C-154, C-173, C-174): satisfied = Live only; alarms on exactly the six triggers | 012-15, 012-18, 012-19, 012-25, 012-27 |
| 012-A17 (C-142, C-185, C-186): fan-in outside any group sized to the largest downstream A, else configured budget, else refused with `NoSkewBudget`; re-aligned on a changed inherited A, a change after admission arriving in receiver reports (C-198) | 012-26, 012-28 |
| 012-A18 (C-139, C-185): control-degraded accepts no new connection, refuses with `ControlDegraded`, keeps admitted ones, raises the alarm | 012-27 |
| 012-A19 (C-211, C-216): changed A beyond buffer memory caps the ceiling, raises cannot-meet-A and holds it until a fitting change, reports the given A, refuses and tears down nothing | 012-29 |
| 012-A20 (C-214, C-222): a sink's own changed A queues new budgets for its inputs, clamped as for a fan-in beyond its buffer memory; state changes only on data-plane evidence | 012-30 |

## Remaining gaps
1. **Producer slot state after `ProducerStopped`, and exits from Demoted other than re-admission.** Affects 012-11, 012-15. Answer: control state machines epic 005.
2. **Who aggregates "both endpoints locked"** for Locking → Aligning. Affects 012-11, 012-15. Answer: epic 005, with epic 009. Each module's own lock state is published by the servo core, epic 008a (C-187).
3. **What a connection does when the source bumps its session (C-19) during Live or Interrupted.** Affects 012-11, 012-15. Answer: epic 005.
4. **Who re-requests a wanted connection after Dead or Ended**, including an edge waking from sleep (C-124) and a requester refused with `ControlDegraded` after the operator restart (C-185). Affects 012-15, 012-25, 012-27. Answer: epic 005.
5. **Whether epic 004 delivers a codec or only byte layouts**, and the stamp section layout. Affects 012-08, 012-23. Answer: wire specification epic 004.
6. **Path-MTU probe timeout value.** Affects 012-09, 012-14. Answer: proposed in 012-09, confirmed by a measurement (L1, task 019).
7. **Re-probe thresholds.** Affects 012-10. Answer: proposed in 012-10, confirmed by a measurement (L1, task 019).
8. **Per-platform receiver limits** (packets/s, slice rate, inputs, buffer, CPU per slice) and the link rate on each platform. Affects 012-02, 012-03, 012-17. Answer: a measurement (L1, C-92, C-93) and platform tasks 015, 021, 022, 026.
9. **Bad-input processing bound** (C-164: linear, at most 10 µs per packet, a proposal). Affects 012-08, 012-21. Answer: a measurement (L1).

## Settled since the split
- Former gap 1, required E has no carrier: C-103 (sink declares E or "unknown"), with `PresentationTerms` in the epic.
- Former gap 2, peer reasons: narrowed. `FollowerCap` settled by C-35 and C-114; `ModuleCap` value by C-34; domain id by C-11. Narrowed again in the last round (below); the rest is remaining gap 1.
- Former gap 3, `PathMtuTooSmall`: narrowed by C-05, C-100 and C-116; settled in the last round by C-173 (below).
- Former gap 5, receiver limits and skew budget: skew budget settled by C-96 (the group's A); cost-bound unit by C-106. Per-platform limits are remaining gap 11.
- Former gap 7, `on_tick` without a peer map: epic change, `on_peer_map` added and restricted to Pending.
- Former gap 8, missing evidence variants: epic change, `Disconnect`, `AnnouncedRealign`, `Demoted` (C-109) and `LargePacketsVanished` added.
- Former gap 11, enforcers of C-50 and C-54: C-101 (persistently late is the C-54 dwell, measured in epic 009); the epic states both arrive as evidence.
- Former gap 12, table full: epic change, `request` returns `RequestError::TableFull` locally; a consumer without a free slot refuses with `InputCount`.
- Former gap 13, graph view: narrowed. Shape fixed in the epic and 012-18; Pending unsatisfied by C-45; the rest settled in the last round by C-154 (below).
- Former gap 14, regime stamp location of the path MTU: C-18 (dedicated sectioned stamp message) and C-47 (per-connection summary); new ticket 012-23. Codec ownership remains as gap 8.
- Former gap 15, location of operator documentation: C-130 (`doc/design/spec/` in the design set's repository), refined by C-171 in the last round (operator guide in `doc/operations/`).
- Former gap 16, evidence tag list: the epic now lists all tags of `00-context.md`, plus `[standard]` (C-132).
- Epic open sub-question, buffer memory for a bundle subset: C-96 with C-04 and C-27.
- Epic open sub-question, Pending timeout: C-45 (none; operator cancel, new ticket 012-24).
- Epic open sub-question, session owner: C-19 (source-assigned, carried in the request).
- Last round (C-127 to C-176, C-35 owner change, C-104 and C-105 revised):
  - Previous remaining gap 1, `PathMtuTooSmall`: C-173 (the 1 280-byte probe also fails). 012-09 now probes at the fallback and returns `TooSmall`; the caller hook is gone.
  - Previous remaining gap 2, peer reasons: narrowed again. `ForeignDomain` by C-173 (different non-zero domain; a domain 0 hello is ignored and counted). The rest is remaining gap 1.
  - Previous remaining gap 3, member-list reason: C-173 (16th reason `BadMemberList`, also for more than 32 members). 012-01, 012-05, 012-08, 012-13, 012-20, 012-21, 012-22 updated.
  - Previous remaining gap 4, skew budget outside a group: C-142 (largest downstream A, learned at admission, else configured budget, else refused). New ticket 012-26. The unnamed refusal reason is remaining gap 2.
  - Previous remaining gap 5, satisfied mapping: C-154 (only Live). 012-18 updated.
  - Previous remaining gap 6, alarm triggers: C-173 and C-174 (refusal, interrupted, dead, demoted, cannot-meet-A, control-degraded). `CannotMeetA` evidence added; 012-15 and new 012-27 updated.
  - Stamp counters: C-159 completes C-47, with C-135 for the other-epoch counter. 012-23, 012-25 and the epic's `GapCounters` updated. Sending the stamp is epic 015's (C-144).
  - Dead clock and other-epoch slices: C-135. 012-10, 012-11, 012-15, 012-25 updated.
  - Header cap: C-137 (a default packet fits the 1 280-byte fallback). 012-09, 012-10 updated.
  - Numeric E: C-140 (commissioned sinks only). 012-07 updated.
  - Housekeeping restart limit: C-139 (no new connection while control-degraded). New ticket 012-27; the reply to a requester is remaining gap 3.
  - Sink-only device liveness: C-168 (producer timeout). 012-15 updated.
  - Drain cycle: C-156 (one slice period). 012-17 updated.
  - Bad-input bound: C-164 (proposal, pending L1). 012-08, 012-21 updated; remaining gap 12.
  - Documentation locations: C-171 (operator guide in `doc/operations/`, architecture in `doc/design/01-architecture.md`). 012-22 updated.
  - `FollowerCap` value: C-35's owner is now L1 (task 019). 012-06 notes it.
  - Structural latency (C-104, C-105 revised, C-134): no effect on this epic; nothing to remove.
- Final round (C-185, C-186):
  - Previous remaining gap 1, peer reasons at admission: C-185 (`VersionMismatch`, `NameTaken`, `ModuleCap` are raised at discovery, not at admission). 012-06 drops their provisional conditions and shrinks to XS; 012-13, 012-20, 012-22 updated.
  - Previous remaining gap 2, fan-in with no budget and a changed inherited A: C-185 (`NoSkewBudget`, replacing the provisional `BufferMemory`) and C-186 (each input that inherited the old A re-enters aligning at an announced position, rings and ratchet ceiling resized, replacing the provisional "keep the A used at admission"). 012-26 updated. The carrier of a downstream group's A change is the new remaining gap 1.
  - Previous remaining gap 3, reply at a control-degraded module: C-185 (refused with `ControlDegraded`, answered by the housekeeping supervisor's minimal refusal path, replacing the provisional "not answered"). 012-27 adds the stateless refusal function; 012-21 fuzzes it.
  - Reason count: C-185 (18). 012-01 (discriminants 0..=17), 012-08 (reason byte ≥ 18 rejected), 012-20, 012-21, 012-22 updated.
- Carrier round (C-198):
  - Previous remaining gap 1, carrier of a downstream group's A change after admission: C-198 (the receiver report, C-40, carries the consumer's current A, own or inherited; a change propagates one hop per report, on the data plane, and each fan-in that inherited the old value re-aligns, C-186). New ticket 012-28 (S); 012-26 and 012-22 updated; the epic's open sub-question is now decided. Remaining gaps 2 to 10 are renumbered 1 to 9.
- Sweep of C-177 to C-212 (with C-08, C-134, C-141, C-153, C-169, C-181, C-184 revised):
  - Controller member (C-08, C-181, C-205, C-209): counts towards the 32 members. 012-01, 012-04, 012-05 (new criterion 6), 012-08, 012-22 updated.
  - Required E (C-180, C-194): refused on the sink's own E whatever the other sinks'; a commissioned sink with an unlocked clock keeps its numeric E and waits in Locking. 012-07 (new criterion 6) and epic 012-A4 updated.
  - Changed A (C-208, C-211): the fan-in picks and announces its own re-alignment position (012-26, 012-28 updated); a change beyond its buffer memory caps the ceiling and raises cannot-meet-A. New ticket 012-29 (XS), epic criterion 012-A19.
  - Conformance vectors (C-192, C-193): epic 005 supplies the admission and connection scenario scripts, epic 004 the wire layouts and reject reasons; vectors in the crate at the workspace root; fuzzing tool the team's choice. 012-19 (now depends on epics 004 and 005), 012-21, 012-25 updated.
  - Commissioning window (C-153, C-178): production figures unqualified while open, stamped at stamp level; 012-23 notes the record needs no flag.
  - Lateness window (C-182): cannot-meet-A judged by epic 009 over a rolling 60 s window. 012-15 updated.
  - Lock state publisher (C-187): servo core, epic 008a; noted in 012-11 and remaining gap 2, which stays with epic 005.
  - No subject in this epic: C-134, C-141, C-169, C-177, C-179, C-183, C-184, C-188 to C-191, C-195 to C-197, C-199 to C-204, C-207, C-210, C-212. C-206 leaves the commissioning tool's presentation of the graph view to epic 028. C-185, C-186 and C-198 were applied in earlier rounds and are unchanged.
- Round of C-181 (revised: permissive), C-214, C-216, C-219:
  - Controllers of a single-stream source (C-181): a second stream is always allowed, required above the 100-byte cap, and the audio packet may be shared while they fit; checked by the compiler. Admission accepts either layout. 012-05 (new criterion 7) and the epic updated.
  - Wire cost of a controller stream (C-219): like an audio stream, one packet per slice of (headers up to 100 bytes + controller bytes per slice + 66 bytes), at 2 000 packets/s, the same figure as the commissioning tool. 012-03 (new criterion 8) and the epic's units table updated.
  - Clamped A (C-216): the cannot-meet-A alarm stays raised until a later change of A that fits, replacing 012-29's one event per capping (new `a_clamped`); the receiver report carries the A given, not the clamped value, confirming 012-29's reading. 012-29, epic 012-A19 updated.
  - Sink switch position (C-214): at its own position, one lead ahead, unannounced. New ticket 012-30 (XS) for the re-budget of a sink's inputs on a change of its own group's A (`set_group_a_ns`), epic criterion 012-A20; 012-26 points to it. The sink case of an A beyond buffer memory is new remaining gap 10.
- Round of C-220 to C-225:
  - Previous remaining gap 10, a sink's own changed A beyond its buffer memory: C-222 (the clamp of C-211 applies to any consumer that buffers to meet A, sinks included; C-216 applies to a sink in the same way). 012-30 now runs the 012-29 capping on its path (new criterion 4, depends on 012-29); epic 012-A20 and the open sub-question updated.
  - Commissioned change of a sink's own A (C-224): through the commissioning tool, inside a declared window (C-153), buffer memory validated first (C-211), then announced to the group's members; `set_group_a_ns` takes the announced value. 012-30 and the epic updated.
  - No subject in this epic: C-220, C-221, C-223, C-225 (ladder assertions, election, ratchet step at a sink, all outside admission).
  - No ticket added or deleted; tally unchanged.
- Not yet settled and carried forward: probe timeout (gap 6), re-probe thresholds (gap 7), endpoint aggregation (gap 2), producer state (gap 1), session bump (gap 3), re-request (gap 4), codec ownership (gap 5), receiver limits (gap 8), bad-input bound (gap 9).
