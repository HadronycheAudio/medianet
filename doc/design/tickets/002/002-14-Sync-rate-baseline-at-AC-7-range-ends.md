# 002-14 — Sync rate baseline at the ends of the AC-7 range
**Epic:** 002-POC-archaeology · **Effort:** XS · **Depends on:** 002-12
## Context
Part of epic 002 (POC archaeology, check 0b), deliverable **(e) Run confirmations**, item 3. Epic 002 provides the clean POC baseline for system criterion **AC-7** (achieved sync rate within 2 % of configured). The accepted sync request rate is 10 to 200 per second and AC-7 is checked over that range (C-112, constants table `02-constants.md`); the design default is 50 per second (C-36). Ticket 002-12 measures 50 Hz and the anomaly figure 1000 Hz (P-02); this ticket measures the two ends of the range, 10 and 200 Hz.

Mechanism already known `[code]`: the POC clock thread paces requests with a 5 ms socket receive timeout (`module.rs:1126`), which behaves as about 13 ms on a `CONFIG_HZ=250` kernel (P-03, `[measured]`). At 200 Hz the request period is 5 ms, so the same effect may cap the achieved rate below 200; at 10 Hz the period is 100 ms, where the timeout may not matter. Neither is known; this ticket measures both.

002-12 sets up and records the method that gives one single-writer metrics file per module (separate processes with their own `--metrics` path, or loopback in the scratch copy with one path per module) and names the metrics column that counts completed exchanges. The frozen POC is not modified; any logging lives in the scratch copy.

Strict mode (C-147, settled): POC runs do not run in strict mode, and their figures go only into POC reference rows. These runs are unprivileged like the 002-12 runs, so the (e) 3 table stays comparable.
## Work
1. Reuse the method and column recorded by 002-12, on the same host.
2. Run 3 runs of at least 30 s each at `--sync-hz 10` and at `--sync-hz 200`, unprivileged (C-147, see Context).
3. Check every metrics file used with the 002-09 checker (or the same rule by hand): every line after the header parses, `unix_ms` never decreases.
4. Compute completed exchanges per second from the clock holder's peer metrics; report mean and range of the 3 runs and the percentage of the configured rate.
5. Add the two results to the (e) 3 part of section (e) of `doc/design/reports/archaeology/REPORT-0b.md`, tagged `[measured]`, in one table with the 50 and 1000 Hz results of 002-12, and state for each of 10, 50 and 200 Hz whether the POC is within 2 % of the configured rate.
## Acceptance criteria
1. Every metrics file used is single-writer: 100 % of lines after the header parse, 0 `unix_ms` regressions.
2. Achieved exchanges per second at 10 and 200 Hz are reported (mean and range of 3 runs) with the percentage of configured rate (epic criterion 002-AC-6, part of (e) 3).
3. The (e) 3 table covers 10, 50, 200 and 1000 Hz, and each of 10, 50 and 200 Hz is marked "within 2 %" or "not within 2 %" (contribution to system AC-7, epic criterion 002-AC-7).
4. The frozen-tree fingerprint from 002-01 is unchanged.
## Definition of done
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Diagnostic logging used for (b) and (e) lives in a scratch copy of the POC, not in the frozen tree
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
## Out of scope
- The 50 and 1000 Hz measurements, the clean-metrics method and the 5 ms → about 13 ms explanation (002-12).
- Meeting AC-7 in the runtime (later epics); this ticket provides only the POC baseline.
- Editing the constants table (002-13).
