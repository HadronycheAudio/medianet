# 013-18 — Zero-sample check family failure
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-15

## Context
- Rule 4: **a check family with zero samples is a failed run.** `[decision]`
- `[measured]` In the POC's first measured run, two check families reported "0 of 0 windows" on a run that looked green.
- Deliverable in `hadronyche-std`: an **end-of-run check that fails the run and names any check family with zero samples**.
- AC-9 (013-AC-3): a run in which a check family analysed zero samples **exits non-zero and names the family**.
- Records carry a `CheckFamily` in their `RecordHeader` (`CheckFamily` owned by epics 015 and 017).
- The end-of-run summary is one of the outputs that carry labels (epic Rule 13, ticket 013-19).

## Work
1. At end of run, count samples analysed per expected check family (the families the run configured, from the record-set writer or the checks registry).
2. If any family has 0, write a failure entry naming each such family into the output, print it, and make the process exit with a non-zero status.

## Acceptance criteria
1. A run where 1 of N families has 0 samples exits non-zero and its output names that family.
2. A run where 2 families have 0 samples names both.
3. A run where every family has ≥ 1 sample exits 0 (for this check).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- Not applicable (C-125): "Every parser touched is fuzzed" (no parser); "Regime stamp fields touched are complete and documented", "Every stamp field has a stated source on each of the four profiles" and "The stamp format is versioned independently of the media format" (no stamp change); "Design documents updated where behaviour differs, constants table first" (no behaviour differs); "No new open item without an owner and a falsifier" (none opened).

## Out of scope
- Strict-mode start-up refusal (013-20); the checks themselves (epics 015, 017).
