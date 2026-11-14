# Epic 005 tickets — Control state machines specification

## Summary
Epic 005 (phase P1, scope L, depends on 002 and 004, blocks 007, 011, 012) specifies the four control state machines of the runtime — discovery map, clock election, admission handshake and connection states (with a small producer-side machine) — as pure tables *(state, input message or timer) → (new state, output messages, events)* with no I/O, so one `no_std` implementation in `hadronyche-core` serves the hosted and embedded profiles and task 007 can generate scenario vectors from them. It delivers `doc/design/spec/control.md`, a machine-readable form (`doc/design/spec/control.toml`), the `RefuseReason` (18 reasons, C-173, C-185) / `ConnectionState` / `ProducerState` / `ControlEvent` enumerations, the regime stamp coupling, connection alarms, and prose scenario scripts; the tables must be total, preserve invariants L-1 to L-8, state worst-case memory and work with 63 peers (C-217), and a model check must show a single holder after convergence. Tools (checker, explorer) live in the `hadronyche-tools` crate (C-133).

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 005-01 | Control spec skeleton and conventions | XS | epic 004 |
| 005-02 | Machine-readable table format | S | 005-01 |
| 005-03 | Totality checker | S | 005-02 |
| 005-04 | Control enumerations | S | 005-01 |
| 005-06 | Discovery map states and inputs | S | 005-02, epic 004 |
| 005-07 | Discovery map transition table | M | 005-03, 005-04, 005-06, 005-23 |
| 005-08 | Election tie-break decision | XS | 005-01 |
| 005-09 | Election states and inputs | S | 005-02, 005-08, epic 004 |
| 005-10 | Election transition table | M | 005-03, 005-04, 005-09 |
| 005-11 | Election model check | M | 005-10 |
| 005-13 | Admission handshake states and inputs | S | 005-02, epic 004 |
| 005-14 | Admission handshake transition table | M | 005-03, 005-04, 005-13, 005-23 |
| 005-15 | Connection states and inputs | S | 005-02, epic 004 |
| 005-16 | Connection transition tables | M | 005-03, 005-04, 005-15, 005-23, 005-24 |
| 005-17 | Invariant preservation analysis | M | 005-07, 005-10, 005-14, 005-16 |
| 005-18 | Worst-case memory and work summary | XS | 005-07, 005-10, 005-14, 005-16 |
| 005-19 | Scenarios: cold start, holder loss, partition and heal | S | 005-07, 005-10 |
| 005-20 | Scenarios: id collision and gossip robustness | S | 005-07, 005-10, 005-16 |
| 005-21 | Scenarios: refusal for each reason | S | 005-07, 005-10, 005-14, 005-27 |
| 005-22 | Spec closure and open items | XS | 005-11, 005-17, 005-18, 005-19, 005-20, 005-21, 005-23, 005-25, 005-26, 005-27, 005-28, 005-29 |
| 005-23 | Regime stamp coupling | S | 005-06, 005-13, 005-15, epic 004 |
| 005-24 | Connection health guards and demotion | S | 005-15 |
| 005-25 | Scenarios: Pending and demotion | S | 005-14, 005-16 |
| 005-26 | Connection alarms | S | 005-04, 005-16, 005-28 |
| 005-27 | Fan-in A inheritance and alignment opt-out | S | 005-14, 005-16, 005-24 |
| 005-28 | Changed A propagation and fan-in re-alignment | S | 005-15, 005-16, 005-27 |
| 005-29 | Sink switch to a changed A | S | 005-15, 005-16, 005-28 |

Deleted since the split:
- **005-05 Id collision quarantine decision** — decided by C-46 (quarantine until one nonce has been seen for 3 s). The rejoin timeline it was to prove moved into 005-07 (Work 5, AC 5) and 005-20 (S4).
- **005-12 Pending timeout decision** — decided by C-45 (no timeout; ends when the target appears or the operator cancels; shown as unsatisfied). Applied in 005-13, 005-14, 005-15, 005-16, 005-25.

Added since the split: 005-23 (regime stamp coupling: C-18, C-47, C-103, C-106), 005-24 (health guards and Demoted: C-19, C-101, C-102, C-108, C-109, C-111, C-124), 005-25 (Pending and demotion scenarios: C-45, C-109, C-124), 005-26 (connection alarms: C-173, C-174, C-139), 005-27 (fan-in A inheritance and alignment opt-out: C-142, C-165), 005-28 (changed A propagation and fan-in re-alignment: C-198, C-186), 005-29 (sink switch to a changed A: C-214).

No ticket was added or deleted after C-220 to C-225: 005-08 shrinks from S to XS because C-221 decides the tie-break it had to decide (it now only writes the winner function and its test table); 005-01, 005-02, 005-09, 005-10, 005-11, 005-15, 005-16, 005-19, 005-22, 005-24, 005-25 and 005-29 were corrected in place without changing their effort.

No ticket was added or deleted in the C-177 to C-212 sweep: the rules that apply here (C-185, C-188, C-208, C-211 and the rest listed under "Settled since the split") refine existing tickets.

No ticket was deleted in the C-127 to C-176 refinement. The revised C-104 and C-105 (whole-sample L, rational L withdrawn) and C-134 (resampler only inside an edge module) touch no ticket of this epic: the control machines never handled structural latency, so there was no rational-number handling, rounding of L or fractional delay line to remove.

## Effort tally
- XS: 4 (1 day)
- S: 17 (17 days)
- M: 6 (15 days)
- Total: 27 tickets, about 33 days (33) — scope L under C-126

## Coverage

### Deliverables

| Epic deliverable | Tickets |
|---|---|
| D1 `control.md`: state list with data and fixed memory bound | 005-01, 005-06, 005-09, 005-13, 005-15 |
| D1 input alphabet (messages, timers, local events) | 005-06, 005-09, 005-13, 005-15, 005-23, 005-28, 005-29 |
| D1 complete transition tables incl. "ignore" | 005-07, 005-10, 005-14, 005-16 (guards from 005-24), 005-26, 005-27, 005-28, 005-29 |
| D1 outputs and events per transition | 005-07, 005-10, 005-14, 005-16 |
| D1 invariants preserved per transition, by id | 005-07, 005-10, 005-14, 005-16, 005-17 |
| D1 connection outputs feeding C-159 regime stamp counters | 005-23, 005-16 |
| D1 rows raising C-173 connection alarms | 005-26 (rows from 005-14, 005-16) |
| D2 machine-readable form (`control.toml`; tool output formats in the tools' documentation, C-206) | 005-02, 005-03, 005-07, 005-10, 005-14, 005-16, 005-23, 005-28 |
| D3 `RefuseReason` (18: `BadMemberList`, C-173; `ControlDegraded`, `NoSkewBudget`, C-185), `ConnectionState`, `ProducerState`, `ControlEvent` | 005-04 (alarm carrier from 005-26) |
| D4 scenario: cold start of three hosts | 005-19 |
| D4 scenario: holder loss | 005-19 |
| D4 scenario: partition and heal | 005-19 |
| D4 scenario: id collision | 005-20 |
| D4 scenario: refusal for each reason | 005-21 |
| D4 scenario: hello suppressed 30 s with live streams | 005-20 |
| D4 scenario: hello with a changed address during a live stream | 005-20 |
| D4 scenario: Pending persists and is cancelled | 005-25 |
| D4 scenario: persistently late at the ceiling, demoted | 005-25 |
| Tie-break among candidates with usable time, decided by C-221 (with C-213, C-113): winner function and test table | 005-08 |
| Open sub-question: event per non-admission refusal (this epic) | 005-04 |
| Open sub-question: admission machine split (this epic) | 005-13 |
| Open sub-question: equal-epoch merge (this epic) | 005-10 |
| Open sub-question: discovery details, removal of a live-pinned peer (this epic) | 005-06 |
| Open sub-question: producer machine (this epic) | 005-15 |
| Open sub-questions: new session, exits from Locking/Aligning/Demoted (this epic) | 005-24 |
| Open sub-question: locking-member silence carrier (this epic with 004) | 005-16 |
| Open sub-question: reaction to an unusable endpoint clock, C-162 (this epic) | 005-24 |
| Open sub-question: requester's rows for a peer that never answers (this epic; the C-139 case is a `ControlDegraded` refusal, C-185) | 005-14 |
| Open sub-question: alarm carrier, C-173 (this epic) | 005-26 |
| Open sub-question: arcs of an opted-out input, C-165 (this epic; the fan-in refusal is `NoSkewBudget`, C-185) | 005-27 |
| Open sub-questions owned elsewhere: I/O pressure (task 002), the Locked state (servo core, epic 008a, C-187; candidacy rule C-213), C-35 follower cap (L1, task 019) | recorded in 005-22 |

### Acceptance criteria

| Epic acceptance criterion | Tickets |
|---|---|
| 005-AC-1 every (state, input) pair has an explicit row | 005-03, 005-07, 005-10, 005-14, 005-16 |
| 005-AC-2 invariant analysis L-1 to L-8, `ControlDegraded` emitter named | 005-17 (with row annotations in 005-07, 005-10, 005-14, 005-16) |
| 005-AC-3 worst-case memory with 63 peers (C-217) / 64 followers and work per input | 005-06, 005-07, 005-09, 005-10, 005-13, 005-14, 005-15, 005-16, 005-18 |
| 005-AC-4 partition heal to one holder within 1.4 s of restored connectivity (system AC-36) | 005-10, 005-11, 005-19 |
| 005-AC-5 id collision detected within 1 s, rejoin within 2.5 s, quarantine per C-46 (system AC-22) | 005-07, 005-20 |
| 005-AC-6 closed refusal enumeration of 18 (C-173, C-185), a scenario per reason, Pending unsatisfied without timeout, only Live satisfied (system AC-12, C-45, C-154) | 005-04, 005-14, 005-15, 005-21, 005-25 |
| 005-AC-7 contributes to AC-20 (a), (b), (d); (c) owned by epic 015 (C-144) | 005-17, 005-20 |
| 005-AC-8 C-159 counters and C-173 alarms mapped to transitions; follower cap with synthetic followers (C-114) | 005-16, 005-23, 005-26, 005-10, 005-21 |

### Definition of done

| Item | Tickets |
|---|---|
| Tests green in CI, unprivileged | 005-03 and every ticket with tests; verified in 005-22 (no test of this epic needs a privilege, C-146) |
| Every parser touched is fuzzed | 005-03 (table loader, C-95 budget); verified in 005-22 |
| Regime stamp fields touched are complete and documented | 005-23 (C-18, C-103, C-106, C-140, C-152, C-159, C-204); verified in 005-22 |
| Design documents updated, constants table first | 005-01, 005-22 |
| No new open item without owner and falsifier | 005-04, 005-06, 005-08, 005-10, 005-13, 005-14, 005-15, 005-16, 005-24, 005-26, 005-27, 005-22 |
| The frozen POC is not modified | 005-01, 005-22 |
| Every new factual claim carries an evidence tag | all tickets |
| Transition tables total, checked mechanically | 005-03, 005-07, 005-10, 005-14, 005-16, 005-26, 005-27, 005-28, 005-29 |
| Model check: no two holders after convergence | 005-11 |

No checklist line is "not applicable" in this epic (C-125); see the note under the epic's definition of done.

## Remaining gaps
1. **Wire message names and fields** come from epic 004: hello (stamp version, authentication counter used as liveness counter), regime stamp message and unicast request, admission and MTU probe messages, operator force and operator cancel, and the observed-DSCP stamp fields (C-152). Affects 005-06, 005-09, 005-13, 005-15, 005-23 and transitively 005-07, 005-10, 005-14, 005-16. Owner: wire specification epic 004.
2. **State lists of machines 1, 2 and 3** are proposed by tickets, not fixed by the epic. Affects 005-06, 005-09, 005-13. Owner: this epic (005).
3. **Table file format and enumeration module path** ("`control.toml` or similar", `control::types` proposed). Tool location is settled (C-133); tool output formats and file names go in the tools' documentation, not the specification (C-206). Affects 005-02, 005-04. Owner: this epic (005).
4. **Event emitted by each non-admission refusal** (`VersionMismatch`, `ForeignDomain`, `ModuleCap`, `NameTaken`, `FollowerCap`); where VersionMismatch, NameTaken and ModuleCap arise is settled (C-185: at discovery). Affects 005-04, 005-21. Owner: this epic (005).
5. **Admission: one machine per role or one joint machine.** Affects 005-13, 005-14. Owner: this epic (005).
6. **Merge of equal election epochs naming different holders.** Affects 005-10, 005-11. Owner: this epic (005).
7. **Holder-eligibility inputs:** whether the I/O-pressure rule is inherited (task 002); the definition of the Locked state (servo core, epic 008a, which publishes the lock state and bound, C-187; C-62 pending simulation; the post-lapse state is settled, C-162, and so is the candidacy rule, C-213: Locked, or in holdover with the claim not lapsed); the operator-force message (epic 004). Affects 005-08, 005-09, 005-10. Owners: task 002, epic 008a and the servo simulation (a cross-reference, not a dependency, C-196), epic 004.
8. **Discovery details:** removal of a live-pinned peer after its last connection ends (a version-mismatched module is stopped from joining and not inserted, C-185). Affects 005-06, 005-07. Owner: this epic (005).
9. **Producer machine** initial state, entry to Sending, restart after Stopped. Affects 005-15, 005-16. Owner: this epic (005).
10. **Reaction to slices with a new session number** (source restart, C-19) in Live or Interrupted. Affects 005-24, 005-16. Owner: this epic (005).
11. **Exits from Locking, Aligning and Demoted** when data stops or an end-of-stream marker arrives, and the reaction of a post-admission connection when an endpoint clock becomes unusable (C-162). Affects 005-24, 005-16. Owner: this epic (005).
12. **Carrier of a locking member's "declared silent" status.** Affects 005-16. Owner: this epic (005), with epic 004 for the carrier.
13. **Followers per holder, C-35 = 64**, now pending measurement. Falsifier: holder serving lateness grows with followers below 64 ("no growth" is within 20 % from 1 to 64 followers, C-167). Affects 005-09, 005-10, 005-18, 005-21 only if the value changes. Owner: L1 measurement (task 019).
14. **Requester's rows when the peer never answers.** A module whose housekeeping restarts have stopped is no longer such a case: its supervisor refuses with `ControlDegraded` (C-185). Affects 005-14. Owner: this epic (005).
15. **Carrier of a C-173 connection alarm** (new `ControlEvent` variant or derived). Affects 005-26, 005-04. Owner: this epic (005).
16. **Arcs of an input opted out of alignment** (C-165). The refusal of a fan-in's aligned inputs with no downstream group and no budget is settled (`NoSkewBudget`, C-185). Affects 005-27, 005-16. Owner: this epic (005).

## Settled since the split
- Quarantine duration for a colliding id — C-46 (ticket 005-05 deleted).
- Pending timeout — C-45 (ticket 005-12 deleted); former gap 10 (no falsifier) closed with it.
- Former gap 4, `ControlEvent::ConnectionState` payload — epic change: deliverable 3 defines `ConnectionState` (11 states, including Demoted) and `ProducerState`.
- Former gap 5, "connection shows unsatisfied" — C-45 plus epic change: unsatisfied is a displayed property of Pending and Refused, not a state.
- Former gap 6, slice period — C-02 (48 frames, 0.5 ms) with C-01 (96 kHz only).
- Former gap 7, meaning of C-50, C-53 ceiling and C-54 — C-50 read as the 4 s aligning duration, C-101 (dwell as a test), C-109 (Demoted at the ceiling).
- Former gap 8, start of the dead threshold — C-111.
- Former gap 13, start of the 1.4 s heal bound — epic change (005-AC-4: counted from restored connectivity).
- Former gap 17, L-7 / L-8 and the `ControlDegraded` emitter — epic change (by construction; emitted by the housekeeping supervisor).
- Former gap 18, regime stamp and fuzzing lines of the definition of done — C-18, C-47, C-95, C-125, with new ticket 005-23.
- Former gap 11, in part — election with no locked candidate settled by C-113; the rest stays in remaining gap 7.
- Former gap 14, in part — stall rule settled by C-115 with C-42; the rest stays in remaining gap 8.
- Former gap 3, in part — checker and explorer location settled by C-133 (`hadronyche-tools`), output location by C-130; the rest stays in remaining gap 3.
- Former gap 19 and former remaining gap 13, owner of system AC-20 (c) — C-144: the hosted backend, epic 015.
- Epic defects fixed: task 002 added to the header's dependencies; acceptance criteria given ids 005-AC-1 to 005-AC-8; scope recomputed from this tally (C-126).
- Domain 0 hellos (part of remaining gap 8) — C-173: ignored and counted, not refused; ForeignDomain is for a different non-zero domain.
- Refusal enumeration — C-173 adds a 16th reason, `BadMemberList`, and fixes when `PathMtuTooSmall` applies (applied in 005-04, 005-13, 005-14, 005-21).
- Display of unsatisfied connections — C-154: only Live is satisfied (applied in the epic, 005-13, 005-14, 005-15, 005-25).
- Per-connection stamp counters — C-159 completes C-47; the connection machine owns interrupted events, dead and demoted rounds (005-03, 005-16, 005-23).
- Other-epoch slice — C-135: restarts the dead clock, is not a miss, has its own counter (005-15, 005-16, 005-24).
- Last ratchet step — C-163: steps to A exactly (005-15, 005-16, 005-24, 005-25).
- Holder stall detection — C-136 with C-67 (005-09, 005-10).
- Clock state after a holdover lapse — C-162: Unlocked, time unusable; not a locked candidate (005-08, 005-09, 005-10); the connection reaction stays in remaining gap 11.
- Commissioned-only numeric E — C-140 (005-04, 005-13, 005-14, 005-21, 005-23).
- Peer DSCP verification — C-152 (005-06, 005-07, 005-09, 005-23).
- Sink-only device liveness — C-168: producer timeout only (005-15, 005-16).
- Housekeeping supervisor policy and control isolation — C-139, C-161 (epic background, 005-17); alarms and fan-in A gave new tickets 005-26 (C-173, C-174) and 005-27 (C-142, C-165).
- Owner of C-35 changed to L1 (task 019): recorded as remaining gap 13.
- C-104, C-105 (revised) and C-134: no effect on this epic.
- Carrier of a changed A — C-198 with C-186: the receiver report carries the consumer's current A, one hop per report; fan-ins that inherited the old A re-align. No ticket covered a change of A after admission: new ticket 005-28 (005-27 and the epic point to it; epic 004 asked for the report's A field).
- Sync requests above 200 per second — C-204: dropped before any work, counted per follower in the holder's stamp (005-09, 005-10, 005-23; counter allowed in 005-02, 005-03).
- Presentation details of tools — C-206: the checker's and explorer's output formats and file names are recorded in their own documentation, not in `control.md` (005-03, 005-11; part of remaining gap 3).
- Refusal reasons completed at 18 — C-185 adds `ControlDegraded` (answered by the housekeeping supervisor's minimal refusal path) and `NoSkewBudget`; VersionMismatch, NameTaken and ModuleCap arise at discovery (epic, 005-03, 005-04, 005-06, 005-07, 005-13, 005-14, 005-17, 005-21, 005-26, 005-27). Closes the reason half of remaining gap 16, the C-139 example of gap 14 and the version-mismatch half of gap 8.
- Ring fit after a changed inherited A — C-211: the commissioning tool refuses a change that does not fit; a fan-in reached anyway keeps its deepest ring as ceiling, raises cannot-meet-A and keeps streaming (epic, 005-26, 005-28). Former remaining gap 17 (design owner) closed.
- Re-alignment position after a changed A — C-208 with C-43: each fan-in picks its own, one lead ahead, announced in its own output (epic, 005-28).
- Bundle cap with a controller member — C-08 revised with C-209: the controller member counts towards the 32 (epic, 005-02, 005-13, 005-14, 005-21).
- Sync requests on the clock thread — C-188 (epic, 005-09, 005-17, 005-23).
- Publisher of the lock state — C-187: servo core, epic 008a; owner of "locked" in remaining gap 7, recorded as a cross-reference under C-196 (epic, 005-08, 005-09, 005-22).
- Holdover entry delay — C-195: a follower is already in holdover when it calls an election (epic, 005-09, 005-19).
- Cannot-meet-A window — C-182: rolling 60 s window (epic, 005-02, 005-26).
- What this epic supplies to the conformance vectors — C-193 (epic dependencies).
- Constants copied into the epic grow from 73 to 84 ids (C-08, C-43, C-182, C-185, C-187, C-188, C-193, C-195, C-208, C-209, C-211; 005-01, 005-02).
- C-177 to C-212 otherwise have no subject in this epic (thread pinning, commissioning windows, loss soak, transport state, resamplers, L2/L3 parameters, kit epics).
- Who may take over as holder — C-213 with C-113 revised: a candidate needs usable time (Locked, or in holdover with the claim not lapsed, C-65, C-169) and takes over with the timeline epoch kept; the epoch is bumped only when no candidate has usable time. The provisional "locked candidate" reading is corrected (epic, 005-08, 005-09, 005-10, 005-11, 005-19, 005-22); remaining gap 7 narrows to the Locked state itself.
- Holdover entry delay — C-195 now a hard limit at C-37 (1.5 s); its value stays pending simulation, epic 008b (epic, 005-09, 005-19; S2 now keeps the timeline epoch).
- Ratchet step position — C-215: a re-alignment at a position announced one lead ahead, never silently on the next round (epic, 005-15, 005-16, 005-24, 005-25); the sink case is settled by C-225.
- Sink switch to a changed A — C-214: at its own position, one lead ahead, no announcement. No ticket covered a sink's own change of A (005-28 covers inherited A only): new ticket 005-29 (epic, 005-22, 005-28).
- Peer map capacity — C-217: 63 peers. The provisional 64-entry map and "64 peers" memory bounds are corrected (epic 005-AC-3, 005-06, 005-07, 005-09, 005-13, 005-15, 005-18, 005-21; constant added in 005-02).
- Constants copied into the epic grow from 84 to 88 ids (C-213, C-214, C-215, C-217; 005-01, 005-02).
- Ratchet step in a sink — C-225: a sink has no output to announce in, so it takes the step at its own position one lead (C-43) ahead, as for a changed A (C-214). Closes former remaining gap 17 (design owner); the open item is removed from 005-24, and 005-15, 005-16, 005-25 (new sink variant of S10) and the epic state it.
- Tie-break among candidates with usable time — C-221: a Locked candidate is preferred over one in holdover; among equals the lowest module id wins; consistent with C-213 (who may be a candidate) and C-113 (no candidate with usable time: lowest id, epoch bumped). Closes the epic's tie-break sub-question owned by 005-08, which shrinks from S to XS (epic, 005-08, 005-09, 005-10, 005-11, 005-19, 005-22). After a holder loss every follower is in holdover (C-195), so the lowest-id eligible follower with its claim not lapsed takes over (005-19 S2).
- Changed A at a sink — C-224 (a commissioned change reaches the sink from the commissioning tool, inside a declared window, buffer memory validated first, announced to the group's members) and C-222 (the C-211 clamp and C-216 apply to sinks too): 005-29 names the announcement as the source of "own A changed" and adds the clamp outcome to its row; the epic's alarm and changed-A paragraphs updated.
- Constants copied into the epic grow from 88 to 92 ids (C-221, C-222, C-224, C-225; 005-01, 005-02). C-220 and C-223 (L3 assertions, handover test drift) have no subject in this epic.
