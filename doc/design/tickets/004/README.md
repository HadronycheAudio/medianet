# Epic 004 — Wire specification: tickets

## Summary
Epic 004 (phase P1, scope L: 29 tickets, about 30.5 days) writes the field-level wire specification of the Hadronyche Medianets protocol in `doc/design/spec/wire.md`: media packets (packet header, slice header, in-band announcements, measurement extension, content messages, checksum), control messages (hello, election, admission handshake, manifest, path-MTU probe pair, regime stamp sections and the stamp request, commissioning window announcement and close), sync request and reply, and the measurement messages (receiver report, one-way probe), with exact offsets, widths, byte order, versioning, reserved fields, size budgets at 1 472 and 1 252 bytes of UDP payload (IPv4 only, C-100; header cap C-137), and the required outcome for every malformed case. It also delivers a machine-readable form of the tables, checked by a tool in the `hadronyche-tools` crate (C-133), and a closed enumeration of rejection reasons. It blocks epics 005, 006, 007, 009 to 014.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 004-01 | wire.md skeleton and encoding conventions | S | none |
| 004-02 | Machine-readable tables and round-trip check | M | 004-01 |
| 004-03 | Rejection reason enumeration | S | 004-01, 004-02 |
| 004-04 | Versioning rules, magic and version fields | S | 004-01, 004-03 |
| 004-05 | Reserved fields and their required values | XS | 004-01, 004-03 |
| 004-06 | Media packet header layout | S | 004-02, 004-03, 004-04, 004-05 |
| 004-07 | Timeline epoch wrap analysis (updated for C-213) | XS | 004-06 |
| 004-08 | Slice header and send-timestamp format | S | 004-06 |
| 004-09 | In-band announcement layouts (updated for C-208) | S | 004-06, 004-08 |
| 004-10 | Media measurement extension layout | XS | 004-06 |
| 004-11 | Content messages: audio and dense controllers (updated for C-205, C-181, C-209; C-181 permissive corrected) | S | 004-08 |
| 004-12 | Content messages: sparse keyed controllers and transport state (updated for C-199, C-205; C-141, C-200 and C-181 permissive corrected) | S | 004-11 |
| 004-13 | Slice checksum algorithm, width and placement | S | 004-08 |
| 004-14 | Media packet size budget at 1 472 and 1 280 bytes (updated for C-205, C-181; C-181 permissive corrected) | S | 004-06, 004-08, 004-09, 004-10, 004-11, 004-13 |
| 004-15 | Recompute capacity figures from the final layout (updated for C-219: controller-stream wire cost) | XS | 004-14 |
| 004-16 | Hello message layout and 32-member size bound | M | 004-03, 004-04, 004-05 |
| 004-17 | Election message layouts (updated for C-213: usable time, epoch kept on takeover) | S | 004-16 |
| 004-18 | Admission handshake messages and refusal reasons (was M; manifest and probe moved to 004-26, 004-27; updated for C-185: 18 reasons, C-209; C-219: controller-stream declared rates) | S | 004-16 |
| 004-19 | Sync request and reply layouts (updated for C-204, C-213) | S | 004-03, 004-04, 004-05 |
| 004-20 | Receiver report and one-way probe layouts (updated for C-198: consumer's current A) | S | 004-03, 004-04, 004-05 |
| 004-21 | Validity rules completeness audit | S | 004-06, 004-08 to 004-13, 004-16 to 004-20, 004-24 to 004-29 |
| 004-22 | Traceability table and review against the architecture specification | S | 004-07, 004-14, 004-15, 004-21 |
| 004-23 | Design document updates and epic close-out | S | 004-22 |
| 004-24 | Regime stamp message: sections, version and fetch request (new, C-18) | M | 004-16 |
| 004-25 | Regime stamp section types named by the constants table (new, C-05, C-47, C-106, C-107, C-120, C-127, C-129, C-131; updated for C-145, C-147, C-153, C-159, C-161) | S | 004-24, 004-08 |
| 004-26 | Manifest encoding in the admission handshake (new, C-04, C-103 to C-106; updated for C-140, C-158, C-207) | S | 004-18 |
| 004-27 | Path-MTU probe pair (new, C-116; fallback sequence ends in `PathMtuTooSmall`, C-173) | S | 004-18 |
| 004-28 | Regime stamp sections: observed DSCP and alignment group (new, C-152, C-157, C-160; updated for C-204: dropped sync requests per follower; C-182: rolling 60 s window and maximum since live) | S | 004-24, 004-25 |
| 004-29 | Commissioning window announcement and close messages (new, C-153; C-178 corrected) | S | 004-16, 004-25 |

No ticket was deleted: no ruling removed a whole piece of work. The rulings shrank 004-11 (no per-channel id byte, C-04) and 004-18 (manifest and probe split out). The withdrawal of the rational-L rule (C-105 revised: L is always a whole number of samples) removed the rational-pair type from 004-02, the rational accumulation rule from 004-10 and the numerator/denominator encoding of L and join delays from 004-26; no ticket existed only for rational L, so none was deleted.

External dependency: epic 001 (report on receive timestamps and don't-fragment per OS) is non-blocking; it feeds notes in 004-08, 004-19 and 004-23. The receive-timestamp source values themselves are settled (C-131) and stamped (004-25).

## Effort tally
- XS: 4 (1 day)
- S: 22 (22 days)
- M: 3 (7.5 days)
- Total: 29 tickets, about 30.5 days: scope L (C-126)

## Coverage

### Deliverables

| Epic deliverable | Tickets |
|---|---|
| D1 `wire.md`: byte order and alignment rules | 004-01 |
| D1 field table: media packet header | 004-06 |
| D1 field table: slice header | 004-08 |
| D1 field table: each announcement type | 004-09 |
| D1 field table: measurement extension | 004-10 |
| D1 field table: each content message type | 004-11, 004-12 |
| D1 field table: checksum | 004-13 |
| D1 field table: hello | 004-16 |
| D1 field table: each election message | 004-17 |
| D1 field table: each admission message | 004-18 |
| D1 field table: manifest | 004-26 |
| D1 field table: path-MTU probe pair | 004-27 |
| D1 field table: regime stamp sections and stamp request | 004-24, 004-25, 004-28 |
| D1 field table: commissioning window announcement and close (C-153) | 004-29 |
| D1 field table: sync request and reply | 004-19 |
| D1 field table: receiver report; one-way probe | 004-20 |
| D1 size budget, default packet (1 472 bytes) | 004-14 |
| D1 size budget at the 1 280-byte fallback (1 252 bytes UDP): 6 channels with headers and checksum ≤ 100 bytes (C-137) | 004-14 |
| D1 validity rules per message with required outcome | 004-06, 004-08 to 004-13, 004-16 to 004-20, 004-24 to 004-29, consolidated in 004-21 |
| D1 versioning rules (minor/major, receiver behaviour) | 004-04 |
| D1 reserved fields and required values (copy index, authentication reservation per C-115) | 004-05 |
| D1 closed enumeration of refusal reasons: the 18 named in C-185; `VersionMismatch`, `NameTaken`, `ModuleCap` raised at discovery | 004-18, 004-27 |
| D1 traceability table, rules 1 to 22 | 004-22 |
| D2 machine-readable field tables and checker in `hadronyche-tools` | 004-02 (format and checker); every table ticket adds its entries |
| D3 closed enumeration of rejection reasons | 004-03, audited in 004-21 |

### Acceptance criteria

| Epic acceptance criterion | Tickets |
|---|---|
| AC 1 — rules 1 to 22 traced to a field or validity rule | 004-22 |
| AC 2 — 6 channels fit 1 472 bytes, spare stated; headers and checksum ≤ 100 bytes (C-137) | 004-14 |
| AC 3 — every control message, stamp section and the manifest ≤ 1 252 bytes (C-17), incl. 32-member hello (C-08); probe pair the only exemption (C-116) | 004-16, 004-17, 004-18, 004-24 to 004-29 |
| AC 4 — exhaustive invalid forms per message with reason | 004-21 (with per-message rules in 004-06 to 004-20, 004-24 to 004-29) |
| AC 5 — no destination-specific / per-receiver media field | 004-06, 004-10, 004-22 |
| AC 6 — capacity figures (C-20, C-21, C-22) recomputed with IPv4 and Ethernet overhead, difference stated; controller-stream wire cost (C-219) | 004-15 |
| AC 7 — contributes to system AC-1 and AC-23 | 004-03 (counting model, named event), 004-21 (bounds and reason lists for fuzzing and vectors), 004-02 (tables for vectors) |
| AC 8 — every wire-visible constant has a field with width and unit | 004-11, 004-16, 004-18, 004-20, 004-24 to 004-26, 004-28, 004-29; checked in 004-22 |

### Definition of done

| Epic DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | 004-02 and every ticket touching the machine-readable file; 004-23 |
| Every parser touched is fuzzed | Not applicable (C-125): no wire parser in this epic; bounds and reason lists handed over in 004-21; applies to 004-02 only if its file parser is hand-written |
| Regime stamp fields touched are complete and documented | 004-08, 004-16, 004-24, 004-25, 004-28, 004-29 |
| Design documents updated where behaviour differs, constants table first | 004-07, 004-11, 004-14, 004-15, 004-16, 004-18, 004-23 |
| No new open item without an owner and a falsifier | all; checked in 004-23 |
| The frozen POC is not modified | 004-23 (and all) |
| Every new factual claim carries an evidence tag (incl. `[standard]`, C-132) | all; checked in 004-23 |
| `wire.md` reviewed against the architecture specification, traceability attached | 004-22 |
| Machine-readable tables parse and round-trip | 004-02, 004-23 |

### Open sub-questions owned by this epic

| Question | Ticket |
|---|---|
| 16-bit timeline epoch wrap within plausible uptime | 004-07 |
| Headers and checksum fit the 100-byte cap (C-137); optional parts beside them | 004-14 |
| 32-member hello fits 1 252 bytes; hello maxima; transport role values; alarm encoding (set fixed by C-173) | 004-16 |
| Checksum algorithm and width | 004-13 |
| Send-timestamp width and unit | 004-08 |
| Magic, version width and scheme, v1 value | 004-04 |
| Election message set | 004-17 |
| Admission message set (refusal reasons settled: all 18 named, C-185) | 004-18 |
| Re-alignment and transport reference change payloads (re-alignment serves ratchet and changed A, C-208) | 004-09 |
| Dense controller and transport state field sets | 004-11, 004-12 |
| Residence-time unit and accumulated-field widths (declared delay is a whole-sample sum, C-105) | 004-10 |
| Bound on inserted join delays in the manifest | 004-26 |
| Counting key for "input" | 004-03 |
| Copy index values accepted in v1 | 004-05 |
| Authentication reservation on sync and measurement messages | 004-19, 004-20 |
| What the one-way probe measures | 004-20 |

## Remaining gaps

Only questions still unanswered by the constants table. The previous R9 (Demoted carriage) and R16 (architecture specification path) are settled and listed below.

| # | Gap | Tickets | Who answers |
|---|---|---|---|
| R1 | Election message set and fields (C-113 and C-213 constrain them: usable time, that is Locked or holdover with the alignment claim not lapsed; holder eligibility; module id; the winner's timeline epoch, kept on a takeover with usable time, bumped otherwise) | 004-17 | Epic 004 proposes; control state machines epic 005 confirms |
| R2 | Admission message set (the refusal reasons are settled: all 18 named in C-185) | 004-18 | Epic 004 proposes; epic 005 confirms |
| R3 | Magic value, version width and scheme, v1 version value | 004-04 | Epic 004 |
| R4 | Payloads of the re-alignment and transport reference change announcements (the re-alignment payload serves both a ratchet step and a changed A, each fan-in announcing its own position one lead ahead, C-186, C-208) | 004-09 | Epic 004 proposes; epic 005 confirms |
| R5 | Field sets of dense controllers (initial value is in the manifest, C-158) and read-only transport state (gap behaviour fixed by C-141 and C-200; initial state stopped, playhead zero, tempo 120 beats per minute, C-184, C-199; controller placement fixed by C-181 (permissive), C-205; controller-stream wire cost by C-219) | 004-11, 004-12 | Epic 004 |
| R6 | Unit of accumulated residence time; widths of the accumulated fields (declared delay is a whole-sample sum, C-105) | 004-10 | Epic 004 |
| R7 | Hello maxima (name, ports, content types, capability bits); transport role values; alarm field encoding (the alarm set is fixed by C-173) | 004-16 | Epic 004; epic 005 confirms the transport role values |
| R8 | Regime stamp section types beyond those the constants table names | 004-24, 004-25, 004-28 | Each epic that stamps state, as a minor change |
| R10 | Bound on inserted join delays (C-104) in the manifest | 004-26 | Epic 004 |
| R11 | Whether sync and measurement messages carry the C-12 reservation (C-12 names control messages only) | 004-19, 004-20 | Epic 004 |
| R12 | Copy-index values a v1 receiver accepts (C-07 is reserved for a deferred feature) | 004-05 | Epic 004 |
| R13 | Counting key for traffic matching no admitted input (C-16 and C-109 cover admitted inputs only; C-173 adds a counted domain-0 hello) | 004-03 | Epic 004 |
| R14 | What the one-way probe measures, given that its reply closes a round trip | 004-20 | Epic 004 |
| R15 | Plausible timeline epoch bump rate (bump events: epoch switch, an election with no candidate having usable time C-113 and C-213, holder stall C-136; a takeover with usable time keeps the epoch) | 004-07 | Epic 004 estimates from C-32, C-33, C-37; epic 005 confirms |
| R17 | Processing bound for one bad input, 10 µs per packet (C-164 is a proposal) | 004-21 | Measurement: L1 |

None needs the design owner.

## Settled since the split

Former gaps (numbering of the previous "Gaps in the epic" list):

1. IP version and header sizes: IPv4 only, TTL 1; 1 472 and 1 252 bytes of UDP payload (C-100). Closed.
2. MTU probe pair versus C-17: the probe is exempt and sent at the intended media packet size (C-116). Closed; new ticket 004-27.
3. Authentication reservation "zeroed" versus live counter: scheme and tag zero, counter live (C-115). Closed.
4. Election message set: narrowed by C-113; remains as R1.
5. Admission set, refusal reasons, required E: E declaration and `RequiredEExceeded` settled (C-103), no Pending timeout (C-45), manifest content settled (C-04, C-104, C-105, C-106); the message set and other reasons remain as R2. New ticket 004-26.
6. Magic and version: remains as R3 (owned by this epic).
7. Undescribed payloads and units: remain as R4, R5, R6.
8. Hello maxima: hello gains only the stamp version number (C-18); maxima remain as R7.
9. Regime stamp fields undefined: transport settled (C-18); contents named by C-05, C-47, C-106, C-107, C-120, C-127, C-129, C-131. Closed; new tickets 004-24 and 004-25; further section types are R8.
10. Location of `Reject`, checker language and schema: checker lives in the `hadronyche-tools` crate (C-133), specifications in `doc/design/spec/` (C-130), schema chosen by 004-02, `Reject` code location assigned to tasks 009 to 014 (epic deliverable 3). Closed.
11. DoD lines assuming code: marked not applicable with the reason (C-125, note after the epic checklist). Closed.
12. Architecture specification path: was R16; closed by C-171 (item 22).
13. C-13, C-20, C-21, C-40 missing from the epic table; Ethernet overhead: added to the epic table with C-18, C-19, C-22, C-100; 28 bytes IPv4/UDP (C-100) and 38 bytes Ethernet framing stated in 004-15, to be tagged `[standard]` with the clause (C-132). Closed.
14. Authentication reservation on sync and measurement messages: remains as R11.
15. Copy-index values: C-07 status "reserved" fixes it as a deferred feature; receiver acceptance remains as R12.
16. "Input" in AC-23: the design's meaning for admitted inputs (C-16, C-109); unmatched traffic remains as R13.
17. One-way probe naming: remains as R14.
18. Epoch-bump rate: bump events named (C-113, epoch switch; C-19 links session bumps); rate remains as R15.

Settled in the final refinement (rules C-104, C-105 revised; C-127 to C-176):

19. Structural latency L: always a whole number of samples; the rational-L rule is withdrawn (C-105 revised); join padding is a whole-sample delay line, and a join on a path with frequency-dependent delay is flagged in the manifest (C-104 revised). Removed rational handling from the epic, 004-02, 004-10 and 004-26; former R6 narrowed.
20. Resampler: only inside an edge module, its delay part of D (C-134, C-170). Epic background and 004-11, 004-26.
21. Demoted carriage (former R9): the demoted-rounds counter of C-159, in the receiver report and the regime stamp. Closed.
22. Architecture specification (former R16): `doc/design/01-architecture.md` (C-171); 004-22 identifies the section. Closed.
23. Stamp and report counters: the eleven counters of C-159, other-epoch per C-135. 004-20, 004-25.
24. Header cap: all headers and the checksum at most 100 bytes beside 1 152 bytes of audio; optional parts omitted on a fallback path (C-137). 004-01, 004-06, 004-08 to 004-11, 004-13 to 004-15; the size sub-question is restated against the cap.
25. Epoch bump by holder stall (C-136). 004-07, 004-19; R15.
26. Admission details (C-173): 16 refusal reasons, four named; domain-0 hello ignored and counted; alarm set. 004-03, 004-06, 004-16, 004-18, 004-21, 004-27; R2 and R7 narrowed.
27. Declared E numeric only for commissioned sinks (C-140); dense controller initial values in the manifest (C-158); downstream A learned at admission (C-142). 004-18, 004-26.
28. Stamp contents: panic injection (C-145), device capabilities by construction (C-147), control isolation (C-161), calibration procedure (C-176) in 004-25; observed DSCP (C-152) and alignment group (C-157, C-160) in new ticket 004-28.
29. Commissioning window life cycle (C-153): new ticket 004-29 for the announcement and close messages.
30. Transport state on a gap (C-141): receiver behaviour, no wire value. 004-12.
31. Processing bound for a bad input (C-164, proposal): linear parse required in 004-21; the 10 µs figure is R17.
32. Duties of epic 015 (C-144): gap-slice emission and sending the stamp are out of scope in 004-06 and 004-24.

Settled in the last rulings (C-198, C-199, C-204, C-205):

33. Carrier of a changed A (C-198): the receiver report carries the consumer's current A, own or inherited; a change propagates one hop per report and inheriting fan-ins re-align (C-186). New field in 004-20; epic background and measurement table.
34. Default tempo (C-199): 120 beats per minute, assumed before any transport state has arrived (C-184); the transport state includes a tempo field and needs no "unset" value. 004-12; R5 narrowed.
35. Sync requests above 200 per second (C-204, C-112): dropped before any work and counted per follower in the holder's stamp; no request field (004-19), counter added to the holder's per-follower entry in 004-28. No new ticket.
36. Controllers of a bundle source (C-205): always in their own member stream; validity rules for controllers in an audio member (and audio in the controller member) in 004-11 and 004-12; budget note in 004-14.

C-198, C-204 and C-205 are added to the constants traced in 004-22. No ticket added; tally unchanged.

Settled in the closing sweep (C-177 to C-212, with the late revisions of C-08, C-134, C-141, C-153, C-169, C-181, C-184):

37. Where controllers travel (C-181): a single-stream source's controllers share the audio packet within the 100-byte cap of C-137, else a second stream. Facts and a validity rule in 004-11, note in 004-12, controller allowance inside the cap in 004-14 (new work step and criterion); epic background and open sub-question.
38. Rolling window of the stamp's lateness figures (C-182): percentiles, maximum and reclaimable latency over 60 s, with the maximum since live beside them. New field, ordering rule and criterion in 004-28; epic stamp contents and AC-8.
39. Initial transport state (C-184 revised): stopped, playhead zero, tempo 120 (already in 004-12); epic background completed. Transport state on a gap (C-141 revised, C-200): a stopped playhead does not advance; 004-12 corrected (it said the playhead advanced by the positions elapsed); epic background gains the bullet.
40. Refusal reasons completed (C-185): the closed set of 18, all named (`PortCeiling`, `PacketRate`, `SliceRate`, `InputCount`, `BufferMemory`, `CostBound`, `ContentMismatch`, `NotAnInput`, `VersionMismatch`, `ForeignDomain`, `PathMtuTooSmall`, `RequiredEExceeded`, `FollowerCap`, `ModuleCap`, `NameTaken`, `BadMemberList`, `ControlDegraded` and `NoSkewBudget`), so no reason name is left to epic 005; `VersionMismatch`, `NameTaken`, `ModuleCap` raised at discovery, never in an admission refusal. 004-18 (enumeration, validity rule, criterion), 004-03 (`VersionMismatch` and `ForeignDomain` shared with packet rejections, separate values), 004-22, 004-23; epic background, rule 17, deliverable 1, AC-8 and open sub-question; R2 reduced to the message set.
41. Re-alignment after a changed A (C-186, C-208): each fan-in announces its own position one lead ahead with the re-alignment announcement, as for a ratchet step; the payload must serve both. 004-09; epic background and open sub-question; R4 narrowed.
42. Conversion delay of an edge (C-207, C-134 revised, C-189): counted only in D, reported separately in the manifest on the device-facing port. New field and validity rule in 004-26; epic background, manifest content, rule 17 and AC-8.
43. Controller member counts towards the bundle cap (C-209, C-08 revised): at most 31 audio members with controllers; `BadMemberList` counts the controller member. Epic C-08 row, 004-11, 004-16, 004-18.
44. Commissioning window disqualification (C-178, C-153 revised): production measurements are unqualified while open; the tool's own measurements are valid and recorded with the window. 004-29 corrected (it said every measurement in a window is unqualified); the window identifier is what those records reference; epic background.
45. Conformance vector supply (C-193): this epic supplies wire layouts and reject reasons; epic dependency on 007 cites it.

C-178, C-181, C-182, C-185, C-207, C-208 and C-209 are added to the constants traced in 004-22. The work fits the existing tickets' effort; no ticket added or deleted; tally unchanged.

C-177, C-179, C-180, C-183, C-187 to C-192, C-194 to C-197, C-201 to C-203, C-206 (no tool presentation question is open in this epic; the checker's output is 004-02's own), C-210 to C-212 and the revision of C-169 create no wire work in this epic. C-196 holds: the only dependency, on epic 001, is non-blocking; 026 is a consumer, not a dependency.

Settled in the last design rulings (C-181 revised, C-213, C-219):

46. Where controllers travel (C-181 revised: permissive): a single-stream source may always put its controllers in a second stream, must when they exceed the 100-byte cap, and may share the audio packet only while they fit. 004-11 corrected (it read sharing as required while within the cap): facts, a validity rule for audio in the controller second stream, and a criterion that a controller second stream is never rejected for controllers that would have fitted; 004-12 note and 004-14 allowance restated as an upper bound for sharing; epic background and open sub-question; R5.
47. Who may take over as holder (C-213): a holder-eligible module with usable time (Locked, or holdover with the alignment claim not lapsed, C-65, C-169) takes over with frequency continuity and keeps the timeline epoch; the epoch is bumped (C-113) only when no candidate has usable time. 004-17 corrected (it made lock state alone visible and treated "no candidate locked" as the bump condition): time-state field, validity rule and criterion; 004-07 bump event and rate ceiling, 004-19 epoch note; epic background, open sub-questions; R1 and R15 narrowed.
48. Wire cost of a controller stream (C-219): one packet per slice of (headers up to the 100-byte cap + controller bytes + 66 bytes overhead) at 2 000 packets/s, used by admission and the commissioning tool. New work step and criterion in 004-15, declared-rate note in 004-18; epic background and AC-6.

C-213 and C-219 are added to the constants traced in 004-22. The work fits the existing tickets' effort; no ticket added or deleted; tally unchanged.

C-35's owner change, C-138, C-139 (beyond the alarm), C-143 (epic not split: 30.5 days), C-146 to C-151, C-154 to C-156, C-162, C-163, C-165 to C-170, C-172, C-174 and C-175 create no wire work in this epic.

Also corrected in the epic: C-04 (192 bytes = 48 × 4, no id byte; the old "1 id byte + 48 × 4" did not add up), C-01 (one rate, one timeline per domain), the "hop limit" wording (IPv4 TTL), the unnumbered control, sync and measurement rules (now rules 14 to 22), task 026 added to the dependencies, and the placeholder count in 004-01 (the old list had 18 sections, not 19; it now has 21).
