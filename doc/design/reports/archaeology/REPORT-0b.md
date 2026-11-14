# REPORT-0b — POC archaeology (check 0b)

**Purpose:** this report is the deliverable of pre-implementation check **0b** of epic 002 (POC archaeology). It records the diagnosis of the known instrument anomalies of the frozen proof of concept `hadropoc` down to "instrument bug, mechanism named", the reading of the POC source files never fully read, and the run-level confirmation of two behavioural claims that were so far only read from code. It was created as a skeleton by ticket 002-01; sections (a) to (e) are filled in by the other tickets of the epic (002-02 … 002-14).

**Evidence tags in use** (as defined in `doc/design/00-context.md`): `[code]` verified by reading the POC source, `[measured]` observed in an actual run on this machine, `[kernel-doc]` documented operating-system behaviour checked against the documentation, `[memory]` recalled and not re-verified, `[decision]` a design choice, not a fact. Every new factual claim in this report carries one of these tags.

**Verdict-line format:** each section (a) to (e) ends with exactly one verdict line, either of the two allowed forms:

- **instrument bug, mechanism: …**
- **not a bug, explanation: …**

tagged `[code]` (read in the POC source) or `[measured]` (observed in a run). While a section is unfilled it carries the placeholder **verdict: pending**.

## Environment

All values in this subsection are `[measured]` on the test host on 2026-11-14.

- host: hackrobustus `[measured]` (2026-11-14)
- kernel: 6.12.115+deb13.1-amd64 `[measured]`
- CONFIG_HZ=250, read from `/boot/config-6.12.115+deb13.1-amd64` on this host `[measured]` — this **matches** POC reference value P-03 (=250, the original measuring machine) `[measured]`
- toolchain: rustc 1.100.0 (0ef1bc386 2026-11-09), cargo 1.100.0 (3acc79c0c 2026-11-05) `[measured]`
- run user: uid 1000, unprivileged (no root, no elevated privileges used) `[measured]`
- frozen-tree fingerprint, taken 2026-11-14: `1c39ae238eeafbc75e3910b5b1a1796718b28fc96bc88c5b4243877da2daa873` `[measured]`
  - convention (fixed by the team lead, identical in `checks/002-01.sh` and 003-01): sha256 over the LC_ALL=C sorted concatenation of `path:`<relative-path> NUL <file-contents> NUL for every regular file under `src/`, `tools/`, `experiments/` (pruning `target/`, `.git/`, `build/`, excluding `.gitignore`) plus `Cargo.toml`, `Cargo.lock`, `README.md` `[decision]`
- scratch copy: `/home/dorian/workspace/hadronyche/poc-scratch` — copied from the frozen tree at `/home/dorian/workspace/hadropoc` on 2026-11-14 with `cp -a` of `src/`, `tools/`, `experiments/`, `Cargo.toml`, `Cargo.lock`, `README.md` (no `target/`); the frozen tree itself was not written to `[measured]`
- scratch-copy fingerprint at creation (2026-11-14): `1c39ae238eeafbc75e3910b5b1a1796718b28fc96bc88c5b4243877da2daa873` `[measured]` — identical to the frozen-tree fingerprint, so the copy is content-identical at creation
- poc-untouched tag-tree fingerprint (tracked files only, git-reproducible), taken 2026-11-14: `ac24375dabbc1fa7926e00c91eefd3503d6a7f23648ce7a785d55b0726ea7675` `[measured]`
  - The original working-tree fingerprint above includes the 27 untracked experiment-data files, whose per-file SHA-256s are committed at `checks/002-01-frozen-data.sha256` and are guarded there.

## Loopback run (AC 1)

Built with `cargo build --release` in the scratch copy, unprivileged (`Finished \`release\` profile [optimized] target(s) in 14.40s`) `[measured]`. The binary the POC builds is `hadropoc`; the subcommand is `loopback` with `--duration <seconds>` (read in `src/main.rs`: `Loopback` variant of `enum Cmd`, `duration: Option<f64>` in `Common`) `[code]`.

The POC's default presentation is a full-screen TUI, which needs a terminal; run unattended (no controlling TTY) it panics in `ratatui` terminal init and exits 101 `[measured]`. To run the exact intended command unattended, it was given a pseudo-terminal with `script -qec`; the command and its exit status as run `[measured]`:

```console
$ cd /home/dorian/workspace/hadronyche/poc-scratch
$ TERM=xterm-256color script -qec './target/release/hadropoc loopback --duration 5' /dev/null
$ echo $?
0
```

exit status: 0 — the run completed its 5-second loopback and exited cleanly, as an unprivileged user (uid 1000) `[measured]`.

A TTY-free variant with the POC's own `--no-tui` flag also runs to completion with the same result `[measured]`:

```console
$ cd /home/dorian/workspace/hadronyche/poc-scratch
$ TERM=dumb script -qec './target/release/hadropoc loopback --duration 5 --no-tui' /dev/null
$ echo $?
0
```

exit status: 0 `[measured]`. Output tail of that run (first and last lines of the 5-second window; full lines are long) `[measured]`:

```text
[   0s] net: sender.events waits for synth1.events
[   0s] net: mixer joined from 127.0.0.1 (mixer; 3 inputs, 1 outputs)
[   1s] clock UNLOCKED drift +0.00 ppm err 0.0 µs | rtt mean 0.077 p99 0.144 max 0.465 ms | … | audio 48 ch: mixer-vs-sender identical (90/90 windows exact, 0 lost, 0 sender gaps), vs ideal: mixer -145.2 dB sender -145.2 dB | …
[   1s] map: 4 modules · clock: nobody · transport reference: nobody
[   5s] (final window; run ends and the process exits with status 0)
```

## (a) Inert check families

<!-- Filled by ticket 002-05 (baseline run + transport trace) and ticket 002-06 (music trace, minimal condition, final verdict, hand-off). -->

Run source: the 25 s run below was made from the scratch copy at `/home/dorian/workspace/hadronyche/poc-scratch` — a copy of the frozen tree that is byte-identical to `poc-untouched` (content-identical at creation, fingerprint-verified by ticket 002-01) [measured]. The frozen tree at `/home/dorian/workspace/hadropoc` was neither run nor written to: its working tree now carries the epic-003 fixes (a read-only `git log` there shows the 003-03 and 003-04 commits on top of the initial untouched commit) [measured], so a run from it would not have been poc-untouched-equivalent.

The exact command as run, unprivileged (uid 1000; the binary's own warnings confirm no real-time scheduling and no memlock); the `--group` flag (per `loopback --help`: "Multicast group and port the modules find each other on") is set to an isolated group and port per the swarm rule for concurrent runs [measured]:

```console
$ cd /home/dorian/workspace/hadronyche/poc-scratch
$ ./target/release/hadropoc loopback --duration 25 --csv out-002-05.csv --no-tui --group 239.72.68.44:44250
$ echo $?
0
```
[measured]

Reproduction status: the anomaly **did not reproduce** in this run [measured]. The music and transport families were inert — `music 32 voices on synth1: NO DATA (steady: 0 of 0 ...)`, `transport: NO DATA`, `playhead: no transport state yet` — only during the first seconds of the run; from the 2 s line on the transport counters climb and the playhead reads PLAYING, and from the 5 s line on both families read PASS and stay PASS for the rest of the 25 s. The final summary line of the run, verbatim:

```text
[  25s] clock LOCKED drift -69.99 ppm err 0.3 µs | rtt mean 0.085 p99 0.107 max 0.205 ms | fwd mean 0.009 p99 0.014 max 0.136 ms | ret mean 0.016 p99 0.019 max 0.038 ms | inside mean 0.060 p99 0.082 max 0.132 ms | Round trip: synth 2 mean 0.087 p99 0.111 max 0.217 ms | Round trip: feed only mean 0.082 p99 0.105 max 0.142 ms | playable(99.9%) 0.521 ms | sent 49964 lost 0 (fwd 0) dropped-at-sender 0 | audio 48 ch: mixer-vs-sender identical (2338/2338 windows exact, 0 lost, 1 sender gaps), vs ideal: mixer -145.4 dB sender -145.4 dB | volume controller: PASS (0 of 2338 windows over epsilon 1e-4), 49964 messages, sent 0.9761 measured 0.9761, range 0.002..0.997, max error 2.5e-8 | music 32 voices on synth1: PASS (steady: 0 of 48103 over epsilon, max level error 2.3e-5; moving: 0 of 13177 over, max 2.5e-2), voice channels 1915/1915 windows exact, max pitch error 2.0e-3 Hz (1915 windows) | parts sent 19546 (637 notes started, 14 replaced while waiting), refreshed 380750, deepest queue 160, longest wait 9.50 ms, samples without controller values at the mixer 0 | transport: PASS (9 changes, 55 pulses, 789 repeats sent; synth playhead identical for 40880/40880 datagrams, 0 changes arrived late; metronome: 47/47 clicks on the right sample, 0 wrong (furthest 0 samples), channel exact in 1915/1915 windows) | least accurate voice 0 (23 % of epsilon), now: pitch 1068.25 Hz (silent), level 0.0000/0.0000, timbre 0.0000/0.0000, articulation 0.0000/0.0000 | also synth2: music PASS (0 of 48129 steady and 0 of 13183 moving measurements over epsilon), voice channels 1916/1916 windows exact, 0 lost, transport PASS (playhead identical for 40909/40909 datagrams, 47/47 clicks right) | mix peak 2.461 | playhead: PLAYING  bar 4 beat 6 (19.321)  6/8  240.000 BPM  time code 01:00:07:24 @ 25
```
[measured]

The inert phase the ticket describes is visible only as a startup transient [measured]: the 1 s line shows both families at 0 of 0 with `playhead: no transport state yet`, the 1 s map shows the sender still `transport reference (claiming)` with the reference slot `nobody`, the transport reference election lands at 2 s, the 2 s line already has the playhead PLAYING while the voice windows are still 0 of 0, and the 5 s line is the first with analysed voice windows:

```text
[   1s] clock UNLOCKED drift +0.00 ppm err 0.0 µs | rtt mean 0.075 p99 0.120 max 0.356 ms | Round trip: synth 2 mean 0.078 p99 0.297 max 0.364 ms | Round trip: feed only mean 0.072 p99 0.092 max 0.353 ms | sent 1965 lost 0 (fwd 0) dropped-at-sender 0 | audio 48 ch: mixer-vs-sender identical (91/91 windows exact, 0 lost, 0 sender gaps), vs ideal: mixer -145.3 dB sender -145.3 dB | volume controller: PASS (0 of 91 windows over epsilon 1e-4), 1965 messages, sent 0.9924 measured 0.9924, range 0.048..0.997, max error 1.8e-8 | music 32 voices on synth1: NO DATA (steady: 0 of 0 over epsilon, max level error 0.0e0; moving: 0 of 0 over, max 0.0e0), voice channels 0/0 windows exact, max pitch error 0.0e0 Hz (0 windows) | parts sent 1003 (37 notes started, 6 replaced while waiting), refreshed 15301, deepest queue 160, longest wait 9.50 ms, samples without controller values at the mixer 0 | transport: NO DATA (1 changes, 1 pulses, 0 repeats sent; synth playhead identical for 0/0 datagrams, 0 changes arrived late; metronome: 0/0 clicks on the right sample, 0 wrong (furthest 0 samples), channel exact in 0/0 windows) | also synth2: music NO DATA (0 of 0 steady and 0 of 0 moving measurements over epsilon), voice channels 0/0 windows exact, 0 lost, transport NO DATA (playhead identical for 0/0 datagrams, 0/0 clicks right) | mix peak 1.853 | playhead: no transport state yet
[   1s] map: 4 modules · clock: nobody · transport reference: nobody
[   1s] map: *sender   0d63f047 127.0.0.1       (I/O pressure 0.0 %; load average 0.26 on 12 cores = 0.02 per core; cpu 14.1 %, late 11 µs, worst 334 µs, machine 1 %, 4 conn) · clock UNLOCKED · transport reference (claiming) · generator,transport
[   2s] net: election 1 requested by synth2: nobody holds the clock
[   2s] net: transport: sender is the reference
[   2s] net: election 1: 4 candidates (synth2; synth1; sender; mixer)
[   2s] net: clock: synth2 holds it (election 1)
[   2s] clock UNLOCKED drift +0.00 ppm err 0.0 µs | rtt mean 0.068 p99 0.093 max 0.356 ms | Round trip: synth 2 mean 0.070 p99 0.100 max 0.364 ms | Round trip: feed only mean 0.065 p99 0.088 max 0.353 ms | sent 4048 lost 0 (fwd 0) dropped-at-sender 0 | audio 48 ch: mixer-vs-sender identical (188/188 windows exact, 0 lost, 0 sender gaps), vs ideal: mixer -145.4 dB sender -145.4 dB | volume controller: PASS (0 of 188 windows over epsilon 1e-4), 4048 messages, sent 0.0423 measured 0.0423, range 0.002..0.997, max error 1.8e-8 | music 32 voices on synth1: NO DATA (steady: 0 of 0 over epsilon, max level error 0.0e0; moving: 0 of 0 over, max 0.0e0), voice channels 0/0 windows exact, max pitch error 0.0e0 Hz (0 windows) | parts sent 1795 (69 notes started, 6 replaced while waiting), refreshed 31173, deepest queue 160, longest wait 9.50 ms, samples without controller values at the mixer 0 | transport: NO DATA (1 changes, 3 pulses, 19 repeats sent; synth playhead identical for 0/0 datagrams, 0 changes arrived late; metronome: 0/0 clicks on the right sample, 0 wrong (furthest 0 samples), channel exact in 0/0 windows) | also synth2: music NO DATA (0 of 0 steady and 0 of 0 moving measurements over epsilon), voice channels 0/0 windows exact, 0 lost, transport NO DATA (playhead identical for 0/0 datagrams, 0/0 clicks right) | mix peak 3.764 | playhead: PLAYING  bar 1 beat 3 (2.078)  4/4  120.000 BPM  time code 01:00:01:00 @ 25
[   5s] clock LOCKED drift -69.83 ppm err 1.3 µs | rtt mean 0.064 p99 0.087 max 0.356 ms | fwd mean 0.007 p99 0.012 max 0.024 ms | ret mean 0.011 p99 0.015 max 0.026 ms | inside mean 0.043 p99 0.055 max 0.263 ms | Round trip: synth 2 mean 0.067 p99 0.091 max 0.364 ms | Round trip: feed only mean 0.062 p99 0.084 max 0.353 ms | playable(99.9%) 0.519 ms | sent 9989 lost 0 (fwd 0) dropped-at-sender 0 | audio 48 ch: mixer-vs-sender identical (465/465 windows exact, 0 lost, 1 sender gaps), vs ideal: mixer -145.4 dB sender -145.4 dB | volume controller: PASS (0 of 465 windows over epsilon 1e-4), 9989 messages, sent 0.9968 measured 0.9968, range 0.002..0.997, max error 2.1e-8 | music 32 voices on synth1: PASS (steady: 0 of 1091 over epsilon, max level error 9.6e-6; moving: 0 of 253 over, max 1.7e-2), voice channels 42/42 windows exact, max pitch error 1.7e-3 Hz (42 windows) | parts sent 4073 (144 notes started, 6 replaced while waiting), refreshed 76423, deepest queue 160, longest wait 9.50 ms, samples without controller values at the mixer 0 | transport: PASS (1 changes, 8 pulses, 139 repeats sent; synth playhead identical for 929/929 datagrams, 0 changes arrived late; metronome: 0/0 clicks on the right sample, 0 wrong (furthest 0 samples), channel exact in 42/42 windows) | least accurate voice 31 (17 % of epsilon), now: pitch 8751.99 Hz (+5.6e-4), level 0.9644/0.9644, timbre 0.2939/0.2939, articulation 0.7656/0.7656 | also synth2: music PASS (0 of 1115 steady and 0 of 261 moving measurements over epsilon), voice channels 43/43 windows exact, 0 lost, transport PASS (playhead identical for 948/948 datagrams, 0/0 clicks right) | mix peak 3.320 | playhead: PLAYING  bar 2 beat 4 (7.014)  4/4  90.000 BPM  time code 01:00:04:00 @ 25
```
[measured]

The companion CSV of this run is kept alongside the report at `doc/design/reports/archaeology/out-002-05.csv` (49 964 rows, one row per measured packet) [measured].

The four trace points, traced in the scratch copy's source (read-only) on top of the 002-03 reading notes in section (d):

1. The transport reference in loopback is the sender [code]: main.rs:408 constructs the sender with `TransportRole::Reference`, and `--transport` names `reference` as the default for a sender (main.rs:287-293) [code]; the run agrees — `net: transport: sender is the reference` at 2 s, and the 3 s map marks the sender `transport REFERENCE` [measured].

2. `ctx.is_reference()` becomes **true** in loopback, about 2 s into the run [code]: `is_reference` compares the gossip-elected reference with the module id (module.rs:997-998), the sender claims the role once its listen delay has passed (module.rs:740-744), and the first claimant wins the election (module.rs:746-753); sender.rs:229 consumes it as the `conducting` flag [code]. The run shows the transition — `transport reference (claiming)` with the reference slot `nobody` at 1 s, `net: transport: sender is the reference` at 2 s, and the transport counters climbing from then on (1 changes, 1 pulses, 0 repeats sent at 1 s → 1 changes, 3 pulses, 19 repeats sent at 2 s) [measured].

3. The listen-before-claim delay is C-32, `LISTEN_NS = 1_500_000_000` ns = 1.5 s (module.rs:55), applied at module.rs:741: a module with the Reference role sets its claim stamp only after it has listened for 1.5 s since joining [code]. Its effect on when the reference starts conducting: for the first 1.5 s it conducts nothing — the 1 s map shows `transport reference (claiming)` with the reference slot `nobody`, and the claim plus the clock election (held back by the same gate, module.rs:676) land only at 2 s (`election 1 requested by synth2: nobody holds the clock`, then `net: transport: sender is the reference`) [measured] — so transport state exists on the network only from about 2 s on.

4. Transport and music datagrams are routed through the events connections [code]: the sender declares the `events` output carrying MUSIC | TRANSPORT and the `audio` output carrying AUDIO | CONTROL | TRANSPORT (sender.rs:200-201); `fire` copies header plus tail — transport state, pulses and musical events — into the events datagram and sends it to the synths, while the same transport bytes ride the audio feed to the mixer (sender.rs:314-317); loopback connects `sender.events` to `synth1.events` and `synth2.events` (main.rs:502) [code]. The run shows the routing live — `out events:60987 (music+transport) → synth1.events, synth2.events` in the map lines — and its effect: `synth playhead identical for 929/929 datagrams` at 5 s, `40880/40880` at 25 s [measured]. So both transport and music reach the synths via the events connections once conducting.

Interim conclusion — where transport state stops: the chain is reference claim and election (module.rs:740-753) → the `is_reference` test at sender.rs:229 gating `conductor.fill` and `fire` (transport.rs:776-798, sender.rs:314-317) → the events connections (main.rs:502) → synth ingestion. The link where transport state would stop reaching the synths is the `is_reference` test at sender.rs:229: while it reads false, `discard()` empties the outbox (transport.rs:802-805), so `fill` writes no transport bytes into any datagram tail and no state, pulse or click can reach a synth regardless of routing [code]. In this run that gate opened at 2 s and stayed open, so the whole-run inertness was not observed; the residual inert window of the first seconds is the C-32 claim delay plus the settle horizon, which the conducting transition moves forward (audio.rs:644) so that early voice windows are skipped as settling (audio.rs:716-720) [code]. The music-family trace and the section verdict belong to 002-06.

The three music points, traced in the same pristine scratch source as the transport trace above [code]:

**Music point 1 — the settling exclusions (SETTLE_SECS).** On the returned side of place_voices, a voice-frame position that jumps by more than 1 s of samples (`frame.pos > last + rate`, audio.rs:603) moves the settle horizon to the resumption position plus `SETTLE_SECS = 3` s of samples (audio.rs:604); every conducting transition moves the same horizon forward to the newest finished window plus 3 s (audio.rs:644, constant `SETTLE_SECS: u64 = 3` at audio.rs:34), and every window before the horizon is skipped and counted as settling instead of being analysed (audio.rs:716-720) — so each return jump or conducting transition buys the synth a 3 s settling exclusion, and this is exactly what held the music family at `0 of 0` through the 4 s line of both captured runs, with the first analysed voice windows at the 5 s line [code][measured].

**Music point 2 — the voices flag.** The voices flag is not a command-line option: main.rs:404-405 creates the feed's analyzer with `voices = false` (the sum path) and every synth's analyzer with `voices = true`, and that fixed value is what arms the music family — the per-voice measurement against the music tolerances in analyse_voices (audio.rs:801) — in every loopback run [code]. The flag is threaded through Analyzer::new (audio.rs:511), selects the complete-window condition on the voice channels (audio.rs:664-667) and routes analyse to analyse_voices (audio.rs:715-723); its value for the music family is `voices = true` unconditionally, so no flag combination turns the music family off, and none is needed to turn it on [code].

**Music point 3 — voice frames and place_voices.** Voice frames do reach place_voices (audio.rs:601): poll pops the sender's replica frames into it at audio.rs:633 and the frames echoed by the synths at audio.rs:636 [code]. The sender pushes its replica frames unconditionally for every connected synth path (sender.rs:340-348, gated only on the events datagram going out, sender.rs:331), so frames arrive from the moment the events connections form; the conducting gate (sender.rs:334) holds only the transport reports and clicks, never the voice frames [code]. The conditions under which frames do not reach place_voices are a dead events path or a synth that never echoes — neither active in any captured run — so the anomaly's mechanism cannot be missing voice frames [code].

**Minimal condition.** Both families need no extra flags or graph wiring: the music family is armed by construction (`voices = true` at main.rs:405) and produces its first non-zero windows once two constants have elapsed — the 1.5 s listen-before-claim delay C-32 (module.rs:55, applied at module.rs:741), after which the sender starts conducting, plus the 3 s settle horizon that the conducting transition installs (audio.rs:644) — and the transport family needs the same claim before any playhead report exists to compare; so any loopback run longer than about 5 s shows both families with non-zero windows, and the default command below, run for 25 s from the scratch copy at `/home/dorian/workspace/hadronyche/poc-scratch` (byte-identical to `poc-untouched`, verified file by file against the tag before the run; the frozen tree untouched), is the minimal condition in practice [code]:

```console
$ cd /home/dorian/workspace/hadronyche/poc-scratch
$ ./target/release/hadropoc loopback --duration 25 --csv out-002-06.csv --no-tui --group 239.72.68.44:44251
$ echo $?
0
```
[measured]

Final summary line of the same run, verbatim [measured]:

```text
[  25s] clock LOCKED drift -104.88 ppm err 0.4 µs | rtt mean 0.407 p99 6.931 max 15.273 ms | fwd mean 0.154 p99 3.763 max 8.669 ms | ret mean 0.092 p99 2.177 max 12.950 ms | inside mean 0.161 p99 2.661 max 8.153 ms | Round trip: synth 2 mean 0.389 p99 5.999 max 15.581 ms | Round trip: feed only mean 0.236 p99 4.526 max 10.413 ms | playable(99.9%) 7.734 ms | sent 50107 lost 34 (fwd 0) dropped-at-sender 0 | audio 48 ch: mixer-vs-sender identical (2323/2323 windows exact, 12 lost, 5 sender gaps), vs ideal: mixer -145.4 dB sender -145.4 dB | volume controller: PASS (0 of 2323 windows over epsilon 1e-4), 50107 messages, sent 0.9865 measured 0.9865, range 0.002..0.997, max error 2.5e-8 | music 32 voices on synth1: PASS (steady: 0 of 47938 over epsilon, max level error 2.3e-5; moving: 0 of 13118 over, max 2.5e-2), voice channels 1849/1908 windows exact, max pitch error 2.0e-3 Hz (1908 windows) | parts sent 19627 (637 notes started, 14 replaced while waiting), refreshed 381797, deepest queue 160, longest wait 9.50 ms, samples without controller values at the mixer 0 | transport: PASS (9 changes, 55 pulses, 791 repeats sent; synth playhead identical for 40869/40869 datagrams, 0 changes arrived late; metronome: 47/47 clicks on the right sample, 0 wrong (furthest 0 samples), channel exact in 1908/1908 windows) | least accurate voice 0 (23 % of epsilon), now: pitch 1068.25 Hz (silent), level 0.0000/0.0000, timbre 0.0000/0.0000, articulation 0.0000/0.0000 | also synth2: music PASS (0 of 47760 steady and 0 of 13104 moving measurements over epsilon), voice channels 1849/1902 windows exact, 17 lost, transport PASS (playhead identical for 40879/40879 datagrams, 46/46 clicks right) | mix peak 3.639 | playhead: STOPPED  bar 4 beat 6 (19.333)  6/8  tempo —  time code 01:00:08:00 @ 25
```
[measured]

The captured counts of this run [measured]: music — `voice channels 1849/1908 windows exact`, with 0 of 47938 steady and 0 of 13118 moving measurements over epsilon; transport — `synth playhead identical for 40869/40869 datagrams`, 47/47 metronome clicks right, channel exact in 1908/1908 windows; both families read PASS from the 5 s line to the end of the run, after reading NO DATA at `0 of 0` during the transient [measured]. This run also had real loss (34 datagrams lost, 12 audio windows lost, 5 sender gaps) and both families still reported PASS, so the minimal condition does not depend on a lossless run [measured]. The transient shape matched the 002-05 run: at 2 s the playhead already reads PLAYING while the music family is still `0 of 0` (settle horizon), at 4 s it is still `0 of 0`, at 5 s the first voice windows are analysed (42/42 exact) [measured]. The companion CSV of this run is kept alongside the report at `doc/design/reports/archaeology/out-002-06.csv` (50 075 rows) [measured].

**What remains unexplained.** The transient inert window is fully explained: transport state cannot exist before the claim (module.rs:741) and voice windows cannot be analysed before the settle horizon (audio.rs:716-720) [code]. But the original anomaly reported music 0 of 0 windows for a whole 25 s run while audio was 2345/2345 exact, and that did not reproduce — neither in the 002-05 run nor in this one [measured]. The trace bounds its possible causes to two: voice frames never completing a window because the events path never delivered them (audio.rs:665-666), or return gaps of more than 1 s recurring within every 3 s settle horizon (audio.rs:603-604) — neither occurred in any captured run, so the cause of the original whole-run inertness cannot be named from this run's evidence [code].

**For task 003 (AC-9: a check family with zero samples fails the run).** What made the families inert: the transport family has no samples until the sender claims the transport reference after the 1.5 s C-32 listen delay (module.rs:741), because playhead reports and clicks are pushed only while conducting (sender.rs:334-338); the music family has no analysed samples until the settle horizon passes — 3 s after each conducting transition (audio.rs:644) and after each return jump of more than 1 s (audio.rs:603-604) — and windows before it are skipped, not analysed (audio.rs:716-720) [code]. Both mechanisms make the families legitimately inert at 0 of 0 early in every run, so 003's zero-sample check must judge a family empty only from a part of the run that is past the claim and past the settle horizon (roughly the first 5 s of a loopback run), or a whole-run emptiness like the original anomaly will false-alarm [code][measured].

**verdict: not a bug, explanation: the music and transport families are inert only during the startup claim+settle window — the transport family has no state to compare until the sender claims the reference after the 1.5 s listen delay (module.rs:741), and the music family's voice windows are skipped for SETTLE_SECS = 3 s after the conducting transition moves the settle horizon (audio.rs:644, audio.rs:716-720); the original whole-run inertness (music 0 of 0 windows) did not reproduce — in the captured 25 s summary the music family reads voice channels 1849/1908 windows exact and the transport family reads playhead identical for 40869/40869 datagrams [measured] — and its cause cannot be named from this run's evidence [code]**

## (b) The 21.7 M counter

To be filled by ticket 002-08 and successors.

**verdict: pending**

## (c) Experiment metrics integrity

Filled by ticket 002-09. The single-writer rule, restated (settled by the epic's Background; also in the checker's documentation, `hadronyche-tools/src/metrics_check.rs`): a metrics file is **single-writer** exactly when the metrics header (`module.rs:1292`) appears exactly once, as the first line; every line after it parses against the header (38 comma-separated fields, each of the type the POC writes — integers, decimals and the two names at the columns defined by the fixed row format at module.rs:1408) [code]; `unix_ms` never decreases from one parseable line to the next; and no line is interleaved [decision].

The checker is the `metrics-check` binary of the `hadronyche-tools` crate (C-133; not in the frozen tree). It is read-only: it walks `experiments/` recursively and reports every regular file whose first line is the metrics header, so the `runs.csv`, log and analysis files in the same directories are not reported. Its output line format (fixed by the checker's documentation, C-206) is `<path>: single-writer <yes|no> (lines <total>, parseable <parseable>, unix_ms violations <count>)` [decision]. It found **16 metrics files** under `experiments/` (`pilot`, `run1`, `run2`, `run3`, four processes each); the run (2026-11-14, unprivileged, `cargo run --release --bin metrics-check -- /home/dorian/workspace/hadropoc/experiments`) reported [measured]:

```text
/home/dorian/workspace/hadropoc/experiments/pilot/mixer.1.csv: single-writer yes (lines 295, parseable 294, unix_ms violations 0)
/home/dorian/workspace/hadropoc/experiments/pilot/sender.1.csv: single-writer yes (lines 315, parseable 314, unix_ms violations 0)
/home/dorian/workspace/hadropoc/experiments/pilot/synth1.1.csv: single-writer yes (lines 315, parseable 314, unix_ms violations 0)
/home/dorian/workspace/hadropoc/experiments/pilot/synth2.1.csv: single-writer yes (lines 295, parseable 294, unix_ms violations 0)
/home/dorian/workspace/hadropoc/experiments/run1/mixer.1.csv: single-writer yes (lines 10752, parseable 10751, unix_ms violations 0)
/home/dorian/workspace/hadropoc/experiments/run1/sender.1.csv: single-writer yes (lines 10893, parseable 10892, unix_ms violations 0)
/home/dorian/workspace/hadropoc/experiments/run1/synth1.1.csv: single-writer yes (lines 10893, parseable 10892, unix_ms violations 0)
/home/dorian/workspace/hadropoc/experiments/run1/synth2.1.csv: single-writer yes (lines 10758, parseable 10757, unix_ms violations 0)
/home/dorian/workspace/hadropoc/experiments/run2/mixer.1.csv: single-writer yes (lines 10873, parseable 10872, unix_ms violations 0)
/home/dorian/workspace/hadropoc/experiments/run2/sender.1.csv: single-writer yes (lines 10871, parseable 10870, unix_ms violations 0)
/home/dorian/workspace/hadropoc/experiments/run2/synth1.1.csv: single-writer yes (lines 10873, parseable 10872, unix_ms violations 0)
/home/dorian/workspace/hadropoc/experiments/run2/synth2.1.csv: single-writer yes (lines 10873, parseable 10872, unix_ms violations 0)
/home/dorian/workspace/hadropoc/experiments/run3/mixer.1.csv: single-writer yes (lines 1841, parseable 1840, unix_ms violations 0)
/home/dorian/workspace/hadropoc/experiments/run3/sender.1.csv: single-writer yes (lines 1840, parseable 1839, unix_ms violations 0)
/home/dorian/workspace/hadropoc/experiments/run3/synth1.1.csv: single-writer yes (lines 1840, parseable 1839, unix_ms violations 0)
/home/dorian/workspace/hadropoc/experiments/run3/synth2.1.csv: single-writer yes (lines 1841, parseable 1840, unix_ms violations 0)
```

Every one of the 16 files shows parseable lines = total lines − 1 (the header), so every data line parses against the header; 0 `unix_ms` violations; the single-writer verdict is yes for 16 of 16 [measured].

**I/O-pressure finding: survives.** The rule "a machine under I/O pressure cannot hold the clock" rests on the experiment metrics, and its basis holds: all 16 metrics files, up to 10 893 lines each, are intact single-writer files — every line parses, `unix_ms` is monotonic with zero violations across `pilot` and the three multi-machine runs [measured]. The reason these files are intact where the loopback metrics file was not (only about 6 parseable lines out of about 80, measured during the epic's investigation, ticket 002-09's context) is the write topology: `tools/metric_experiment.py` gives each of the four processes its own `--metrics` path — `synth1.{g}.csv`, `synth2.{g}.csv`, `mixer.{g}.csv`, `sender.{g}.csv` — so no experiment file is ever opened by two writers [code], while `hadropoc loopback` clones one `--metrics` path into all four modules (`main.rs:317`), each of whose housekeeping threads `File::create`s it with its own writer thread, truncating from its own offset 0 (`module.rs:1354`) [code]. The loopback clobbering is an instrument bug of the loopback harness (the fix is task 003's), not corruption of the experiment data and not a property of the machines.

Sub-question "Does the I/O-pressure finding survive?" (Open sub-questions): answered — yes; the falsifier (experiment files corrupt or multi-writer) did not hold [measured].

**verdict: I/O-pressure finding survives — mechanism: the experiment harness writes one metrics file per process (`metric_experiment.py:51-56`), so no file has two writers, and all 16 metrics files under `experiments/` check single-writer with every data line parsing and zero `unix_ms` violations; the multi-writer truncation seen in loopback (one `--metrics` path cloned into four modules, main.rs:317, each truncating from its own offset 0, module.rs:1354) never touches these files [measured]**

## (d) Reading notes

To be filled by the reading tickets of the epic.

### audio.rs — analyse / analyse_voices

`poll` (audio.rs:620) drains the queues and places every incoming audio block and voice frame into per-window buckets (`place` audio.rs:576, `place_fields` audio.rs:589, `place_voices` audio.rs:601), then hands every finished window to `analyse` (audio.rs:667-672, audio.rs:715) [code]. Without the `voices` flag, `analyse` compares the returned sum channel sample for sample against the sender's reference, counting a sample as matching only when the f64 bit patterns are equal (`w.returned[n].to_bits() != w.reference[n].to_bits()`, audio.rs:756); a window with zero mismatched samples is counted exact (audio.rs:763-765); it also checks the applied volume against eps.volume (default 1e-4, dimensionless 0..1, main.rs:193) and reduces FFT spectra (audio.rs:767-798) [code]. With the flag set, `analyse` skips any window before the settling horizon (audio.rs:716-720) and calls `analyse_voices` (audio.rs:801), which does two distinct things [code]: (i) it compares the returned voice and click channels bit for bit against the sender's own copy of the instrument — the `same` closure tests the raw u32 wire fields for slice equality, `a[from..to] == b[from..to]` (audio.rs:808-810) — counting voice_exact (audio.rs:811) and, while conducting, click_exact (audio.rs:813; when not conducting, click_exact is counted without a check, audio.rs:815-817); (ii) separately, it decodes each returned voice, recomputes from `music::expected` what the sender's fields were doing, and measures level error against eps.music (default 1e-4, level 0..1) or eps.moving (default 0.1, where a field moves within the window) and pitch error against eps.pitch (default 0.01 Hz), a window being bad when the normalised error exceeds 1.0 (audio.rs:823-846; defaults main.rs:193-208) [code].

The five loss-handling facts from the ticket's Context, checked against the code:

- Comparison is per 1024-sample window (WINDOW = 1024, audio.rs:26; buckets per window in `win`, audio.rs:556-557) — confirmed [code] (audio.rs:26, audio.rs:556-557).
- A window is compared only when it is complete on both sides — in voices mode `n_voice_ref >= WINDOW && n_fields >= WINDOW` and `n_voice_ret >= WINDOW`, otherwise `n_ref >= WINDOW` and `n_ret >= WINDOW`; one still incomplete waits and is not analysed, because comparing a half-received window would be meaningless — confirmed [code] (audio.rs:664-667).
- A window still lagging more than MAX_LAG_WINDOWS = 256 windows behind the newest (256 × 1024 samples at the 96 kHz default rate, main.rs:167, about 2.7 s) is dropped and never analysed; it is counted as a lost window if its sending side was complete, otherwise as a sender gap, because the per-window bookkeeping must stay bounded — confirmed [code] (audio.rs:31, audio.rs:673-684).
- Settling: when the returned voice frames jump by more than 1 second of samples (`frame.pos > l + rate`, audio.rs:603), the settle horizon moves to the resumption position plus SETTLE_SECS = 3 s (audio.rs:604), and it also moves on a conducting transition (audio.rs:644); windows before the horizon are skipped and counted as settling (audio.rs:716-720), because a just-started or just-resumed synth needs time to stabilise (echo reports before the horizon are skipped too, audio.rs:657) — confirmed [code] (audio.rs:34, audio.rs:603-604, audio.rs:644, audio.rs:716-720).
- The sender's replica never applies gaps and is never subject to the settle logic: gap detection and the settle extension run only on the returned side (`if returned`, audio.rs:602-604), because the reference stream is the sender's own output, which has no transport losses — confirmed [code] (audio.rs:576-587, audio.rs:602-604).
- voices flag off: with the flag off, finished windows are analysed on the sum path instead and the voice path is not used, because such a run checks the mix rather than the instruments — [code] (audio.rs:511, audio.rs:664-667, audio.rs:715-723).

Verdict statement: `analyse` compares each finished window of the returned audio bit for bit against the sender's reference — voice and click channels by exact equality of the raw wire fields (audio.rs:808-813) and the sum channel by equality of the f64 bit patterns (audio.rs:756) — while the per-voice measurement against the music, moving and pitch tolerances (audio.rs:823-846) is a separate check family, so the exact tier for voices rests on the replica comparison [code].

Voice comparison bit-level: yes

The yes refers to the replica-vs-returned comparison of the voice and click channels (audio.rs:808-813); the tolerance measurement against `music::expected` (audio.rs:823-846) is a separate check family and does not weaken the exact tier. No instrument bug was found in this reading [code].

### transport.rs — the transport state machine and its wire encoding

`transport.rs` defines what every instance on the network must agree on about the timeline — playing or stopped, the playhead in beats and on the timeline, the tempo, the time signature and the time code — all in integer arithmetic tied to the shared media clock, so that every instance finds the same position for the same sample [code] (transport.rs:1-8). **Who acts as transport reference:** the sender is the transport reference by default; `main.rs:408` constructs the sender with `TransportRole::Reference`, and `--transport reference|follower` overrides that per instance [code] (main.rs:287-293).

Wire encoding: three message kinds travel as `K_TRANSPORT` messages, the kind in the `channels` byte of the message header — `T_STATE` = 0, `T_PULSE` = 1, `T_REPORT` = 2 [code] (transport.rs:37-40). `State` is 40 bytes: serial 4 bytes, flags 1 byte, time signature 2 bytes, time-code rate 1 byte, `at` 8 bytes, playhead 8 bytes in signed Q32.32 beats, timeline 8 bytes in samples, bar 4 bytes, bar_beat 4 bytes [code] (transport.rs:109-150). `Pulse` is 24 bytes: serial 4 bytes, sub-sample fraction 4 bytes, media sample 8 bytes, beat 8 bytes [code] (transport.rs:163-186). `Report` is 40 bytes: the follower echoes its playhead (beats, timeline, bar, beat-in-bar, late count, time code in frames, tempo in thousandths of a beat per minute) [code] (transport.rs:200-248). There is no tempo on the wire: the tempo is the spacing of the pulses [code] (transport.rs:17-20). Time-code rates: 24/25/29.97/30 fps, with drop-frame numbering and HH:MM:SS;FF labels at transport.rs:79-89 [code]. One forward datagram reserves `ROOM = 4 * (MSG + STATE_LEN)` bytes for transport messages [code] (transport.rs:47).

Follower side: `Transport` (transport.rs:332-341) holds the state in force, future states queued by `at`, and the pulses of the current serials; `message()` dispatches state/pulse by kind and ignores anything else [code] (transport.rs:365-371). Caps: at most 8 queued states and 16 kept pulses [code] (transport.rs:374-400).

`report()` (transport.rs:427-449) puts queued states in force and interpolates the playhead along the pulses; with no state in force it returns a default `Report` whose flags lack `KNOWN`, and `Report::label()` then prints the string "no transport state yet" — the string the loopback display showed [code] (transport.rs:430, transport.rs:255-258). A state arriving after the sample it applies from counts into `late` [code] (transport.rs:379-381); `Metronome` renders the test clicks from the block's beat onsets, a square wave fading over 2 ms [code] (transport.rs:498-553).

Sender side: `Conductor` (transport.rs:620-817) runs an 8-entry test scenario that loops every 16 s (`CYCLE_SECS`, transport.rs:588), issuing a `State` on each Start/Stop/Continue/Signature change and one `Pulse` per beat, with the beat clock kept `lead` samples ahead of the audio [code] (transport.rs:745-761). States and recent pulses are re-sent every 100 ms so that losses repair and late joiners sync [code] (transport.rs:762-770). **How transport datagrams are sent:** `Conductor::fill` writes the queued messages, header included, into the datagram tail [code] (transport.rs:776-798); the sender declares an `events` output connection carrying MUSIC | TRANSPORT and an `audio` output carrying AUDIO | CONTROL | TRANSPORT [code] (sender.rs:200-201); `fire` copies header plus tail into the events datagram and sends it on `events` to the synths, while the same transport bytes ride the audio feed to the mixer [code] (sender.rs:314-317); the follower's `Report` travels back with each echo, written by the synth [code] (transport.rs:25-26, synth.rs:147). When the instance is not the reference, `discard()` empties the outbox and the scenario runs on silently [code] (transport.rs:802-805). **How `ctx.is_reference()` is decided:** not in these files — `is_reference` compares the transport reference elected by the gossip plane with this module's id (`routes.reference == self.id`), and sender.rs:229 only consumes it [code] (module.rs:992-999).

### control.rs — the controller plane, not the control plane

`control.rs` is the **controller plane**: slow control signals that travel next to the audio. It is not the control plane — module discovery, elections and the clock live in module.rs — and these notes keep the two names apart [code] (control.rs:1-2). A controller is a stream of 32-bit float values tagged with an 8-bit channel id, clocked at the audio clock divided by `DIVIDER = 4` audio samples per tick; tick k belongs to audio sample `k * DIVIDER` [code] (control.rs:21-22). Wire payload after the header: one id byte per controller, then interleaved 32-bit values, one row per tick; in the header `frames` is the tick count and `sample_pos` the audio sample position of the first tick [code] (control.rs:7-12). Between ticks the value is a straight line [code] (control.rs:72-74).

The only controller defined is `VOLUME = 0` (control.rs:23-24), which drives the volume of the echo's sum; values are float, or unsigned Q0.32 when the header carries `F_FIXED` [code] (control.rs:9-10). The test signal is a triangle ramp: 0 up to 1 in 1 s, back to 0 in the next 1 s, with no jumps — float (control.rs:30-34), Q0.32 twin (control.rs:38-42), and the exact continuous form for comparison (control.rs:63-67) [code]. Q0.32 interpolation truncates the quarter step, and a Q1.31 sample times a Q0.32 volume keeps only the high word [code] (control.rs:46-59).

Sender side: `render` (control.rs:87-95) writes the ramp payload — the `VOLUME` id byte, then one 4-byte value per tick; `apply_ramp`/`apply_ramp_fixed` (control.rs:98-113) are what the echo is expected to do, per audio sample [code]. `span` (control.rs:79-83) maps an audio block to its tick range, including the tick after the last sample, which the interpolation of the last samples needs [code].

Echo side: `Volume` (control.rs:117-184) keeps a ring of 8192 slots (`SLOTS`, control.rs:26) of received ticks keyed by tick number, interpolates between the tick at and the tick after each audio sample, and starts at unity gain [code] (control.rs:128-131). When a tick never arrived the last value stays and the sample counts in `starved` [code] (control.rs:160-183). Silently inactive paths: `Volume::store` (control.rs:138-151) rejects a payload whose byte length does not match the declared controller and tick counts, or whose `sample_pos` is not a multiple of `DIVIDER`, and silently ignores a payload without the `VOLUME` id while returning true [code].

The control-plane timers of the constants table, C-30 to C-33, are not in these files: they live in module.rs, and each matches the table [code]:

- C-30 hello period 1 s — `HELLO_NS = 1_000_000_000 ns` [code] (module.rs:50) — matches.
- C-31 hello expiry 3 s, a peer not heard for that long having left — `EXPIRE_NS = 3_000_000_000 ns` [code] (module.rs:52, applied at module.rs:624) — matches.
- C-32 listen-before-claim 1.5 s — `LISTEN_NS = 1_500_000_000 ns` [code] (module.rs:55): a newcomer waits this long, hearing the network before it claims the clock when nobody holds it (module.rs:676) or before it claims the transport reference role (module.rs:741) — matches.
- C-33 election window 400 ms — `ELECTION_NS = 400_000_000 ns` [code] (module.rs:57) — matches.
### sender.rs — deadlines, wire encoding and the return path

- Purpose: the sender owns the media clock, sends musical events, transport and the audio feed to the synths and the mixer on absolute deadlines, and measures everything the mixer sends back (sender.rs:1-3) [code].
- Main types: `Sender` (sender.rs:105) holds the send buffers, phrase and queue, the sender's own replica of the instruments (`voices`, sender.rs:124) and one `Path` per synth; `MasterShared` (sender.rs:26) carries the atomic counters the display reads (`sent` sender.rs:29, `dropped` sender.rs:53); `SumTaps`/`SynthTaps` (sender.rs:65-79) are the rtrb queues feeding the reference-vs-returned comparison [code].
- Wiring: `module` (sender.rs:146) declares the single input "return", handled by `Sender::returned` (sender.rs:202), and a block timer running `prepare` then `fire` (sender.rs:203) [code].
- `prepare` (sender.rs:209) renders block n ahead of its deadline: the audio message (sender.rs:215), the controller values in the same datagram (sender.rs:219-224), the transport scenario while conducting — `ctx.is_reference()` gates it (sender.rs:229) — queued musical events (sender.rs:256-262), and the reference sum, computed with the mixer's own mixing code (sender.rs:271-284) [code].
- `fire` (sender.rs:290) stamps the `Header` (sender.rs:297-312) and sends two datagrams with that same header: events to the synths, the feed to the mixer (sender.rs:316-317) [code].
- TX pacing lateness: `fire` receives the block's scheduling lateness as `late: i64` in ns and clamps it into the header field `tx_late` (i32) at sender.rs:308; the value rides every outgoing header and is read back per returned record (sender.rs:430) [code].
- The return path `returned` (sender.rs:368) classifies each datagram by reusing a header field as a direction marker: `direct = h.ack_seq == ctx.id` (sender.rs:371) decides feed-echo vs synth reply, and otherwise `h.ack_seq` selects the synth path (sender.rs:378); reordering is inferred from `h.seq` against the per-stream highest sequence (sender.rs:381-382) [code].
- Loss hop: the echo header field `rx_lost` — a u32, 4-byte count of packets, no unit — is copied unconverted from the returned datagram's header into the sender's `Record` at sender.rs:435 (`rx_lost: h.rx_lost`), next to the timing fields derived from the header timestamps (fwd = t2k−t1, turn = t3−t2k, ret = recv−t3, sender.rs:422-427) [code].
- Flagged as input to 002-08 (field reuse by direction, factual only): `ack_seq` doubles as stream discriminator (sender.rs:371) and sender identity (sender.rs:307); `h.ack_t4` carries the mixer's starved-sample count back (sender.rs:397, field comment sender.rs:38-40); `path_delay` is filled with the sample rate, rather than a delay (sender.rs:309) [code].
- Offset symmetry, checked for the flag above: the header's `rx_lost` (u32, proto.rs:177) is written at bytes 72..76 and read back at the same offset (proto.rs:205, proto.rs:233), so sender-side encode and decode agree on it [code].
- Open question for 002-08: could a return stream's `rx_lost` value have been computed by the echo against a different direction's counters? Owner: ticket 002-08. Falsifier: showing that each echo handler fills `rx_lost` from the per-input counters matching the stream the record identifies [code].

### stats.rs — record ingestion and the snapshot

- Purpose: drains per-packet `Record`s from the real-time threads and turns them into a `Snapshot` for the display (stats.rs:1-2) [code].
- `Record` (stats.rs:20-50) carries all durations in ns (stats.rs:18): the send-lateness field ("master send time minus its deadline", stats.rs:42-43) and `rx_lost` as u32 (stats.rs:48); `stream` (stats.rs:21) tags the return path a record came from — 0 for the primary, `DIRECT` = 100 for the feed's direct path (sender.rs:24) [code].
- The `Aggregator` (stats.rs:189) polls its rtrb consumers and calls `ingest` per record (stats.rs:287-299) [code].
- Loss hop: at stats.rs:324 (`self.lost_fwd = r.rx_lost as u64`) the value is widened to a u64 counter of packets — no unit conversion — and stored by plain overwrite, so the latest record wins: it is one pooled value across all return streams with no per-stream separation, regardless of the stream that delivered the record; which streams reach the overwrite is decided by the gate at stats.rs:303-312 (the primary synth path, stream 0, always does — its first record sets `primary_seen`, stats.rs:313-314; the other synth paths return early at stats.rs:310-311 and only feed their extra rows, stats.rs:306-308; the feed's direct path reaches it only until that first stream-0 record sets `primary_seen`) [code].
- `snapshot` (stats.rs:392) assembles the display: the latency rows are ns summaries (stats.rs:400-418), and the row labelled "TX pacing lateness" at stats.rs:416 is fed from the TX_LATE metric (stats.rs:343, pushed from the record's `tx_late` in ns); the lateness is computed at the sender, sender.rs:308, in ns, and converted to ms only for display (`ms` divides by 1e6, stats.rs:357, tui.rs:36-38), never in the stored value [code].
- `lost_total` is computed independently of `lost_fwd`, from first/highest sequence numbers and `received` (stats.rs:444-449); `Snapshot.lost_fwd` is u64 (stats.rs:168) [code].
- The CSV export writes the lateness raw, in ns, as `tx_late_ns` (stats.rs:241, stats.rs:377), straight from the value stamped at sender.rs:308 [code].

### tui.rs — the one-line status and the dashboard

- Purpose: the live display — a full-screen ratatui dashboard, or one line per second with `--no-tui` (tui.rs:1) [code].
- `run_headless` (tui.rs:52) polls the aggregator, snapshots every 100 ms and prints `headless_line` once per second (tui.rs:71-74); `run_tui`/`draw` (tui.rs:248, tui.rs:285) render the dashboard [code].
- Loss hop, one-line status: the branch at tui.rs:107-114 matches on (`sent`, `lost_total`) — on the sender side it prints "sent … lost … (fwd …) dropped-at-sender …", otherwise "received … lost …"; both branches print `s.lost_fwd`, the u64 packet count stored at stats.rs:324, as a raw integer with no unit conversion, so a printed value near 21.7 × 10^6 is that counter as displayed — a magnitude that also fits u32 (max 4 294 967 295), so the width of the printed number alone does not decide its type [code].
- Dashboard loss panel: `draw_loss` (tui.rs:450) shows the same numbers; in the sender branch it splits the pooled `lost_fwd` against `lost_total` — "forward" is `s.lost_fwd.min(lost)`, "return" is `lost.saturating_sub(s.lost_fwd)` (tui.rs:458-463) [code].
- The row "TX pacing lateness" appears in the dashboard's latency table, in ms (`draw_table`, tui.rs:393-423); it is computed at sender.rs:308 in ns [code].
- `ms` (tui.rs:36-38) divides ns by 1e6 for display only; the one-line status omits the sender's lateness row (its row list, tui.rs:95, covers only rtt/fwd/ret/inside/turnaround) [code].

**verdict: pending**

## (e) Run confirmations

To be filled by ticket 002-14 and successors.

**verdict: pending**

## Open sub-questions

Copied from the Notes of ticket 002-01 (`doc/design/tickets/002/002-01-Scratch-copy-and-report-skeleton.md`):

| Question | Owner | Falsifier |
|---|---|---|
| Is the voice comparison bit-level? | This epic (ticket 002-02) | answered: **yes** — the replica-vs-returned voice/click comparison is exact bit equality (audio.rs:808-813), so the "exact tier for voices" description stands; the falsifier ("compares within tolerance only") did not hold [code] |
| Does the I/O-pressure finding survive? | This epic (ticket 002-09) | answered: **yes** — all 16 metrics files under `experiments/` check single-writer (every data line parses, `unix_ms` monotonic, zero violations, REPORT-0b section (c)), so the eligibility rule's basis holds; the falsifier (experiment files corrupt or multi-writer) did not hold [measured] |
| Is the 21.7 M value a decoding error or a display error? | This epic (ticket 002-08) | Neither; the value is a real counter and the loss accounting is wrong |
| Why does a 5 ms timeout behave as about 13 ms? | This epic (ticket 002-12; informational only) | None needed; the runtime paces by deadline |
