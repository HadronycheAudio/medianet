# 003-06 — Fail headless run on zero-sample family
**Epic:** 003-POC-smoke-fixes · **Effort:** S · **Depends on:** 003-01, epic 002

## Context
- Epic 003 applies three minimal fixes to the frozen proof of concept **hadropoc** (`/home/dorian/workspace/hadropoc`, Rust). This is fix 3. Fixes must be minimal and must not change wire behaviour, the servo, the analyzer, or any measured quantity other than the achieved sync rate.
- Bug 3 `[measured]`: in a 25-second loopback run the music and transport check families reported "0 of 0 windows" for the whole run, and the run looked identical to a good one.
- Design rule `[decision]`: **a check family with zero samples is a failed run.** The cause of the zero samples is diagnosed in epic 002; this ticket only makes the condition fatal in headless mode. System criterion AC-9: a run with an inert check family exits non-zero and names the family.
- Exit status `[decision]` (epic background): any non-zero status plus the family's name; no specific exit code is required.
- In-scope families (epic deliverable 4): audio, music and transport, unless epic 002 finds that music or transport cannot produce samples in loopback without code changes; such a family is then out of scope for this check and listed as known-inert in `BASELINE.md` (003-08). Epic 002 owns this question; falsifier: no flag combination makes the family produce samples.
- The wording of the zero-sample line is a presentation detail decided by this ticket (C-206) and recorded in `BASELINE.md` (003-08); `tools/smoke.sh` asserts only the exit status, so no other epic or script depends on the wording.
- Headless mode is used when there is no terminal (`main.rs:255-262`, `main.rs:535-537`) `[code]`. `--duration` exists.

## Work
1. In the headless end-of-run path (around `main.rs:535-537`), when the run was started with `--duration`, after the final summary, inspect the number of windows analysed per in-scope check family.
2. For every in-scope family with zero windows, print its name (one line per family, e.g. `check family <name>: 0 windows analysed`) to stderr, and pass the chosen wording to 003-08 for `BASELINE.md` (C-206).
3. If any in-scope family had zero windows, exit with a non-zero status; otherwise exit as today.
4. Implement the in-scope family set as one obvious list, set from epic 002's answer, so that its scope is a one-line change.
5. Do not change the analyzer or how windows are counted; only read the existing counts.

## Acceptance criteria
1. A headless `--duration` run in which an in-scope family has 0 windows exits with a non-zero status and prints that family's name (epic 003-AC4, system AC-9). Verified with all three families in scope and the untouched smoke flags, where music and transport are inert `[measured]`.
2. A headless `--duration` run in which every in-scope family has more than 0 windows exits with status 0.
3. Interactive (TUI) mode behaviour is unchanged.
4. The diff touches only the headless exit path.
5. The two existing tests in `transport.rs` still pass.

## Definition of done
- [ ] Tests green in CI, unprivileged (no CI exists: `cargo test`, unprivileged)
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No change to the wire format, the servo algorithm or the analyzer
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Diagnosing or fixing why families are inert (epic 002).
- The smoke script (003-07) and `BASELINE.md` (003-08).
