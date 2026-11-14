# 002-09 — Experiment metrics integrity
**Epic:** 002-POC-archaeology · **Effort:** S · **Depends on:** 002-01
## Context
Part of epic 002 (POC archaeology, check 0b), deliverable **(c) Experiment metrics integrity**.

The design inherits the rule **"a machine under I/O pressure cannot hold the clock"**. It rests on the experiment metrics under `experiments/` of the POC (`run1`, `run2`, `run3`, produced by `tools/metric_experiment.py`). Those multi-machine runs are expected to have written one file per process and to be intact, but that has not been checked. Epic 011 needs to know whether the rule is inherited or dropped.

Why it matters `[code]`: in `hadropoc loopback`, one `--metrics` path is cloned into all four modules (`main.rs:317`), and each module's housekeeping thread calls `File::create` on it (`module.rs:1354`) with its own channel and writer thread, so four handles truncate and write from their own offset 0. A loopback metrics file had only about 6 parseable lines out of about 80 `[measured]`. The metrics header is defined at `module.rs:1292`.

Settled for this ticket (epic 002, Background):
- **Single-writer rule.** A file is single-writer when every line after the header parses, `unix_ms` never decreases, the header appears once, and no line is interleaved. The report restates the rule.
- **Definition of done.** The checker is the only code epic 002 ships, so it is the subject of "Tests green in CI" and "Every parser touched is fuzzed" (C-125). Fuzz budget (C-95): the line parser is fuzzed for 60 s on every change and 1 hour nightly, and the corpus is kept.
- **Location.** The checker is a tool binary in the `hadronyche-tools` crate of the design set's repository (C-133), never in the frozen tree (C-130); that repository's CI runs its tests and fuzz jobs. Its captured output goes to `doc/design/reports/archaeology/` (C-130). "Unprivileged" applies to per-change CI (C-146); the checker needs no privileged job.
- **Presentation (C-206).** The checker's output line format and file names within `doc/design/reports/archaeology/` are decided by this ticket and recorded in the checker's documentation; the report table columns of Work step 3 are the only fixed part.

Open sub-question owned by this epic: **"Does the I/O-pressure finding survive?"** Falsifier: experiment files are corrupt or multi-writer; the eligibility rule loses its basis.
## Work
1. Write a small read-only checker binary in `hadronyche-tools` (C-133; not in the frozen tree), that for each metrics file under the POC's `experiments/`:
   - counts total lines;
   - counts lines that parse against the header (column count and column types as defined at `module.rs:1292`);
   - checks that `unix_ms` is monotonic non-decreasing, and counts violations;
   - gives a single-writer verdict by the rule above, which is restated in the report.
2. Add unit tests for the checker on 3 synthetic files (clean, truncated/interleaved by two writers, non-monotonic) and a fuzz target feeding random bytes to the line parser (it must never crash). Run it for 60 s on every change and 1 hour nightly in CI (C-95); keep the corpus with the checker in `hadronyche-tools`. Document the checker's usage, output line format and output file names in its crate documentation (C-206).
3. Run it and put the table in section (c) of `doc/design/reports/archaeology/REPORT-0b.md`: file, total lines, parseable lines, `unix_ms` violations, single-writer verdict.
4. Re-read the I/O-pressure finding against the files judged intact and state whether it survives; answer the sub-question; write the (c) verdict line, tagged `[measured]`.
## Acceptance criteria
1. The table lists **every** metrics file under `experiments/` (count stated and equal to the number of files found) with total lines, parseable lines and a single-writer verdict (epic criterion 002-AC-4).
2. The checker's 3 synthetic-file tests pass in CI, unprivileged; the fuzz target runs 60 s per change without a crash, a 1-hour nightly fuzz job is configured, and the corpus is stored with the checker (C-95).
3. The report states "I/O-pressure finding: survives" or "does not survive", with the reason.
4. Section (c) ends with exactly one verdict line with a named mechanism and the captured checker output (epic criterion 002-AC-1 for (c)).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
## Out of scope
- Fixing the loopback metrics clobbering (task 003).
- Deciding the clock-eligibility rule of the runtime (epic 011); this ticket only states whether its basis holds.
- Updating the evidence appendix (002-13).
