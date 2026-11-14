# 001-15 — Probe fuzz targets in CI
**Epic:** 001-Receive-stamp-probes · **Effort:** XS · **Depends on:** 001-01, 001-02, 001-06

## Context
Epic 001 builds three parsers: the payload decoder of the Linux peer (`probes/linux-peer/`, 001-01) and the control-message parsers of the macOS and Windows probes (`probes/macos-rxstamp/`, 001-02; `probes/windows-rxstamp/`, 001-06). Each has a fuzz target. The fuzz budget is settled: each parser is fuzzed for 60 s on every change and for 1 hour nightly, and the corpus is kept (C-95). No other ticket of this epic wires that budget into CI.

The probe parsers are written without OS-specific APIs so all three can run on the Linux CI runners, unprivileged; "unprivileged" applies to per-change CI (C-146). The probes live under `probes/` in the design set's repository (C-133).

## Work
- Add a per-change CI job that runs each of the three fuzz targets for 60 s.
- Add a nightly CI job that runs each for 1 hour.
- Keep the corpus of each target: committed seed inputs (the captured byte buffers from the unit tests) plus the corpus grown by the nightly job, stored where the next run picks it up.
- Fail the job on any crash and keep the crashing input as an artefact.

## Acceptance criteria
1. A change to any probe parser triggers a 60 s run of each fuzz target, unprivileged, and the job fails on a crash.
2. A nightly run fuzzes each target for 1 hour and its grown corpus is available to the next run (C-95).
3. Seed corpora for the three targets are committed.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Probe sources committed so the measurements can be repeated

Not applicable (C-125): "Regime stamp fields touched" — none; "Design documents updated" — no behaviour change; "No new open item" — opens none; "`REPORT-0a.md` delivered before task 004" — 001-13.

## Out of scope
- Writing the parsers and fuzz targets (001-01, 001-02, 001-06).
- Fuzzing parsers outside `probes/`.
