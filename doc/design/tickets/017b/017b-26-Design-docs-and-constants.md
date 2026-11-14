# 017b-26 — Design documents and constants for the L0 harness
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017b-01 to 017b-25

## Context
- Definition-of-done items of epic 017b that span all its tickets: design documents updated where behaviour differs (**constants table first**); regime stamp fields documented; no new open item without an owner and a falsifier; every new factual claim carries an evidence tag (`[code]`, `[measured]`, `[kernel-doc]`, `[memory]`, `[decision]`, and `[standard]` for a figure cited from a named published standard with its clause, C-132); the frozen POC is not modified (task 003 is the only exception).
- **Locations (settled, C-130, C-171):** constants in `doc/design/02-constants.md`; evidence in `doc/design/06-evidence-appendix.md`; reports and results (the resynchronisation-bound measurement of 017b-11) in `doc/design/reports/`; specifications in `doc/design/spec/`; the architecture specification is `doc/design/01-architecture.md`. Nothing is written into the frozen POC tree. Harness binaries live in `hadronyche-tools` (C-133).
- Behaviour changes introduced by this part that the design must reflect: the injection API in `hadronyche-std::inject`; the test-only hooks (017b-06, 017b-07, 017b-08; C-120, C-145); new stamp fields for the injection log (017b-05); the record-file format (017b-09), which later epics read and which therefore goes into the specifications, while the runner's other presentation details stay in the tool's documentation (C-206); the definition of the L0 baseline scenario (017b-10, C-192); the resynchronisation bound after loss (decided in 017b-11); the `housekeeping` scenario's assertions (017b-23, from C-139).
- Open sub-questions to carry with owner and falsifier: resync bound (017b-11); reorder slot definition (017b-03); record-file format (017b-09); `ModuleHandle`, `StreamHandle`, `ThreadRole`, `SocketKind` and the sync-rate mechanism (epics 015, 016).
- Ported POC pieces: this part ports none (C-125); provenance is audited in 017a-12.

## Work
1. Update the constants table with any new constant from this part (resync bound, reorder slot definition), then the prose.
2. Document every stamp field this part adds (injection log, `injected`, hook records) in the stamp documentation owned by epic 013.
3. Check the frozen POC tree is unchanged.
4. List any new open item with owner and falsifier.

## Acceptance criteria
1. Constants table contains every constant introduced by tickets 017b-01 to 017b-25, each with an evidence tag.
2. Every stamp field added by 017b-05 to 017b-08 is documented (name, type, unit, meaning).
3. Diff of the frozen POC tree against its frozen state is empty.
4. Every open item added has an owner and a falsifier.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Regime stamp fields touched are complete and documented
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- New behaviour or code (all earlier tickets).
- Provenance of the ported modules and their constants (017a-12).
