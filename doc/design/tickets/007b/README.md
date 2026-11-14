# Epic 007b — Scenario and reference vectors: tickets

## Summary
Epic 007b (phase P1, scope L (24 tickets, about 37.5 days), depends on epics 002–006 and 007a, blocks 009–014, 025, 026) is the second part of epic 007 (Conformance vectors), split under C-143; a reference to "epic 007" means both parts. It builds on 007a (formats, reader, `UnderTest`, generator codec, wire vectors, parser fuzz targets) and delivers scenario vectors for the control state machines (discovery, election and holder including holder stall and holdover lapse, admission including the C-173 and C-142 details, connection lifecycle, alignment health, connection alarms, reassembly, stamp distribution, commissioning window, live change of A) with a mechanical transition-coverage check; timeline vectors; the POC transport scenario and gap-semantics vectors for transport state and dense controllers (C-141, C-158); exact-tier reference data for the reference echo and mixer in f32 and Q1.31, checked on x86-64 and 64-bit ARM (C-151); state-machine fuzz targets and the C-95 CI jobs; the full conformance run; and `doc/design/spec/conformance.md` (C-130).

## Tickets

| Id | Former id | Title | Effort | Depends on |
|---|---|---|---|---|
| 007b-01 | 007-14 | Discovery scenario vectors | M | 007a-04, epic 004, epic 005, epic 006 |
| 007b-02 | 007-15 | Election scenario vectors | S | 007a-04, epic 004, epic 005, epic 006 |
| 007b-03 | 007-33 | Holder follower cap and sync request rate vectors | S | 007b-02 |
| 007b-04 | new (C-136, C-162) | Holder stall and holdover-lapse scenario vectors | M | 007b-02 |
| 007b-05 | 007-16 | Admission scenario vectors | M | 007a-04, epic 004, epic 005, epic 006 |
| 007b-06 | new (C-142, C-173) | Admission details scenario vectors | M | 007b-05 |
| 007b-07 | 007-17 | Connection lifecycle scenario vectors | M | 007b-05 |
| 007b-08 | 007-31 | Alignment health scenario vectors | M | 007b-07 |
| 007b-09 | new (C-173, C-174) | Connection alarm scenario vectors | S | 007b-05, 007b-07, 007b-08 |
| 007b-10 | 007-32 | Multi-packet reassembly scenario vectors | S | 007a-04, 007a-07 |
| 007b-11 | 007-30 | Regime stamp distribution scenario vectors | M | 007a-04, 007a-09, epic 005 |
| 007b-12 | new (C-107, C-153) | Commissioning window scenario vectors | S | 007a-04, 007a-08, 007a-09, epic 005 |
| 007b-13 | 007-18 | Transition coverage check | S | 007b-01 to 007b-12, 007b-24 |
| 007b-14 | 007-19 | Timeline vectors | S | 007a-02, 007a-04, epic 004, epic 005, epic 006 |
| 007b-15 | 007-20 | POC transport scenario vector | M | 007a-07, 007a-04, epic 002, epic 003 |
| 007b-16 | new (C-141, C-158) | Gap semantics vectors: transport state and dense controllers | S | 007a-07, 007b-15 |
| 007b-17 | 007-21 | Reference instrument data from the POC | S | 007a-02, epic 002, epic 003 |
| 007b-18 | 007-22 | Reference echo and mixer vectors | M | 007b-17 |
| 007b-19 | 007-23 | Cross-machine reference hash check | S | 007b-18 |
| 007b-20 | 007-25 | Fuzz targets for state machines | S | 007a-15, 007b-13 |
| 007b-21 | 007-26 | Fuzz budget and CI job | S | 007a-15, 007b-20 |
| 007b-22 | 007-27 | Full conformance CI run | S | 007a-10, 007a-12, 007a-13, 007a-14, 007b-13, 007b-14, 007b-15, 007b-16, 007b-19, 007b-21, 007b-24 |
| 007b-23 | 007-28 | Conformance specification document | S | 007b-21, 007b-22 |
| 007b-24 | new (C-186) | Live A change scenario vectors | S | 007b-06, 007b-08 |

No ticket was deleted: none existed only for the withdrawn rational-L rule (C-105); the one rational-L statement (in 007b-18) is replaced by whole-sample L. 007b-11 grew from S to M (full C-159 counter set, C-152 DSCP verification, C-160 percentiles). Five tickets are new (007b-04, 007b-06, 007b-09, 007b-12, 007b-16). Other changes: 007b-01 (C-173 domain 0), 007b-03 (C-35 owner, C-167), 007b-05 (C-140, C-154, C-173 reason count), 007b-07 (C-135, C-168), 007b-08 (C-159, C-163), 007b-10 (C-156, C-159), 007b-14 (C-135), 007b-18 (C-105, C-122), 007b-19 (C-151), 007b-20 and 007b-23 (C-164, C-151), 007b-21 (C-146). After the last rulings: 007b-24 is new (C-186); 007b-01 (C-185 discovery reasons), 007b-02, 007b-14 (C-193), 007b-05 and 007b-06 (C-185), 007b-12 (C-178), 007b-16 (C-184, C-192), 007b-13, 007b-20, 007b-21, 007b-22 (C-192, C-193, 007b-24). After the final rulings: 007b-03 (C-204), 007b-16 (C-199, C-200), 007b-24 (C-198); no ticket added. In the last consistency sweep (C-177 to C-212): 007b-04 grew from S to M (C-169 unknown-E lapse, C-180 relock of a commissioned sink, C-195 holdover entry before the election); 007b-02 (C-195), 007b-05 and 007b-06 (C-209 controller member in the 32), 007b-09 (C-182), 007b-11 (C-182, C-157), 007b-12 (C-178 disqualification now tested), 007b-16 (C-181, C-205), 007b-24 (C-208, C-211); no ticket added or deleted. After the last design rulings (C-213, C-195 hard limit, C-215, C-216, C-181 revised): 007b-02 (C-213 takeover by any candidate with usable time, epoch kept; the epoch bump only when none has usable time; C-195), 007b-04 (C-195 hard limit with a scenario at 1.5 s, C-213), 007b-08 (C-215 ratchet step at an announced position), 007b-09 (C-216), 007b-16 (C-181 permissive: a single-stream source may carry its controllers in a second stream while they fit), 007b-24 (C-216 alarm held until a fitting change, receiver report carries the given A); no ticket added or deleted, no effort changed.

## Effort tally
- XS: 0 (0 days)
- S: 15 (15 days)
- M: 9 (22.5 days)
- Total: 24 tickets, about 37.5 days → scope L (C-126)

## Coverage

### Deliverables

| Part deliverable | Tickets |
|---|---|
| 1. `vectors/control/*.json` (control state machines and settled behaviours) | 007b-01 to 007b-12, 007b-24 |
| 1. `vectors/timeline/*.json` | 007b-14 |
| 1. `vectors/transport/*.json` (POC transport scenario; gap semantics C-141, C-158) | 007b-15, 007b-16 |
| 1. `vectors/reference/*` (echo and mixer, f32 and Q1.31, with gaps; hash and script format chosen here) | 007b-17, 007b-18, 007b-19 |
| 2. Transition coverage check | 007b-13 |
| 3. State-machine fuzz targets and C-95 CI jobs | 007b-20, 007b-21 |
| 4. Full conformance run against the generator-codec adapter | 007b-22 |
| 5. `doc/design/spec/conformance.md` (C-130) | 007b-23 (budget input from 007b-21) |

### Acceptance criteria

| Criterion | Tickets |
|---|---|
| 007-AC3 — every transition row exercised, checked mechanically | 007b-01 to 007b-12, 007b-24, 007b-13 |
| 007-AC5 — transport vector reproduces POC clicks for block sizes 48 and 7 | 007b-15 |
| 007-AC6 — reference vectors identical on x86-64 and 64-bit ARM (C-151), both formats | 007b-18, 007b-19 |
| 007-AC7 — system AC-1: 100 % pass both directions (against the generator-codec adapter), zero panics/hangs within the C-95 budget | 007a-15, 007b-20, 007b-21, 007b-22 |

### Definition of done
Each DoD line of the part is copied verbatim into the tickets it applies to. "The fuzz budget ... is stated in `conformance.md`" is in 007b-21 and 007b-23; "No vector is extracted from the POC before task 002 ..." is in 007b-15, 007b-17, 007b-18. "Regime stamp fields touched are complete and documented" has no subject in this part (it reads stamp fields defined by epic 004 and adds none); per C-125 it is marked not applicable in the part and in 007b-11, 007b-12 and 007b-23. "Unprivileged" applies to per-change CI (C-146); no ticket needs the privileged job.

## Settled since the split
- Rational L withdrawn: L is always a whole number of samples (C-105 revised); 007b-18 now uses whole-sample L and states the C-122 stamp rule exactly (output stamped p degraded for an input gap at p, emitted while input p + L is processed). The resampler node exists only inside an edge module (C-134) and C-104 pads with whole-sample delay lines; neither creates work here.
- Machine pair for 007-AC6: x86-64 and 64-bit ARM (C-151). 007b-19.
- Bounded time per input: C-164 (pending L1); 007b-20, 007b-23.
- C-35 falsifier owner is L1, task 019; serving lateness defined by C-167; 007b-03.
- Other-epoch slices restart the dead clock and have their own counter (C-135); 007b-07, 007b-14.
- Sink-only device liveness by producer timeout (C-168); 007b-07.
- Last ratchet step lands on A (C-163), demoted rounds counter (C-159); 007b-08.
- Incomplete reassemblies counter (C-159), drain cycle (C-156); 007b-10.
- Admission: 16 refusal reasons, PathMtuTooSmall, ForeignDomain versus domain 0, BadMemberList (C-173); commissioned-only numeric E (C-140); fan-in A inheritance (C-142); satisfied means Live (C-154); 007b-01, 007b-05, 007b-06.
- New behaviours with new tickets: holder stall and holdover lapse (C-136, C-162; 007b-04), connection alarms and cannot-meet-A (C-173, C-174; 007b-09), commissioning window life cycle (C-153; 007b-12), transport state and dense controllers on a gap (C-141, C-158; 007b-16).
- Stamp distribution: full counter set (C-159), percentiles (C-160), peer DSCP (C-152); the message is defined by epic 013 and sent by epic 015 (C-144); 007b-11.
- Privileged tests are a separate nightly job (C-146); this part uses none; 007b-21.
- Refusal reasons completed, 18 in all: ControlDegraded and NoSkewBudget added (the latter is the fan-in no-budget refusal); VersionMismatch, NameTaken and ModuleCap raised at discovery, so a 65th module is stopped from joining with ModuleCap (C-185); 007b-01, 007b-05, 007b-06.
- Which specification supplies what: epic 004 wire layouts and reject reasons, epic 005 state tables and scenario scripts for discovery, election, admission and connection, epic 006 timeline and epoch scenarios (C-193); 007b-01, 007b-02, 007b-05, 007b-13, 007b-14, 007b-20.
- Transport state before any has arrived: stopped, playhead at zero, default tempo (C-184); the reference dense controller is the mixer's volume (C-192); 007b-16.
- A changed while inputs are live: inheriting fan-ins re-enter aligning at an announced position with rings and ratchet ceiling resized (C-186); new ticket 007b-24.
- An open commissioning window disqualifies ordinary production measurements only; the tool's own measurements are valid and recorded with their window (C-178); 007b-12.
- Vectors crate at the workspace root; fuzzing tool is the implementing team's choice (C-192); 007b-20, 007b-21.
- Sync requests above 200 per second from one follower are dropped before any work and counted per follower in the holder's stamp (C-204); 007b-03 adds the over-rate scenario.
- The changed A travels upstream in the receiver report, which carries the consumer's current A, one hop per report (C-198); 007b-24 adds a two-hop chain scenario.
- Default tempo 120 beats per minute (C-199); a stopped playhead does not advance through a gap, a playing one moves on at its tempo (C-200, C-141); 007b-16 vectors corrected for the stopped case.
- Holdover entry delay at most 1.5 s, a hard limit, so a follower is always in holdover when it calls an election; its value within the limit is pending simulation (epic 008b), which reports a finding if none works (C-195); lapse with E unknown at 2 × C-62 (C-169); a commissioned sink stays commissioned while unlocked, its claim suspended and resumed on relock without re-commissioning (C-180); 007b-04 (now M), 007b-02.
- The controller member counts towards the 32 members of a bundle; a bundle source's controllers ride in their own member stream (C-08, C-205, C-209); 007b-05, 007b-06 (33-in-all case and 31-plus-controller boundary). Where controllers travel (C-181); 007b-16 runs the dense-controller vectors on the controller member and, for a single-stream source, in the audio packet and in a second stream (C-181 revised, below).
- Lateness figures, reclaimable latency and the cannot-meet-A alarm use a rolling 60 s window, maximum since live beside them (C-182, C-157); 007b-09, 007b-11.
- After a changed A each fan-in picks its own position one lead ahead and announces it (C-208); a changed A beyond a fan-in's buffer memory keeps the deepest ring, raises cannot-meet-A, streams continue (C-211); 007b-24.
- What an open commissioning window disqualifies (C-178) is now tested, not only stated; 007b-12.
- Who may take over as holder: any holder-eligible module with usable time (Locked, or in holdover with its claim not lapsed), with frequency continuity and the epoch kept; the epoch is bumped by the lowest-id candidate only when none has usable time (C-213, C-113); 007b-02 corrected (its "no candidate locked" case read as "no candidate with usable time", usable-time takeover scenarios added), 007b-04.
- A ratchet step takes effect at a position the fan-in announces one lead ahead in its own output, never silently on the next round (C-215); 007b-08.
- A clamped A: the cannot-meet-A alarm stays raised until a later change of A that fits; the fan-in's receiver report carries the A it was given, not the clamped value (C-216); 007b-24, 007b-09.
- Where controllers travel, revised to permissive: a single-stream source may always put its controllers in a second stream and must beyond the 100-byte cap; it may share the audio packet while they fit; the compiler checks this from the manifest (C-181); 007b-16 corrected (both carriages tested, the compiler check out of scope).
- No subject here: C-177, C-179, C-183, C-187 to C-191, C-194, C-196, C-197, C-201 to C-203, C-206, C-207, C-210, C-212 (C-206: the hash algorithm and script format stay this epic's own choice, recorded in `conformance.md`).
- C-144 duties (gap slice on own missed deadline, processing-overrun event, AC-20 (c), single-writer file service, sending the stamp) belong to epic 015 and are not taken here.

## Remaining gaps
1. **Which machine owns alignment health, alarms, reassembly, stamp distribution, the commissioning window and the live change of A** (epic 005 supplies the discovery, election, admission and connection tables, C-193). Affects 007b-08 to 007b-13, 007b-20, 007b-24. Owner: epic 005.
2. **Names of the remaining refusal reasons** (nine of the 18 named: RequiredEExceeded, PathMtuTooSmall, ForeignDomain, BadMemberList, ControlDegraded, NoSkewBudget, VersionMismatch, NameTaken, ModuleCap; C-103, C-173, C-185), **of the follower-cap event (C-35), the capacity refusal (C-23, C-117) and the six connection alarms (C-173).** Affects 007b-03, 007b-05, 007b-09. Owner: epic 005.
3. **Whether an epoch-switch announcement is also carried in the slice at the named position** (C-44). Affects 007b-14. Owner: epic 006 (epoch scenarios, C-193).
4. **Timeline units and rounding** (rate is 96 000 Hz, C-01; time, origin and position units, rounding and time before origin still open). Affects 007b-14. Owner: epic 006 (timeline tables, C-193).
5. **f32 hash agreement across listed targets beyond the C-151 pair.** Affects 007b-19. Owner: task 025.
6. **POC voice comparison bit-level, and the sample rate of the POC transport scenario.** Affects 007b-15, 007b-17, 007b-18. Owner: task 002.
7. **Confirmation of the C-164 bound** for state-machine steps. Affects 007b-20. Owner: L1 measurement.
