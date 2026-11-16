# 06 — Evidence appendix

What is actually known, how it is known, and the three pre-implementation checks. Tags are defined in [00-context.md](00-context.md).

## 1. POC run results `[measured]`

One loopback run series by the product owner, on the development machine: Linux 6.12, unprivileged (no real-time scheduling, no memory lock), POC built in release mode. These were the only measurements behind this design until task 002 (archaeology verdicts below).

| # | Observation | Mechanism | Status |
|---|---|---|---|
| M-1 | Sync rate far below configured: `--sync-hz 50` gives 37 completed exchanges/s; `--sync-hz 1000` gives 77/s (historical record, read from the corrupt file of M-2; superseded by the clean figures 36.1 and 71.4: P-02, [REPORT-0b](reports/archaeology/REPORT-0b.md) section (e), entry REPORT-0b (e) below) | The clock thread paces requests with a 5 ms socket receive timeout. After each reply nothing wakes it until the timeout expires. One effective timeout of about 13 ms explains both figures: 1000/13 ≈ 77, and two timeouts to pass a 20 ms period give 26 ms ≈ 38/s | Mechanism `[code]`; the 13 ms value is fitted. The kernel ticks at 250 Hz (`CONFIG_HZ=250`), which rounds 5 ms up to 8 ms; the remaining 5 ms is unexplained (unprivileged scheduling is a guess) |
| M-2 | The metrics CSV is corrupt: about 6 parseable lines of about 80 expected, lines fused | In loopback every module receives the same `--metrics` path and calls `File::create` on it, each with its own channel and writer thread. Four handles truncate and write from their own offset 0, overwriting each other | Mechanism `[code]` |
| M-3 | The one-line display shows a "lost" value near 21.7 million, random-walking, in a clean run (2 345 of 2 345 audio windows exact, zero datagram loss) | **Resolved by task 002** (entry REPORT-0b (b) below; unresolved when this run was recorded). The traced chain is saturating arithmetic on sequence numbers and only about 50 000 packets were sent, so the value is not a packet count. Its magnitude looks like nanoseconds (21.7 ms). The display takes the latest record's value across all return streams with no per-stream separation | Task 002: resolved, instrument bug with its mechanism named (entry REPORT-0b (b)) |
| M-4 | The music and transport check families report "0 of 0 windows" for a whole 25 s run that looks green; "playhead: no transport state yet" persists | **Resolved by task 002** (entry REPORT-0b (a) below; unresolved when this run was recorded). | Task 002: resolved, not a bug (entry REPORT-0b (a)); the whole-run signature was reproduced with a second instance on the same group (REPORT-0b (b)) |

M-1 figures were read from the file described in M-2. They fit the model, but should be re-read from a clean file after task 003. (Done by task 002 from clean single-writer files of separate-process runs: 36.1 and 71.4 per follower, entry REPORT-0b (e) below `[measured]`.)

Consequences already drawn in the design:

- Sync pacing is by deadline; the achieved rate is reported (AC-7).
- A file has one writer, owned by the process (AC-31).
- A check family with zero samples fails the run (AC-9).
- No smoke assertion depends on sync-rate timing (AC-8).

### Archaeology verdicts (task 002, REPORT-0b)

One entry per verdict of [REPORT-0b](reports/archaeology/REPORT-0b.md) (check 0b, epic 002), in the format of the table above. Runs were unprivileged on a `CONFIG_HZ=250` host, from scratch copies of the `poc-untouched` tag; the frozen tree was never run.

| # | Observation | Mechanism | Status |
|---|---|---|---|
| REPORT-0b (a) | Inert check families (M-4): music and transport read NO DATA at 0 of 0 only in the first seconds of a loopback run; in two 25 s runs both read PASS from the 5 s line on (voice channels 1849/1908 windows exact, playhead identical for 40869/40869 datagrams); the whole-run inertness did not reproduce from one instance `[measured]` | Not a bug: transport state exists only after the sender claims the reference after the 1.5 s listen delay C-32 (`module.rs:741`), and voice windows are skipped for SETTLE_SECS = 3 s after the conducting transition (`audio.rs:644`, `audio.rs:716-720`) `[code]`; a second instance on the same discovery group reproduces the whole-run signature (section (b)) `[measured]` | Verdict (a), `[code]` `[measured]`; task 003 judges a family empty only past claim and settle (about the first 5 s) |
| REPORT-0b (b) | The loss figure near 21.7 million (M-3) is the genuine output of the per-input loss counter, neither a decoding nor a display error; with two instances on one group the poisoned value runs end to end, hop 1 `lost=220033` to the display's `lost 220 033 (fwd 220 033)` `[measured]` | Instrument bug: the counter (`mixer.rs:78`) is sequence-span arithmetic over an echo input whose sequence space is not bounded to one sender, because the echo stamps its own session (`synth.rs:155-156`) and same-named modules of two instances cross-wire (`module.rs:781`); second bug: the latest-record-wins `lost_fwd` store (`stats.rs:324`) prints return-link loss as forward loss `[code]` | Verdict (b), `[code]` `[measured]`; not fixed by task 003 (epic 003's allowed diff is the metrics writer, the clock wait and the headless exit path), so `poc-baseline` keeps both and POC loss figures are never cited from it (open item in REPORT-0b, close-out); the zero-loss 21.7 M magnitude itself was not reproduced |
| REPORT-0b (c) | All 16 metrics files under `experiments/` (pilot, run1 to run3) are single-writer: every data line parses, 0 `unix_ms` violations, up to 10 893 lines each `[measured]` | The experiment harness gives each process its own `--metrics` path (`metric_experiment.py:51-56`); the corruption of M-2 is the loopback harness's (`main.rs:317`, `module.rs:1354`) `[code]` | Verdict (c), `[measured]`; the I/O-pressure eligibility rule keeps its basis and is inherited (epic 011) |
| REPORT-0b (d) | The voice comparison is bit-level: yes `[code]` | Not a bug: `analyse` compares each finished window bit for bit with the sender's reference, voice and click channels by equality of the raw wire fields (`audio.rs:808-813`), the sum channel by equality of f64 bit patterns (`audio.rs:756`); the tolerance measurement against `music::expected` (`audio.rs:823-846`) is a separate check family; the five loss-handling facts are confirmed `[code]` | Verdict (d), `[code]`; the POC may be cited as the exact-tier baseline (epics 007, 017) |
| REPORT-0b (e) | Run confirmations: a 10× flood (20 000 well-formed datagrams/s, all accepted) leaves every sender block firing, worst lateness 3 846.1 µs against the 10 ms skip threshold, and a 10-pair sign test finds no flood effect on the ticker p99 (F = 0, B = 0); the servo steps published time at all 1078 locked updates (abs Δ median 14 730 ns, s median 27 ns); clean sync rates 9.34 / 36.1 / 71.5 / 71.4 exchanges/s per follower at 10 / 50 / 200 / 1000 Hz, none within 2 % `[measured]` | Instrument bug: sync requests are paced by the effective receive timeout of `module.rs:1126` (timed-out waits of 3 or 4 ticks of 4 ms, about 13 ms), not by `--sync-hz`; the other confirmations found no bug: starvation refuted (`module.rs:1669`, `module.rs:1694`), phase step confirmed (`clock.rs:362`) `[measured]` | Verdict (e), `[measured]`; the pacing is fixed in the POC by task 003 (deadline-paced clock wait); supersedes M-1's figures (P-02); the runtime bounds the drain (C-16, C-156) and paces by deadline |

### Baseline at `poc-baseline` (task 003, ticket 003-10)

One entry per recorded fact of the fixed POC, in the format of the tables above. Source of the facts of the fixes, the lock and the check families: `BASELINE.md` and the commit log of the POC at the tag `poc-baseline` (commit 461f24a); source of the sync rates, the machine and the lock time: `BASELINE.md` sections 3 and 4 at the tag `poc-baseline`; source of the three smoke runs: the body of POC commit 461f24a, verified against the run logs in the review of PR #28. The POC is only read, through `git show` at the tag `[code]`; each entry carries its own tags.

| # | Observation | Mechanism | Status |
|---|---|---|---|
| BASE-1 | Achieved sync rate per follower after the fixes, duration-corrected: 49.86 exchanges/s at `--sync-hz 50` and 944.77 at `--sync-hz 1000` (one 25 s loopback run per rate, two runs in all, with `--sim-offset-ms 3 --sim-drift-ppm 20`; like for like before the fixes: 36.1 and 71.4, P-02) `[measured]` | Measured by ticket 003-05 at POC commit 4a4885e (BASELINE.md section 3), which holds at the tag `poc-baseline`: `git diff --stat 4a4885e poc-baseline` touches only the end of `main()` in `src/main.rs`, `tools/smoke.sh` and `BASELINE.md`, not the clock path `[code]`. Machine: host hackrobustus, CPU i7-1365U, kernel 6.12.115+deb13.1-amd64, `CONFIG_HZ=250` (P-03). Both runs were unprivileged and not in strict mode (C-147) `[measured]` | `[measured]`; the figures go only in the POC reference row P-07, because POC runs are not strict mode (C-147); 49.86 is inside the 2 % band of 003-AC3 (49.0 to 51.0) |
| BASE-2 | The three fixes recorded at `poc-baseline` `[code]`: (1) one process-owned writer per metrics file, commit 7a2d2e4 (003-03; `src/lib.rs`, `src/metrics.rs`, `src/module.rs`); (2) a clock-sync wait bounded by the next request deadline with a 100 ms ceiling, commit 4a4885e (003-04; `src/module.rs`); (3) a non-zero headless exit naming any in-scope check family (audio, music, transport) with zero windows, commits b0dbdfd and 4a13ac1 (003-06; `src/main.rs`) | Fix 1 replaces the four writers that truncated one path (M-2). Fix 2 replaces the fixed 5 ms receive timeout (M-1). Fix 3 prints `check family <name>: 0 windows analysed` on standard error and exits 1, and judges only a process that has an analyzer (a sender or a loopback run) and only the first-listed synth for music and transport (BASELINE.md sections 2 and 6) `[code]` | `[code]`; the commits are the ones in `git log poc-untouched..poc-baseline` |
| BASE-3 | Lock sub-question: deadline pacing still reaches LOCKED. In both runs of BASE-1 the clock reached LOCKED at 4.1 s inside the 25 s run, so it still locks after fix 2 `[measured]` | The lock rule is P-04 (`clock.rs:370-379`, identical at `poc-untouched` and `poc-baseline`; the exit hysteresis is at 375-376) `[code]`; the falsifier "lock no longer reached within the 25 s smoke run" did not fire (BASELINE.md section 4) `[measured]` | `[measured]`, `poc-baseline` |
| BASE-4 | No in-scope check family (audio, music, transport) is known-inert at `poc-baseline` `[code]` | Music and transport produce no windows only until the sender claims the transport reference after the 1.5 s listen delay (C-32) and the 3 s settle horizon has passed, about the first 5 s of a run; see REPORT-0b (a) (BASELINE.md section 5) `[code]` | `[code]`; task 003 judges a family empty only past that horizon |
| BASE-5 | Three recorded smoke runs at `poc-baseline` (`tools/smoke.sh`, unprivileged, one `--group` each, three passes in a row, none repeated), audio windows exact 2346 of 2346, 2345 of 2345 and 2346 of 2346; synth1 and synth2 both verified in each run; `cargo test` 2 passed of 2 (transport time_code and scenario) `[measured]` | Source: the body of POC commit 461f24a, verified against the run logs in the review of PR #28. In each run every module's metrics `lock` column reached LOCKED, which is an assertion of the script itself (BASELINE.md section 7) `[measured]` | `[measured]`, citable as stated and no further (BASE-7) |
| BASE-6 | Disclosure: the three runs of BASE-5 were made with `HADROPOC_BIN` set to a pass-through wrapper that passed the arguments and the exit status through and copied standard output, to record the order of the synth sections; the path of the binary it ran was not logged, and this is not written in the commit body of 461f24a `[measured]` | The review of PR #28 found that the wrapper changes nothing the script asserts: the binary's own exit status is returned, standard error is inherited, the metrics file is written by the binary, and the copy of standard output carries the same bytes; the release binary built from the tree was byte-identical to an independent build and to every other build of that source tree on disk, and the output shows post-fix behaviour, so the residual risk is that a non-identical binary was used `[measured]` | `[measured]`, disclosed here so that BASE-5 is read with it |
| BASE-7 | `[decision]` Owner of the design set, 2026-11-16: "poc-baseline: tag as is, keep the loss counter bug". POC loss figures, including the "0 lost" printed by the three runs of BASE-5, are not citable; the loss counter defect of REPORT-0b (b) was kept at `poc-baseline`. Also not citable from those three runs: the sync rate and the drift (no metrics file was kept), and the per-second headless clock text as lock evidence | The loss counter (`mixer.rs:78`) and the latest-record-wins `lost_fwd` store (`stats.rs:324`) are unchanged: neither file differs between `poc-untouched` and `poc-baseline` `[code]`; the headless clock field is the sender module's own view, which is trivially LOCKED from about 2 s when the sender holds the clock, and its drift depends on which module won the election (review of PR #28, section C; BASELINE.md section 7) `[measured]` | `[decision]`, BASELINE.md section 9; the smoke loss checks can only fail a run |

## 2. POC code pointers `[code]`

Line numbers refer to the POC as found, before any change. Files: `/home/dorian/workspace/hadropoc/src/`.

| Claim | Where |
|---|---|
| Wire header is 88 bytes, version 11, one datagram per slice | `proto.rs` (`HDR`, `VERSION`, `Header`) |
| `t2k` is documented as kernel receive time but written with userspace time, with the kernel-timestamp flag set | `proto.rs` (`Header.t2k`, `F_KTS`) against `synth.rs:162` |
| Header fields overloaded by direction (`ack_seq`, `ack_t4`, `path_delay`) | `proto.rs` field comments; `synth.rs:165-166`; `mixer.rs:163-165` |
| Default feed is 48 channels × (1 + 48 × 4) = 9 264 bytes of audio per datagram | `audio.rs:129` (`payload_len`); `main.rs` (`--channels` default 48, `--frames` default 48) |
| Module id is a 31-bit mix of wall time, pid and monotonic time | `module.rs`, in `Module::start` (the `let id = …` line) |
| A module's own hello is recognised by id only | `module.rs:402-409` |
| Clock sync replies carry the request's own `t1`; exchanges are built from the reply alone | `module.rs:1576-1583` |
| Clock thread: 5 ms receive timeout used as the request pacer | `module.rs:1126` (timeout), `module.rs:1504-1600` (loop) |
| Clock thread takes a mutex shared with housekeeping every iteration | `module.rs` in `spawn_clock` (`inner.target.lock()`) |
| Clock thread runs under `SCHED_FIFO` as well as the data thread | `module.rs` in `spawn_clock` (`rt::setup_thread`) |
| Data thread drains every input completely before checking its deadline | `module.rs` in `spawn_rt` (the `while let Some(..) = net::recv_now(..)` loop) |
| Data-thread ticker runs on local monotonic time, not network time | `module.rs` in `spawn_rt` (the `period` closure) |
| Route table is cloned (allocates) on the data thread at each route change | `module.rs` in `spawn_rt` (`routes = r.clone()`) |
| Connection is unilateral: the output owner resolves the address and sends | `module.rs:775` (`routes`) |
| Name resolution takes the first peer with a matching name | `module.rs:775-785` |
| Holder's hello expiring opens an election | `module.rs:575-585` (`leave`) |
| Gossip constants: hello 1 s, expiry 3 s, listen 1.5 s, election 400 ms, 2 mandates, silence 1.5 s | `module.rs:50-72` |
| Servo steps the phase estimate at each update; gains settle at 0.1 and 0.005 (wired) | `clock.rs`, `Servo::update` |
| Lock requires at least 8 updates and smoothed error below max(20 µs, path delay / 4) | `clock.rs`, `Servo::update` |
| Holdover begins after 2 s and never expires; it stays "usable" | `clock.rs` (`HOLDOVER_NS`, `LockState::usable`, `Servo::tick`) |
| Seqlock publication of the servo view | `clock.rs` (`SharedSync`) |
| Simulated crystal: injectable offset and drift | `clock.rs` (`LocalClock`); `main.rs` (`--sim-offset-ms`, `--sim-drift-ppm`) |
| Kernel receive timestamps via `SO_TIMESTAMPNS`, compared against `CLOCK_REALTIME` | `net.rs` (`configure`, `recv_flags`); `module.rs` (`wake = real_ns() - kts`) |
| Discovery multicast with hop limit 1 | `net.rs` (`discovery`) |
| Mixer keeps the latest block per input and mixes on arrival | `mixer.rs:108-127` |
| Mixer validates message lengths before slicing | `mixer.rs:93-98` |
| Synth resets its transport when the gossiped reference changes | `synth.rs:76-80` |
| Deterministic sine from additions and multiplications only | `music.rs:263` |
| Fixed-point sine table built with libm `f64::sin` | `music.rs:311` |
| Events are state parts: one field of one voice moving to a value over the block | `music.rs` module documentation; `proto.rs` (`Part`) |
| Comparison is per 1 024-sample window; incomplete windows are dropped after 256 windows; 3 s settling | `audio.rs:26-34`, `audio.rs:620-685` (`poll`) |
| The sender's replica receives every part regardless of what the echo received | `sender.rs` (the `prepare` path feeding `self.voices.part`) |
| Loss display takes the latest record's value across streams | `stats.rs:324` |
| Real-time setup is best-effort with collected warnings | `rt.rs` |
| Only two tests exist, both in the transport code | `transport.rs:864-947` |

Not read, or only partly: `audio.rs` `analyse`; most of `sender.rs`, `stats.rs`, `control.rs`, `tui.rs`; the body of `transport.rs`.

## 3. Operating-system facts

| Fact | Tag |
|---|---|
| `SO_TIMESTAMPNS` delivers `CLOCK_REALTIME` timestamps | `[kernel-doc]` socket(7), checked by the product owner |
| `SO_RXQ_OVFL` (since Linux 2.6.33) delivers a cumulative per-socket drop counter in a control message | `[kernel-doc]`, checked by the product owner |
| The development machine's kernel ticks at 250 Hz | `[measured]` `/boot/config` on that machine: `CONFIG_HZ=250` |
| Socket receive timeouts are tick-granular | `[memory]` |
| On that machine: `ipfrag_time` 30 s, `ipfrag_high_thresh` 4 MB, loopback MTU 65 536, Ethernet MTU 1 500 | `[measured]` |
| A kernel that exhausts reassembly memory rejects new fragments host-wide | `[memory]`; moot once fragmentation is forbidden |
| A reassembled datagram is stamped with its last fragment's arrival | `[memory]`; moot once fragmentation is forbidden |
| On a pure layer-2 path a too-small hop drops silently, with no error returned | `[memory]`; drives the probe-pair design |
| macOS has kernel UDP receive timestamps on a monotonic basis | `[memory]`; task 001 |
| Recent Windows builds have a UDP receive timestamp control in QPC units | `[memory]`; task 001 |
| macOS and Windows have no per-socket drop counter | `[memory]`; task 001 |
| Ethernet bit-error figures (1e-12; 1e-10 for gigabit copper) behind the loss floor | `[memory]`; task 001 |
| Receive cost of 2 to 4 µs per packet | `[memory]`, a guess; L1 |

## 4. Pre-implementation checks

All three are non-blocking for specification drafting and blocking for anything that cites their subject. Full task files: [001](implementation/001-Receive-stamp-probes.md), [002](implementation/002-POC-archaeology.md), [003](implementation/003-POC-smoke-fixes.md).

### 0a — Receive-stamp probes (task 001)

1. Write one small prototype per OS that opens a UDP socket, enables whatever receive-timestamp option exists, receives packets from a peer, and prints the stamp beside a userspace clock read.
2. **macOS:** establish whether a kernel stamp exists, and on which basis (monotonic or wall clock).
3. **Windows:** establish whether a receive timestamp control exists, from which build, and in which units.
4. On each OS, verify the don't-fragment option and whether a per-socket drop counter exists.
5. Look up the published Ethernet bit-error figure behind constant C-90.
6. Report at specification time; update the capability table and the capability bits.

Falsifier: stamps absent. Then the clock bound and E targets on that platform rise.

### 0b — POC archaeology (task 002)

1. Diagnose M-3 and M-4 to the level "instrument bug, mechanism named".
2. Check the integrity of the experiment metrics files (`experiments/run1` to `run3`) behind the rule "machines under I/O pressure cannot hold the clock". Multi-machine runs are expected to have one file per process and be intact; confirm it.
3. Read the files listed above as not read.
4. Confirm by a run the claims that are only read from code: the unbounded drain starving the ticker, and the phase step per servo update.

Falsifier: the voice comparison turns out not to be bit-level, or the I/O-pressure finding does not survive. Then the exact-tier description or the eligibility rule changes.

### 0c — POC smoke basics (task 003)

1. Put the POC under version control and tag the untouched state.
2. Fix the metrics writer: one writer per file, owned by the process.
3. Fix sync pacing: deadline-based.
4. Make a zero-sample check family fail the run.
5. Tag the fixed state. Both tags are kept; the fixed state is the citable baseline.

This is the only change the POC ever receives: task 003 is the single exception to the freeze (C-130), and the POC is frozen after the tag `poc-baseline`, which ended that task `[decision]`.
