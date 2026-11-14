# 017a-12 — Design documents, constants and provenance for the reference modules
**Epic:** 017a-Reference-modules · **Effort:** S · **Depends on:** 017a-01 to 017a-11, 017a-13

## Context
- Definition-of-done items of epic 017a that span all its tickets: design documents updated where behaviour differs (**constants table first**); no new open item without an owner and a falsifier; every new factual claim carries an evidence tag (`[code]`, `[measured]`, `[kernel-doc]`, `[memory]`, `[decision]`, and `[standard]` for a figure cited from a named published standard with its clause, C-132); each ported POC piece cites its source file and the archaeology (epic 002) finding that cleared it; the frozen POC is not modified (task 003 is the only exception).
- **Locations (settled, C-130, C-171):** constants in `doc/design/02-constants.md`; evidence in `doc/design/06-evidence-appendix.md`; reports and results (including epic 002's archaeology findings and the cross-machine result of 017a-10) in `doc/design/reports/`; specifications in `doc/design/spec/`; the architecture specification is `doc/design/01-architecture.md`. Nothing is written into the frozen POC tree. The reference modules live in `hadronyche-reference`; protocol types in `hadronyche-core` (C-133).
- Behaviour changes introduced by this part that the design must reflect: the reference mixer is a position-aligned fan-in at `T(p) + offset` (POC on-arrival mixing retired), taking its ceiling from the downstream presentation group or an explicit budget, refusing with NoSkewBudget otherwise (C-142, C-185), and following a changed inherited A (C-186, C-198, C-208, C-211, ticket 017a-13); the sender's volume rides in the audio feed packet and its parts in a second controller stream (C-181); the analyzer report and cross-machine report layouts are recorded in this part's documentation (C-206); the fixed-point sine table is shipped as a constant; transport state is held on a gap and its playhead advanced only while playing (C-141, C-200) and starts stopped, at zero, at 120 beats per minute (C-184, C-199); the mixer's volume is the reference dense controller (C-192); reference modules declare a whole-sample L and contain no resampler (C-105, C-134).
- Open sub-questions to carry with owner and falsifier: voice windows exact vs tolerance and tolerance epsilons (epic 002); resync bound (017b-11); received-positions bitmap (deferred past v1); echo degraded-output timing (epic 009). The dense-controller user is settled: the mixer's volume (C-192).
- Regime stamp fields: this part adds none (C-125); stamp fields added by epic 017b are documented in 017b-05 and 017b-26.

## Work
1. Update the constants table with any new constant from this part (analyzer constants 1 024 samples / 256 windows / 3 s, tolerance epsilons), then the prose.
2. Audit every ported file for its POC citation and epic 002 finding (from `doc/design/reports/`); add missing ones.
3. Check the frozen POC tree is unchanged.
4. List any new open item with owner and falsifier.

## Acceptance criteria
1. Constants table contains every constant introduced by tickets 017a-01 to 017a-11 and 017a-13, each with an evidence tag.
2. Every file under `hadronyche-reference/src/{instrument,analyzer,transport}` and `sender.rs`, `echo.rs`, `mixer.rs` has a provenance header with a POC path and an epic 002 finding id.
3. Diff of the frozen POC tree against its frozen state is empty.
4. Every open item added has an owner and a falsifier.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Each ported POC piece cites its source file and the archaeology finding that cleared it

## Out of scope
- New behaviour or code (all earlier tickets).
- Injection API, hooks, L0 runner and scenarios, and their stamp fields (017b-26).
