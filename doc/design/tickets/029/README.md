# Epic 029 — Monitor: tickets

## Summary
Epic 029 (phase P7, optional for v1, depends on epics 008a, 013, 014, 015, 017b and 019, blocks nothing; scope L, 25 tickets, about 27 days) builds `hadronyche-monitor`, a terminal monitor in crate `hadronyche-tools` (C-133) that shows the live state of a Hadronyche Medianet. It is a strictly read-only consumer of what the runtime already publishes — regime stamps (including the per-connection summary counters received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events and dead, C-159, and the observed DSCP, C-47) and hellos (C-30) — and adds no measurement of its own `[decision]`. It does not see receiver reports. It joins the IPv4 discovery and control groups read-only (C-100) and receives stamps multicast on any regime change and every 10 s, in sections of at most 1 252 bytes (C-17, C-18), never sending the unicast stamp request; or it reads a recorded stamp file. It keeps a model keyed by module (with instance nonce), connection and bundle member in which every regime change or new session number (C-19) starts a new series, and renders seven `ratatui` panels (Medianet, Clock, Connections, Bundles, Ports, Sinks, Regime) plus a `--headless` one-line-per-interval mode for logs. Its rules come from POC failures `[measured]`: blanks before clock lock, zero-sample families shown as failed, injected and commissioning runs labelled, no cross-regime averages, no last-writer-wins aggregation across streams.

Epic criteria are named 029-AC-1 to 029-AC-8, to keep them apart from the global criteria (AC-9, AC-31 and others).

## Tickets

| Ticket | Title | Effort | Depends on |
|---|---|---|---|
| 029-01 | Crate skeleton and observation types | S | epic 013, epic 014, epic 015 |
| 029-02 | Stamp section reassembly and version tracking | S | 029-01, epic 013, epic 015 |
| 029-03 | Live read-only collector | M | 029-01, 029-02, epic 015 |
| 029-04 | Recorded stamp file source | S | 029-01, 029-02, epic 015 |
| 029-05 | Fuzz observation decoders | S | 029-02, 029-03, 029-04 |
| 029-06 | Observation model with regime series | M | 029-01, 029-02 |
| 029-07 | Clock state and holdover countdown | XS | 029-06, epic 008a |
| 029-08 | Honest blanks, failed families, injection and commissioning labels | S | 029-06, 029-07 |
| 029-09 | Connection states and gap classes | S | 029-06 |
| 029-10 | Terminal UI shell | S | 029-03, 029-04, 029-06 |
| 029-11 | Medianet panel | S | 029-08, 029-10 |
| 029-12 | Clock panel | XS | 029-07, 029-10 |
| 029-13 | Connections panel | S | 029-08, 029-09, 029-10 |
| 029-14 | Bundles panel | S | 029-09, 029-10 |
| 029-15 | Ports panel | XS | 029-08, 029-10 |
| 029-16 | Sinks panel | S | 029-08, 029-10 |
| 029-17 | Regime panel | S | 029-08, 029-10 |
| 029-18 | Headless mode and CI log formatter | S | 029-08, 029-09, 029-10 |
| 029-19 | Replay equivalence test | S | 029-11 … 029-18, 029-24, 029-25 |
| 029-20 | Read-only verification in baseline scenarios | S | 029-03, 029-10, epic 017b, epic 019 |
| 029-21 | End-to-end honesty, regime and gap-class tests | M | 029-13, 029-17, 029-18 |
| 029-22 | Scale test | S | 029-03, 029-10 |
| 029-23 | Monitor documentation and design updates | S | 029-18 … 029-22, 029-24, 029-25 |
| 029-24 | Id collision and departure marks (C-46, C-124) | S | 029-06, 029-11 |
| 029-25 | Connection alarms, refusal reasons and stopped housekeeping (new, C-139, C-173, C-174; updated C-182, C-185) | S | 029-09, 029-11, 029-13 |

No ticket was deleted. The rulings removed one piece of work inside 029-01: the `ReceiverReport` variant and `ReportView` type (C-47). The withdrawal of the rational-L rule (C-104, C-105 revised) deletes nothing here: no ticket handled rational latency, rounding of L or fractional delay; the monitor only displays the degraded flag, which lags by a whole-sample L (C-105, C-122). The C-35 owner change does not touch this epic. In the final round 029-25 was added (C-139, C-173, C-174); 029-01, 029-06, 029-07, 029-08, 029-09, 029-11 to 029-14, 029-16 to 029-21 and 029-23 were updated in place. After the split, C-182, C-185 and C-192 added no ticket; 029-01, 029-05, 029-09, 029-13, 029-17, 029-20, 029-23 and 029-25 were updated in place. C-206 then added no ticket; 029-18 and 029-23 were updated in place. The last sweep (C-169, C-178, C-180, C-187, C-204, C-205, C-209, revised C-08 and C-153) added and deleted no ticket; 029-01, 029-06, 029-07, 029-08, 029-12, 029-14, 029-16, 029-17 and 029-23 were updated in place.

## Effort tally
- XS: 3 (0.75 day)
- S: 19 (19 days)
- M: 3 (7.5 days)
- Total: 25 tickets, about 27 days (27.25) → scope L (C-126)

## Coverage

### Deliverables

| Epic deliverable | Tickets |
|---|---|
| Crate `hadronyche-tools`, binary `hadronyche-monitor` (C-133) | 029-01 |
| `Observation` enum (Hello, Stamp) and `ObservationSource` trait | 029-01 |
| `collect.rs` — read-only IPv4 group join (C-100), multicast stamps in ≤1 252-byte sections, stamp version in hello (C-17, C-18) | 029-02, 029-03 |
| `collect.rs` — read a recorded stamp file | 029-04 |
| `collect.rs` — never sends anything except IGMP membership; never the stamp request | 029-03, 029-20 |
| `model.rs` — keyed by module (with nonce), connection, bundle member; regime change or new session starts new series (C-19) | 029-06 (with 029-07, 029-08, 029-09, 029-24 derivations) |
| `views/` Medianet panel (incl. id collisions C-46, departures C-124, commissioning C-107, C-153, alarms C-173, stopped housekeeping C-139) | 029-11, 029-24, 029-25 |
| `views/` Clock panel (holdover lapse to Unlocked, C-162; reference 2 × C-62 when E is unknown, C-169; lock state from the servo core, C-187) | 029-07, 029-12 |
| `views/` Connections panel (only Live satisfied C-154, Demoted C-109, C-159 counters, C-160 percentiles over the 60 s window with maximum since live C-182, alarms C-173, 18 refusal reasons C-185) | 029-09, 029-13, 029-25 |
| `views/` Bundles panel (C-159 counters per member; up to 32 members, controller member included, C-08, C-205, C-209) | 029-14 |
| `views/` Ports panel (per direction, C-117) | 029-15 |
| `views/` Sinks panel (E unknown unless commissioned C-103, C-140; maximum beside E C-73; D per C-170; claim suspended while unlocked C-180; commissioning results not unqualified C-178) | 029-16 |
| `views/` Regime panel (four capability states C-129, C-131 source values, control isolation C-161, peer DSCP C-152, C-157, C-160, dropped sync requests per follower C-204) | 029-17 |
| `--headless` mode | 029-18, 029-25 |
| `ratatui` dependency for terminal UI | 029-10 |

### Acceptance criteria

| Epic criterion | Tickets |
|---|---|
| 029-AC-1 Read-only (IGMP only, no stamp request; L0 (epic 017b, C-192) and L1 counters unchanged) | 029-03 (partial), 029-20 |
| 029-AC-2 Honest blanks, injected and commissioning labels (contributes to global AC-9, AC-31) | 029-07, 029-08, 029-10, 029-13, 029-17, 029-18, 029-21 |
| 029-AC-3 Regime change (path-MTU fallback to 1 280 bytes) and new session | 029-06, 029-13, 029-21 |
| 029-AC-4 Per-stream separation | 029-06, 029-13, 029-21 |
| 029-AC-5 Gap classes (late after C-108 reclassification, missing, marked, dead after 2 s per C-111; other-epoch C-135), within one 10 s refresh | 029-09, 029-13, 029-21 |
| 029-AC-6 Replay renders identically | 029-04, 029-10, 029-19, 029-24, 029-25 |
| 029-AC-7 Scale (64 modules, hellos at 1 s, stamps at 10 s, 32 streams, regime-change burst) | 029-03, 029-10, 029-22 |
| 029-AC-8 Decoders fuzzed per C-95, zero panics | 029-05 |

### Definition of done

| Epic DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all (per-change CI; privileged packet capture of 029-20 runs in the privileged nightly job, C-146) |
| Every parser touched is fuzzed | 029-02, 029-03, 029-04, 029-05 |
| Regime stamp fields touched are complete and documented | not applicable (C-125): the monitor adds or changes no stamp field; marked so in the epic and in 029-01, 029-02, 029-06, 029-09, 029-17, 029-25 |
| Design documents updated where behaviour differs, constants table first | 029-08, 029-23 |
| No new open item without an owner and a falsifier | 029-21, 029-23 |
| The frozen POC is not modified (task 003 is the only exception) | 029-01, 029-03, 029-10, 029-23 |
| Every new factual claim carries an evidence tag | all |
| The headless mode is used by at least one CI job as a log formatter | 029-18 |
| A screenshot or text capture of each panel is in the tool's documentation | 029-11 … 029-17, 029-24, 029-25, 029-23 (operator guide, `doc/operations/`, C-171) |

### Open sub-questions
- Decided in the epic: third-party observation of stream health (C-47, C-159; its falsifier is checked in 029-23 using 029-21 results); stamp fetch without a request (C-18); summary counters (C-159, C-108, C-135); definition of D (C-170); holdover countdown (C-64, C-65, C-70); 029-AC-7 load (C-18, C-30); L0 baseline scenario owner (C-192); headless line content and default interval (C-206).
- Build it in v1 at all: owned by Product at the start of P7, not this task; noted in 029-01 and 029-23.
- The remaining open sub-questions are listed below; each has an owner and a falsifier in the epic.

## Remaining gaps
1. **Recorded stamp file format, and whether it records hellos.** Needed for identical replay and for hello-derived fields. Affects 029-04, 029-19. Owner: epic 015 (stamp collector and single-writer file service, C-144).
2. **Is each bundle member its own connection with its own C-159 summary?** Affects 029-06, 029-14. Owner: epic 005 (control state machines), with epic 004 for the stamp layout.
3. **Where some displayed fields are published:** connection state, the Demoted health state (its rounds are counted, C-159), admitted and link rate per port and direction (C-117), sink D, E and certification state, check-family sample counts, election epoch, the commissioning window's expiry (C-153), whether stopped housekeeping (C-139) is distinguishable from ordinary control-degraded, and the discovery-time refusals VersionMismatch, NameTaken and ModuleCap that stop a module joining (C-185). Panels show `NotPublished` meanwhile. Affects 029-01, 029-08, 029-09, 029-11, 029-15, 029-16, 029-25. Owner: epic 013 (stamp object), with epic 004 for the hello.
4. **Stamp version semantics:** whether the version number changes only on a regime change or also when a 10 s refresh carries new counters. Affects 029-02, 029-06. Owner: epic 013, with epic 004.
5. **Value of the loss alarm threshold (C-91).** Rule proposed as 10 × the measured clean-wire loss (C-167); the number awaits L1. The monitor reads the published alarm and needs no value; only the floor annotation text waits on it. Affects 029-09. Owner: measurement (L1).
6. **Build the monitor in v1 at all.** Affects every ticket. Owner: Product, at the start of P7.

## Settled since the split
- Former gap 1, `ReceiverReport` variant versus invisible reports: closed by an epic change under C-47; the variant and `ReportView` are removed.
- Former gap 2, stamp fetch "on request" versus read-only: closed by C-18 (stamps multicast on any regime change and every 10 s; the monitor never sends the request).
- Former gap 4, gap-class counter mismatch: closed by C-47 (received, late, missing, marked gap, interrupted events, dead, observed DSCP) and C-108 (late versus missing).
- Former gap 8, holdover start time: closed by C-64, C-65 and C-70; the countdown is derived from the published bound.
- Former gap 10, 029-AC-7 load model and "the development machine": closed by C-18 and C-47 and an epic change (load restated as hellos at 1 s, stamps at 10 s and a regime-change burst; the machine's specification is recorded with the result in `doc/design/reports/`, C-130).
- Former gap 12, L0 and L1 baseline scenarios: narrowed by C-94 (L1 is task 019, now a dependency of the epic and of 029-20); the L0 owner remains as remaining gap 6.
- Former gap 7, D undefined: narrowed by C-74 and C-81 (typical values and restart spread); the definition was later settled by C-170 (final round below).
- Former gaps 3, 5, 6, 9 and 11 remain open (now remaining gaps 1, 2, 3, 7 and 5); gap 6 now also covers Demoted (C-109) and per-direction port rates (C-117).
- Internal defects of the epic fixed: the Goal no longer lists receiver reports as an input; criteria now carry ids 029-AC-1 to 029-AC-8, distinct from the global AC-9 and AC-31; dependency 019 added to the header and Dependencies; the inapplicable definition-of-done line is marked under C-125; the scope label is recomputed from the tally (C-126).
- Final round (rules C-127 to C-176, revised C-104 and C-105):
  - Former remaining gap 5, definition of D: closed by C-170 (total plumbing delay from hand-off to emission); 029-16 and the epic updated.
  - Counter list of the connection summary completed by C-159 (adds other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds) with C-135 for other-epoch; 029-01, 029-06, 029-09, 029-13, 029-14, 029-17, 029-21, 029-23 updated.
  - Satisfied means Live only; Pending, Locking, Aligning, Interrupted and Demoted are unsatisfied by name (C-154); 029-09, 029-13, 029-18 updated.
  - Lateness percentiles 50th, 99th, 99.9th and maximum (C-160) and reclaimable latency (C-157) are in the alignment group; 029-01, 029-13, 029-17 updated; former gap 3 narrowed accordingly.
  - Numeric E only for commissioned sinks (C-140); 029-08, 029-16, 029-18 updated.
  - Holdover lapse ends in Unlocked with time unusable (C-162); 029-07, 029-08, 029-12 updated.
  - Commissioning window life cycle and unqualified measurements (C-153); 029-08, 029-11, 029-17 updated.
  - Control isolation capability (C-161), peer-observed DSCP on sync and hellos (C-152), panic injection hook (C-145), holder stall bumps the timeline epoch (C-136), sink-only liveness by producer timeout (C-168) added to the relevant tickets.
  - Connection alarm kinds, 16 refusal reasons, cannot-meet-A and stopped housekeeping (C-139, C-173, C-174): new ticket 029-25.
  - Privileged capture in the separate nightly job (C-146): 029-20. Tool documentation in the operator guide under `doc/operations/` (C-171): 029-23. The stamp message is sent by epic 015 (C-144): 029-03, 029-04, epic Dependencies.
  - Loss alarm rule proposed by C-167; the value still awaits L1 (remaining gap 7).
  - Rational-L withdrawal (C-104, C-105 revised, C-134) and the C-35 owner change: no effect on this epic; nothing deleted.
- Rules settled after the ticket split (C-182, C-185, C-192):
  - Former remaining gap 6, which epic defines the L0 baseline scenario: closed by C-192 (epic 017b); epic 017b added to the epic's dependencies and to 029-20, whose blocked-L0 criterion now names epic 017b. Former remaining gaps 7 and 8 are now 6 and 7.
  - Fuzzing tool is the implementing team's choice (C-192): 029-05 no longer prescribes `cargo-fuzz`.
  - Lateness percentiles, reclaimable latency and cannot-meet-A use a rolling 60 s window, with the maximum since the connection went live reported beside them (C-182): 029-01, 029-13, 029-17, 029-25 and the epic updated.
  - Refusal reasons completed at 18 by ControlDegraded and NoSkewBudget (C-185); VersionMismatch, NameTaken and ModuleCap are discovery-time refusals that stop a module joining, not connection refusals: 029-09 and 029-25 corrected (count 16 → 18; discovery-time refusals shown against the module on the Medianet panel, not as connection rows), the epic updated; where the discovery-time refusals are published joins remaining gap 3.
  - Not relevant to this epic: the remaining points of C-192 (corpus program counting, reference dense controller, vectors crate location).
- Rules settled after the ticket split (C-206):
  - Former remaining gap 5, headless line content and default interval: closed by C-206 (output line formats and panel layouts are decided by the epic that builds the tool and recorded in its documentation; only what another epic or an operator's script depends on goes into the specifications). 029-18's format and 1 s default are now this epic's decision, not a proposal awaiting the design owner; its criterion 6 now requires them to be recorded in the tool documentation. 029-23 and the epic's open sub-questions updated. No ticket added; former remaining gaps 6 and 7 are now 5 and 6.
- Rules settled after the ticket split, last sweep (C-177 to C-212, revised C-08, C-134, C-141, C-153, C-169, C-181, C-184):
  - What an open commissioning window disqualifies (C-178, C-153 revised): only ordinary production measurements (latency, loss, alignment) are unqualified; D, restart spread, certification and soak are valid commissioning results recorded with their window. 029-08 corrected (it labelled every figure of an overlapping series `unqualified`; its Work item 5 and criterion 4 now exempt commissioning results), 029-16 and 029-17 updated, the epic's Commissioning row corrected.
  - Unlocked commissioned sink (C-180): stays commissioned with its claim suspended and left out of the group's required-E check until it locks again; 029-16 (Work and two new criteria) and the epic's Sinks panel and presentation-group text updated.
  - Holdover reference when E is unknown (C-169): 2 × C-62, 50 µs; 029-07 (constant `REFERENCE_E_UNKNOWN`, new criterion), 029-12 and the epic updated.
  - Bundle cap includes the controller member, and a bundle source's controllers ride in their own member (C-08 revised, C-181, C-205, C-209): 029-06, 029-14 (new fixture in criterion 1) and the epic's Bundle text updated.
  - Publisher of the lock state and bound (C-187): epic 008a added to the epic's dependencies and to 029-07; 029-01 and 029-17 note it.
  - Sync requests dropped above the accepted rate, counted per follower in the holder's stamp (C-204): added to the epic's Clock group, 029-01's `StampView` and 029-17 (new criterion).
  - No subject in this epic: C-177 and C-201 (a module that refuses to start publishes nothing to show), C-179, C-183, C-184, C-186, C-188 to C-191, C-193 to C-200, C-202, C-203, C-207, C-208, C-210 to C-212, and revised C-134 and C-141. C-182, C-185, C-192 and C-206 were already applied. The cannot-meet-A alarm raised under C-211 and the re-alignment after a changed A (C-186, C-208) reach the monitor as published alarms and connection states, which 029-09 and 029-25 already display without change.
  - Remaining gaps unchanged; none is owned by the design owner.
