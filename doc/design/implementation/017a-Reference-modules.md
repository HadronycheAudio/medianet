# 017a — Reference modules

**Phase:** P4 · **Scope:** L (13 tickets, about 21 days) · **Depends on:** 002, 009, 010, 013, 015 · **Blocks:** 017b, 019, 020, 025, 027

## Goal

Port the POC's test instrument — sender, echo (synth) and mixer — onto the new runtime as the permanent **reference modules**, with the ported instrument, analyzer and transport pieces they need, and prove that they reproduce their own replicas bit for bit on two different machines. This is the first of the two parts of epic 017 (C-143); the second, 017b, builds the L0 loopback harness and its scenarios around these modules.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's, over IPv4 only, with multicast at TTL 1 (C-100). Modules exchange *slices* — all content of one stream for one stretch of media time — addressed by *position* on a domain-wide timeline (`position = (network_time − epoch_origin) × rate`, where `rate` is the domain rate of 96 kHz, the only rate on the wire, C-01). Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers.

**Split of epic 017 (C-143).** Epic 017 exceeded 40 days by ticket tally and is split into 017a (this part: the reference modules) and 017b (the injection API, test-only hooks, the `hadronyche-l0` binary and its 15 scenarios, and the CI gate). 017b depends on 017a: its scenarios drive these modules and its runner uses this part's analyzer. A reference elsewhere to "epic 017" means both parts.

**What the reference modules are for `[decision]`.** The POC's bit-exact claim is a property of its *test instrument*, not of a transport: the sender reruns the echo's code and compares. The runtime keeps that instrument permanently as reference modules. Verification has tiers:

| Tier | Proves | Mechanism |
|---|---|---|
| Transport integrity | Bytes delivered equal bytes sent, per slice | Slice checksum |
| Exact | Deterministic modules produce identical bits | The sender's replica runs the same code; compare |
| Tolerance | Non-exact modules are within stated epsilons | Analyzer windows and per-voice projections |

A reference module is hand-written, not DSL-compiled. Where it claims exactness it is checked on the exact tier; where it does not, its output falls in the bounded class (C-123) and is checked on the tolerance tier. Whether the voices claim exactness is the first open sub-question below (owner task 002).

**The POC instrument `[code]`.**

- Sender (`src/sender.rs`): owns the media clock; at each block it prepares (`:209`) and fires (`:290`) two datagrams with the same header — musical-event parts to the synths, a multi-channel audio feed to the mixer — and compares what comes back (`:368`).
- Synth (`src/synth.rs`) with `music::Voices` (`src/music.rs:287`): plays voices from parts. A *part* is one field of one voice moving linearly to a value over the block — a **sparse keyed controller**, repaired by re-sending current values. Float rendering uses `sine()` (`music.rs:263`), built from additions and multiplications only "so that it gives the same bits on every machine". **Hole to close:** the fixed-point table is built with libm `f64::sin` on each machine (`music.rs:311`); ship it as a constant or generate it with add/multiply only.
- Mixer (`src/mixer.rs`): sums the feed with a controller-set volume, and mixes synth inputs **on arrival** using each input's latest block, with no alignment (`mixer.rs:108-124`). That behaviour is retired: the reference mixer becomes a position-aligned fan-in.
- Analyzer (`src/audio.rs`): 1 024-sample windows (`:26`). A window is compared only when complete on both sides; a window still incomplete 256 windows later (`MAX_LAG_WINDOWS`, `:31`, about 2.7 s at 96 kHz) is counted lost or as a sender gap and never analysed. A synth silent for over a second gets a 3-second settling exclusion (`SETTLE_SECS`, `:34`).
- Loss handling today: the sender's replica never applies gaps; stateful divergence heals itself (state refresh; oscillator phase resets when the gate is zero).

**POC instrument bugs observed in a real run `[measured]`.** A 25-second loopback that looked green had: a corrupt metrics CSV (four `File::create` handles on one path, `module.rs:1354`); a "lost" counter random-walking around 21.7 million with zero actual loss; and the entire music and transport check families reporting "0 of 0 windows". Task 002 (POC archaeology) diagnoses these and reads the files the design never read (`analyse` in `audio.rs`, most of `sender.rs`, the body of `transport.rs`). **Do not cite the POC as the exact-tier baseline until task 002 is done.** The analyzer of this part exposes a sample count per check family so that 017b can fail any run with an inert family.

**Rules for the ported modules `[decision]`.**

- Hand-written Rust first; re-expressed as DSL programs in task 025.
- Content domains: audio; dense controllers; sparse keyed controllers; read-only transport state. Transport *authoring* stays in the reference sender only (it is deferred in the DSL).
- Audio payload at defaults: 6 channels of 48 samples of 4 bytes, that is 192 bytes per channel and 1 152 bytes of audio per slice, with no per-channel id byte; the channel map is in the manifest (C-03, C-04). A default slice fits 1 252 bytes of UDP payload: 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum; in-band announcements and the measurement extension fit inside that cap or are omitted on a fallback path (C-137).
- The reference sender assigns one session number per stream, bumped on restart and on timeline epoch change; every consumer of a stream sees the same value (C-19).
- **Where the sender's controllers travel (C-181).** The reference sender is a single-audio-stream source (one 6-channel feed), so its controllers share the audio feed packet while headers plus controllers stay within the 100-byte cap (C-137); beyond that they go in a second stream. The mixer's volume rides in the audio feed packet; the parts for the echo are the sender's second, controller stream. No compiler checks a hand-written module, so the sender's own tests check this from its manifest.
- **Structural latency.** Each reference module declares its structural latency L in its manifest as a whole number of domain-rate samples (C-105). No reference module contains a resampler node: a resampler may appear only inside an edge module (C-134).
- **Reference mixer on a gap:** a missing input contributes zero and the output slice is marked *degraded*.
- **Degraded flag through a module with latency (C-122).** Stamps denote content position: while a module processes input position p it emits the output stamped p − L, whose content came from input position p − L; that output is degraded if the input at p − L was gapped, so the flag is delayed by L together with the content. This applies to the reference echo as well as to the mixer.
- **Transport state on a gap (C-141):** the last known state is held and the playhead advanced as that state dictates: a playing transport moves on at its tempo, a stopped one does not advance (C-200); the output is marked degraded. **Before any transport state has arrived (C-184):** stopped, playhead at zero, tempo 120 beats per minute (C-199).
- **Exact tier and loss:** windows touched by loss or by a degraded slice are excluded and counted; the reference instrument must resynchronise within a stated bound (inherit: 3 s settling; the bound is measured and decided by 017b's `loss` scenario).
- The reference echo is arrival-driven (single input); the reference mixer is a fan-in that fires at the fixed time `T(p) + offset`. The **offset** is the input's alignment offset from task 009: measured during aligning (4 s, C-50) as the maximum observed lateness plus headroom (C-51, C-52), then raised by the ratchet one slice period at a time up to the ceiling A (C-53), the last step landing on A exactly when less than one slice period remains (C-163). Alignment rings are sized to A (C-96).
- **The mixer's A (C-142).** A is set per presentation group with no default (C-55). The mixer, a fan-in outside any presentation group, inherits the A of the downstream presentation group it feeds (the largest if several), learned at admission; with no downstream group it needs an explicit budget in configuration, or its aligned inputs are refused. In L0 and in the cross-machine check the mixer feeds the reference sender, so it is given an explicit budget. With no downstream group and no budget the refusal reason is NoSkewBudget (C-185).
- **A changed while the mixer's inputs are live (C-186, C-198, C-208, C-211).** The receiver report carries the consumer's current A, its own or the one it inherited, so a change reaches the mixer one hop per report and the mixer's own reports carry its current A upstream (C-198). When the A it inherited changes, the mixer re-enters aligning with rings and ratchet ceiling resized (C-186), at a position it picks itself one lead (C-43) ahead and announces in its own output (C-208). If the new A needs more ring than its buffer memory holds, it keeps the deepest ring it can hold as its ceiling, raises the cannot-meet-A alarm (C-174) and keeps its streams running (C-211). The re-alignment mechanism is epic 009's; this part wires the mixer to it (ticket 017a-13).
- A dense controller declares an initial value in the manifest (default 0), used when a gap precedes any received value (C-158). The reference dense controller is the mixer's volume (C-192).
- Multi-packet slices: an arrival-driven input (the echo) reassembles within 2 slice periods (C-15); an aligned input (the mixer) reassembles until its own deadline for that position (C-110).
- No libm in any reference module's signal path.

**Gap classes the reference modules exhibit `[decision]`** (implemented by task 009; asserted in L0 by 017b).

- Late versus missing: at the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span it is reclassified late (missing −1, late +1) and discarded (C-108).
- Interrupted: after 8 consecutive missed slices (C-38). Only slices absent at the deadline count as misses — missing, or arriving late; a marked gap or a slice on another timeline epoch proves the upstream alive and resets the count (C-102).
- Dead: 2 s (C-39) counted from the last slice received with content or a marked gap (C-111); a slice on another timeline epoch also restarts the dead clock and has its own counter (C-135).
- Persistently late: at least one late slice in each of 10 consecutive 1 s windows (C-101) triggers a ratchet step (C-53, C-54). An input persistently late with its offset already at A enters the health state **Demoted**: treated as gapped, counted separately, cleared only by re-admission (C-109).

**Cross-machine bit-exactness.** The ported modules must reproduce their own replicas bit for bit on two different machines, float and fixed. The machine pair is x86-64 and 64-bit ARM (C-151). The check needs two machines, so it runs outside the single-process L0 gate and records each machine's regime stamp (task 013) with the results.

**Where outputs live (C-130, C-133, C-171).** The reference modules live in the `hadronyche-reference` crate; protocol types in `hadronyche-core`. The cross-machine result goes to `doc/design/reports/`; evidence to `doc/design/06-evidence-appendix.md`; new constants to `doc/design/02-constants.md`; the architecture specification is `doc/design/01-architecture.md`. Task 002's archaeology findings, which provenance headers cite, are in `doc/design/reports/`. Nothing is written into the frozen POC tree. Evidence tags include `[standard]` for a figure cited from a named, published standard with its clause (C-132).

**Fuzzing.** Budget for every touched parser: 60 s on every change and 1 hour nightly, corpus kept (C-95); the fuzzing tool is the implementing team's choice and the vectors crate sits at the workspace root (C-192). The parsers this part touches are the echo's parts payload decoder and the mixer's audio and controller payload decoder.

**Constants used.** Domain rate 96 kHz, the only rate on the wire (C-01); slice 48 frames = 0.5 ms = 2 000 slices/s (C-02); 4-byte samples (C-03); 6 channels per stream, 192 bytes per channel per slice, 1 152 bytes of audio, no per-channel id byte (C-04); single-packet default slice, multi-packet regime stamped (C-05); members per bundle at most 32, the controller member included (C-08, C-209); reassembly deadlines (C-15, C-110); session number ownership (C-19); aligning 4 s (C-50); offset estimator and headroom (C-51, C-52); ratchet one slice period, ceiling A (C-53), after 10 s dwell (C-54) expressed as persistent lateness (C-101), last step to A (C-163); A per presentation group, no default (C-55); fuzz budget (C-95); rings sized to A (C-96); IPv4 only (C-100); interrupted, dead and foreign-epoch rules (C-38, C-39, C-102, C-111, C-135); whole-sample L (C-105); late versus missing (C-108); Demoted (C-109); degraded flag with latency (C-122); bounded class (C-123); output and code locations (C-130, C-133, C-171); `[standard]` tag (C-132); resampler only in edge modules (C-134); header cap (C-137); transport state on a gap (C-141); fan-in A (C-142); machine pair (C-151); controllers in the audio packet within the cap, else a second stream (C-181); refusal reason NoSkewBudget (C-185); changed A at a fan-in, carried by the receiver report, re-alignment position, A beyond buffer memory (C-186, C-198, C-208, C-211); tool presentation details decided here (C-206); dense-controller initial value (C-158); transport state before any has arrived (C-184); default tempo 120 beats per minute (C-199); playhead held through a gap while stopped (C-200); smaller settled points: mixer volume as the reference dense controller, fuzzing tool, vectors crate (C-192).

## Deliverables

Crate `hadronyche-reference`:

- `sender.rs`, `echo.rs`, `mixer.rs` — the three reference modules against the runtime's module API.
- `instrument/voices.rs`, `instrument/phrase.rs` — ported from `music.rs`; fixed-point table shipped as a constant.
- `analyzer/` — ported windows, projections and comparison (tolerance and exact tiers), with per-family sample counts.
- `transport/` — ported state machine and its two existing tests (`transport.rs:864`), with a read-only view that holds state on a gap and advances the playhead only while playing (C-141, C-200).
- Fuzz targets for the echo's and the mixer's payload decoders.
- A cross-machine bit-exactness report in `doc/design/reports/`. Its file name, and the layout of the analyzer's report, are decided by this part and recorded in its documentation; only what 017b's runner depends on (per-family counts, inert families, mismatches) is a contract (C-206).

## Dependencies

- **Needs:** archaeology results (002) before any exact-tier claim; alignment ring and gap classes (009); packetiser and announcements (010); the stamp (013) for recording the cross-machine runs; the hosted runtime and its module API (015).
- **Provides:** the reference modules and analyzer that epic 017b's L0 harness drives; the reference peers for L1 (019), L2 (020) and L3 (027); the programs task 025 re-expresses in the DSL.
- **Between parts:** 017b depends on 017a; 017a does not depend on 017b.

## Acceptance criteria

1. **AC-017a.1 (echo).** Driven by 20 000 parts slices (10 s), the reference echo's output is bit-identical to `instrument::voices` rendering the same parts offline, in float and in fixed mode; each output carries the content position of its input, L being a whole number of samples (C-105, C-122); each dropped input slice yields exactly one degraded output slice at that position.
2. **AC-017a.2 (mixer).** The reference mixer fires at `T(p) + offset`, never on arrival; its output equals `mix_offline` bit for bit; a missing input contributes zero and marks the output degraded; a late arrival within the ring span is counted missing then reclassified late (C-108); with no downstream presentation group and no explicit budget its aligned inputs are refused with reason NoSkewBudget (C-142, C-185); a change of the inherited A carried by a receiver report makes it re-align at an announced position of its own choosing with ring and ceiling resized (C-186, C-198, C-208), and an A beyond its buffer memory leaves it at its deepest ring with the cannot-meet-A alarm raised and its streams running (C-211).
3. **AC-017a.3 (sender and replicas).** Sender → echo → sender and sender → mixer → sender on loopback with no injection for 10 s give zero exact-tier mismatches with the `audio`, `music` and `transport` families each non-empty; every default audio packet, the volume controller included, has at most 1 252 bytes of UDP payload, controllers beyond the cap travelling in a second stream (C-137, C-181); restarting the sender bumps every stream's session number and consumers agree (C-19).
4. **AC-017a.4 (analyzer).** Every window ends in exactly one state; with zero injected loss over 25 s the lost count is exactly 0; the analyzer reports per-family sample counts and names every family with no compared window.
5. **AC-017a.5 (transport).** Both ported POC transport tests pass; echo and mixer cannot author transport; on a gap the state is held and the playhead advanced while playing, not while stopped (C-141, C-200); before any state has arrived the view reads stopped, playhead at zero, 120 beats per minute (C-184, C-199).
6. **AC-017a.6 (cross-machine).** Float and fixed modes give zero exact-tier mismatches over at least 60 s between an x86-64 and a 64-bit ARM machine (C-151), recorded as `[measured]` evidence.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The ported modules interoperate bit-exactly with their own replicas on two different machines (float and fixed)
- [ ] Each ported POC piece cites its source file and the archaeology finding that cleared it

**Not applicable in this part (C-125):** "Regime stamp fields touched are complete and documented" — this part adds no stamp field; it only reads stamps (the injection and test-hook fields are epic 017b's). Every other line has a subject: the part writes code and tests, touches two parsers, changes design documents, ports POC code and runs the bit-exactness check.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Are voice windows compared bit for bit in the POC, or only within tolerance? (`analyse` was never read) | Task 002 | Reading shows tolerance-only; the exact-tier description for voices changes and voices fall in the bounded class (C-123) |
| Tolerance-tier epsilons | Task 002 (reading of `analyse`) | The reading yields no epsilon, leaving the tolerance tier without an oracle |
| Resynchronisation bound after loss (inherit 3 s?) | Epic 017b (ticket 017b-11), from task 002 findings and the `loss` scenario's measurement | A loss scenario shows divergence lasting longer |
| Received-positions bitmap, letting the replica apply the echo's gaps | Deferred past v1 | Excluding loss-touched windows hides a real defect class |
| Archaeology finding ids (the findings live in `doc/design/reports/`, C-130) and the names of the two transport tests at `transport.rs:864` | Task 002 | Provenance headers or the transport port cannot be completed |
| `ModuleHandle`, `StreamHandle` and the module API | Task 015 | The reference modules cannot be written against them |
| When an arrival-driven module (the echo) emits the degraded output for an input position that never arrives | Task 009 | Consumers downstream of the echo count interrupted while the parts source is alive |
