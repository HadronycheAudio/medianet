# 013-19 — Unqualified and privilege labels in outputs
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-10, 013-15, 013-17

## Context
- Rule 9: numbers produced before the self-test passes are labelled **unqualified**. `Stamp::qualified()` is false while the self-test has not passed, any injection is active, or a commissioning window is open (C-153). `[decision]`
- 013-AC-6: unqualified numbers are labelled as such **in every output**. The epic (Rule 13) enumerates the outputs of this epic: **the record-set files, the collector's store and the end-of-run summary**. The commissioning tool and monitor (epics 028, 029) read the stamp themselves.
- AC-31 (013-AC-2): injection and missing privileges are labelled. The epic (Rule 13) defines **missing privileges as the capabilities `rt_scheduling` and `memory_locked` when Absent or Unverified**. A capability in the fourth state, **not requested** (C-129), is a deliberate configuration choice, not a denial, and is labelled `NOT REQUESTED: <capability>`, never as a missing privilege.
- Rule 7: an injected run can never pass as a real one. The test-only hooks of C-120 (forced module id and nonce, forced timeline epoch, raw packet) and C-145 (panic injected into a data, clock or housekeeping thread) are injections and are labelled like any other.
- `[code]` Retired POC behaviour: warnings about missing privileges are collected in a process-global list and shown in the UI (`rt.rs:7-19`) but never written next to the numbers.
- Commissioning window (C-153, C-178): ordinary production measurements (latency, loss, alignment figures) are unqualified while a window is open. An open window therefore sets `UNQUALIFIED` on them and is also named by its own label. The commissioning tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results recorded with the window they were taken in (C-178); they are epic 028's output and are not labelled unqualified by this ticket.
- The label text and its placement are presentation details of this epic's outputs (C-206): chosen here and recorded in `doc/design/spec/stamp.md` only where another epic or an operator's script depends on them.

## Work
In `hadronyche-std`, for each of the three outputs:
1. Derive labels from the record set's stamp: `UNQUALIFIED` when `qualified()` is false; `INJECTED` with the injection kinds when an injection is active; `MISSING PRIVILEGE: <capability>` for `rt_scheduling` or `memory_locked` when Absent or Unverified; `NOT REQUESTED: <capability>` when either is not requested (C-129); `COMMISSIONING WINDOW` when the window is active.
2. Write labels next to the numbers, not only in a separate warning list.

## Acceptance criteria
1. A run with failed self-test: every emitted figure in each of the 3 outputs is labelled unqualified (test scans all outputs).
2. A run with one active injection (one test case with a classic kind, one with a C-120 hook, one with the C-145 panic hook): outputs are labelled unqualified and name the injection kind.
3. A run with a commissioning window open and the self-test passed: outputs are labelled `UNQUALIFIED` and `COMMISSIONING WINDOW` (C-153).
4. A run with `rt_scheduling = Absent`, and one with `memory_locked = Unverified`: outputs name the missing privilege next to the figures.
5. A run with `memory_locked = NotRequested`: outputs show `NOT REQUESTED: memory_locked` and no missing-privilege label.
6. A qualified run with both privileges Present: no such labels.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Every parser touched is fuzzed" (no parser); "Regime stamp fields touched are complete and documented", "Every stamp field has a stated source on each of the four profiles" and "The stamp format is versioned independently of the media format" (reads the stamp, changes no field); "Design documents updated where behaviour differs, constants table first" (follows the epic's Rule 13, C-153, C-178 and C-206).

## Out of scope
- UI of the commissioning tool and monitor (epics 028, 029); recording the commissioning tool's own results with their window (epic 028, C-178).
