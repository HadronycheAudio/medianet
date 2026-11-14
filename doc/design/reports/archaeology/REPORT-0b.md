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

To be filled by ticket 002-02 and successors.

**verdict: pending**

## (b) The 21.7 M counter

To be filled by ticket 002-08 and successors.

**verdict: pending**

## (c) Experiment metrics integrity

To be filled by ticket 002-09 and successors.

**verdict: pending**

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
- Loss hop: at stats.rs:324 (`self.lost_fwd = r.rx_lost as u64`) the value is widened to a u64 counter of packets — no unit conversion — and stored by plain overwrite, so the latest record wins: it is one pooled value across all return streams with no per-stream separation, regardless of the stream that delivered the record; which streams update it is decided by the gate above (synth-path records always do, the feed's direct path only until a synth answers, stats.rs:303-312) [code].
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
| Does the I/O-pressure finding survive? | This epic (ticket 002-09) | Experiment files are corrupt or multi-writer; the eligibility rule loses its basis |
| Is the 21.7 M value a decoding error or a display error? | This epic (ticket 002-08) | Neither; the value is a real counter and the loss accounting is wrong |
| Why does a 5 ms timeout behave as about 13 ms? | This epic (ticket 002-12; informational only) | None needed; the runtime paces by deadline |
