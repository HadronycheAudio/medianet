# 003-10 — Update design set with baseline
**Epic:** 003-POC-smoke-fixes · **Effort:** XS · **Depends on:** 003-05, 003-09

## Context
- Epic 003 produced the tagged baseline `poc-baseline` of the proof of concept **hadropoc** after three fixes (one process-owned writer per metrics file; deadline-paced clock sync with a 100 ms ceiling; non-zero headless exit naming any in-scope check family with zero windows).
- The design set (Hadronyche Medianet runtime rewrite) must reflect this (epic deliverable 8):
  - the constants table `doc/design/02-constants.md` holds the POC reference values; the pre-fix sync rates are row P-02 (37 / 77 exchanges/s at `--sync-hz 50` / `1000`) and the measuring machine's kernel tick is P-03. The post-fix rates go into the same section first ("constants table first");
  - the evidence appendix is `doc/design/06-evidence-appendix.md` (C-130) and must record the post-fix achieved sync rates as `[measured]`;
  - the design set previously stated the rule "no code goes into hadropoc"; C-130 now states it with task 003 as its single exception ("nothing is written into the frozen POC tree except by task 003"). Any other design document that still states the unqualified rule (for example the architecture specification `doc/design/01-architecture.md`, C-171) is aligned to C-130.
- C-147 settles that POC runs do not run in strict mode and that their figures go only in the POC reference rows (P-nn); the post-fix figures are therefore entered beside P-02.
- Design rules introduced `[decision]`: "a file has one writer, owned by the process, never by a module" (system criterion AC-31); "a check family with zero samples is a failed run" (system criterion AC-9); "smoke assertions never depend on sync-rate timing" (system criterion AC-8).
- Definition-of-done applicability (rule C-125, recorded in the epic): regime stamps (C-18) are a runtime control message and no fix touches a stamp field, so "Regime stamp fields touched are complete and documented" is not applicable.

## Work
1. In `doc/design/02-constants.md`, section "POC reference values", add a row next to P-02 with the post-fix achieved sync rates at `--sync-hz 50` and `--sync-hz 1000` from 003-05, tagged `[measured]`, citing the `poc-baseline` tag and the measuring machine. Do not change P-02.
2. Add the same figures to the evidence appendix `doc/design/06-evidence-appendix.md`, tagged `[measured]`, with machine identity, kernel `CONFIG_HZ` and the `poc-baseline` tag.
3. Check that C-130 states the exception (task 003 only); align any other statement of the unqualified rule "no code goes into hadropoc" in the design set to C-130, adding that the POC is frozen after `poc-baseline`.
4. Update any other design document wherever this epic's behaviour differs from what is written (e.g. clock-sync pacing, metrics file ownership).
5. Record the answer to the lock sub-question from 003-05 (does deadline pacing still reach lock within 25 s). If epic 002 left families known-inert, record that the baseline covers the audio family only.

## Acceptance criteria
1. The constants table has a POC reference row with the 2 post-fix sync-rate figures, tagged `[measured]`, referencing `poc-baseline` (C-147: POC figures go only in the POC reference rows); P-02 is unchanged.
2. `doc/design/06-evidence-appendix.md` contains the same 2 figures, tagged `[measured]`, referencing `poc-baseline`.
3. No design document states the rule "no code goes into hadropoc" without the task 003 exception of C-130.
4. Every edited claim carries an evidence tag.
5. No open item is added without an owner and a falsifier.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] The evidence appendix of the design set records the post-fix sync rates as `[measured]`
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] Regime stamp fields touched are complete and documented (not applicable, C-125: no stamp field touched)

## Out of scope
- Any POC change (frozen after 003-09).
- Conformance vectors (epic 007) and reference modules / L0 harness (epic 017).
