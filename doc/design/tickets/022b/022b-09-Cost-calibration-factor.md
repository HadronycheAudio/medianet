# 022b-09 — Cost calibration factor for the Windows target
**Epic:** 022b-Windows-edge-L2-certification · **Effort:** XS · **Depends on:** 022a-17, 022b-01, epic 025

## Context
- Part of epic 022b (Windows edge L2 certification).
- C-106: the compiler's cost bound is counted in abstract operations per slice, identical on every machine. Each target has a calibration factor (time per operation), measured once and recorded in the regime stamp; admission converts with it. The Windows edge runs the kit's resampler node (C-80), so admission on it needs the factor.
- Procedure (C-176): the DSL conformance suite (epic 025) runs a reference workload once per listed target and records time per operation; the value is carried in the regime stamp. This ticket does not run the workload: it takes the factor epic 025 records for each listed Windows target and carries it in the stamp.
- The factors are also listed in the certification record in `doc/design/reports/` (C-130).

## Work
1. For each listed Windows target on the rig (022b-01), take the factor recorded by the epic-025 conformance suite (C-176).
2. Write the factor per target into the crate data file the stamp builder reads (022a-17) and into the results report.

## Acceptance criteria
1. Each listed Windows target has the factor recorded by epic 025 (C-176) in the data file, and its stamp carries that factor.
2. A machine without a recorded factor stamps the factor *absent* (C-129).

## Definition of done
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The reference workload and its runs (epic 025, C-176) and the admission conversion (core).
