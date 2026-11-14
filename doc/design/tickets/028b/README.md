# Epic 028b — Commissioning survey, certification and operations: tickets

## Summary
Epic 028b (phase P7, scope L: 24 tickets, about 31.5 days) is the second of the two lettered parts into which epic 028 (commissioning tool) was split under C-143. It depends on epic 028a, which delivers the binary `hadronyche-commission` (crate `hadronyche-tools`, C-133) with its stubs, the configuration format, the capacity arithmetic, the offline `validate` command, the buffer-memory check (C-211) and the A proposal computation (C-148). This part implements everything that needs a running medianet. It surveys a medianet from regime stamps (C-18, C-47): capability states (C-129) and strict-mode readiness (C-127, C-128), coalescing, path MTU, observed DSCP, unverified platforms, identity collisions, and foreign traffic including all IPv6 (C-100). It proposes the minimum feasible audition offset A per group under a provisional A with the commissioning rule (maximum lateness since the connection went live + D + headroom, C-148), and applies a commissioned change of A by announcing it to the group's members inside a commissioning window after the buffer-memory check, sinks included (C-224, C-211, C-222), certifies sink E/D (L2 marker loop) and group membership (L3 pairwise skew, C-79), runs burst and 24-hour loss-soak tests (C-118), and raises post-hoc alarms, including cannot-meet-A and Demoted inputs as "sinks not meeting A" (C-109, C-173, C-174). It announces and closes commissioning windows (C-153); outside a window stamped by every module it only reads stamps and writes configuration; test traffic is confined to that window (C-107). It also delivers per-OS and switch checklists and a commissioning guide in `doc/operations/`, and stores certificates beside the collected stamps (C-171). It depends on epics 028a, 008a (published clock lock state and bound, C-187), 012 (receipt of an announced A, C-224), 013, 015, 019, 020, 021, 022 and 027.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 028b-01 | Read-only stamp reader client | S | 028a-01, epic 013, epic 015 |
| 028b-02 | `survey`: framework, identity collisions and path MTU | S | 028b-01 |
| 028b-03 | `survey`: interrupt coalescing, EEE and timestamp source | S | 028b-02, epic 019 |
| 028b-04 | `survey`: DSCP as observed by peers | S | 028b-02 |
| 028b-05 | Platform reference list and unverified combinations | S | 028b-02, epic 020, epic 021, epic 022 |
| 028b-06 | `survey`: foreign multicast on the medianet interface | S | 028b-02 |
| 028b-07 | `survey`: capability states and strict-mode readiness (C-127 to C-129) | S | 028a-03, 028b-02 |
| 028b-08 | Commissioning-window guard (C-107) | S | 028b-01, epic 013 |
| 028b-09 | `window open` and `window close` command (C-153) | S | 028b-01, 028b-08, epic 013 |
| 028b-10 | Certificate record type | S | 028a-01, epic 013 |
| 028b-11 | `propose-a` command | M | 028a-03, 028a-10, 028a-11, 028b-01, 028b-08, epic 008a, epic 012, epic 015, epic 019 |
| 028b-12 | `certify-sink` command | M | 028b-01, 028b-05, 028b-08, 028b-10, epic 008a, epic 020 |
| 028b-13 | Decide on-site capture hardware for `certify-group` | S | epic 027 |
| 028b-14 | `certify-group` command and membership certificate | M | 028a-03, 028b-08, 028b-10, 028b-12, 028b-13, epic 027 |
| 028b-15 | `burst-test` command | M | 028a-02, 028b-01, 028b-08, epic 019 |
| 028b-16 | `soak` command and loss alarm threshold | S | 028b-01, 028b-08, 028b-15, epic 019 |
| 028b-17 | `watch` command and trunk-rate alarm | S | 028a-02, 028a-03, 028a-07, 028b-01 |
| 028b-18 | `watch`: loss, late-slice and A alarms | S | 028b-16, 028b-17 |
| 028b-19 | Queue configuration per switch family | S | 028b-04, 028b-15 |
| 028b-20 | `checklist-switch.md` | S | 028b-03, 028b-04, 028b-15, 028b-16, 028b-19 |
| 028b-21 | `checklist-linux.md` | S | 028b-03, 028b-06, 028b-07, epic 020 |
| 028b-22 | `checklist-macos.md` | S | 028b-03, 028b-06, 028b-07, epic 021 |
| 028b-23 | `checklist-windows.md` | S | 028b-03, 028b-04, 028b-06, 028b-07, epic 022 |
| 028b-24 | `commissioning-guide.md` | M | 028a-04 to 028a-10, 028b-02 to 028b-09, 028b-11, 028b-12, 028b-14 to 028b-18, 028b-20 to 028b-23 |

The order is a valid execution order. Dependencies on epic 028a: 028b-01 and 028b-10 (028a-01), 028b-07 and 028b-14 (028a-03), 028b-11 (028a-03, 028a-10, 028a-11), 028b-15 (028a-02), 028b-17 (028a-02, 028a-03, 028a-07), 028b-24 (028a-04 to 028a-10). Epic 028a depends on nothing here.

### Old ticket numbers

| Epic 028 ticket | New id |
|---|---|
| 028-09 | 028b-01 |
| 028-10 | 028b-02 |
| 028-11 | 028b-03 |
| 028-12 | 028b-04 |
| 028-13 | 028b-05 |
| 028-14 | 028b-06 |
| 028-33 | 028b-07 |
| 028-32 | 028b-08 |
| 028-34 | 028b-09 |
| 028-15 | 028b-10 |
| 028-18 | 028b-11 |
| 028-19 | 028b-12 |
| 028-20 | 028b-13 |
| 028-21 | 028b-14 |
| 028-22 | 028b-15 |
| 028-23 | 028b-16 |
| 028-24 | 028b-17 |
| 028-25 | 028b-18 |
| 028-26 | 028b-19 |
| 028-27 | 028b-20 |
| 028-28 | 028b-21 |
| 028-29 | 028b-22 |
| 028-30 | 028b-23 |
| 028-31 | 028b-24 |
| 028-16 | deleted before the split (margin term settled by C-148) |
| 028-01 to 028-08, 028-17, 028-35, 028-36 | epic 028a (see its index) |

File names kept their slug after the new id. Ticket ids inside ticket texts (dependencies, cross-references) were renumbered; nothing else in the tickets changed apart from the epic name in each header.

### History before the split (old numbers of epic 028)

Changes since the split into tickets: 028-05 went from S to XS (the follower check is implied by the module limit, C-114). The capability-gap finding moved from 028-10 to the new 028-33, and 028-10 gained the identity-collision finding (C-46); file renamed to `028-10-Survey-framework-collisions-and-path-MTU.md`. 028-26 now covers queue configuration only, the DSCP values being settled (C-13); file renamed to `028-26-Queue-configuration-per-switch-family.md`.

Changes in the final update (rules up to C-176): **028-16 deleted** (file `028-16-A-proposal-margin-decision.md` removed): its only purpose was deciding the margin term, which C-148 settles as the consumer headroom C-52. 028-17 went from S to XS (no margin option left; a fixed formula, truncation check and rounding). New tickets: 028-34 announces and closes the commissioning window (C-153); 028-35 checks the operator policy that the configuration now carries (C-165) and fan-in budgets (C-142). 028-01 gained the `window` subcommand stub; 028-03 gained the policy and budget fields; 028-33 gained the configured-versus-stamped strict-mode finding. No ticket dealt with the withdrawn rational-L rule (C-104, C-105 revised); 028-13, 028-29 and 028-30 now state that the edge resampler exists only inside an edge module, for 2:1 and 1:2 only, and that its delay is part of D (C-105, C-134, C-170).

Changes in the sweep of rules C-177 to C-212: **new 028-36** (XS) checks declared buffer memory against the A each aligning module holds and is called by `propose-a --write` (C-211); 028-03 gained the optional `buffer_kb` field. 028-17 and 028-18 read lateness over the stamp's rolling 60 s window with the since-live maximum beside it (C-182); 028-18 gained the re-alignment after a changed provisional A (C-186, C-198, C-208) and the buffer-memory refusal on `--write` (new AC 7). 028-15 and 028-19 carry the sink's and holder's stamps and the embedded device's declared restart spread (C-191); 028-15 and 028-21 keep a commissioned sink's record while its clock is unlocked (C-180); 028-21 applies the pair tolerance rule (C-203). 028-35 names NoSkewBudget (C-185); 028-33 states that a failed or unverified pin stops the module in every mode (C-177, C-201); 028-25 uses the rolling window and the C-211 fallback; 028-02 and 028-06 note controller streams (C-181, C-205, C-209). 028-18 and 028-19 depend on epic 008a (C-187).

## Effort tally
- XS: 0 (0 days)
- S: 19 (19 days)
- M: 5 (12.5 days)
- Total: 24 tickets, about 31.5 days → scope L (C-126)

## Coverage

### Deliverables

| Deliverable | Tickets |
|---|---|
| `survey` (stamp collection) | 028b-01, 028b-02 |
| `survey`: path MTUs, identity collisions | 028b-02 |
| `survey`: capability gaps and strict-mode readiness | 028b-07 |
| `survey`: coalescing state, EEE, timestamp source | 028b-03 |
| `survey`: DSCP as observed by peers | 028b-04 |
| `survey`: unverified platform combinations | 028b-05 |
| `survey`: clean-survey foreign-multicast check (for AC-9) | 028b-06 |
| `propose-a <group>` (since-live lateness, C-148; buffer-memory refusal on `--write` and `--apply`, C-211, C-222; `--apply` announces A inside a window, C-224) | 028b-11 (computation 028a-11, check 028a-10, guard 028b-08) |
| `certify-sink <sink>` | 028b-12 |
| `certify-group <group>` | 028b-13, 028b-14 |
| `burst-test` | 028b-15 |
| `soak` | 028b-16 |
| Commissioning-window guard for test activity | 028b-08 |
| `window open` / `window close` (C-153) | 028b-09 |
| `watch`: trunk rates per direction versus declared | 028b-17 |
| `watch`: loss above threshold, late slices, sinks not meeting A (cannot-meet-A, Demoted) | 028b-18 |
| Certificate record type | 028b-10 |
| `checklist-switch.md` | 028b-19, 028b-20 |
| `checklist-linux.md` | 028b-21 |
| `checklist-macos.md` | 028b-22 |
| `checklist-windows.md` | 028b-23 |
| `commissioning-guide.md` | 028b-24 |

### Acceptance criteria

| Epic AC | Tickets |
|---|---|
| AC-4 A proposal (zero late slices for 1 h on the C-92 rig; per-sink breakdown) | 028b-11 (with 028a-11) |
| AC-6 Certification (L2 within 20 %; L3 skew reproduced) | 028b-12, 028b-14 |
| AC-7 Survey (coalescing, path MTU, unverified platform, capability gap as named findings) | 028b-02, 028b-03, 028b-05, 028b-07 |
| AC-8 Post-hoc trunk alarm (per direction) | 028b-17 |
| AC-9 Checklists yield a clean `survey` (no foreign multicast, including IPv6, for 10 min) | 028b-06, 028b-21, 028b-22, 028b-23 |
| AC-10 Read-only outside a commissioning window | 028b-01, 028b-06, 028b-08, 028b-09, 028b-11, 028b-12, 028b-14, 028b-15, 028b-16, 028b-17 (the rule is fixed in the skeleton, 028a-01) |
| AC-11 A against declared buffer memory, `propose-a` half (C-211, C-222) | 028b-11 (check 028a-10) |
| AC-12 Applying a commissioned A inside a window (C-224) | 028b-11, 028b-08 |

### Definition of done

| Epic DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all code tickets |
| Every parser touched is fuzzed (budget C-95) | 028b-01, 028b-05, 028b-06, 028b-10, 028b-14 |
| Regime stamp fields touched are complete and documented | 028b-01 to 028b-05, 028b-07, 028b-08, 028b-09, 028b-10, 028b-11, 028b-12, 028b-17, 028b-18 (reading only; marked not applicable for defining fields in the epic, C-125) |
| Design documents updated where behaviour differs, constants table first | 028b-13, 028b-16, 028b-19, 028b-24 |
| No new open item without an owner and a falsifier | 028b-13, 028b-19, 028b-22, 028b-23 |
| The frozen POC is not modified | applies to all; code lives in `hadronyche-tools` (C-133), created by 028a-01 |
| Every new factual claim carries an evidence tag | all tickets that state facts |
| `validate` runs with no network and has tests for every refusal | not applicable: `validate` is epic 028a's (C-125) |
| Each checklist item states how to verify it and what the stamp shows | 028b-20 to 028b-23 |
| The commissioning guide walks through one complete medianet | 028b-24 |

### Open sub-questions

| Question | Handled in |
|---|---|
| Queue configuration per switch family; families proposed here, approved by the product owner (C-172) | 028b-19 (ticket) |
| Discovery chatter (including IPv6) suppression on macOS / Windows (owners 021, 022, then this task) | notes in 028b-22, 028b-23 |
| On-site capture hardware for `certify-group` (approval and recording per C-172) | 028b-13 (ticket) |
| Stamp fields for coalescing/EEE, path MTU, platform tuple, connection alarms (C-173), window and expiry; encoding of the window announcement and of the A announcement (C-224) | epic 013 for the stamp fields; epics 004 and 005 for the encoding (C-228); 028b-08, 028b-09 and 028b-11 isolate the dependency |

## Remaining gaps

1. **Stamp fields and window encoding not specified.** Which stamp fields carry coalescing and EEE state, path MTU per connection, the platform tuple (OS build, driver, audio device), the runtime's connection alarms (C-173, including demoted and cannot-meet-A) and the commissioning-window state with its expiry (C-153); and the encoding of the window announcement and close, and of the announcement of a commissioned change of A (C-224 settles who sends it and when, not its encoding). C-47, C-129, C-131, C-152, C-159 and C-160 settle the counters, capability states, DSCP observations and percentiles, not these. Affects 028b-02, 028b-03, 028b-05, 028b-18, 028b-08, 028b-09, 028b-11. Owner: epic 013 (regime stamps), with the wire specification epic 004 for the encoding. The encoding is owned by epics 004 and 005; epic 013 owns the stamp fields only (C-228).
2. **Reference platform list contents.** Reference audio hardware (proposed after P0, approved by the product owner, C-172) and supported OS versions (after task 001, C-138) are not yet recorded. Affects 028b-05. Owner: measurements of epics 020, 021, 022 and task 001, then product owner approval.
3. **Reference figures.** The L2 Linux record figures (for the "within 20 %" check) and the L3 skew result are measurements not yet made. Affects 028b-12, 028b-14. Owner: measurements of epics 020 and 027.
4. **C-91 value.** Rule settled as 10 × the clean-wire loss (C-167, a proposal pending first runs); the loss itself awaits the L1 measurement. Affects 028b-16, 028b-18. Owner: measurement (epic 019, via 028b-16 on the L1 rig).
5. **Supported switch families and capture hardware not yet chosen.** The process is settled (C-172): this epic proposes, the product owner approves, the constants table records. Affects 028b-13, 028b-19, 028b-20. Owner: measurement in 028b-13 and 028b-19, then product owner approval.

Gap numbering of epic 028: items 1 to 5 keep their numbers. Gaps 6 and 7 of epic 028 concern only the tickets of epic 028a and are listed there. Gap 8 (epic split owed, C-143) is closed by this split.

## Settled since the split

| Former gap | Closed by |
|---|---|
| Epic split owed (C-143; epic 028 gap 8) | This split: epic 028 (35 tickets, about 40.25 days) became 028a (11 tickets, about 8.75 days, scope M) and 028b (24 tickets, about 31.5 days, scope L) |
| 3. Receiver reports versus C-47; whether stamp counters carry DSCP, late, lost | C-47 (per-connection counters received, late, missing, marked gap, interrupted, dead, observed DSCP; no receiver reports), with C-108 for late versus missing |
| 5. Trunk capacity direction | C-117 (per direction) |
| 6. AC-10 versus active subcommands | C-107 (test traffic only inside a commissioning window; read-only outside); epic AC-10 rewritten |
| 11. Foreign-multicast detection mechanism | Epic change: passive receive-only capture on the survey host, and C-100 (IPv4 only, so all IPv6 is foreign) |
| 13. "Capability gaps" undefined | C-127, C-128, C-129 (four states; absent or unverified is a gap; strict-mode requirements), C-131 |
| 14. "Sinks not meeting A" undefined | C-101 and C-109 (an input Demoted at the ratchet ceiling A) |
| Stale figures: per-channel id byte, "1 500-byte packet" loss floor, 85/170 Mbit/s, 1 017 Mbit/s, 12-hour soak | C-04, C-20 to C-22, C-90, C-118 (192 bytes per channel, 1 152 bytes of audio; floor per default packet; 84/169/338 Mbit/s; 1 014 Mbit/s; 24-hour two-bundle soak, about 58 losses) |
| Fuzz duration unspecified (10 minutes assumed) | C-95 (60 s per change, 1 hour nightly, corpus kept) |
| Stamp transport and fetch timing (5 s collection assumed) | C-18 (sections within C-17, 10 s refresh, unicast fetch) |
| Code location ("the workspace") | C-133 (`hadronyche-tools` in the design set's repository), C-130 for reports |
| How a commissioning window is opened and closed (former gap 2) | C-153 (announced by the tool on the control group with an expiry; stamped while open; closes at expiry or on explicit close); new ticket 028b-09; 028b-08 checks expiry against the run. The stamp field stays with gap 1 |
| Operator policy without a deliverable (former gap 3) | C-165 (the commissioning configuration sets strict mode, alignment opt-out, pacing override); fields in 028a-03, checks in new 028a-09 with C-155 and C-147, stamped-versus-configured finding in 028b-07 |
| Certificate storage and operator documents (former gap 4) | C-171 (certificates beside the collected stamps in the deployment's data directory; checklists and guide in `doc/operations/`); 028b-10, 028b-20 to 028b-24 |
| Who names the supported switch families (former gap 5, process part) | C-172 (proposed by this epic in P7, approved by the product owner, recorded in the constants table); the choice itself is gap 5 |
| Margin term in the A proposal (epic open sub-question; ticket 028-16) | C-148 (commissioning rule: maximum lateness + D + headroom, the headroom of C-52); 028-16 deleted |
| "Sinks not meeting A" detection from 10 s stamps | C-173, C-174 (the runtime raises the cannot-meet-A and demoted connection alarms; `watch` reads them); 028b-18 |
| DSCP of sync and control traffic | C-152 (holder reports sync DSCP per follower; every module reports hello DSCP per peer); 028b-04 |
| Which E a pass criterion means; numeric E for uncertified sinks | C-149 (platform target E), C-140 (commissioned only; others unknown); 028b-10, 028b-12, 028b-14 |
| L2/L3/L1 test parameters used by `certify-sink`, `certify-group`, `burst-test`, the C-91 rule | C-166, C-167 |
| Counter set and lateness percentiles in stamps | C-159, C-160, C-135 |
| D definition and edge resampler (withdrawn rational-L rule) | C-170, C-105 (revised), C-134 |
| Privileged tests (live capture, NIC changes, network sandbox) | C-146 (separate privileged nightly job); 028a-04, 028b-03, 028b-06 |
| Strict mode on devices and in measurement runs | C-147; 028b-13, 028b-16, 028b-19, 028b-07, 028a-09 |
| Meaning of "unqualified" in C-153 (former gap 2) | C-178 (C-153 revised): ordinary production measurements in stamps are unqualified while a window is open; the tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results, recorded with the window. The provisional reading stands; 028b-10, 028b-12, 028b-14, 028b-16 state it; 028b-02, 028b-17 mark production figures taken in a window; 028b-11 now marks a proposal measured inside a window unqualified (new AC 6); 028b-24, 028b-08, 028b-09 reworded |
| Epic 014 dependency unexplained (former gap 3) | C-196: the unused dependency on epic 014 is removed from the epic header, its Dependencies section and this index; no ticket depended on it |
| Buffer memory against a changed A (no former gap) | C-211: new 028a-10 (`validate`), used by `propose-a --write` (028b-11 AC 7); 028a-03 `buffer_kb`; 028b-18 states the runtime fallback (deepest ring, cannot-meet-A) |
| A changed while inputs are live (no former gap) | C-186, C-198, C-208: 028b-11 restarts its collection after re-alignment; 028b-24 explains it |
| Window of the stamp's lateness figures (no former gap) | C-182: rolling 60 s with the since-live maximum beside it; 028a-11, 028b-11, 028b-18 |
| Certification record contents and embedded restart spread (no former gap) | C-191: sink and holder stamps; an embedded device declares its own restart spread; 028b-10, 028b-12 |
| Commissioned sink with an unlocked clock (no former gap) | C-180: stays commissioned, claim suspended; 028b-10, 028b-14 |
| Tolerance for a pair of different sinks (no former gap) | C-203: the larger of the two, both reported; 028b-14 |
| Failed or unverified pin (no former gap) | C-177, C-201: module refuses to start in every mode, seen as a missing module; 028b-07 |
| Publisher of the clock bound used in headroom and E (no former gap) | C-187: epic 008a added to the epic dependencies, 028b-11, 028b-12 |
| Lateness figure used by the A proposal (no former gap) | C-148: the maximum since the connection went live, never the rolling 60 s figure; 028b-11 (new AC 9) |
| How a commissioned change of A reaches the sinks (no former gap; the epic said "propose, never set live") | C-224: through the tool, inside a declared window (C-153), buffer memory first (C-211), then announced to the group's members; `propose-a --apply` in 028b-11 (new AC 8, epic AC-12), allowed by the guard 028b-08 and noted in 028b-09; epic 012 added as a dependency (receipt, 012-30); 028b-24 walks through it |
| Switch positions after a changed A and ratchet steps (no former gap) | C-208, C-214 (fan-ins and sinks each at their own position; the tool picks none), C-215, C-225 (ratchet steps announced in a fan-in, at its own position in a sink); epic Background, 028b-11, 028b-24 |
| Sinks under the buffer-memory check and the clamp in detail (no former gap) | C-222 (sinks included), C-216 (alarm until a fitting change; receiver report carries the given A); 028b-11, 028b-18, 028b-24 |
| Controller streams in observed trunk rates (no former gap) | C-219 (packet size of a controller stream), C-209 (bundle cap); 028b-17 (new AC 5), epic Background |
| Holder takeover during a certification run or a `watch` (no former gap) | C-213, C-221 (usable-time candidate takes over, epoch kept, Locked preferred, lowest id among equals): streams are not restarted; 028b-12 and 028b-10 record each holder followed, 028b-17 resets no baseline |
| Peer map size (no former gap) | C-217 (63 peers); epic Background item 4 |
| When measured soak loss reopens the redundancy decision (no former gap) | C-179: above 10 × the published floor (C-90), about 580 losses in the 24-hour soak (C-118), reopens it; between 1 × and 10 × it is recorded and investigated as a cabling or host problem. 028b-16 classifies the soak result (new AC 7); 028b-20 states it |

The other closed gaps of epic 028 (configuration shape, default-group A, AC-5's runtime half, follower check, fan-in outside a group, NoSkewBudget) concern only the tickets of epic 028a and are listed in its index.
