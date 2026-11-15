# 002 — POC archaeology

**Phase:** P0 · **Scope:** M (14 tickets, about 13 days) · **Depends on:** none · **Blocks:** 003 (confirmation of the two bugs it fixes, and the answer to (a)), 007, 017 (exact-tier citations), 011 (the I/O-pressure eligibility rule), 005

## Goal

Diagnose the POC's known instrument anomalies to the level of "instrument bug, mechanism named", read the source files that were never fully read, and confirm by a run the behavioural claims that were only read from code. This is pre-implementation check **0b**. Until it is done, the POC must not be cited as the exact-tier baseline and no conformance vector may be extracted from it.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules exchange *slices* addressed by *position* on a domain-wide timeline. Linux hosts are the backbone, macOS and Windows machines are edges, embedded devices are followers. The runtime is a rewrite; the proof of concept **hadropoc** (`/home/dorian/workspace/hadropoc`, Rust, about 8 700 lines, one binary) is frozen as the reference instrument: its sender, synth and mixer become the reference sender, echo and mixer, and its analyzer becomes the tolerance tier of the test suite.

**Evidence tags.** `[code]` read in the POC source · `[measured]` observed in a run · `[kernel-doc]` (as defined in `00-context.md`) · `[memory]` recalled, unverified · `[decision]` design choice · `[standard]` cited from a named published standard with the clause (C-132). The tags are defined in `00-context.md`; every number in this epic agrees with the constants table `02-constants.md`, which wins on any difference.

**What the POC does.** Modules (sender, synth, mixer) find each other by multicast gossip, elect a clock holder, then stream 96 kHz audio, controllers, musical-event parts and transport in datagrams, and the sender verifies the returned content by rerunning the echo's code on its own copy. Its default feed is 48 channels in datagrams of about 9.6 KB, 7 IP fragments at MTU 1500, about 150 Mbit/s (P-01) `[code]`. `hadropoc loopback` runs the whole graph (sender, synth1, synth2, mixer) in one process (`main.rs:483-500`) `[code]`. The POC's 96 kHz rate is the domain rate of the design (C-01), so window durations below convert at 96 000 samples per second.

**Anomalies observed in a 25-second unprivileged loopback run on kernel 6.12** `[measured]`:

1. **Inert check families.** The music and transport checks reported "0 of 0 windows" for the whole run and the display said "playhead: no transport state yet", while the audio check was green (2345/2345 windows exact, 0 datagram loss). A run where half the verification is inert looked identical to a good run. Cause unknown.
2. **Garbage loss counter.** The one-line display showed a "lost" value of about 21.7 million, random-walking, during that clean run. The traced chain is:
   - per-input counters in the mixer (`mixer.rs:78`) and synth (`synth.rs:88`): `(highest − first + 1) − rx_count`, saturating arithmetic on sequence numbers `[code]`;
   - written to the echo header field `rx_lost` (`synth.rs:168,195`, `mixer.rs:167,194`) `[code]`;
   - copied into the sender's record (`sender.rs:435`) `[code]`;
   - stored as `lost_fwd` in the statistics, where the latest record wins across all return streams with no per-stream separation (`stats.rs:324`) `[code]`;
   - printed by the one-line display (`tui.rs:107-114`) `[code]`.
   Only about 50 000 packets were sent, so the value is not a packet count. Its magnitude suggests nanoseconds (21.7 ms). Suspects: a header field overloaded by direction, or a field decoded from the wrong place. Unresolved. (The runtime replaces the single "latest record wins" counter with per-connection summary counters per gap class, C-47, full list C-159; this epic only diagnoses the POC.)
3. **Corrupt metrics file.** Only about 6 parseable lines out of about 80. Mechanism established `[code]`: loopback clones one `--metrics` path into all four modules (`main.rs:317`), and each module's housekeeping thread calls `File::create` on it (`module.rs:1354`) with its own channel and writer thread, so four handles truncate and write from their own offset 0. (This one is fixed in task 003, not here.)
4. **Sync rate.** `--sync-hz 50` achieved 37 completed exchanges per second (superseded by the clean 36.1, P-02); `--sync-hz 1000` achieved 77 (superseded by the clean 71.4, P-02) `[measured]`. Mechanism `[code]`: the clock thread paces requests with a 5 ms socket receive timeout (`module.rs:1126`); one effective timeout of about 13 ms explains both figures on a `CONFIG_HZ=250` kernel (P-03) `[measured]`. Both figures were read from the corrupt metrics file and should be re-read from a clean one. (Re-read from clean single-writer files as 36.1 and 71.4, REPORT-0b section (e), P-02 `[measured]`.)

**The eligibility rule at stake.** The design inherits the rule "a machine under I/O pressure cannot hold the clock". It rests on the experiment metrics under `experiments/` (run1, run2, run3, produced by `tools/metric_experiment.py`). Those multi-machine runs are expected to have written one file per process and to be intact, but that has not been checked.

**Files never fully read.** `audio.rs` functions `analyse` (line 715) and `analyse_voices` (line 801), most of `sender.rs`, the body of `transport.rs` (947 lines; only its two tests were read), most of `stats.rs`, `control.rs`, `tui.rs`. Consequence: it is not known whether voice windows are compared bit for bit or only within tolerance.

**Claims read from code but never run.**

- *Unbounded drain.* The data thread empties every input socket (`while let Some(..) = net::recv_now(..)`, `module.rs:1669`) before it looks at its own deadline, so one flooding input should starve the ticker `[code]`. (The runtime bounds the drain to 2 × the admitted packets per slice per input per cycle, C-16, where a cycle is one slice period, C-156; this epic only confirms or refutes the POC behaviour.)
- *Phase step per servo update.* `Servo::update` sets `theta0 = pred + alpha·w·err` at each window (`clock.rs`, in `update`), which is a step in published time of a fraction of the error every 200 ms `[code]`. The 200 ms window is inherited by the design (C-66); the POC declares lock when the smoothed error is below max(20 µs, path delay / 4) after at least 8 windows (P-04) `[code]`. The measured step distribution is an input to the servo simulation that must confirm the slew limits C-60 and C-61.

**Control-plane timers the trace in (a) meets.** Hello period 1 s (C-30), listen-before-claim 1.5 s (C-32), election window 400 ms (C-33), all inherited from the POC; the trace confirms the values in code.

**Sync-rate range.** System criterion AC-7 (achieved sync rate within 2 % of configured) is checked over the accepted request range of 10 to 200 per second (C-112); the design default is 50 per second (C-36). The POC baseline therefore covers 10, 50 and 200 Hz; 1000 Hz is outside the accepted range and is re-measured only because it is the anomaly figure P-02 and exposes the timeout mechanism.

**Known loss handling in the POC** `[code]` (`audio.rs:20-34`, `audio.rs:620-685`): comparison is per 1 024-sample window; a window is compared only when complete on both sides; one still incomplete 256 windows later (`MAX_LAG_WINDOWS`, about 2.7 s at 96 kHz) is counted lost or as a sender gap and never analysed; a synth silent for more than a second gets a 3-second settling exclusion (`SETTLE_SECS`). The sender's replica never applies gaps. The design inherits this as the exact-tier gap baseline, which is why `analyse` must be read.

**Rules that settle how this epic is run.**

- *Scope* is the ticket tally (C-126): 3 XS, 10 S, 1 M, about 13 days, label M.
- *Definition-of-done lines without a subject* are marked not applicable with the reason, never skipped (C-125); see the note after the checklist.
- *The only code this epic ships* is the read-only metrics checker of (c), a tool binary in the `hadronyche-tools` crate of the design set's repository (C-133), whose CI runs it. It is the subject of "Tests green in CI" and "Every parser touched is fuzzed"; its line parser is fuzzed for 60 s on every change and 1 hour nightly, and the corpus is kept (C-95). Its output line format and file names are decided by this epic and recorded in the checker's documentation (C-206).
- *Scratch copy.* Any path outside `/home/dorian/workspace/hadropoc`; the path and a content fingerprint of the frozen tree and of the copy at creation are recorded in the report. All diagnostic logging lives there; a standalone flood sender is a throwaway probe in `probes/` of the design set's repository (C-133); nothing goes into the frozen tree (C-130).
- *Report location.* The report and its companion files (logs, patches, captured outputs) live in `doc/design/reports/archaeology/` of the design set's repository (C-130), never in the frozen POC tree. The checker's code lives in `hadronyche-tools` (C-133).
- *Constants table* means `02-constants.md`. A verdict that changes a POC reference value updates its P-row there first (P-02 for the sync rate, P-03 for `CONFIG_HZ`, the loss-handling facts if `analyse` differs); other design documents that repeat the changed statement (for example "exact tier for voices") are found by searching the design set, including the architecture specification `doc/design/01-architecture.md` (C-171), and updated after.
- *Evidence appendix* is `doc/design/06-evidence-appendix.md` (C-130); each verdict gets one entry in the entry format that file already uses.
- *Strict mode and P-rows.* C-147 settles it: POC runs do not run in strict mode, and their figures go only into the POC reference rows (P-nn), never into design constants. The (e) 3 runs are unprivileged like the anomaly run and update P-02 as POC figures.

## Deliverables

One document, `doc/design/reports/archaeology/REPORT-0b.md` (C-130), with a section per item below. Each section ends with a verdict line: **instrument bug, mechanism: …** or **not a bug, explanation: …**, tagged `[code]` or `[measured]`.

**(a) Inert check families.** Procedure:
1. Run `hadropoc loopback --duration 25 --csv out.csv` and capture the final summary.
2. Trace why no transport state reaches the synths: who is the transport reference in loopback (the sender defaults to reference per `main.rs` help text for `--transport`), whether `ctx.is_reference()` becomes true (`sender.rs:229`), the listen-before-claim delay (1.5 s, C-32), and whether transport and music datagrams are routed (`events` output connections in loopback).
3. Trace why the music family has zero windows: settling exclusions (`audio.rs:604`, `audio.rs:644`), the `voices` flag in the analyzer, and whether voice frames reach `place_voices`.
4. State the minimal condition under which both families produce samples, and confirm it with a 25 s run.

**(b) The 21.7 M counter.** Procedure:
1. Log `rx_lost` at each hop of the chain above for one 25 s loopback run from the scratch copy.
2. Identify the first hop where the value is not a small packet count.
3. Name the field that actually holds the printed value.

**(c) Experiment metrics integrity.** For every metrics file under `experiments/`: count lines, count lines that parse against the header (`module.rs:1292`), check monotonic `unix_ms`, and state whether each file had a single writer. A file is judged single-writer when every line after the header parses, `unix_ms` never decreases, the header appears once, and no line is interleaved; the report restates this rule. State whether the I/O-pressure finding survives.

**(d) Reading notes.** For each unread file, a half-page: what it does, and specifically for `analyse`: are voice windows compared bit for bit, within tolerance, or both; which windows are excluded and why. The verdict line of (d) is **not a bug, explanation: …** stating what `analyse` compares, unless the reading finds an instrument bug, in which case it is **instrument bug, mechanism: …**.

**(e) Run confirmations.**
1. Measure the nominal datagram rate of one mixer input, then flood that input at ten times that rate with well-formed datagrams from a standalone sender or a scratch-copy option. Run a baseline and a flooded run of at least 25 s each and record the sender-side "TX pacing lateness" and the mixer's ticker lateness (deadline against actual tick time, logged in the scratch copy if the POC does not expose it); confirm or refute starvation.
2. Log published `theta0` before and after each servo update while locked, for at least 200 locked updates (a run of at least 60 s); report the step size distribution.
3. Re-measure achieved sync rate at `--sync-hz 50` and `1000`, and at the ends of the AC-7 range, `10` and `200` (C-112), from clean single-writer metrics files: either each module as a separate process with its own `--metrics` path, or loopback in the scratch copy with one path per module. Three runs of at least 30 s per rate.

## Dependencies

- Needs: nothing. Items (a), (b), (e) need the POC built; (c) needs the files under `experiments/`.
- Provides to 003: confirmation of the two bugs it fixes, and the answer to (a), since 003 makes a zero-sample family fail the run.
- Provides to 007 and 017: clearance to cite the POC as exact-tier baseline and to extract vectors, and the statement of what `analyse` compares.
- Provides to 011: whether the I/O-pressure eligibility rule is inherited or dropped.

## Acceptance criteria

1. **002-AC-1.** Each of (a) to (e) has a verdict line with a named mechanism and at least one `file:line` pointer or a captured run output.
2. **002-AC-2.** For (a): a command line is given under which both families report non-zero windows, or the report states that none exists and why.
3. **002-AC-3.** For (b): the report names the source field of the printed value, with the log that shows the first wrong hop.
4. **002-AC-4.** For (c): a table of every metrics file with total lines, parseable lines and a single-writer verdict.
5. **002-AC-5.** For (d): the question "is the voice comparison bit-level" is answered yes or no.
6. **002-AC-6.** For (e): the flood result, the step distribution and the achieved sync rate at 10, 50, 200 and 1000 Hz, with numbers.
7. **002-AC-7.** Contributes to system criterion AC-9 (a check family with zero samples fails the run) by explaining the inert families, and to system criterion AC-7 (achieved sync rate within 2 % of configured, over 10 to 200 per second, C-112) by providing the clean POC baseline.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Diagnostic logging used for (b) and (e) lives in a scratch copy of the POC, not in the frozen tree
- [ ] The evidence appendix of the design set is updated with each verdict

**Not applicable in this epic (C-125).** "Regime stamp fields touched are complete and documented": the epic writes no runtime code and touches no regime stamp field. "Tests green in CI, unprivileged" and "Every parser touched is fuzzed" apply only to the metrics checker of (c), the one piece of code the epic ships (fuzz budget C-95); every other ticket has no subject for them. "Unprivileged" refers to per-change CI (C-146); the checker needs no privileged nightly job.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Is the voice comparison bit-level? | This epic (ticket 002-02) | `analyse` compares within tolerance only; the "exact tier for voices" description changes |
| Does the I/O-pressure finding survive? | This epic (ticket 002-09) | Experiment files are corrupt or multi-writer; the eligibility rule loses its basis |
| Is the 21.7 M value a decoding error or a display error? | This epic (ticket 002-08) | Neither; the value is a real counter and the loss accounting is wrong |
| Why does a 5 ms timeout behave as about 13 ms? | This epic (ticket 002-12; informational only) | None needed; the runtime paces by deadline |
