# 003-03 — Process-owned metrics writer
**Epic:** 003-POC-smoke-fixes · **Effort:** S · **Depends on:** 003-01

## Context
- Epic 003 applies three minimal fixes to the frozen proof of concept **hadropoc** (`/home/dorian/workspace/hadropoc`, Rust). This is fix 1. Fixes must be minimal and must not change wire behaviour, the servo, the analyzer, or any measured quantity other than the achieved sync rate.
- Bug 1 `[measured]` `[code]`: a 25-second loopback run produced a metrics file with about 6 parseable lines out of about 80. `hadropoc loopback` runs sender, synth1, synth2 and mixer in one process. Loopback clones the single `--metrics` path into every module's options (`main.rs:317`); each module's housekeeping thread does `std::fs::File::create(p)` on it (`module.rs:1354`), each with its own `mpsc` channel and its own writer thread (`module.rs:1355-1361`). Four handles truncate the file and write from their own offset 0, overwriting each other.
- Design rule `[decision]`: **a file has one writer, owned by the process, never by a module.** Contributes to system criterion AC-31 (one writer per file).
- The epic fixes the metrics layout as "unchanged": the header text, column set and row format stay as the POC emits them at `poc-untouched` (epic deliverable 2).
- Definition-of-done applicability (rule C-125, recorded in the epic): this ticket changes a writer and touches no parser, so "Every parser touched is fuzzed" is not applicable. The POC has no CI, so "Tests green in CI" is met by `cargo test` run unprivileged.

## Work
1. Add a process-level registry mapping a metrics path to exactly one `Sender<Vec<u8>>` and one writer thread. The first request for a path creates the file once, writes the header line once, and spawns the writer thread; later requests return a clone of the same sender.
2. Change the module housekeeping code around `module.rs:1354-1361` to obtain a sender clone from the registry instead of calling `File::create` and spawning its own channel and writer.
3. Each module formats a full row into one `Vec<u8>` (including the trailing newline) and sends it whole. Remove any per-module `BufWriter` that could split a row.
4. The writer thread is the only owner of the file handle; it writes each received `Vec<u8>` in full and flushes on channel close.
5. Keep the row format, column set and header text unchanged.

## Acceptance criteria
1. `grep` shows no `File::create` on the metrics path inside module code; the only creation of a metrics file is in the process-level registry.
2. A 25-second run of `hadropoc loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20 --metrics <tmp>/metrics.csv` produces a file in which 100 % of lines parse against the header (same field count as the header), and the header appears exactly once (epic 003-AC2).
3. The file has one row per module per second: 4 × 25 = 100 data rows within ±2 in total, and, if a column identifies the module, 25 ± 2 rows per module.
4. The header line is byte-identical to the one the POC emits at `poc-untouched`.
5. The two existing tests in `transport.rs` still pass (epic 003-AC6).
6. The diff of this ticket touches only the metrics writer code.

## Definition of done
- [ ] Tests green in CI, unprivileged (no CI exists: `cargo test`, unprivileged)
- [ ] Every parser touched is fuzzed (not applicable, C-125: no parser touched)
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No change to the wire format, the servo algorithm or the analyzer
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The CSV file written by `--csv` (not named as affected by the epic).
- The runtime's single-writer file service: it belongs to the hosted backend, epic 015 (C-144).
- Automated checking in `tools/smoke.sh` (003-07).
- Sync pacing (003-04) and the zero-sample exit (003-06).
