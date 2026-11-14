# 027-08 — CI analysis on recorded files
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-02, 027-03, 027-04, 027-07

## Context
- Epic 027 definition of done requires: "The skew analysis runs in CI on recorded two-channel files" and "Tests green in CI, unprivileged". The hardware rig cannot run in CI; the analysis can.
- Tool: `hadronyche-l3` (crate `hadronyche-tools`, in the design set's repository, C-133). Skew = channel 2 (embedded device) minus channel 1 (Linux sink). Pass rule (C-79): the 99.9th percentile of |skew| is at most E_linux + E_device; E_linux target 50 µs (C-70).
- Fuzz budget (C-95): each parser 60 s on every change and 1 hour nightly; the corpus is kept. The parsers are the capture reader (027-02), the stamp file reader (027-07) and the stamp message decoder (027-22).
- "Unprivileged" applies to per-change CI; a test that needs a privilege runs in the separate privileged nightly job (C-146). This ticket's jobs need none.
- Until real rig recordings exist (027-11 onward), the fixture set is synthetic (027-04). Each scenario ticket then adds a short excerpt of its real recording to this fixture set.

## Work
1. Subcommand `hadronyche-l3 analyze --capture <file> --markers <plan> --linux-stamp <json> --device-stamp <json> [--interchannel-ns <n>] --out <report>` running read → measure → summarize → report.
2. Fixture directory (for example `hadronyche-tools/tests/l3-fixtures/`) with: the 37 µs self-check file, a drift file, a step file, and stamp JSON files; each with an `expected.json` (pass/fail, p50 within tolerance).
3. A CI job (unprivileged, no audio hardware, no network privileges) that runs `selfcheck` and `analyze` on every fixture and compares with `expected.json`, and runs every fuzz target for 60 s.
4. A nightly CI job that runs every fuzz target for 1 hour and keeps the corpus (C-95).
5. Size budget for fixtures: keep each under a few MB (short excerpts); document the budget.
6. Document in the crate README how to add a recorded excerpt as a new fixture.

## Acceptance criteria
1. The CI job runs without root and without any audio or network device, and is green.
2. Changing an `expected.json` verdict makes the job fail (negative check done once in the PR).
3. The job runs the 37 µs self-check (027-AC-8) on every CI run.
4. Total per-change CI job time under 5 minutes, fuzzing included.
5. The nightly job runs each fuzz target for 1 hour and its corpus persists between runs.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The skew analysis runs in CI on recorded two-channel files
- [ ] Every parser touched is fuzzed (scheduling of the C-95 budget)
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Regime stamp fields touched" — none; "Design documents updated" — no design behaviour; "No new open item" — opens none; "Every new factual claim carries an evidence tag" — no factual claim; "The rig is photographed or diagrammed" — no rig work; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Real rig recordings (added by 027-10 to 027-19).
- Writing the fuzz targets (027-02, 027-07, 027-22).
