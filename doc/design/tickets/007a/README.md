# Epic 007a — Wire vectors and harness: tickets

## Summary
Epic 007a (phase P1, scope L (15 tickets, about 22.5 days), depends on epics 004–006, blocks 007b, 009–014, 025, 026) is the first part of epic 007 (Conformance vectors), split under C-143; 007b (scenario and reference vectors) builds on it, and a reference to "epic 007" means both parts. 007a delivers the crate `hadronyche-vectors` (data plus a tiny `no_std` reader) with the schemas of all six vector kinds; the `UnderTest` runner trait with its observation extension and drivers; the hosted generator `gen` in the `hadronyche-tools` crate (C-133) that derives wire vectors from the specification tables; valid wire vectors for every message type in both directions (media and control, including the sectioned regime stamp message, its request, the commissioning window messages and the path-MTU probe pair); invalid vectors for every validity rule; bad-input outcome vectors (AC-23) with the C-164 time bound; dedup vectors; and the parser fuzz targets. All figures follow the constants table `doc/design/02-constants.md`: one wire rate of 96 000 Hz (C-01), 192 bytes per channel per slice with no per-channel id byte (C-04), a default slice within 1 252 bytes of UDP payload with at most 100 bytes of headers and checksum (C-137), IPv4 only (C-100).

## Tickets

| Id | Former id | Title | Effort | Depends on |
|---|---|---|---|---|
| 007a-01 | 007-01 | Crate skeleton and vector layout | XS | none |
| 007a-02 | 007-02 | Vector file formats and version stamp | M | 007a-01 |
| 007a-03 | 007-03 | No-std vector reader | S | 007a-02 |
| 007a-04 | 007-04 | UnderTest runner trait | S | 007a-03 |
| 007a-05 | 007-05 | Generator core and header vectors | M | 007a-02, epic 004 |
| 007a-06 | 007-06 | Valid vectors: announcements and measurement extension | S | 007a-05 |
| 007a-07 | 007-07 | Valid vectors: content messages and slice checksum | M | 007a-05 |
| 007a-08 | 007-08 | Valid vectors: control messages | S | 007a-05, epic 004 |
| 007a-09 | 007-29 | Valid vectors: regime stamp message and request | S | 007a-05, epic 004 |
| 007a-10 | 007-09 | Message-type coverage check and generator reproducibility | S | 007a-06, 007a-07, 007a-08, 007a-09 |
| 007a-11 | 007-10 | Invalid vectors per validity rule | M | 007a-05, 007a-06, 007a-07, 007a-08, 007a-09 |
| 007a-12 | 007-11 | Rejection reason coverage check | XS | 007a-11 |
| 007a-13 | 007-12 | Bad-input outcome vectors (AC-23) | M | 007a-04, 007a-11 |
| 007a-14 | 007-13 | Dedup scenario vectors | S | 007a-07, 007a-04 |
| 007a-15 | 007-24 | Fuzz targets for parsers | M | 007a-04, 007a-11, 007a-13 |

No ticket of this part was deleted: none existed only for the withdrawn rational-L rule (C-105). No effort changed. Changes in this refinement: 007a-02 (C-159 counters, C-160, C-164 bound), 007a-04 (C-136 clock input, C-145 not used, alarms as events), 007a-05, 007a-06, 007a-07 and 007a-10 (C-137 header cap), 007a-08 (C-153 window messages, C-173 PathMtuTooSmall), 007a-09 (C-135, C-152, C-153, C-157, C-159, C-160, C-161), 007a-13 and 007a-15 (C-164; C-173 domain 0), 007a-14 (C-159 duplicates counter). Later rulings: 007a-01 and 007a-15 (C-192 crate location and fuzzing tool), 007a-04 (C-193 `Machine` set), 007a-05 and 007a-08 (C-193 table sources; dependency on epic 004 only), 007a-08 (C-185 18 refusal reasons), 007a-11 (C-193 reject reasons from epic 004). Final sweep (C-177 to C-212): 007a-05 (C-206 file naming), 007a-07 (C-181, C-205 controller packaging; C-08, C-209 controller member), 007a-08 (C-198 A in the receiver report; C-209), 007a-09 (C-182 rolling window and maximum since live; C-204 dropped sync requests per follower), 007a-10 (C-181 controllers inside the header cap), 007a-11 (C-209). No ticket added, no effort changed.

## Effort tally
- XS: 2 (0.5 day)
- S: 7 (7 days)
- M: 6 (15 days)
- Total: 15 tickets, about 22.5 days → scope L (C-126)

## Coverage

### Deliverables

| Part deliverable | Tickets |
|---|---|
| 1. Crate `hadronyche-vectors` (data + tiny `no_std` reader, formats of all six kinds, reader API, data directories) | 007a-01, 007a-02, 007a-03 |
| 1. `vectors/wire/valid/*.bin` + `*.json` (min/typical/max per message type, media and control) | 007a-05, 007a-06, 007a-07, 007a-08, 007a-09, 007a-10 |
| 1. `vectors/wire/invalid/*.bin` + `*.json` (per validity rule with reason; bad-input outcomes; dedup) | 007a-11, 007a-12, 007a-13, 007a-14 |
| 2. Generator `gen` in `hadronyche-tools` (hosted only, from spec tables; C-133) | 007a-01, 007a-05, 007a-06, 007a-07, 007a-08, 007a-09, 007a-10, 007a-11 |
| 3. Runner trait `UnderTest` with observation extension | 007a-04 |
| 4. Parser fuzz targets in `hadronyche-tools` | 007a-15 |

### Acceptance criteria

| Criterion | Tickets |
|---|---|
| 007-AC1 — valid vectors both directions for every message type | 007a-05, 007a-06, 007a-07, 007a-08, 007a-09, 007a-10 |
| 007-AC2 — every rejection reason has an invalid vector, checked mechanically | 007a-11, 007a-12 |
| 007-AC4 — generator reproduces committed wire vectors byte for byte | 007a-05, 007a-10 |
| 007-AC8 — system AC-23: no crash, no state change, counted per input and reason, named version event, within the C-164 bound | 007a-04, 007a-13 (run in full by 007b-22) |

### Definition of done
Each DoD line of the part is copied verbatim into the tickets it applies to. Per C-125 the part marks three lines not applicable with the reason: regime stamp fields (encoded, not defined, here), POC extraction (none in this part) and the fuzz budget in `conformance.md` (007b-21, 007b-23). "Unprivileged" applies to per-change CI (C-146).

## Settled since the split
- Bounded time for one bad input: C-164 (linear in packet length, at most 10 µs per packet; proposal pending L1). Applied in 007a-02, 007a-13, 007a-15.
- Room for headers and in-band content in a default slice: C-137 (at most 100 bytes of headers and checksum, default slice within 1 252 bytes; announcements and measurement extension fit or are omitted on a fallback path). Replaces the earlier "about 320 bytes" assumption in 007a-06. Applied in 007a-05, 007a-06, 007a-07, 007a-10.
- Full per-connection counter set in the stamp: C-159 (with C-135 other-epoch, C-160 percentiles, C-157 reclaimable latency, C-152 peer DSCP, C-161 control isolation). Applied in 007a-02, 007a-09, 007a-14.
- Commissioning window messages: C-153. Applied in 007a-08, 007a-09.
- Domain 0 versus ForeignDomain, and PathMtuTooSmall only after the fallback probe fails: C-173. Applied in 007a-08, 007a-13.
- Housekeeping duties and the panic hook belong to epic 015 (C-144, C-145); the trait does not use the panic hook (007a-04).
- Earlier: observation extension and types (deliverable 3, C-120), generator-codec self-check, reader formats and API, C-125 applicability, evidence tags (C-132), fuzz budget (C-95), locations (C-130, C-133).
- Which specification supplies what: C-193 (epic 004 wire layouts and reject reasons; epic 005 state tables and scenario scripts for discovery, election, admission and connection, which fixes the `Machine` set; epic 006 timeline and epoch scenarios). Closes former gaps 1 and 2. Applied in 007a-04, 007a-05, 007a-08, 007a-11.
- Crate location and fuzzing tool: C-192 (`hadronyche-vectors` at the workspace root; fuzzing tool is the implementing team's choice). Closes former gap 8. Applied in 007a-01, 007a-15.
- Refusal reasons completed, 18 in all: C-185 (adds ControlDegraded and NoSkewBudget; VersionMismatch, NameTaken and ModuleCap raised at discovery, not admission). Applied in 007a-08.
- Where controllers travel: C-181 and C-205 (a bundle source's controllers in their own member stream; a single-stream source's controllers inside the 100-byte cap of C-137, else a second stream). Applied in 007a-07, 007a-10.
- The controller member counts towards the 32 members of a bundle: C-08 (revised), C-209. Applied in 007a-07, 007a-08, 007a-11.
- The receiver report carries the consumer's current A: C-198. Applied in 007a-08.
- Window of the stamp's lateness figures: C-182 (rolling 60 s; maximum since live beside them). Applied in 007a-09.
- Dropped excess sync requests counted per follower in the holder's stamp: C-204. Applied in 007a-09.
- Vector file names and generator output layout are decided by this part and recorded in the crate documentation: C-206. Applied in 007a-05.
- C-104, C-105 (revised) and C-134 create no work in this part; nor do C-141, C-153 (beyond the window messages already applied), C-169, C-177 to C-180, C-183, C-184, C-186 to C-197, C-199 to C-203, C-207, C-208, C-210 to C-212, whose subjects are runtime behaviour, the compiler, the clock, scenarios (007b) or ladder runs.

## Remaining gaps
1. **Full list of control message types and their layouts** (narrowed: hello, stamp sections, stamp request, probe pair, receiver report (carrying the consumer's current A, C-198), admission request and refusal with the 18 reasons of C-185, commissioning window announcement and close, no query message). Affects 007a-08, 007a-09. Owner: epic 004 (C-193).
2. **Full list of validity rules and the rejection reason enumeration** (narrowed by C-115, C-116, C-17, C-18, C-137). Affects 007a-11, 007a-12. Owner: epic 004 (C-193).
3. **Receiver behaviour on a non-advancing liveness counter** (C-115). Affects 007a-11, 007a-13. Owner: epic 004.
4. **Format of the specification version stamp in vector files.** Affects 007a-02. Owner: epic 004.
5. **Confirmation of the C-164 bound.** Affects 007a-02, 007a-13, 007a-15. Owner: L1 measurement.
