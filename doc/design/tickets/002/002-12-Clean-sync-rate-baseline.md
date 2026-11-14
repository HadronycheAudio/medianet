# 002-12 — Clean sync rate baseline
**Epic:** 002-POC-archaeology · **Effort:** S · **Depends on:** 002-01
## Context
Part of epic 002 (POC archaeology, check 0b), deliverable **(e) Run confirmations**, item 3. Re-measures the anomaly figures at 50 and 1000 Hz. System criterion **AC-7** (achieved sync rate within 2 % of configured) is checked over the accepted range of 10 to 200 requests per second (C-112); 1000 Hz is outside that range and is measured only because it is anomaly figure P-02 and exposes the timeout mechanism. The ends of the AC-7 range, 10 and 200 Hz, are measured in 002-14 with the method set up here.

Anomaly `[measured]`: `--sync-hz 50` achieved **37** completed exchanges per second; `--sync-hz 1000` achieved **77** (P-02 in `02-constants.md`). Mechanism `[code]`: the clock thread paces requests with a **5 ms** socket receive timeout (`module.rs:1126`); one effective timeout of about **13 ms** explains both figures on a `CONFIG_HZ=250` kernel (P-03, `[measured]`). Both figures were read from a **corrupt** metrics file and must be re-read from a clean one.

Why the file was corrupt `[code]`: in `hadropoc loopback` one `--metrics` path is cloned into all four modules (`main.rs:317`) and each calls `File::create` on it (`module.rs:1354`), so four writers truncate and write from offset 0. Task 003 fixes this in the POC; this ticket must not wait for it and must not touch the frozen tree.

Open sub-question owned by this epic (informational only): **"Why does a 5 ms timeout behave as about 13 ms?"** No falsifier needed; the runtime paces by deadline.

Strict mode (C-147, settled): POC runs do not run in strict mode; their figures go only into the POC reference rows, here P-02, never into design constants. Runs are unprivileged like the anomaly run.
## Work
1. Obtain a single-writer metrics file per module by one of the two methods the epic allows: each module as a separate process with its own `--metrics` path, or loopback in the scratch copy only with a distinct metrics path per module. Record which method was used, as a reusable command or script for 002-14.
2. Check each file is clean: every line after the header parses and `unix_ms` is monotonic (reuse the 002-09 checker if available, otherwise the same rule by hand).
3. Run 3 runs of ≥ 30 s each at `--sync-hz 50` and at `--sync-hz 1000`, unprivileged, and compute completed exchanges per second from the clock holder's peer metrics (name the metrics column used, with its `module.rs` pointer); report mean and range.
4. Record the host's `CONFIG_HZ` and, informationally, explain the 5 ms → ≈ 13 ms effect (e.g. measured receive-timeout duration in a scratch-copy log) to answer the sub-question.
5. Write the result and verdict line in section (e) of `doc/design/reports/archaeology/REPORT-0b.md`, tagged `[measured]`, with the percentage of configured rate achieved.
## Acceptance criteria
1. Every metrics file used is single-writer: 100 % of lines after the header parse, 0 `unix_ms` regressions.
2. Achieved exchanges per second at 50 Hz and at 1000 Hz are reported (mean and range of 3 runs) with the percentage of configured rate (epic criterion 002-AC-6, part of (e) 3).
3. The effective receive-timeout duration is reported in ms with the host `CONFIG_HZ`.
4. The verdict line names the pacing mechanism with `module.rs:1126` (epic criterion 002-AC-1).
5. The clean 50 and 1000 Hz figures are given in a form ready to replace P-02 in the constants table (002-13 writes them).
## Definition of done
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Diagnostic logging used for (b) and (e) lives in a scratch copy of the POC, not in the frozen tree
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
## Out of scope
- Fixing the metrics clobbering in the POC (task 003).
- Measuring 10 and 200 Hz (002-14).
- Editing the constants table (002-13).
- Meeting AC-7 in the runtime (later epics); this ticket provides only the POC baseline.
