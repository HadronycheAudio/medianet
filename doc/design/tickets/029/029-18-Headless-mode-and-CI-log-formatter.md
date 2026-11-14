# 029-18 — Headless mode and CI log formatter
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-08, 029-09, 029-10

## Context
- `hadronyche-monitor --headless` prints **one summary line per interval**, for logs (epic 029 deliverable).
- Definition of done of epic 029: the headless mode is used by at least one CI job as a log formatter.
- The same honesty rules apply as in the UI `[decision]`: one-way figures blank until lock and with bound after; a check family with zero samples is **failed**; an injected run is labelled **injected**; a run inside a commissioning window is labelled (C-107); no average across a regime change; per-stream figures are not merged.
- The monitor writes no measurement files; headless output goes to stdout only. `[decision]`
- Line content and default interval: decided by this epic and recorded in the tool's documentation (C-206); this ticket decides them. Only what another epic or an operator's script depends on goes into the specifications.

## Work
1. `--headless [--interval <seconds>]` (default 1 s, the hello period C-30; chosen by this epic under C-206) on either source (live or `--file`). With `--file` and no real-time pacing, intervals are measured in recorded time.
2. Line format (stable, documented, one line per interval; decided by this epic under C-206): timestamp, modules present / total, holder, clock summary (worst state, worst bound), connections satisfied (Live only) and unsatisfied, with counts by state name (Pending, Locking, Aligning, Interrupted, C-154), Demoted count, worst connection's late / missing / marked / interrupted / dead, failed families, `INJECTED` flag, `COMMISSIONING` flag, regime-change count in the interval.
3. Blank values print as `-` (unlocked), `FAILED` (zero-sample family), `unknown` (declared unknown, C-103, C-140) and `n/p` (not published), never `0`.
4. Add a CI job that runs an existing scenario recording through `hadronyche-monitor --headless --file <recording>` and attaches the output to the job log.

## Acceptance criteria
1. A 10 s recording at interval 1 s produces exactly 10 lines.
2. Before lock the one-way field prints `-`; after lock it prints value and `±bound`.
3. A zero-sample family prints `FAILED`; an injected run has `INJECTED` on every line; a commissioning window has `COMMISSIONING` on every line it covers.
4. Output is identical for two runs over the same file.
5. At least one CI job uses headless mode as its log formatter and is green.
6. The line format and default interval are recorded in the tool's documentation (C-206, 029-23); if another epic or an operator's script depends on any part of the line, that part is also entered in the specifications.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The headless mode is used by at least one CI job as a log formatter
- [ ] Every new factual claim carries an evidence tag
- [ ] The line format is documented in the tool's documentation

## Out of scope
- Interactive panels (029-11 to 029-17, 029-24).
