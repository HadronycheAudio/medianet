# Epic 013 — Regime stamp wire object: tickets

## Summary
Epic 013 (phase P3, scope L, depends on epics 001, 004, 005, 007; blocks 014, 015, 017, 019, 020, 026, 028, 029) defines the **regime stamp**: a compact, versioned, `no_std`, allocation-free wire object in `hadronyche-core` that states under which conditions every measurement was taken. It covers eight groups (Identity, Capabilities in four states (C-129), Network, Clock, Alignment, Platform, Drops, Injection), per-connection entries, per-connection summary counters (C-47, C-159), per-peer observed sync and hello DSCP (C-152), per-follower dropped sync request counts in the holder's stamp (C-204), a self-test result, a `StampId` derived from the regime fields only (counters, including the lateness percentiles and reclaimable latency, excluded), and the rules binding records to stamps (new record set on regime change, both stamp ids on two-endpoint records, one process-owned writer per file, zero-sample families fail the run, unqualified, injected, commissioning-window (C-153) and missing-privilege numbers labelled, strict mode per C-127, C-128, C-147 and C-165). The stamp travels as a sectioned control message, multicast on regime change and refreshed every 10 s, with a unicast request (C-18). In `hadronyche-std` the epic adds a stamp publication schedule (epic 015 sends, C-144), a stamp collector, a record-set writer over epic 015's single-writer file service (C-144), an end-of-run zero-sample check and strict mode, plus the spec `doc/design/spec/stamp.md`. It exists because the POC's first measured run `[measured]` produced a corrupt metrics file, a garbage loss counter, "0 of 0 windows" families on a green-looking run, and silent real-time-scheduling failure.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 013-01 | Stamp core types | S | epic 001 |
| 013-02 | Stamp envelope and section framing | M | 013-01, epic 004 |
| 013-03 | Stamp parser fuzz target | S | 013-02 |
| 013-04 | Canonical configuration encoding | S | 013-02 |
| 013-05 | StampId derivation | S | 013-02, 013-04 |
| 013-06 | Identity and capabilities sections | S | 013-01, 013-02, 013-03, 013-04, epic 001 |
| 013-07 | Network and connection sections | S | 013-01, 013-02, 013-03 |
| 013-08 | Clock and alignment sections | S | 013-02, 013-03 |
| 013-09 | Platform, drops and injection sections | S | 013-02, 013-03 |
| 013-10 | Self-test section and qualified() | S | 013-01, 013-02, 013-03, 013-09, 013-25 |
| 013-11 | Stamp completeness check | XS | 013-06, 013-07, 013-08, 013-09 |
| 013-12 | Stamp encoding vectors | S | 013-06, 013-07, 013-08, 013-09, 013-10, 013-13, 013-23, 013-25, 013-26, 013-27, epic 007 |
| 013-13 | Stamp size budget and splitting rule | S | 013-06, 013-07, 013-08, 013-09, 013-10, 013-23, 013-26, 013-27, epic 004 |
| 013-14 | RecordHeader with both stamp ids | XS | 013-01, 013-03 |
| 013-15 | Process-owned record-set writer | S | 013-11, 013-14, epic 015 |
| 013-16 | New record set on regime change | S | 013-05, 013-15, 013-17 |
| 013-17 | Stamp collector | M | 013-05, 013-11, 013-13, 013-15, epic 004, epic 005, epic 015 |
| 013-18 | Zero-sample check family failure | S | 013-15 |
| 013-19 | Unqualified and privilege labels in outputs | S | 013-10, 013-15, 013-17 |
| 013-20 | Strict mode start refusal | S | 013-01, 013-06, 013-10 |
| 013-21 | Socket overflow counter in stamp | S | 013-09 |
| 013-22 | Stamp specification document | S | 013-05, 013-11, 013-12, 013-13, 013-17, 013-23, 013-24, 013-25, 013-26, 013-27, epic 001, epic 004 |
| 013-23 | Per-connection summary section (new, C-47) | S | 013-02, 013-03, 013-05, 013-07 |
| 013-24 | Stamp publication schedule (C-18; sending by epic 015, C-144) | S | 013-05, 013-11, 013-13, epic 004, epic 005, epic 015 |
| 013-25 | Calibration factor and commissioning window fields (C-106, C-107, C-153, C-176) | S | 013-05, 013-09 |
| 013-26 | Per-peer observed DSCP entries (new, C-152) | S | 013-02, 013-03, 013-05, 013-07 |
| 013-27 | Per-follower dropped sync request counter (new, C-204) | S | 013-02, 013-03, 013-05, 013-26 |

No ticket was deleted. The withdrawn rational-L rule (C-104, C-105 revised; C-134) never reached this epic: no ticket handled rational numbers, rounding of L or fractional delays, so nothing was removed for it. C-144 moved two duties to epic 015: 013-24 no longer sends (it became the publication schedule, M → S) and 013-15 no longer owns file exclusivity (it writes through epic 015's single-writer file service, M → S). 013-26 is new (C-152). 013-27 is new (C-204). C-198 adds a context line and one acceptance criterion to 013-08, staying S. The last sweep (C-177 to C-212) added no ticket and changed no effort: C-169, C-180, C-182, C-185 and C-187 extend 013-08 (one counter field, the since-live maximum, and an A value "none"); C-178 refines 013-10, 013-19, 013-25; C-188 adds context to 013-26; C-191 one criterion to 013-14; C-192 and C-193 touch 013-02, 013-03, 013-12; C-206 touches 013-15, 013-17, 013-19; C-177 and C-201 are recorded as out of scope in 013-20; 013-22 lists them all. Earlier: 013-13 changed purpose (C-18); 013-20 grew the C-127/C-128 checks and now also C-147/C-165, staying S.

## Effort tally
- XS: 2 (0.5 day)
- S: 23 (23 days)
- M: 2 (5 days)
- Total: 27 tickets, about 28.5 days → scope **L** (C-126); under the 40-day split threshold of C-143

## Coverage

### Deliverables

| Deliverable | Tickets |
|---|---|
| `CapState` (four states, C-129), `Profile`, `RxStampSource` (C-131) enums | 013-01 |
| `Capabilities` struct (and its wire section), including `cpu_idle_restricted` (C-127) and `control_isolation` (C-161) | 013-01, 013-06 |
| `StampId` (content-derived over regime fields, counters excluded) | 013-01, 013-05 |
| `Stamp<'a>` borrowed view, length-prefixed versioned sections, `parse` | 013-02, 013-03 |
| `Stamp::id()` | 013-05 |
| `Stamp::qualified()` (incl. commissioning window, C-153; production figures only, C-178) | 013-10, 013-25 |
| `Stamp::is_complete()` | 013-11 |
| `StampBuilder::identity`, `capabilities` | 013-06 |
| `StampBuilder::network`, `connection` | 013-07 |
| `StampBuilder::connection_summary`, `ConnSummary` (C-47, C-159) | 013-23 |
| `StampBuilder::peer_dscp`, per-peer sync and hello DSCP (C-152) | 013-26 |
| Per-follower dropped sync request count in the holder's stamp (C-204) | 013-27 |
| `StampBuilder::clock`, `alignment` (certification state C-103, C-140; percentiles C-160; reclaimable latency C-157; unlocked after holdover lapse C-162, C-169; current A, own or inherited, C-198, configured budget or none, C-142, C-185; 60 s window and since-live maximum, C-182; commissioned while unlocked, C-180; lock state and bound from epic 008a, C-187) | 013-08 |
| `StampBuilder::platform`, `drops`, `injection` (incl. C-120 and C-145 hooks) | 013-09 |
| Calibration factor (C-106, C-176), commissioning window and expiry (C-107, C-153) | 013-25 |
| `StampBuilder::self_test`, `SelfTest` | 013-01, 013-10 |
| `StampBuilder::finish` (caller buffer, no allocation) | 013-02, 013-11 |
| Section splitting rule and reassembly reader | 013-13 |
| `RecordHeader` (certification record: sink and holder stamps, C-191) | 013-14 |
| Stamp publication schedule (`hadronyche-std`, C-18; sending by epic 015, C-144) | 013-24 |
| Stamp collector (`hadronyche-std`; deployment data directory, C-171) | 013-17 |
| Record-set writer over epic 015's single-writer file service (C-144), new set on local/peer id change (`hadronyche-std`) | 013-15, 013-16 |
| End-of-run zero-sample check (`hadronyche-std`) | 013-18 |
| Strict mode (`hadronyche-std`, hosted only, from operator policy; C-127, C-128, C-147, C-165) | 013-20 |
| Labels in the outputs of Rule 13 (incl. commissioning window as unqualified, C-153, C-178; label text per C-206) | 013-19 |
| `doc/design/spec/stamp.md` (field tables, encoding, publish/request, task 001 capability table) | 013-06 to 013-10, 013-23, 013-25, 013-26, 013-27 (field tables), 013-22 |
| Open sub-question: compact comparable "full configuration" encoding | 013-04 |
| Open sub-questions owned elsewhere (tasks 001, 015, 021, 022; epics 004, 005, 015, 017; measurement) | recorded in 013-22; noted in 013-01, 013-05, 013-06, 013-07, 013-10, 013-14, 013-18, 013-20, 013-23, 013-24 |

### Acceptance criteria

| Epic AC | Tickets |
|---|---|
| 013-AC-1 Vectors byte for byte (AC-1); every parser fuzzed at the C-95 budget | 013-12, 013-03 (corpus extended by 013-04, 013-06 to 013-10, 013-13, 013-14, 013-15, 013-23, 013-25, 013-26, 013-27) |
| 013-AC-2 AC-31: complete stamp per record set; injection and missing privileges labelled; regime change starts new set; one writer per file | 013-11, 013-15, 013-16, 013-19 |
| 013-AC-3 AC-9: zero-sample family exits non-zero and names it | 013-18 |
| 013-AC-4 Regime field change changes `StampId` and starts a new set (local at once, peer within 10 s); counter change does not | 013-05, 013-06 to 013-09, 013-23, 013-25, 013-26, 013-27 (per-field id tests), 013-16 |
| 013-AC-5 Two-endpoint record carries both ids; collector resolves both | 013-14, 013-15, 013-17 |
| 013-AC-6 `qualified()` false without passed self-test, with injection or with an open commissioning window (C-153); labelled in every output | 013-10, 013-25, 013-19 |
| 013-AC-7 Strict mode refuses on domain 0, failed self-test item, or `rt_scheduling`/`memory_locked` not present; never on a device (C-147) | 013-20 |
| 013-AC-8 AC-34 (stamp part): requested DSCP and peer-observed values, or "unverified"; consumer's observed DSCP; per-peer sync and hello DSCP (C-152) | 013-07, 013-23, 013-26 |
| 013-AC-9 AC-37 (stamp part): induced socket overflow in drop counters | 013-21 |
| 013-AC-10 Device stamp without allocator; every message ≤ 1 252 bytes; splitting rule | 013-02, 013-13 |
| 013-AC-11 Producer and consumer summaries, resolvable by a third party | 013-23, 013-17 |
| 013-AC-12 Emitted on regime change, every 10 s and on unicast request; epic 015 sends (C-144) | 013-24, 013-17 |
| 013-AC-13 Calibration factor, commissioning window and test hooks (C-120, C-145) stamped | 013-09, 013-25 |

### Definition of done
Each ticket lists the epic's checklist lines that apply to it and, per C-125, names every other line as not applicable with the reason. "Every parser touched is fuzzed": 013-03 plus every ticket adding a section or decoder. "The stamp format is versioned independently of the media format, with the rule for unknown sections (skip and count)": 013-02, 013-12, 013-13, 013-22. "Every stamp field has a stated source on each of the four profiles": 013-06 to 013-10, 013-21, 013-22, 013-23, 013-25, 013-26, 013-27. "Unprivileged" applies to per-change CI (C-146); no ticket needs a privileged test. The epic itself has no not-applicable line.

## Remaining gaps

1. **Hello stamp version number versus `StampId`** (C-18): whether they are the same value and whether a counter refresh bumps the version. Affects 013-05, 013-17, 013-24. Owner: wire specification epic 004 with the control state machines epic 005.
2. **Byte layout**: envelope fields, section headers, enum wire values (`Profile`, `RxStampSource`, `CapState`, certification state), counter widths, split-part header. Tickets propose and mark "pending epic 004". Affects 013-01, 013-02, 013-08, 013-12, 013-13, 013-23, 013-26, 013-27. Owner: wire specification epic 004 (vectors then from epic 007).
3. **External types** `ConnId`, `PacingMode` (epic 004) and `CheckFamily` (epics 015, 017); placeholders meanwhile. Affects 013-07, 013-14, 013-18, 013-23. Owner: those epics.
4. **Self-test thresholds**: wake-latency pass limit and receive-stamp consistency tolerance. (Lock bound and packets per slice are settled: C-62, pending simulation; C-05.) Affects 013-10, 013-20. Owner: a measurement (task 001, then L2), recorded in the constants table.
5. **Per-OS facts**: real receive-stamp source and state per OS (task 001, on the OS versions C-138 names) and unprivileged readability of interrupt coalescing (tasks 015, 021, 022). Affects 013-06, 013-07, 013-22. Owner: those tasks (measurements).

None of the remaining gaps needs the design owner.

## Settled since the split

| Former gap | Closed by |
|---|---|
| 2. Receive-stamp source cannot be "unverified" | C-129, C-131: source value (kernel, userspace, mac-hardware, interrupt) plus a four-state capability state |
| 3. `Tri` value 0 undefined (discriminants remain, remaining gap 2) | Epic change and C-129: four states, 0 rejected |
| 5. `StampId` derivation unspecified | Epic change from C-18, C-47: content-derived over regime fields; function chosen in 013-05 |
| 7. Self-test not one of the eight groups | Epic change: completeness counts the eight groups; no self-test means unqualified |
| 8. Self-test thresholds (partly) | C-62 lock bound, C-05 packets per slice; the rest is remaining gap 4 |
| 9. "Within one reporting interval" | C-18: local change at once; peer change within the 10 s refresh |
| 10. Counters versus regime | C-18, C-47 and epic change: Drops and summaries are counters, excluded from `StampId` |
| 11. Rule 12 group and percentiles | C-103, C-140 and epic change: Alignment group, states commissioned / failed / uncertified / self-monitored; percentiles by C-160 |
| 12. Rule 6 and Rule 3 without deliverable | Epic change and C-47: stamp carries lock state, bound, packets per slice and slice-level summary; producers (epics 015, 017) blank and report both loss levels |
| 13. Rule 10 versus AC 7; strict mode not a deliverable | Epic change, C-127, C-128: strict mode in deliverables; refuses on domain 0, failed self-test item, or real-time scheduling / memory locking not present |
| 14. "Every output" and "privileges" undefined | Epic Rule 13 and C-129: three outputs listed; privileges are `rt_scheduling` and `memory_locked`; not requested is labelled separately |
| 15. Record-set and collector formats | Epic change: chosen by 013-15 and 013-17, written into the spec |
| 16. Overlap with epic 014 | C-47 and epic change: local counters here, peer-observed values from 014, consumer's own observed DSCP in its summary |
| 17. Fuzzing bar | C-95: 60 s per change, 1 hour nightly, corpus kept |
| 18. Transport and publish/request flow | C-18 (byte layout remains, remaining gap 2); 013-24 and 013-17; sending by epic 015 (C-144) |
| 19. Location of the constants table | C-130: `doc/design/02-constants.md` |
| 20. Unused constants C-90, C-91 | Epic change: removed from the epic's related constants |
| 1. (and gap 1 of the previous refinement) Meaning of `control_isolation` | C-161: housekeeping cannot delay the data path; present on hosted by construction, absent or unverified on a device unless bounded (013-01, 013-06) |
| Gap 2 of the previous refinement: lateness percentiles | C-160: p50, p99, p99.9 and maximum, fixed fields in a counter section (013-08); C-157 adds reclaimable latency beside them |
| Gap 3 of the previous refinement: commissioning window and `qualified()` | C-153: measurements in a window are unqualified; `qualified()` false while open; expiry stamped (013-10, 013-19, 013-25) |
| Gap 4 of the previous refinement: Demoted inputs in the summary | C-159: demoted rounds counter, with other-epoch (C-135), too-early, duplicates and incomplete reassemblies (013-23) |
| Who sends the stamp, who owns files | C-144: epic 015 sends and provides the single-writer file service; 013-24 and 013-15 rescoped (M → S) |
| Sync and control DSCP verification | C-152: new ticket 013-26 |
| Certification states | C-140: commissioned gives a numeric E; uncertified and self-monitored declare E unknown (013-08) |
| Further rulings applied | C-145 panic hook (013-09, 013-19); C-147 and C-165 strict mode hosted-only, from operator policy (013-06, 013-20); C-162 unlocked after holdover lapse (013-08); C-171 collector location (013-17); C-176 calibration factor from epic 025 (013-25); C-146 privileged tests (none here) |
| Withdrawn rational-L rule (C-104, C-105 revised; C-134) | No effect: this epic never carried L |
| Sync requests above the accepted rate | C-204: dropped beyond 200 per second per follower (C-112), counted per follower in the holder's stamp; new ticket 013-27 (counter, excluded from `StampId`) |
| Which A the stamp carries for a fan-in | C-198 (with C-142, C-186): the current A, own or inherited; a change of the inherited A is a regime change (013-08) |
| What an open commissioning window disqualifies | C-178: ordinary production figures only; the commissioning tool's own measurements are valid and recorded with their window by epic 028 (013-10, 013-19, 013-25) |
| Window of the lateness figures | C-182: rolling 60 s for percentiles and reclaimable latency; the maximum since the connection went live is a new counter field (013-08) |
| Certification of a sink whose clock is unlocked | C-180: stays commissioned, claim suspended until relocked; certification field unchanged (013-08) |
| A for a fan-in with no downstream group | C-142, C-185: configured budget, or A = none with NoSkewBudget refusal (013-08) |
| Last sweep, smaller rulings | C-169 unknown-E lapse reference (013-08); C-187 lock state and bound from epic 008a (013-08); C-188 sync DSCP recorded on the clock thread (013-26); C-191 certification record stamps (013-14); C-192 vectors crate at the workspace root, fuzzing tool the team's choice (013-03, 013-12); C-193 reject reasons from epic 004 (013-02, 013-12); C-206 file names and label text decided here (013-15, 013-17, 013-19); C-177 and C-201 pin refusal in every mode, out of scope (013-20). No other rule of C-177 to C-212 has a subject in this epic |
| 4. External types | Not settled: remaining gap 3 |

Fully settled: 16 of the 20 split gaps (1, 2, 5, 7, 9, 10, 11, 12 to 20). Partly settled: 3 (discriminants, remaining gap 2) and 8 (thresholds, remaining gap 4). Open: 4 (remaining gap 3). Of the gaps raised after the split, the commissioning-window and Demoted questions are closed (C-153, C-159); the hello version question remains (remaining gap 1).
