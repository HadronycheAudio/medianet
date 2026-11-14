# 003-09 — Verify and tag poc-baseline
**Epic:** 003-POC-smoke-fixes · **Effort:** XS · **Depends on:** 003-03, 003-04, 003-05, 003-06, 003-07, 003-08

## Context
- Epic 003 ends with a second commit of the proof of concept **hadropoc** (`/home/dorian/workspace/hadropoc`) and the annotated tag `poc-baseline`, the frozen baseline cited by epics 007 and 017. After this tag the POC is not modified again.
- The first commit, tagged `poc-untouched` (003-01), already contains `.gitignore`.
- Allowed changes between the tags (epic criterion 003-AC1): the metrics writer (003-03), the clock thread wait (003-04), the headless exit path (003-06), `tools/smoke.sh` (003-07) and `BASELINE.md` (003-08). Nothing else, in particular no CI configuration.
- Definition-of-done applicability (rule C-125, recorded in the epic): the POC has no CI and may not gain one, so "Tests green in CI, unprivileged" is met by the unprivileged `cargo test` and the three smoke runs of this ticket, recorded in the second commit message.
- The measuring machine is the machine identified in 003-02 (`CONFIG_HZ=250`, P-03).

## Work
1. On the measuring machine, unprivileged, run `cargo test`: the two existing tests in `transport.rs` pass.
2. Run `tools/smoke.sh` three times in a row; all three exit 0.
3. Commit all fixes, `tools/smoke.sh` and `BASELINE.md` as the second commit (if fixes were merged as separate commits, the final one is the baseline commit). Record the test and smoke results in the commit message.
4. Create annotated tag `poc-baseline`.
5. Check `git diff poc-untouched poc-baseline --stat`.

## Acceptance criteria
1. `git tag` lists `poc-untouched` and `poc-baseline`; both are annotated (`git cat-file -t` prints `tag`) (epic 003-AC1).
2. `git diff poc-untouched poc-baseline --stat` touches only the metrics writer, the clock thread wait, the headless exit path, `tools/smoke.sh` and `BASELINE.md` (epic 003-AC1).
3. `tools/smoke.sh` passed 3 consecutive times, unprivileged, on the measuring machine (epic 003-AC5).
4. The two tests in `transport.rs` pass (epic 003-AC6).

## Definition of done
- [ ] Tests green in CI, unprivileged (no CI exists: `cargo test` and three smoke runs, unprivileged)
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Both tags exist and `BASELINE.md` records before/after figures
- [ ] No change to the wire format, the servo algorithm or the analyzer

## Out of scope
- Any further change to the POC after `poc-baseline`.
- Design-set updates (003-10).
