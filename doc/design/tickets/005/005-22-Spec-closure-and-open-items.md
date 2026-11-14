# 005-22 — Spec closure and open items
**Epic:** 005-Control-state-machines-specification · **Effort:** XS · **Depends on:** 005-11, 005-17, 005-18, 005-19, 005-20, 005-21, 005-23, 005-25, 005-26, 005-27, 005-28, 005-29

## Context
- Final ticket of epic 005 (control state machines specification, `doc/design/spec/control.md` and `doc/design/spec/control.toml`). Downstream consumers: task 007 (vectors), task 011 (discovery and election implementation), task 012 (admission and connection-state implementation), and epic 004 (regime stamp counters, locking-member silence carrier).
- Sub-questions of the epic:
  - decided by the constants table: quarantine of a colliding id (C-46: until one nonce has been seen for 3 s); Pending timeout (C-45: none; ends by target appearing or operator cancel); election with no candidate with usable time (C-113), who may take over as holder (C-213) and the tie-break among candidates with usable time (C-221: Locked before holdover, then the lowest module id; 005-08); holdover entry delay as a hard limit at C-37 (C-195); peer map capacity of 63 (C-217); ratchet step at an announced position (C-215; 005-16, 005-24, 005-25) and in a sink at its own position one lead ahead (C-225; 005-15, 005-16, 005-24, 005-25); sink switch to a changed A (C-214; 005-29), how a commissioned change of a sink's own A reaches it (C-224; 005-29) and the clamp of a sink that cannot hold it (C-222; 005-29); domain 0 hellos (C-173); holder stall detection (C-136); owner of system AC-20 (c): epic 015 (C-144); carrier of a changed A (C-198, with C-186; rows in 005-28); sync requests above the accepted rate (C-204; rows in 005-09, 005-10); presentation details of the tools (C-206; 005-03, 005-11); refusal enumeration closed at 18 and where VersionMismatch, NameTaken and ModuleCap arise (C-185; 005-04, 005-21); refusal for a fan-in without budget, `NoSkewBudget` (C-185; 005-27); a peer whose housekeeping restarts have stopped refuses with `ControlDegraded` (C-185; 005-14); version-mismatched module not inserted (C-185; 005-06); sync requests answered on the clock thread (C-188; 005-09, 005-17); re-alignment position after a changed A (C-208; 005-28); rings that cannot hold a changed A (C-211; 005-28);
  - decided by this epic's tickets: event per non-admission refusal (005-04); discovery details, removal of a live-pinned peer (005-06); admission machine split (005-13); equal-epoch merge (005-10); producer machine (005-15); new-session reaction, exits from Locking, Aligning, Demoted and reaction to an unusable endpoint clock (C-162) (005-24); locking-member silence carrier (005-16, with epic 004); requester's rows for a peer that never answers (005-14); alarm carrier (005-26, C-173); arcs for opted-out inputs (005-27, C-165);
  - owned elsewhere: is the I/O-pressure eligibility rule inherited? — task 002, falsifier: the POC experiment data behind it is not intact; definition of the Locked state — servo core, epic 008a, which publishes the lock state and bound (C-187; C-62 pending simulation; the state after a holdover lapse is settled by C-162 and the candidacy rule by C-213), a cross-reference, not a dependency (C-196); followers per holder C-35 = 64 — L1 measurement (task 019), falsifier: holder serving lateness grows with followers below 64.
- `[code]` POC behaviour retired by this spec: rotation after 2 mandates (`MANDATES = 2`, `module.rs:62`); election opened by `leave()` on holder hello expiry (`module.rs:575-585`); text messages (`"HADRO/1"`); unilateral send (`module.rs:775-820`); waiting forever for a named target. The frozen POC itself is not modified.
- Definition-of-done applicability (C-125): no line of the epic checklist is "not applicable"; "Every parser touched is fuzzed" covers only the table loader (C-95 budget), and "Regime stamp fields touched" covers the fields mapped in 005-23 (C-18, C-103, C-106, C-140, C-152, C-159, C-204).

## Work
1. Fill the "Open items" section of `control.md`: every remaining open item with owner and falsifier (task 002, servo core epic 008a, L1 measurement task 019, and any item a ticket could not close).
2. Fill "Decisions on open sub-questions" with a pointer to each decision above and its C-id or ticket.
3. Add a "Changes from the POC" list with the five retired behaviours above, each with its `[code]` reference.
4. Check that the constants table is first and that every value equals `doc/design/02-constants.md`; any constant a ticket needed and lacked is reported to the design owner, not invented.
5. Run the full checker (totality, invariant checks, scenario closedness, memory summary, model check, fuzz at the C-95 budget) and record the result.
6. Walk the epic's definition-of-done list and tick each item with the ticket that satisfied it; for any line without a subject, write "not applicable" with the reason (C-125).

## Acceptance criteria
1. "Open items" lists the I/O-pressure question (task 002), the Locked-state definition (servo core, epic 008a, C-187) and the C-35 follower cap (L1, task 019), each with a falsifier; AC-20 (c) is recorded as owned by epic 015 (C-144); no item lacks an owner or falsifier.
2. "Changes from the POC" lists 5 entries with `[code]` references.
3. The constants comparison against `doc/design/02-constants.md` finds 0 differences.
4. All CI jobs of this epic are green on the merge commit, including the nightly 1-hour fuzz run (C-95).
5. A diff of the frozen POC directory shows 0 changes.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Transition tables are total (checked mechanically from the machine-readable form)
- [ ] A model check or exhaustive small-scope exploration shows no reachable state with two holders after convergence

## Out of scope
- Any new behaviour; this ticket only closes and verifies.
- Resolving the I/O-pressure question (task 002), the Locked-state definition (epic 008a) or the C-35 measurement (task 019).
- The housekeeping-restart behaviour and AC-20 (c) (epic 015, C-144).
