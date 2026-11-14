# 002-06 — Inert families: music trace, minimal condition and verdict
**Epic:** 002-POC-archaeology · **Effort:** S · **Depends on:** 002-02, 002-05
## Context
Part of epic 002 (POC archaeology, check 0b), deliverable **(a) Inert check families**, steps 3 and 4, and the (a) verdict. Section (a) of `doc/design/reports/archaeology/REPORT-0b.md` already holds the baseline run and the transport trace (002-05).

Anomaly `[measured]`: in a 25 s loopback run the music family reported "0 of 0 windows" while audio was 2345/2345 exact.

Points to trace for music (from the epic):
- settling exclusions at `audio.rs:604` and `audio.rs:644` — a synth silent for more than 1 s gets a 3 s settling exclusion (`SETTLE_SECS`) `[code]`;
- the `voices` flag in the analyzer;
- whether voice frames reach `place_voices`.

Comparison is per 1 024-sample window; a window still incomplete 256 windows later (`MAX_LAG_WINDOWS`, about 2.7 s at 96 kHz) is counted lost or as a sender gap and never analysed `[code]`. The reading notes of 002-02 give the full exclusion list.

System criterion **AC-9**: a check family with zero samples fails the run. Task 003 implements it and needs this answer.
## Work
1. Trace the three music points with `file:line` pointers; where needed, add logging in the scratch copy only and record log lines.
2. Determine the minimal condition (command line, flags, or graph wiring) under which **both** the music and transport families produce non-zero windows. Try it: run it for 25 s and paste the summary.
3. If no such command line exists with the frozen code, state that and why.
4. Write the (a) verdict line: **instrument bug, mechanism: …** or **not a bug, explanation: …**, tagged `[code]` or `[measured]`, covering both families.
5. Add a one-paragraph hand-off note "for task 003" summarising what made the families inert.
## Acceptance criteria
1. Each of the 3 music points has a definite answer with a pointer or log line.
2. The report gives a command line whose captured 25 s summary shows music windows > 0 **and** transport windows > 0, or states that none exists and why (epic criterion 002-AC-2).
3. Section (a) ends with exactly one verdict line with a named mechanism and at least one `file:line` pointer or captured run output (epic criterion 002-AC-1 for (a)).
4. The hand-off note for task 003 is present.
## Definition of done
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Diagnostic logging used for (b) and (e) lives in a scratch copy of the POC, not in the frozen tree
- [ ] No new open item without an owner and a falsifier
## Out of scope
- Fixing the inert families or making them fail the run (task 003).
- Updating the evidence appendix (002-13).
