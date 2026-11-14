# Epic 028a — Commissioning configuration and offline validation: tickets

## Summary
Epic 028a (phase P7, scope M: 11 tickets, about 8.75 days) is the first of the two lettered parts into which epic 028 (commissioning tool) was split under C-143; epic 028b is the second and depends on this part. It builds the offline half of the operator-facing tool `hadronyche-commission` (crate `hadronyche-tools` in the design set's repository, C-133): the binary skeleton with a stub for every subcommand and the read-only rule fixed for the whole tool (C-107, C-153), the capacity arithmetic (C-01 to C-04, C-20 to C-26, with controller streams at the C-219 cost and the controller member in the bundle cap, C-209), the declarative configuration format with operator policy (C-165), fan-in budgets (C-142) and declared buffer memory (C-211), and the `validate` command, which checks a configuration with no network: domain id (C-11), module count (C-34, which also bounds followers, C-114) and a holder-capable node, endpoint port sums at the 70 % ceiling (C-23) and the GbE wall (C-24, C-25), declared trunks per direction (C-117), group A (C-55) and required E including unknown E (C-103), operator policy with the pacing fit (C-155), fan-in budgets with refusal reason NoSkewBudget (C-185), and declared buffer memory against the A each aligning module holds, sinks included (C-211, C-222). It also delivers the pure A proposal computation (commissioning rule: maximum lateness since the connection went live + D + headroom, C-148), which `propose-a` in epic 028b calls. It depends on epic 012 only.

## Tickets

| Id | Title | Effort | Depends on |
|---|---|---|---|
| 028a-01 | Crate and CLI skeleton | XS | none |
| 028a-02 | Capacity arithmetic module | S | 028a-01 |
| 028a-03 | Configuration format and parser | S | 028a-01 |
| 028a-04 | `validate` command and domain id check | XS | 028a-03 |
| 028a-05 | `validate`: scale limits and holder-capable node | XS | 028a-04 |
| 028a-06 | `validate`: endpoint port sums and 10 GbE wall | M | 028a-02, 028a-03, 028a-04, epic 012 |
| 028a-07 | `validate`: trunk capacities per direction | S | 028a-02, 028a-03, 028a-04 |
| 028a-08 | `validate`: presentation groups, A and required E | S | 028a-03, 028a-04 |
| 028a-09 | `validate`: operator policy and fan-in budgets | S | 028a-03, 028a-04, 028a-08 |
| 028a-10 | `validate`: declared buffer memory against A (new, C-211) | XS | 028a-03, 028a-08, 028a-09 |
| 028a-11 | A proposal computation library | XS | 028a-02 |

The order is a valid execution order. No ticket of this part depends on epic 028b. Tickets of epic 028b that depend on this part: 028b-01 and 028b-10 (on 028a-01), 028b-07 and 028b-14 (on 028a-03), 028b-11 (on 028a-03, 028a-10, 028a-11), 028b-15 (on 028a-02), 028b-17 (on 028a-02, 028a-03, 028a-07), 028b-24 (on 028a-04 to 028a-10).

### Old ticket numbers

| Epic 028 ticket | New id |
|---|---|
| 028-01 | 028a-01 |
| 028-02 | 028a-02 |
| 028-03 | 028a-03 |
| 028-04 | 028a-04 |
| 028-05 | 028a-05 |
| 028-06 | 028a-06 |
| 028-07 | 028a-07 |
| 028-08 | 028a-08 |
| 028-35 | 028a-09 |
| 028-36 | 028a-10 |
| 028-17 | 028a-11 |
| 028-16 | deleted before the split (margin term settled by C-148) |
| 028-09 to 028-15, 028-18 to 028-34 | epic 028b (see its index) |

File names kept their slug after the new id. Ticket ids inside ticket texts (dependencies, cross-references) were renumbered; nothing else in the tickets changed apart from the epic name in each header.

### History before the split (old numbers of epic 028)

Changes since the split into tickets: 028-05 went from S to XS (the follower check is implied by the module limit, C-114). The capability-gap finding moved from 028-10 to the new 028-33, and 028-10 gained the identity-collision finding (C-46); file renamed to `028-10-Survey-framework-collisions-and-path-MTU.md`. 028-26 now covers queue configuration only, the DSCP values being settled (C-13); file renamed to `028-26-Queue-configuration-per-switch-family.md`.

Changes in the final update (rules up to C-176): **028-16 deleted** (file `028-16-A-proposal-margin-decision.md` removed): its only purpose was deciding the margin term, which C-148 settles as the consumer headroom C-52. 028-17 went from S to XS (no margin option left; a fixed formula, truncation check and rounding). New tickets: 028-34 announces and closes the commissioning window (C-153); 028-35 checks the operator policy that the configuration now carries (C-165) and fan-in budgets (C-142). 028-01 gained the `window` subcommand stub; 028-03 gained the policy and budget fields; 028-33 gained the configured-versus-stamped strict-mode finding. No ticket dealt with the withdrawn rational-L rule (C-104, C-105 revised); 028-13, 028-29 and 028-30 now state that the edge resampler exists only inside an edge module, for 2:1 and 1:2 only, and that its delay is part of D (C-105, C-134, C-170).

Changes in the sweep of rules C-177 to C-212: **new 028-36** (XS) checks declared buffer memory against the A each aligning module holds and is called by `propose-a --write` (C-211); 028-03 gained the optional `buffer_kb` field. 028-17 and 028-18 read lateness over the stamp's rolling 60 s window with the since-live maximum beside it (C-182); 028-18 gained the re-alignment after a changed provisional A (C-186, C-198, C-208) and the buffer-memory refusal on `--write` (new AC 7). 028-15 and 028-19 carry the sink's and holder's stamps and the embedded device's declared restart spread (C-191); 028-15 and 028-21 keep a commissioned sink's record while its clock is unlocked (C-180); 028-21 applies the pair tolerance rule (C-203). 028-35 names NoSkewBudget (C-185); 028-33 states that a failed or unverified pin stops the module in every mode (C-177, C-201); 028-25 uses the rolling window and the C-211 fallback; 028-02 and 028-06 note controller streams (C-181, C-205, C-209). 028-18 and 028-19 depend on epic 008a (C-187).

## Effort tally
- XS: 5 (1.25 days)
- S: 5 (5 days)
- M: 1 (2.5 days)
- Total: 11 tickets, about 8.75 days → scope M (C-126)

## Coverage

### Deliverables

| Deliverable | Tickets |
|---|---|
| Crate `hadronyche-tools`, binary `hadronyche-commission` (stub for every subcommand, exit codes, CI) | 028a-01 |
| Capacity arithmetic (C-01 to C-04, C-20 to C-26; controller streams, C-209, C-219) | 028a-02 |
| Configuration file format (nodes, modules, trunks with endpoints, groups including `default`, connections, operator policy, fan-in budgets, `buffer_kb`) | 028a-03 |
| `validate`: domain id | 028a-04 |
| `validate`: module count ≤ 64 (bounds followers at 63, C-114), holder-capable node | 028a-05 |
| `validate`: per-port sums ≤ 70 %, ports above ~198 channel-copies flagged for 10 GbE | 028a-06 |
| `validate`: trunks per direction against declared capacities | 028a-07 |
| `validate`: presentation groups, A and required E consistent (including unknown E) | 028a-08 |
| `validate`: operator policy (strict mode, alignment opt-out, pacing fit) and fan-in budgets | 028a-09 |
| `validate`: declared buffer memory against A (C-211), also exposed for `propose-a --write` (028b-11) | 028a-10 |
| A proposal computation (commissioning rule, C-148), used by `propose-a` (028b-11) | 028a-11 |

### Acceptance criteria

| Epic AC | Tickets |
|---|---|
| AC-1 Trunk validation (800 Mbit/s in one direction on 1 000 refused, ceiling 700 per direction) | 028a-07 |
| AC-2 Port wall ("port would carry 1 014 Mbit/s; use 10 GbE") | 028a-02, 028a-06 |
| AC-3 Scale (65 modules; no holder-capable node) | 028a-05 |
| AC-5 Group E (1 ms sink and unknown-E sink refused by `validate` in a 100 µs group) | 028a-08 |
| AC-11 A against declared buffer memory, `validate` half (C-211) | 028a-03, 028a-10 |
| AC-4 per-sink breakdown (part; AC-4 itself is epic 028b's) | 028a-11 |

### Definition of done

| Epic DoD item | Tickets |
|---|---|
| Tests green in CI, unprivileged | all tickets (all are code tickets) |
| Every parser touched is fuzzed (budget C-95) | 028a-03, 028a-04 |
| Regime stamp fields touched are complete and documented | not applicable: no ticket of this part reads or defines a stamp field (C-125) |
| Design documents updated where behaviour differs, constants table first | 028a-02, 028a-03 |
| No new open item without an owner and a falsifier | no ticket opens one; the open sub-questions are owned by epic 012 |
| The frozen POC is not modified | 028a-01 (applies to all; code lives in `hadronyche-tools`, C-133) |
| Every new factual claim carries an evidence tag | all tickets that state facts (028a-02, 028a-03, 028a-10, 028a-11) |
| `validate` runs with no network and has tests for every refusal | 028a-04 to 028a-10 |
| Each checklist item states how to verify it and what the stamp shows | not applicable: checklists are epic 028b's (C-125) |
| The commissioning guide walks through one complete medianet | not applicable: the guide is epic 028b's (C-125) |

### Open sub-questions

| Question | Handled in |
|---|---|
| Runtime port-wall refusal text (1 014 Mbit/s) | epic 012 |

## Remaining gaps

1. **Runtime refusal text for AC-2.** Three copies of a 96-channel source are 1 014 Mbit/s; the runtime's own refusal (runtime AC-12) must quote the same figure. Affects 028a-06. Owner: epic 012.

Gap numbering of epic 028: item 1 here was gap 6; gap 7 (wire cost of a controller stream) is closed by C-219, see below. Gaps 1 to 5 of epic 028 concern only the tickets of epic 028b and are listed there. Gap 8 (epic split owed, C-143) is closed by this split.

## Settled since the split

| Former gap | Closed by |
|---|---|
| Epic split owed (C-143; epic 028 gap 8) | This split: epic 028 (35 tickets, about 40.25 days) became 028a (11 tickets, about 8.75 days, scope M) and 028b (24 tickets, about 31.5 days, scope L) |
| 1. Configuration shape missing nodes, OS, ports, channel counts, sink E | Epic change: configuration shape in the epic Background and sample (nodes, modules, trunk endpoints); holder/follower roles dropped (C-113, C-114) |
| 2. A for the implicit default group | Epic change: a `[[group]]` named `default` with no members, `a_ms` required (C-55) |
| 5. Trunk capacity direction | C-117 (per direction) |
| 16. AC-5's runtime half | Epic change: AC-5 now covers `validate` only and names runtime AC-30 as verified elsewhere; C-103 adds the unknown-E case |
| Stale figures: per-channel id byte, "1 500-byte packet" loss floor, 85/170 Mbit/s, 1 017 Mbit/s, 12-hour soak | C-04, C-20 to C-22, C-90, C-118 (192 bytes per channel, 1 152 bytes of audio; floor per default packet; 84/169/338 Mbit/s; 1 014 Mbit/s; 24-hour two-bundle soak, about 58 losses) |
| Follower check against a named holder | C-113, C-114 (holder elected; at most 63 real followers under the 64-module limit) |
| Fuzz duration unspecified (10 minutes assumed) | C-95 (60 s per change, 1 hour nightly, corpus kept) |
| Code location ("the workspace") | C-133 (`hadronyche-tools` in the design set's repository), C-130 for reports |
| Operator policy without a deliverable (former gap 3) | C-165 (the commissioning configuration sets strict mode, alignment opt-out, pacing override); fields in 028a-03, checks in new 028a-09 with C-155 and C-147, stamped-versus-configured finding in 028b-07 |
| Margin term in the A proposal (epic open sub-question; ticket 028-16) | C-148 (commissioning rule: maximum lateness + D + headroom, the headroom of C-52); 028-16 deleted |
| Fan-in outside any presentation group | C-142 (inherits the downstream group's A; otherwise an explicit budget); 028a-09 |
| D definition and edge resampler (withdrawn rational-L rule) | C-170, C-105 (revised), C-134 |
| Privileged tests (live capture, NIC changes, network sandbox) | C-146 (separate privileged nightly job); 028a-04, 028b-03, 028b-06 |
| Strict mode on devices and in measurement runs | C-147; 028b-13, 028b-16, 028b-19, 028b-07, 028a-09 |
| Epic 014 dependency unexplained (former gap 3) | C-196: the unused dependency on epic 014 is removed from the epic header, its Dependencies section and this index; no ticket depended on it |
| Buffer memory against a changed A (no former gap) | C-211: new 028a-10 (`validate`), used by `propose-a --write` (028b-11 AC 7); 028a-03 `buffer_kb`; 028b-18 states the runtime fallback (deepest ring, cannot-meet-A) |
| Window of the stamp's lateness figures (no former gap) | C-182: rolling 60 s with the since-live maximum beside it; 028a-11, 028b-11, 028b-18 |
| Refusal reason for an unbudgeted fan-in (no former gap) | C-185 (NoSkewBudget); 028a-09 |
| Wire cost of a controller stream (former remaining gap 2, epic 028 gap 7; open sub-question owned by epic 012) | C-219 (one packet per slice of headers up to 100 bytes + controller bytes per slice + 66 bytes, at 2 000 packets/s, the same figure as admission) and C-209 (controller member counts towards the 32; at most 31 audio members, 186 channels): `capacity` function and tests in 028a-02, `controller_bytes` and `header_bytes` in 028a-03, port sums and the `bundle.controller_cap` refusal in 028a-06, trunk sums in 028a-07 |
| Lateness figure used by the A proposal (no former gap) | C-148: the maximum since the connection went live, never the rolling 60 s figure; 028a-11, 028b-11 |
| Sinks under the buffer-memory check and the clamp (no former gap) | C-222 (any consumer that buffers to meet A, sinks included), C-216 (alarm until a fitting change; receiver report carries the given A), C-214 (sink switches at its own position); 028a-03, 028a-10 |
| Who applies a commissioned change of A (no former gap) | C-224 (the tool, inside a declared window, buffer memory first, then the announcement to the group's members); check in 028a-10, applied by 028b-11 |
| Peer map size under the module limit (no former gap) | C-217 (63 peers); 028a-05 notes it, the 64-module refusal stays the only check |

The other closed gaps of epic 028 concern only the tickets of epic 028b and are listed in its index.
