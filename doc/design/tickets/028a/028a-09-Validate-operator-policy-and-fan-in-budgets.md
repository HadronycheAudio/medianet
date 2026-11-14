# 028a-09 — `validate`: operator policy and fan-in budgets
**Epic:** 028a-Commissioning-configuration-and-offline-validation · **Effort:** S · **Depends on:** 028a-03, 028a-04, 028a-08

## Context
The commissioning configuration sets strict mode, per-input alignment opt-out and pacing override (C-165); 028a-03 parses the fields. `validate` checks them offline, together with the budget that a fan-in outside any presentation group needs.
- **Strict mode** requires real-time scheduling on the data and clock threads and locked memory (C-127); it refuses to start on domain 0 (C-11). It is hosted-only: a device stamps real-time scheduling and memory lock as present by construction (C-147).
- **Pacing override.** The intra-slice egress spread is at most 25 % of the slice period (C-14), and phase offset plus spread stay inside the slice period: the phase offset is in [0, 75 %) of the period (C-155). The slice period is 0.5 ms at defaults (C-02).
- **Alignment opt-out** is per input, that is per `[[connection]]` (`align = false`).
- **Fan-in outside any presentation group (C-142).** It inherits the A of the downstream presentation group it feeds (the largest if several), learned at admission. With no downstream group it needs an explicit budget in the configuration (`budget_ms`), or its aligned inputs are refused with reason NoSkewBudget (C-185). A fan-in here is a module that is not a sink and has two or more incoming connections that are not opted out of alignment.

## Work
- Check `policy.strict_on_device` (flag): `strict` set on an embedded node has no effect (C-147).
- Check `policy.pacing_fit`: refuse a pacing override whose spread exceeds 25 % of the slice period (C-14), whose phase offset is outside [0, 75 %) of the period, or whose phase offset plus spread exceeds the period (C-155).
- Check `policy.opt_out_unknown`: refuse `align = false` on a connection whose consumer input does not exist in the configuration.
- Check `fanin.no_budget`: walk the connection graph downstream from each fan-in; if it reaches no sink (every sink belongs to a group, at least `default`) and the module has no `budget_ms`, refuse it, stating that its aligned inputs would be refused with reason NoSkewBudget (C-142, C-185).
- Notice `fanin.inherited_a`: for a fan-in that reaches one or more groups, report the A it will inherit (the largest of those groups' `a_ms`, C-142); flag a `budget_ms` set on such a module as unused.
- Tests for each refusal, flag and notice.

## Acceptance criteria
1. A non-sink module with two aligned inputs whose outputs reach no sink and that has no `budget_ms` is refused; the same module with `budget_ms` passes.
2. A fan-in feeding sinks of groups with A of 6 ms and 20 ms reports an inherited A of 20 ms.
3. A fan-in whose inputs are all but one opted out (`align = false`) is not treated as a fan-in.
4. A pacing override with a phase offset of 75 % of the period, or with offset plus spread above the period, is refused; one with offset 50 % and spread 25 % passes.
5. `strict = true` on an embedded node is flagged, not refused.
6. All tests run with no network.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] `validate` runs with no network and has tests for every refusal

## Out of scope
- Parsing the fields (028a-03). Runtime enforcement of strict mode, opt-out, pacing and the C-142 refusal (runtime epics). Checking stamped strict mode against the configuration (028b-07). Declared buffer memory against A (028a-10), which reuses this ticket's inherited-A walk.
