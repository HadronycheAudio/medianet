# 003-07 — Smoke script
**Epic:** 003-POC-smoke-fixes · **Effort:** S · **Depends on:** 003-03, 003-04, 003-06, epic 002

## Context
- Epic 003 makes the loopback run of the frozen proof of concept **hadropoc** (`/home/dorian/workspace/hadropoc`) a citable baseline. After fixes 003-03 (one process-owned writer per metrics file), 003-04 (deadline-paced clock sync) and 003-06 (non-zero exit when an in-scope check family analysed zero windows in headless `--duration` runs), a smoke script checks the run.
- `hadropoc loopback` runs sender, synth1, synth2 and mixer (4 modules) in one process; headless mode is used when there is no terminal `[code]`.
- Constraint `[decision]`: smoke assertions must **never depend on sync-rate timing**; on an unprivileged machine `--sync-hz 50` gave 37/s and `--sync-hz 1000` gave 77/s pre-fix (P-02) `[measured]`. Contributes to system criterion AC-8 (the loopback smoke run is trustworthy; no assertion depends on sync-rate timing).
- Row count (epic deliverable 5): 4 × 25 = 100 data rows within ±2 in total; where a row identifies its module, also 25 ± 2 per module.
- "Locked" is the POC's own lock rule (P-04) `[code]`; the script checks the POC's indicator for it, not the runtime's targets.
- The exact text of the final summary, the locked-state indicator and the metrics header are read from the POC at `poc-untouched` and recorded as comments in the script (epic deliverable 5).
- Definition-of-done applicability (rule C-125, recorded in the epic): the field-count check is a test assertion over the POC's own output, not a parser of untrusted input, so "Every parser touched is fuzzed" and the fuzz budget C-95 do not apply. Its rejection of corrupt files is covered by the negative tests in the acceptance criteria. The POC has no CI.

## Work
Create `tools/smoke.sh` (bash, executable, `set -euo pipefail`) that:
1. Creates a temporary directory `<tmp>` (`mktemp -d`) and removes it on exit.
2. Runs, headless (stdin/stdout not a terminal):
   `hadropoc loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20 --csv <tmp>/smoke.csv --metrics <tmp>/metrics.csv`
   plus any flags epic 002 provides to make music and transport produce samples (if any). Captures stdout/stderr.
3. Asserts, failing with a clear message on the first violation:
   - exit status 0;
   - every line of `metrics.csv` parses against the header (same number of fields as the header), and the number of data rows is 100 ± 2; if a column identifies the module, 25 ± 2 per module;
   - the final summary reports 0 content mismatches and 0 lost datagrams;
   - the clock reached the locked state.
4. Contains no assertion that mentions a sync rate, a latency or a percentile.
5. Exits 0 when all assertions pass, non-zero otherwise.
6. Records, as comments, the exact patterns used for the summary, the lock indicator and the header, each with the `poc-untouched` source location they come from (`[code]`).

## Acceptance criteria
1. `tools/smoke.sh` exits 0 on the fixed build and non-zero on the `poc-untouched` build (corrupt metrics file).
2. `grep -Ei 'sync.?(hz|rate)|latenc|percentile|p9[59]|p50' tools/smoke.sh` finds no assertion line.
3. Negative tests, run by hand against prepared files: a metrics file with one truncated line, one with two rows joined on one line, and one with a duplicated header each make the script's metrics check fail with a message naming the line number.
4. Row-count check accepts 98–102 data rows and rejects 97 and 103; if the per-module check exists, it accepts 23–27 and rejects 22 and 28.
5. Runs unprivileged (no `sudo`, no capabilities).

## Definition of done
- [ ] Tests green in CI, unprivileged (no CI exists: the script and `cargo test`, unprivileged)
- [ ] Every parser touched is fuzzed (not applicable, C-125: no parser touched)
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Running the script three times on the measuring machine and tagging (003-09).
- Sync-rate measurement (003-02, 003-05).
- Fuzzing the script.
- Runtime harness and tool binaries, which live in the `hadronyche-tools` crate (C-133). `tools/smoke.sh` stays in the POC tree under the task 003 exception of C-130.
