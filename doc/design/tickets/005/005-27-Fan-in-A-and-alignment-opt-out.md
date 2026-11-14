# 005-27 — Fan-in A inheritance and alignment opt-out
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-14, 005-16, 005-24

## Context
- Machine 3 (admission handshake) and machine 4 (connection states) of `doc/design/spec/control.md`; tables in `doc/design/spec/control.toml` (`admission` from 005-14, `connection_consumer` from 005-16, guards from 005-24).
- C-142: a fan-in outside any presentation group inherits the A of the downstream presentation group it feeds (the largest if several), learned at admission. With no downstream group it needs an explicit budget in configuration, or its aligned inputs are refused.
- A change of the inherited A after admission is carried by the receiver reports (C-198) and handled by 005-28; this ticket covers the A learned at admission.
- Admission already checks buffer memory with alignment rings sized to A (C-96, `BufferMemory`). The refusal enumeration is closed at 18 reasons (C-173, C-185); C-185 names the reason for C-142: `NoSkewBudget`.
- C-165: the operator policy in the commissioning configuration sets per-input alignment opt-out. The connection arcs reach Live only through Aligning (C-50); an opted-out input has no arc yet.
- Open sub-question owned by this ticket (epic 005): the arcs of an opted-out input. Falsifier: an opted-out input waits in Aligning. (The refusal reason for a fan-in with no downstream group and no budget is settled: `NoSkewBudget`, C-185.)

## Work
1. Add to the admission state data the A in force for the request: the presentation group's A, the inherited A (largest downstream group's A, C-142), or the configured budget; and the alignment opt-out flag per input (C-165).
2. Add admission rows: A learned at admission per C-142; refusal of an aligned input when there is no downstream group and no budget, with `NoSkewBudget` (C-185); an opted-out input skips the A-dependent checks that do not apply to it.
3. Decide the connection arcs for an opted-out input (for example whether Admitted leads to Live without Aligning) and add the rows to `connection_consumer`; record the decision in "Decisions on open sub-questions" with a `[decision]` tag and the falsifier.
4. Add two scenario scripts to the "Scenarios" section: S12 a fan-in feeding two presentation groups inherits the larger A; S13 a fan-in with no downstream group and no budget has its aligned input refused, and the same fan-in with an opted-out input is admitted.

## Acceptance criteria
1. With two downstream groups, the A used at admission is the larger one (C-142).
2. With no downstream group and no budget, an aligned input is refused with `NoSkewBudget` (C-185); with a configured budget it is admitted against that budget.
3. An opted-out input never stays in Aligning; the decided arc is in both tables and the checker of 005-03 passes.
4. The opted-out arc decision is recorded with its falsifier; S12 and S13 use only inputs present in the tables.

## Definition of done
- [ ] Transition tables are total (checked mechanically from the machine-readable form)
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- How commissioning proposes A (C-55) and the commissioning configuration format (other epics).
- Ring sizing and the alignment data path (other epics).
- Propagation of a changed A after admission and the resulting re-alignment (005-28, C-198, C-186).
