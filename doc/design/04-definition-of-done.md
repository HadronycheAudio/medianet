# 04 — Definition of done

Two tiers: one for each implementation task, one for each roadmap phase.

## DoD-item: every implementation task

A task in [implementation/](implementation/) is done when every line holds. Each task file repeats this checklist and adds its own items.

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag

Clarifications:

- **Unprivileged CI** means no real-time scheduling, no memory locking and no root. Anything that needs privilege or hardware is a ladder run (L1 to L3), not CI.
- **Privileged tests** that need neither a second machine nor audio hardware (a real wall-clock step, for instance) run in a separate privileged nightly job (C-146); "unprivileged" applies to per-change CI.
- **A line that cannot apply to a task** (a task with no parser, or one that touches no stamp field) is marked not applicable with its reason, never silently dropped (C-125).
- **"Constants table first"** means [02-constants.md](02-constants.md) is edited before any other file repeats a changed value.
- **A measured number that replaces a proposal** changes the constant's status from pending to decided in the same change, with the measurement recorded in the evidence appendix.
- **No smoke assertion may depend on sync-rate timing.** The POC's timeout-paced sync delivered 37 and 77 exchanges per second where 50 and 1 000 were configured; an assertion tied to that rate would have encoded the bug.
- **A check family with zero samples is a failure**, in every harness.

## DoD-release: every roadmap phase

A phase in [roadmap.md](roadmap.md) is done when its exit gate holds. The gate is a set of acceptance criteria from [03-acceptance-criteria.md](03-acceptance-criteria.md) plus documents.

| Phase | Exit gate |
|---|---|
| P0 Pre-implementation | POC under version control with the untouched state tagged; tasks 001, 002, 003 reported; every `[memory]` platform claim in the capability table either confirmed or corrected; every POC instrument bug has "mechanism named" |
| P1 Specification | Wire, state-machine and timeline-epoch specifications reviewed; conformance vectors exist for every message in both directions including malformed cases; no decision-shaped item remains in the open list |
| P2 Servo simulation | AC-2 to AC-7 pass in simulation; constants C-60 to C-63 are confirmed or replaced and their status updated; alignment estimator and ratchet (C-50 to C-54) simulated |
| P3 Core crate | AC-1 passes; core builds for a `no_std` target; AC-10, AC-15 to AC-18 and AC-23 pass against the core with a simulated transport |
| P4 Hosted crate, Linux edge, L1 and L2 | AC-8, AC-9, AC-11 to AC-14, AC-19 to AC-22, AC-24 to AC-26, AC-30, AC-31, AC-34, AC-36, AC-37 pass; macOS and Windows backends reach AC-27 and AC-28 or their targets are replaced by measured values |
| P5 DSL compiler | AC-32 and AC-33 pass; the reference echo is a DSL program and still passes the exact tier |
| P6 Embedded and L3 | AC-29 and AC-35 pass |
| P7 Commissioning and documentation | The commissioning tool proposes A, validates trunks, certifies group membership and refuses more than 64 modules; per-OS checklists published; every pending constant is either decided or carries a dated measurement plan |

## Standing conditions, checked at every release gate

- **POC untouched and tagged.** The POC changes only in task 003, and both the before and after states are tagged. Nothing else writes to it.
- **Archaeology before citation.** No document cites the POC as the exact-tier baseline until task 002 has named the mechanism of each instrument bug and the partly read files have been read.
- **Open list hygiene.** Every item in [05-risks-and-open.md](05-risks-and-open.md) has an owner that is a milestone or a verification, and a falsifier. A decision-shaped open item is a defect in the design set and blocks the gate.
- **One value per constant.** A scripted check confirms that numbers repeated in ADRs and task files match the constants table.
- **Retired stays retired.** Reintroducing anything on the retired list requires a new ADR that names the evidence.
