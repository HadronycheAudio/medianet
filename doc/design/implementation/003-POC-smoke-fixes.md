# 003 — POC smoke fixes

**Phase:** P0 · **Scope:** M (10 tickets, about 5.5 days) · **Depends on:** 002 (for the cause of the inert check families) · **Blocks:** 007, 017

## Goal

Put the proof of concept under version control, tag it untouched, then apply three minimal fixes so that its loopback run is a citable baseline: one writer per metrics file, deadline-paced clock sync, and a failed run when a check family has zero samples. Tag again. This is pre-implementation check **0c** and is the only task allowed to modify the POC.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules exchange *slices* addressed by *position* on a domain-wide timeline. Linux hosts are the backbone, macOS and Windows machines are edges, embedded devices are followers. The runtime is a rewrite. The proof of concept **hadropoc** (`/home/dorian/workspace/hadropoc`, Rust, one binary, dependencies clap, libc, ratatui, rtrb) is frozen as the reference instrument.

**Evidence tags.** `[code]` read in the POC source · `[measured]` observed in a run · `[decision]` design choice. The full list is in `00-context.md`.

**Constants used.** Every number below is taken from the constants table `doc/design/02-constants.md`, which wins over this file. Rows used: C-36 (50 sync requests per second), C-112 (accepted sync request rate 10 to 200 per second; system criterion AC-7 is checked over this range), C-18 (regime stamp transport), C-95 (fuzz budget), C-125 (definition-of-done applicability), C-126 (scope label from the ticket tally), C-130 (where task outputs live; nothing is written into the frozen POC tree except by task 003), C-133 (where runtime code lives), C-144 (owner of the runtime's single-writer file service), C-146 (privileged tests), C-147 (strict mode and measurements; POC runs do not run in strict mode), C-206 (presentation details of tools), and the POC reference values P-01 (POC feed), P-02 (pre-fix achieved sync rate), P-03 (kernel tick of the measuring machine) and P-04 (POC lock threshold).

**The POC is not the runtime.** The POC's wire (P-01: 48 channels, about 9.6 KB per datagram, 7 IP fragments at MTU 1 500) differs from the runtime's settled wire (one rate on the wire, 96 kHz, C-01; 6 channels and 1 152 bytes of audio per default packet with no per-channel id byte, C-04; about 1 320 bytes per packet on the wire, C-20; IPv4 only, C-100). This epic does not move the POC toward the runtime wire: "no change to the wire format" below means the POC's own format. The baseline is a behavioural reference for epics 007 and 017, not a wire reference.

**State of the POC.** Before task 003 (at the time of the first run) the directory was not a git repository `[measured]`. It has two unit tests, both in `transport.rs` `[code]`. Not being a repository, it had no CI. `hadropoc loopback` runs sender, synth1, synth2 and mixer (four modules) in one process; `--duration`, `--metrics` and `--csv` exist (`main.rs:253-262`), and the headless mode (one line per second) is used only with `--no-tui` (`main.rs:248-249`, branch `main.rs:536-537`; line numbers identical at `poc-untouched` and `poc-baseline`) `[code]`; without `--no-tui` the POC starts the full-screen display and, when none can be initialised outside a terminal, panics with exit status 101 (BASELINE.md section 7 at `poc-baseline`) `[measured]`. The regime stamp (C-18) is a runtime control message; this epic adds none to the POC and touches none `[decision]`.

**Measuring machine.** The machine on which P-02 was measured, with a `CONFIG_HZ=250` kernel (P-03) `[measured]`. Its identity (host name, CPU, kernel version) is recorded with the first measurement of this epic and every later figure is taken on the same machine.

**Measurement conditions.** Every figure of this epic is taken unprivileged, under the same conditions as P-02, so that before and after are comparable. Strict mode (C-127) is a runtime mode that the POC does not have. C-147 settles that POC runs do not run in strict mode: strict mode is required only of runs whose numbers enter the constants table as design constants, and POC figures go only in the POC reference rows (P-nn). The post-fix figures are therefore taken unprivileged, without strict mode, and added beside P-02. No test of this epic needs a privilege, so the privileged nightly job of C-146 is not used.

**Bug 1: corrupt metrics file** `[measured]` `[code]`. A 25-second loopback run produced a metrics file with about 6 parseable lines out of about 80. Root cause: loopback clones the one `--metrics` path into every module's options (`main.rs:317`), and each module's housekeeping thread does `std::fs::File::create(p)` on it (`module.rs:1354`), each with its own `mpsc` channel and its own writer thread (`module.rs:1355-1361`). Four handles truncate the file and write from their own offset 0, overwriting each other. The design rule that follows: **a file has one writer, owned by the process, never by a module**. In the runtime, the single-writer file service belongs to the hosted backend, epic 015 (C-144); this epic fixes the POC only.

**Bug 2: sync pacing** `[measured]` `[code]`. The clock thread paces its requests with a 5 ms socket receive timeout (`clock_sock.set_read_timeout(5 ms)`, `module.rs:1126`; the loop at `module.rs:1504-1600` sends when `now >= next`, then blocks in `net::recv_ts`). After the reply arrives the thread blocks again for a full timeout. Measured on the measuring machine (P-03): `--sync-hz 50` achieved 36.1 completed exchanges per second, `--sync-hz 1000` achieved 71.4 (P-02). One effective timeout of about 13 ms explains both. The fix is to wait until `min(next request deadline, stop check)`, for example with `ppoll` and a timeout computed from `next − now`, as the data thread already does (`module.rs:1663-1667`).

**Sync rates that matter.** The runtime's default is 50 sync requests per second (C-36) and the accepted range is 10 to 200 per second (C-112) `[decision]`. The POC is therefore checked at `--sync-hz 50`. `--sync-hz 1000` lies outside the accepted range: it is measured only so that the before and after figures can be compared with P-02, and nothing depends on whether it is reached. System criterion AC-7 over the full C-112 range is checked against the runtime's servo, a redesign tested in task 008, not against the POC.

**Bug 3: inert check families look green** `[measured]`. In the same run the music and transport check families reported "0 of 0 windows" for the whole run and the run looked identical to a good one. Design rule: **a check family with zero samples is a failed run**. The cause of the zero samples is diagnosed in task 002; this task only makes the condition fatal in headless mode. The failure is any non-zero exit status plus the family's name; no specific exit code is required `[decision]`.

**Constraints.**

- The fixes must be minimal and must not change wire behaviour, the servo, or any measured quantity other than the achieved sync rate.
- Smoke assertions must **never depend on sync-rate timing**: the 36.1/s and 71.4/s figures (P-02) show that timing on an unprivileged machine is not what the flags say.
- The second tag is the frozen baseline. After it, the POC is not modified again.
- No file outside those listed in criterion 003-AC1 may be added between the tags; in particular no CI configuration is added to the POC.

## Deliverables

1. **Repository.** `git init` in `/home/dorian/workspace/hadropoc`; a `.gitignore` excluding `target/` and `experiments/*/`; first commit of the untouched tree plus that `.gitignore`; annotated tag `poc-untouched`.
2. **Metrics writer fix.** One process-owned writer per metrics path:
   - a process-level registry maps a path to one `Sender<Vec<u8>>` and one writer thread;
   - each module obtains a clone of that sender; rows are sent whole (one `Vec<u8>` per row), never through a per-module `BufWriter` that can split a row;
   - the header line is written once;
   - the header text, column set and row format stay as the POC emits them at `poc-untouched`.
3. **Sync pacing fix.** In `spawn_clock`, replace the fixed receive timeout by a wait bounded by the next request deadline; keep a 100 ms ceiling so the stop flag is still noticed.
4. **Zero-sample failure.** In headless mode, at the end of a run with `--duration`, exit non-zero and print the name of any in-scope check family that analysed zero windows. The in-scope families are audio, music and transport, unless task 002 finds that music or transport cannot produce samples in loopback without code changes; such a family is then out of scope for this check and listed as known-inert in `BASELINE.md`.
5. **Smoke script** `tools/smoke.sh`: runs
   `hadropoc loopback --duration 25 --sim-offset-ms 3 --sim-drift-ppm 20 --csv <tmp>/smoke.csv --metrics <tmp>/metrics.csv`
   plus any flags task 002 supplies to make the music and transport families produce samples, and asserts:
   - exit status 0;
   - every line of the metrics file parses against the header (same field count) and there is one row per module per second: 4 × 25 = 100 data rows in total, within ±2 rows; where a row identifies its module, also 25 ± 2 rows per module;
   - zero content mismatches and zero lost datagrams in the final summary;
   - the clock reached the locked state (the POC's own lock rule, P-04);
   - no assertion mentions a sync rate, a latency or a percentile.
   The exact text of the final summary, of the locked-state indicator and of the metrics header is read from the POC at `poc-untouched` and recorded as comments in the script.
6. Second commit and annotated tag `poc-baseline`.
7. A note `BASELINE.md` in the POC root listing the two tags, the three fixes, the known-inert families if any, and the measured achieved sync rate before and after at `--sync-hz 50` and `1000`, with the measuring machine and the measurement method.
8. **Design-set update.** The post-fix achieved sync rates are added to the POC reference values of the constants table (`doc/design/02-constants.md`) next to P-02, then to the evidence appendix `doc/design/06-evidence-appendix.md` (C-130), tagged `[measured]`. The earlier rule "no code goes into hadropoc" is now stated by C-130 with this task as its single exception ("nothing is written into the frozen POC tree except by task 003"); any other design document that still states the unqualified rule is aligned to C-130. `tools/smoke.sh` and `BASELINE.md` live in the POC tree under that exception, not in `doc/design/reports/` or the `hadronyche-tools` crate (C-130, C-133).

## Dependencies

- Needs from 002: the reason the music and transport families are inert, so the smoke script can be given flags under which they produce samples (or, if none exist, so the zero-sample failure is scoped to the audio family with the others listed as known-inert in `BASELINE.md`).
- Provides to 007 (conformance vectors) and 017 (reference modules and L0 harness): a tagged, citable baseline.
- Provides to the design set: the post-fix figures, and alignment of any remaining statement of the earlier rule "no code goes into hadropoc" with C-130, which already names this task as the single exception (deliverable 8).

## Acceptance criteria

1. **003-AC1.** `git tag` in the POC lists `poc-untouched` and `poc-baseline`, both annotated; `git diff poc-untouched poc-baseline --stat` touches only the metrics writer, the clock thread wait, the headless exit path, `tools/smoke.sh` and `BASELINE.md`. (`.gitignore` is part of the first commit, deliverable 1, and does not appear in this diff.)
2. **003-AC2.** After the fix, a 25-second loopback run produces a metrics file in which 100 % of lines parse.
3. **003-AC3.** After the fix, achieved sync rate is within 2 % of the configured rate at `--sync-hz 50` (C-36, inside the C-112 range), that is between 49.0 and 51.0 completed exchanges per second, measured with the same method as the before figure. The achieved figure at `--sync-hz 1000` is recorded whatever it is; it is outside the C-112 range and is not asserted anywhere.
4. **003-AC4.** A headless `--duration` run with an inert in-scope check family exits non-zero and names the family (system criterion AC-9).
5. **003-AC5.** `tools/smoke.sh` passes three times in a row, unprivileged, on the measuring machine.
6. **003-AC6.** The existing two tests in `transport.rs` still pass.
7. **003-AC7.** Contributes to system criteria AC-8 (the loopback smoke run is trustworthy; no assertion depends on sync-rate timing) and AC-31 (one writer per file).

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Both tags exist and `BASELINE.md` records before/after figures
- [ ] The evidence appendix of the design set records the post-fix sync rates as `[measured]`
- [ ] No change to the wire format, the servo algorithm or the analyzer

**Not applicable in this epic** (rule C-125):

- *Tests green in CI, unprivileged* — the "in CI" part only. The POC has no CI and 003-AC1 forbids adding one between the tags. The line is met by `cargo test` and three consecutive `tools/smoke.sh` runs, unprivileged, on the measuring machine (003-AC5, 003-AC6), recorded with the second commit. No test of this epic needs a privilege, so nothing goes to the privileged nightly job of C-146.
- *Every parser touched is fuzzed* — no parser is touched. Fix 1 changes a writer, fixes 2 and 3 change a wait and an exit path. The field-count check in `tools/smoke.sh` is a test assertion over the POC's own output, not a parser of untrusted input, so the fuzz budget C-95 does not apply; its rejection of corrupt files is covered by negative tests instead.
- *Regime stamp fields touched are complete and documented* — regime stamps (C-18) are a runtime control message; no fix adds or touches a stamp field in the POC.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Can the music and transport families be made to produce samples in loopback without code changes? | Task 002 | No flag combination does; the baseline then covers the audio family only and says so |
| Does deadline pacing change the servo's lock behaviour enough to alter the baseline? | This task (measured in the post-fix run) | Lock (P-04) is no longer reached within the 25 s smoke run |
| How is "completed exchanges per second" observed on the untouched POC without modifying it (an output, a counter, or a packet count on the loopback interface)? | This task (first measurement, by reading the POC at `poc-untouched`) | The chosen method cannot be reproduced by a second engineer within 5 % |

Decided: "Is 1000 exchanges/s reachable unprivileged?" no longer needs an answer. 1000 is outside the accepted range of 10 to 200 per second (C-112), so the figure is recorded for comparison with P-02 and nothing depends on it.

Decided: "Which files of the design set are the evidence appendix and the document that states 'no code goes into hadropoc'?" is answered by C-130. The evidence appendix is `doc/design/06-evidence-appendix.md`; the rule, with task 003 as its single exception, is stated in C-130 itself.

Decided: "Does C-147 (strict mode for runs whose numbers enter the constants table) cover the POC reference rows?" is answered by C-147. It does not: POC runs do not run in strict mode and their figures go only in the POC reference rows (P-nn), so the post-fix figures are added beside P-02.

Decided: the wording of the zero-sample message, the layout of `BASELINE.md` and the comments of `tools/smoke.sh` are presentation details of this epic's own tooling, decided here and recorded in `BASELINE.md` (C-206); no other epic depends on them.
