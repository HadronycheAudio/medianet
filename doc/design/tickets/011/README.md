# Epic 011 — Gossip and discovery core: tickets

## Summary
Epic 011 (phase P3, scope L: 26 tickets, about 29 days) implements discovery and clock election in `hadronyche-core` as pure `no_std` state machines with no I/O and no allocation, shared by the hosted and embedded profiles (a platform supplies only a transport and a clock). It delivers the binary hello (encode/parse, at most 1 252 bytes, IPv4 multicast with TTL 1, carrying the regime stamp version), the fixed 63-entry peer map (C-217), identity handling (64-bit id + 64-bit nonce, collision redraw and quarantine lifted after 3 s of a single nonce, name and module-cap refusal via 1.5 s listen-before-claim, raised at discovery per C-185), stall detection via the liveness counter, regime stamp version tracking, per-peer observed hello DSCP for the stamp (C-152), and the election machine (triggers on reply silence, pressure or force but never hello expiry; 400 ms window; deterministic winner (Locked before holdover, then lowest module id, C-221); epoch merge; frequency-continuous handover, by any eligible candidate with usable time (Locked or in holdover with the alignment claim standing, C-213), or lowest-id eligible winner with a timeline epoch bump when none has usable time; no rotation; 64-follower cap tested with synthetic followers), plus scenario-vector tests from epic 007, fuzz targets run to the C-95 budget, a forced-identity test hook and a memory-footprint `const`. It depends on epics 004, 005, 007, 008, 013 and 002, and feeds 012, 015, 017 and 026.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 011-01 | Core identity types and no_std build | S | none |
| 011-02 | Hello encode | M | 011-01, epic 004 |
| 011-03 | Hello parse | M | 011-02, epic 004 |
| 011-04 | Hello size budget check | XS | 011-02 |
| 011-05 | Hello parse fuzz target | XS | 011-03 |
| 011-06 | Fixed 63-entry peer map | S | 011-03 |
| 011-07 | Discovery machine: hello cadence and expiry | M | 011-02, 011-03, 011-06 |
| 011-08 | Hello reject counters and version event | S | 011-07 |
| 011-09 | Unconfigured domain and strict mode | XS | 011-07 |
| 011-10 | Id collision: redraw and quarantine | M | 011-07, 011-24, epic 005 |
| 011-11 | Name taken: listen before claim | S | 011-07 |
| 011-12 | Stalled data plane detection | S | 011-07, epic 005 |
| 011-13 | Election message codec and fuzz target | S | 011-01, epic 004 |
| 011-14 | Holder eligibility and candidacy | S | 011-06, epic 008a, epic 002 |
| 011-15 | Election machine: triggers and window | M | 011-13, 011-14, epic 005 |
| 011-16 | Epoch merge and partition heal | S | 011-15, epic 005 |
| 011-17 | Handover continuity and timeline epoch bump | S | 011-15, epic 008 |
| 011-18 | Follower cap at 64 | XS | 011-15, epic 005 |
| 011-19 | Hello independence of election: tests | S | 011-07, 011-15 |
| 011-20 | No rotation: 24-hour simulation | S | 011-19 |
| 011-21 | Scenario vector runner | M | 011-10, 011-11, 011-12, 011-16, 011-17, 011-18, 011-23, 011-24, 011-26, epic 007 |
| 011-22 | Regime stamp and design doc update | S | 011-04, 011-06, 011-09, 011-12, 011-17, 011-18, 011-23, 011-25, 011-26, epic 013 |
| 011-23 | Regime stamp version tracking (new, C-18) | XS | 011-07, epic 004 |
| 011-24 | Forced identity test hook (new, C-120) | XS | 011-01 |
| 011-25 | Observed hello DSCP per peer (new, C-152) | XS | 011-06, 011-07 |
| 011-26 | Module cap at discovery (new, C-185) | XS | 011-06, 011-11 |

No ticket was deleted: no ruling removed work from this epic. The epic never contained rational-L, rounding or fractional-delay work, so the revision of C-104/C-105 and C-134 changes nothing here.

## Effort tally
- XS: 8 (2 days)
- S: 12 (12 days)
- M: 6 (15 days)
- Total: 26 tickets, about 29 days, scope L (C-126)

## Coverage

### Deliverables

| Deliverable | Tickets |
|---|---|
| `ModuleId`, `Nonce`, `Domain`, `Addr` (IPv4), `Outbox` | 011-01 |
| `IdSource` trait, deterministic test source | 011-01; platform impls out of scope (015, 026) |
| Test-only forced-identity `IdSource` (C-120) | 011-24 |
| `no_std`, no allocation, fixed capacities | 011-01 (CI build), all code tickets |
| `Hello::encode` (<= 1 252 bytes), `LocalInfo` | 011-02, 011-04 |
| `Hello::parse`, `Reject` (incl. `UnconfiguredDomain`, C-173; linear cost, C-164) | 011-03 |
| `check_start`, `StartError` | 011-09 |
| `MAX_PEERS`, `Peer`, `PeerMap` | 011-06 |
| `MapEvent` (all variants) | 011-06 (type); Joined/Left/MapFull 011-07; ForeignDomain/VersionMismatch 011-08; Collision/Quarantined 011-10; NameTaken 011-11; ModuleCap 011-26; Stalled 011-12; StampVersionChanged 011-23 |
| `Discovery::new`, `on_hello`, `on_tick`, `next_event`, `mark_live` | 011-07 |
| `Discovery::redraw_id` | 011-10 |
| `Discovery::reject_counters`, `RejectCounters` (incl. `unconfigured_domain`, C-173) | 011-08 |
| `on_hello` `dscp` argument, `Discovery::observed_dscp` (C-152) | 011-07 (signature), 011-25 |
| `Candidacy` (`time_usable`, C-213), holder eligibility, fallback candidates (C-113) | 011-14 |
| `Election`, `ElectionInput`, `ElectionEvent`, `Election::step`, `Election::next_event` | 011-15 (core, queue); 011-16 (merge); 011-17 (HolderChanged.continuous, TimelineEpochBump); 011-18 (FollowerRefused) |
| Election message codec | 011-13 |
| Scenario-vector tests from 007 | 011-21 |
| Fuzz target for `Hello::parse` (C-95 budget) | 011-05 |
| Fuzz target for election messages (C-95 budget) | 011-13 |
| `const` reporting the map's memory footprint | 011-06 |
| Holder-eligible bit in regime stamp, docs | 011-22 |
| Per-peer observed hello DSCP exported for the stamp (C-152) | 011-25; documented by 011-22 |

### Acceptance criteria

| Epic AC | Tickets |
|---|---|
| 1. All discovery and election scenario vectors pass (AC-1) | 011-21 |
| 2. AC-22 forced id collision, quarantine until one nonce for 3 s | 011-10, 011-24 |
| 3. AC-20 (b) no election with holder hellos suppressed; AC-20 (a) live peers kept 30 s | 011-19 (both), 011-07 (a), 011-15 (no expiry input) |
| 4. AC-36 one holder within 1.4 s after heal | 011-16 |
| 5. AC-25 (core) 65th follower refused, synthetic followers | 011-18 |
| 6. No rotation over 24 h simulated | 011-20 |
| 7. Eligible + usable time (Locked or holdover claim standing, C-213) elected, epoch kept; none with usable time → lowest-id eligible + timeline epoch bump | 011-14, 011-17 |
| 8. 32-member bundle hello <= 1 252 bytes or constant falsified | 011-04 |
| 9. 64th peer (65th module) not inserted, `MapFull` (C-217) | 011-06, 011-07 |
| 10. AC-23 reject counting (incl. domain 0 ignored, C-173), version event | 011-03, 011-08 |
| 11. Compiles `no_std` without allocator | 011-01 (CI job), all code tickets |
| 12. Fuzz budget 60 s per change, 1 h nightly, corpus kept | 011-05, 011-13 |
| 13. Stamp version in hello, change raises event | 011-02, 011-03, 011-23 |
| 14. Auth scheme 0, tag zero, counter = liveness counter | 011-02 |
| 15. Name taken → silent; else first hello at 1.5 s | 011-11 |
| 16. `observed_dscp()` per peer (C-152) | 011-25 |
| 17. Newcomer hearing 64 modules (full 63-peer map plus a refused id, C-217) raises `ModuleCap`, sends no hello (C-185) | 011-26 |

### Definition of done
Each ticket lists the lines that apply and marks the others not applicable with the reason (C-125).

| Item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all except 011-22 (marked not applicable); no test of this epic needs a privilege, so the privileged nightly job of C-146 is not used |
| Every parser touched is fuzzed | 011-03, 011-05, 011-08, 011-13 |
| Regime stamp fields touched are complete and documented | 011-22 (holder-eligible bit, observed hello DSCP from 011-25) |
| Design documents updated, constants table first | 011-02, 011-04, 011-09, 011-10, 011-12, 011-17, 011-19, 011-22 |
| No new open item without owner and falsifier | all except 011-05, 011-13, 011-19, 011-20 (open no item) |
| Frozen POC not modified | code tickets that touch POC-derived behaviour (011-01, 011-02, 011-03, 011-06, 011-07, 011-10, 011-11, 011-13, 011-15, 011-16, 011-20) |
| Every new factual claim carries an evidence tag | all |
| No election path reachable from hello expiry | 011-07, 011-15, 011-19, 011-21 |
| Holder-eligible bit in regime stamp | 011-22 |

### Open sub-questions
| Question | Owner | Handling |
|---|---|---|
| I/O-pressure eligibility rule inherited? | Task 002 | 011-14 keeps the pressure term behind one switch |
| Later joiner / "joined at once" | Task 005 | 011-10 blocked on it; falsified by 011-21 |
| Stall rule and no-data-plane exemption | Task 005 | 011-12 |
| Equal-epoch merge | Task 005 | 011-16 |
| Follower attach input | Task 005 with 008 (clock thread, C-188) | 011-18 |
| 63-entry map (C-217) fits device memory? | Task 026 | 011-06 supplies the footprint const |
| Holder can serve 64 followers at 50 req/s (and up to 200/s, C-112)? | L1, task 019 (C-35); falsifier per C-167 | 011-18 keeps a single `MAX_FOLLOWERS` const |
| Bad-input processing within 10 µs per packet (C-164)? | L1 measurement | 011-03 and 011-13 parse in one pass and report a per-packet figure |

## Remaining gaps

1. **Hello and election byte layouts, and the regime stamp sections** (field widths, name length, port/bundle limits, message types, width of the stamp version number; the stamp section holding the holder-eligible bit and the field for the per-peer observed hello DSCP). Affects 011-02, 011-03, 011-04, 011-13, 011-22, 011-23, 011-25. Answer: wire specification epic 004 (hello, election), epic 013 (stamp message, C-144).
2. **"Later joiner" and "joined at once" in a collision** (the hello carries no join time). Affects 011-10, 011-21. Answer: control state machines epic 005.
3. **Stall detection rule for C-42**: the span over which the liveness counter must fail to advance, and whether a module with no data plane is exempt. Affects 011-12. Answer: epic 005.
4. **Equal election epochs with different holders after a heal.** Affects 011-16. Answer: epic 005.
5. **Follower attach input**: how the holder's election machine learns of an attaching follower, so it can emit `FollowerRefused`. Affects 011-18. Answer: epic 005, with epic 008 for the clock-thread side, where sync requests are answered (C-188) and over-rate requests dropped (C-204).
6. Settled by C-221 (tie-break among candidates with usable time); see "Settled since the split". The number is kept so that references to gaps 7 to 10 stay valid.
7. **Vector format and the I/O-pressure input source.** Affects 011-14, 011-21. Answer: epic 007 (format; scenario scripts from epic 005, reject reasons from epic 004, C-193), task 002 and hosted platform epic 015 (pressure).
8. **Bad-input cost bound** of 10 µs per packet (C-164, a proposal). Affects 011-03, 011-13. Answer: L1 measurement.
9. **Holder serving 64 followers** without growth of serving lateness beyond 20 % (C-35, C-167). Affects 011-18. Answer: L1 measurement, task 019.
10. **63-entry map (C-217) in device memory.** Affects 011-06. Answer: task 026, on the reference hardware chosen per C-172.

None of these needs the design owner.

## Settled since the split

Former gaps of the first split, and what closed them:

- Gap 2, plumbing types undefined: the epic now defines `Addr` (IPv4, C-100), `Outbox`, `LocalInfo`, `Reject`, `RejectCounters`, `Peer`.
- Gap 3 (part), quarantine resolution: until only one nonce has been seen for 3 s (C-46). The later-joiner part remains (gap 2 above).
- Gap 4, no-locked-candidate contradiction: the lowest-id holder-eligible candidate wins and bumps the timeline epoch (C-113).
- Gap 5, strict mode API: the epic adds `check_start(domain, strict)`; the flag's home is the platform.
- Gap 8 (part), unreachable 65th follower: tested with synthetic followers (C-114). The attach input remains (gap 5 above).
- Gap 9, single-event return: the epic adds a fixed-capacity event queue drained by `next_event()`.
- Gap 10, name taken vs join: the epic defines join as the end of the 1.5 s listening phase; first hello at 1.5 s.
- Gap 11, "logged on both modules": the epic defines it as `MapEvent::Collision` on both modules.
- Gap 12, "counted per reason" API: the epic adds `Discovery::reject_counters()`.
- Gap 13 (part), regime stamp undefined: transport and hello content settled by C-18 (new ticket 011-23). The implementing epic remains (gap 7 above).
- Gap 14, constants with no owner in core: IPv4 multicast with TTL 1 (C-100) and DSCP CS5 are platform transport; sync request rate 10 to 200/s (C-112) is the clock thread's; the epic says so.
- Gap 16, device id derivation and `IdSource` home: the epic places the implementations in platform epics 015 and 026.
- Also applied: authentication reservation in v1 (C-115), fuzz budget (C-95), forced-identity hook (C-120, new ticket 011-24), definition-of-done applicability (C-125), scope label from the tally (C-126), single domain rate (C-01, no rate field in the hello).

Former gaps 1, 6, 7 and 15 remain open as gaps 1, 3, 4 and 7 above.

Settled in the final update (rules C-127 to C-176, C-35 owner change, revised C-104/C-105):

- Home of the regime stamp message (former gap 7, design owner): defined by epic 013 and sent by the hosted backend, epic 015 (C-144). 011-02, 011-22, 011-23 and 011-24 now name these epics; the holder-eligible section comes from epic 013.
- Domain-0 hello: ignored and counted under its own reason, not refused; ForeignDomain is for a different non-zero domain (C-173). Applied in 011-03, 011-08 and 011-09 (`Reject::UnconfiguredDomain`, `RejectCounters::unconfigured_domain`).
- Bundle member limit at parse: more than 32 members (C-08) is malformed; BadMemberList is an admission refusal of epic 012 (C-173). Applied in 011-03.
- Peer verification of control DSCP: every module reports the DSCP observed on each peer's hellos (C-152). New ticket 011-25; `on_hello` gains a `dscp` argument (011-07); 011-06 stores it; 011-22 documents it.
- Strict-mode flag home: the operator policy in the commissioning configuration (C-165); strict mode is hosted-only (C-147), so a device passes `strict = false`; the other strict-mode checks (C-127, C-128) belong to epic 015. Applied in 011-09.
- Bad-input cost (C-164): parsers are single-pass and report a per-packet figure (011-03, 011-13); the 10 µs bound is confirmed at L1 (gap 8).
- Follower-cap measurement owner: L1, task 019 (C-35), with the falsifier of C-167. Applied in 011-18.
- Locked candidate after holdover: a lapsed holdover claim means Unlocked (C-162). Applied in 011-14.
- Holder-stall timeline epoch bump (C-136): outside the election machine, noted as out of scope in 011-17.
- Code and document locations (C-130, C-133, C-171): applied in 011-01, 011-04 and 011-22.
- Panic-injection hook (C-145): platform work, noted out of scope in 011-24. Privileged tests (C-146): none in this epic.
- C-104, C-105 (revised) and C-134: no effect; this epic has no latency, resampler or rational-L content.

Settled in the closing sweep (rules C-177 to C-212 and the late revisions, not applied to this epic before):

- Refusals raised at discovery (C-185): VersionMismatch (011-03, 011-08), NameTaken (011-11) and ModuleCap stop a module joining at discovery, not at admission. ModuleCap had no ticket: new ticket 011-26 raises `MapEvent::ModuleCap` when a newcomer hears 64 active modules during listen-before-claim (epic AC 17); `MapEvent` gains the variant (epic, 011-06).
- Over-rate sync requests (C-204) and where the holder answers them (C-188): clock-thread work of epic 008, dropped before any work and counted per follower in the holder's stamp; recorded in the epic and 011-18, and cited for the follower-attach gap (gap 5).
- Publisher of the lock state (C-187): the servo core, epic 008a, through the published time view; 011-14 now depends on epic 008a.
- Controller member and the bundle cap (C-08 revised, C-205, C-209): the 32 members include the controller member; applied to the parse limit (011-03), the size check (011-04, epic AC 8) and the constants list of 011-22.
- Vectors and fuzzing (C-192, C-193): the vectors crate sits at the workspace root and its scripts come from epic 005, reject reasons from epic 004 (011-21, gap 7); the fuzzing tool is the implementing team's choice (011-05, 011-13).
- No subject here: C-177 to C-184 and C-186, C-189 to C-191, C-194 to C-203, C-205 beyond the bundle cap, C-206 (this epic builds no tool; the runner's count line of 011-21 is its own presentation detail), C-207, C-208, C-210 to C-212, and the revised C-134, C-141, C-153, C-169, C-181, C-184.

Settled by the last rulings (C-213, C-113 as revised, C-217):

- Who may take over as holder (C-213, C-113 revised): candidates need usable time, that is Locked or in holdover with the alignment claim not lapsed (C-65, C-169), not Locked only; such a takeover is frequency-continuous and keeps the timeline epoch, and the epoch is bumped only when no candidate has usable time. The provisional "locked only" reading is corrected: `Candidacy.locked` becomes `time_usable` and the predicates, tests and handover rule follow (epic, 011-14 Work and AC, 011-15, 011-17 Work and AC, 011-21, 011-22). The tie-break question (gap 6) now reads "among candidates with usable time".
- Peer map capacity (C-217): 63 peers, since a module holds no entry for itself. `MAX_PEERS` is 63 and the 64th peer raises `MapFull` (epic AC 9, 011-06, 011-07); the newcomer's ModuleCap check of 011-26 now detects the 64th active module as a further id refused while its 63-peer map is full, instead of a 64-entry map; the device-memory question (gap 10) and 011-04, 011-22 now refer to 63 entries.
- No new ticket: both rulings change existing tickets only; the tally is unchanged.

Settled by the handover rulings (C-221; C-220, C-222 to C-225 have no subject in this epic):

- Tie-break among candidates with usable time (former gap 6, epic 005; open sub-question of the epic): a Locked candidate is preferred over one in holdover, and among equals the lowest module id wins (C-221). Applied in the epic (constants, election rules, AC 7, POC hash draw retired), 011-15 (rule, Work, AC 3; open question removed), 011-17 (AC 3) and 011-21 (no longer a falsifier of an open question). The ticket file 011-06 is renamed to match its 63-entry title (C-217). No new ticket; the tally is unchanged.
