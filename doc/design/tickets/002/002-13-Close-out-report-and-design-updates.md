# 002-13 — Close-out: report audit and design updates
**Epic:** 002-POC-archaeology · **Effort:** S · **Depends on:** 002-02, 002-03, 002-04, 002-06, 002-08, 002-09, 002-10, 002-11, 002-12, 002-14
## Context
Final ticket of epic 002 (POC archaeology, check 0b). Once it is merged, the POC may be cited as the exact-tier baseline and conformance vectors may be extracted from it (unblocks epics 007 and 017), epic 011 knows whether the I/O-pressure eligibility rule is inherited, and task 003 has confirmation of the two bugs it fixes plus the answer to (a).

Epic acceptance criteria to check against `doc/design/reports/archaeology/REPORT-0b.md`:
1. **002-AC-1.** Each of (a) to (e) has a verdict line with a named mechanism and at least one `file:line` pointer or a captured run output.
2. **002-AC-2.** (a): a command line under which both families report non-zero windows, or a statement that none exists and why.
3. **002-AC-3.** (b): the source field of the printed value is named, with the log showing the first wrong hop.
4. **002-AC-4.** (c): a table of every metrics file with total lines, parseable lines and a single-writer verdict.
5. **002-AC-5.** (d): "is the voice comparison bit-level" answered yes or no.
6. **002-AC-6.** (e): the flood result, the step distribution and the achieved sync rate at 10, 50, 200 and 1000 Hz, with numbers.
7. **002-AC-7.** Contributes to system criterion **AC-9** (a check family with zero samples fails the run) by explaining the inert families, and to system criterion **AC-7** (achieved sync rate within 2 % of configured, checked over 10 to 200 per second, C-112) by the clean POC baseline.

Verdict line format: **instrument bug, mechanism: …** or **not a bug, explanation: …**, tagged `[code]` or `[measured]`. The (d) verdict is **not a bug, explanation: …** stating what `analyse` compares (from 002-02), unless a reading in 002-02 to 002-04 found an instrument bug, in which case it is **instrument bug, mechanism: …**.

Settled rules this ticket applies (epic 002, Background):
- **Constants table** means `02-constants.md`. POC reference values changed by a verdict are updated there first: P-02 with the clean sync rates from 002-12, P-03 if the test host's `CONFIG_HZ` differs and the effect is re-explained, and the loss-handling facts if 002-02 corrected them. Other design documents repeating a changed statement (for example "exact tier for voices") are found by searching the design set and updated after.
- **Definition of done (C-125).** "Regime stamp fields touched are complete and documented" is not applicable: epic 002 writes no runtime code and touches no stamp field; the report says so with that reason. "Tests green in CI" and "Every parser touched is fuzzed" apply only to the 002-09 checker (fuzz budget C-95); this ticket checks them there.
- **Evidence appendix** is `doc/design/06-evidence-appendix.md` (C-130); entries follow the format that file already uses.
- **Architecture specification** is `doc/design/01-architecture.md` (C-171); it is among the design documents searched for changed statements.
- **Strict mode (C-147, settled).** POC runs do not run in strict mode and their figures go only into POC reference rows; P-02 is written from the unprivileged 002-12 runs as a POC figure.
## Work
1. Audit the report against the 7 criteria above; add a checklist at the top of the report with one tick per criterion and the section that satisfies it.
2. Check every factual sentence of the report carries an evidence tag; fix any that does not.
3. Write the (d) verdict line in the form above, from 002-02's statement of what `analyse` compares and any bug flagged by 002-02 to 002-04.
4. Close the open sub-questions table: the four investigation questions marked answered, with the answer and section.
5. Add a "Hand-offs" section: to 003 (bugs confirmed, (a) answer), to 007 and 017 (clearance to cite the POC as exact-tier baseline, statement of what `analyse` compares), to 011 (I/O-pressure rule inherited or dropped).
6. Update the evidence appendix `doc/design/06-evidence-appendix.md` (C-130) with one entry per verdict ((a) to (e)), tagged, in the entry format the file already uses.
7. Update `02-constants.md` first (P-02, and P-03 or the loss-handling facts where a verdict changed them), then every other design document that repeats a changed statement, including `doc/design/01-architecture.md` (C-171); list the documents changed in the report.
8. State in the report that no regime stamp field was touched, with the reason (C-125), and check that the 002-09 checker's CI tests and fuzz jobs are green.
9. Re-check the frozen-tree fingerprint recorded in 002-01; it must be unchanged.
## Acceptance criteria
1. The checklist shows all 7 epic criteria (002-AC-1 to 002-AC-7) satisfied, each with a section reference.
2. Five verdict lines exist, none "pending".
3. The four investigation sub-questions of the report are marked answered.
4. The evidence appendix has 5 new entries, one per verdict, each with an evidence tag.
5. P-02 in `02-constants.md` holds the clean 50 and 1000 Hz figures from 002-12, tagged `[measured]`, as POC figures from runs without strict mode (C-147); any other changed document is listed in the report.
6. The report carries the "not applicable" line for regime stamp fields with its reason.
7. Frozen-tree fingerprint identical to the 002-01 value.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Diagnostic logging used for (b) and (e) lives in a scratch copy of the POC, not in the frozen tree
- [ ] The evidence appendix of the design set is updated with each verdict

Not applicable (C-125): "Regime stamp fields touched are complete and documented" — no stamp field exists in this epic. "Tests green in CI" and "Every parser touched is fuzzed" are met through the 002-09 checker, the only code of the epic.
## Out of scope
- New investigation; any missing measurement goes back to its ticket (002-05 to 002-12, 002-14).
- Fixing any bug found (task 003).
