# 019-22 — Constants write-back
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-06, 019-07, 019-08, 019-09, 019-10, 019-11, 019-12, 019-13, 019-14, 019-15, 019-16, 019-17, 019-18, 019-19, 019-20, 019-21, 019-24, 019-25, 019-26, 019-27

## Context
Epic 019 must measure or check, and write back to the constants table `doc/design/02-constants.md`:

| Constant | Currently | Measured how | Ticket |
|---|---|---|---|
| C-93 receive cost per packet | 2 to 4 µs, guessed `[memory]` | Data-thread time in batch receive divided by packets, at 2 000, 32 000 and 64 000 packets/s | 019-10 |
| C-92 two-bundle capacity | pending-measurement | `two-bundles` (AC-13) | 019-08 |
| C-90 loss floor | about 1.1e-8 per default packet, from memory `[memory]` | `loss-soak`, 24 h at 64 000 packets/s, aggregate (C-118) | 019-20 |
| C-91 loss alarm threshold | rule: 10 × the measured clean-wire loss (C-167); value unset | Applied to the clean-network loss measured here | 019-21 |
| C-94 wired jitter profile | provisional, synthetic | One-way delay distribution | 019-24 |
| C-35 followers per holder | 64, decided (owner L1, task 019) | `serving`: serving lateness against follower count (C-167) | 019-16 |
| C-14 egress spread | 25 % of the slice period, decided | Falsifier check, spread on and off | 019-09 |
| C-16 bounded drain | 2 × admitted packets per slice, decided | Falsifier check, largest legitimate burst | 019-25 |
| Coalescing bias | unknown | `coalescing` | 019-11 |
| C-164 bound on processing one bad input | linear in packet length, at most 10 µs per packet, proposal pending L1 | `bad-input` | 019-27 |

The falsifier of C-50 to C-52 (late slices after alignment on a clean network) is checked by AC-14 (019-07) and AC-13 (019-08).

Only strict-mode runs may supply a value to the constants table (C-147): a report with `eligible_for_constants = false` (019-04) is not used. Each value is stated with the A the harness computed for its run (C-148).

It also provides to the commissioning tool (task 028) the measured constants and switch findings, and receive-cost data that decides whether the 10 GbE rule (any port above about 198 channel-copies, C-25) needs to be tightened. The criterion is the C-93 falsifier: tighten if measured receive cost makes C-92 fail. The measured jitter profile goes to the simulation owner (C-94). Design documents must be updated where behaviour differs, constants table first.

## Work
- Update the constants table: for each measured row above, the measured value (or reason why not), tag `[measured]` (C-91: 10 × the `[measured]` clean-wire loss, C-167), status changed from pending-measurement where the value is now measured, and a link to its `doc/design/reports/l1-results.md` section. For C-14, C-16 and C-35 record the falsifier verdict; for C-164 record the measured bound and whether the proposal holds.
- Finalise `doc/design/reports/l1-results.md`: every constant section has measured value, hardware, stamp; summary of acceptance-criterion results (AC-13, AC-14, AC-24, AC-25, AC-19 L1 part, AC-34, AC-37, AC-11 L1 part, 019-A, 019-B, 019-C, 019-D) with pass/fail and link to the report.
- C-25: state, from the C-92 result and the C-93 data at 64 000 packets/s, whether the C-93 falsifier triggered and so whether the 10 GbE rule should be tightened; if so open an item (owner, falsifier).
- Collect the switch findings (spread/burst, blackhole) into a short hand-off section for task 028, and the measured jitter profile into a hand-off for the simulation.
- For C-90, keep the predicted floor's tag as the table has it (`[memory]`, or `[standard]` with clause once checked, C-132) beside the `[measured]` L1 value.
- Update any design document whose described behaviour differs from what L1 measured.
- Check that every source report used is strict-mode eligible (C-147) and states its A (C-148).
- Review every open item created by 019 tickets for owner and falsifier.

## Acceptance criteria
1. The constants table has 10 updated rows (C-93, C-92, C-90, C-91, C-94, C-35, C-14, C-16, coalescing bias, C-164), each with a value or falsifier verdict tagged `[measured]` (C-91 with its C-167 basis) or a stated reason why not.
2. `l1-results.md` lists all 12 epic acceptance criteria with pass/fail and a link to the evidence.
3. A C-25 statement (tighten / do not tighten, with reason from the C-93 falsifier) exists.
4. A task-028 hand-off section lists the switch findings, and a simulation hand-off gives the C-94 profile.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Each pending-measurement constant in the table above has a measured value tagged `[measured]`, or a stated reason why not
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The named hardware pair (hosts, NICs, switch, cables) is recorded
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Running any measurement (earlier tickets).
- Implementing changes in the commissioning tool (task 028), the simulation or the runtime.
