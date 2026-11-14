# 028b-19 — Queue configuration per switch family
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028b-04, 028b-15

## Context
Open sub-question owned by this task: queue configuration per supported switch family. The DSCP values themselves are settled (C-13): sync **CS6 (48)**, media and one-way probes **EF (46)**, control **CS5 (40)**. Settled (C-172): supported switch families are proposed by the implementing team in P7, that is by this task, approved by the product owner and recorded in the constants table. Falsifier: a recommended configuration that still drops sync under media bursts.

Bursts: two 96-channel bundles into one port = 32 packets ≈ 42 KB per 0.5 ms slice, spread over ≤ 25 % of the slice (C-14). On an unmanaged switch marking does nothing.

## Work
- Propose the switch families to support (the families present in the lab at least), and once the product owner approves them record them in the constants table first (C-172).
- For each switch family, define the queue mapping for 48/46/40 and buffer settings.
- Verify inside a commissioning window (C-107) with `burst-test` (028b-15) plus sync traffic that sync is not dropped under media bursts; verify marking with `survey` DSCP findings for all three classes (028b-04, C-47, C-152). Runs whose results enter the constants table run in strict mode (C-147).
- Write a report in `doc/design/reports/` (C-130) per family: settings, measured result, evidence tag. Close or re-own the epic's open sub-question.

## Acceptance criteria
1. For each tested family, a burst test of two 96-channel sources into one port shows zero sync drops with the recommended configuration.
2. `survey` shows no `dscp.mismatch` on the tested switch.
3. The approved families are recorded in the constants table (C-172), and the epic's open sub-questions table is updated.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Writing the operator checklist (028b-20).
