# 025a — DSL conformance corpus, probe and reference programs

**Phase:** P5 · **Scope:** L (22 tickets, about 26 days) · **Depends on:** 004, 007, 017, 017b, 023, 024 · **Blocks:** 025b, 026

## Goal

Prove on the host that the DSL compiler's compile-time promises are true, and keep proving it in CI: the figures it derives (structural latency, cost bound as a count of abstract operations, stack and state size, inserted join delays), the determinism class it assigns, and the gap behaviour it generates. Deliver the **conformance corpus** with hand-derived expectations and host vectors, the CI job `dsl-conformance`, the **latency probe tool**, and the **reference echo and mixer re-expressed as DSL programs**, shown bit-identical to the hand-written modules.

This is the first part of epic 025, split under C-143 because the whole epic exceeded 40 days by ticket tally. Part **025b** (record and replay, cross-target fingerprint suite, cost calibration and the listed-target set) builds on this part's corpus, vectors, CI job and DSL echo. A reference elsewhere to "epic 025" means both parts.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules exchange *slices* — all content of one stream for one stretch of media time — addressed by *position* on a domain-wide timeline. Every stream on the wire is at the domain rate, 96 kHz (C-01); an edge whose hardware runs at another rate converts at the edge. Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers. The module-construction DSL is one of the three product deliverables.

**Why this suite exists `[decision]`.** The proof of concept's bit-exact guarantee worked because its sender reran the receiver's exact code. That is impossible for arbitrary user modules — unless the module's code is generated, deterministic and replayable. The DSL revives the exact tier for **user** modules, and moves trust from each module author to the compiler. The compiler therefore needs its own conformance suite.

**Verification tiers.**

| Tier | Proves | Mechanism |
|---|---|---|
| Exact | exact-class modules produce identical bits | Record input slices and output hashes; replay the same generated code offline; compare (this part: host vectors; replay and cross-target runs are part 025b) |
| Tolerance | bounded-class modules are within stated epsilons | Analyzer windows and per-voice projections |

**The compiler's promises under test.** The compiler (front end 023, generator, ABI and kit 024) implements every promise below, including the fingerprint; this epic tests them and files any defect against 023 or 024. It never patches an expectation to match the compiler.

1. **Process function:** `process(state, inputs at position p with per-input gap flags, position) -> outputs`; never blocks; **no allocation**; state size and worst-case stack emitted as constants (`state_bytes`, `stack_bytes`).
2. **No libm, no transcendental functions**; only the kit (strict f32, Q1.31 / Q0.32 fixed point, shipped tables, the resampler node at 2:1 and 1:2 only, C-105). Float operations never reordered.
3. **Cost bound:** an upper bound per slice in **counted abstract operations**, identical on every machine. Each target has a calibration factor (time per abstract operation), measured once and recorded in the regime stamp; admission converts the bound to time with it (C-106). This part checks the operation count; the time half and the factors are part 025b.
4. **Structural latency L**, a compile-time constant, **always a whole number of domain-rate samples** in the manifest and the ABI (C-105; the earlier rational-L rule is withdrawn): explicit delays, lookahead, block buffering, linear-phase filter centres. L is the module's structural delay in the domain-rate timeline; an edge's resampler delay is never part of it (C-207). An output reached through a recursive filter is additionally flagged as having non-constant (frequency-dependent) delay, a property beside its structural L (C-104). The runtime stamps output at p − L.
5. **Multi-rate** only through the kit's resampler node, at 2:1 or 1:2, whose delay is a whole number of samples; other ratios are out of v1 (C-105). The node may appear only inside an edge module, and the compiler rejects it anywhere else; an edge's conversion delay is counted only in its plumbing delay D and never in L, and the manifest reports it separately, on the device-facing port (C-134, C-170, C-207). An edge whose hardware runs at 48 kHz converts 2:1 through this node (C-80).
6. **Determinism classes:**
   - **exact-fixed** — bit-exact everywhere, including between a host and a device;
   - **exact-float** — bit-exact only where the startup **fingerprint** matches;
   - **bounded** — a runtime outcome, never a compiler assignment: an exact-float module whose fingerprint mismatched, or a hand-written (non-DSL) module that does not claim exactness. The DSL has no non-deterministic node in v1, so every accepted DSL program is exact-fixed or exact-float (C-123).
7. **Fingerprint:** a compile-time vector (including subnormal and NaN paths) evaluated at start **on the processing thread itself**, because floating-point modes are per thread and audio stacks change them. Match ⇒ exact-float eligible. Mismatch ⇒ the module runs bounded-class and the regime stamp says so. **The listed-target set is whatever CI verified; nothing else claims exactness.** Tested in part 025b.
8. **Gap defaults** per content type, overridable: audio zero-fill; dense controllers hold last value, and each dense controller declares an initial value in the manifest (default 0), used when a gap precedes any received value (C-158); sparse keyed controllers rely on state refresh; read-only transport state holds the last known state and advances the playhead as that state dictates: a stopped transport stays where it is, a playing one moves on at its tempo, and the output is marked degraded (C-141, C-200); before any transport state has arrived the state is stopped, playhead at zero, tempo 120 beats per minute (C-184, C-199). **Degraded marking per output from dataflow**: the degraded flag follows the content, so an output at position p is degraded if its input at p − L was gapped (C-122), L being the output's whole-sample structural L also for a non-constant-delay output (C-104, C-105).
9. **Content domains v1:** audio; dense controllers; sparse keyed controllers; read-only transport state. Transport authoring is absent.
10. **Latency joins:** where paths of different latency meet, the compiler pads the shorter path with a whole-sample delay line, reports the inserted delay in the manifest, and the module's L is the longest path; an explicit construct opts out per join. A path with frequency-dependent delay is padded on its structural part only and the join is flagged in the manifest (C-104). On the device side of an edge module, paths that differ by an odd number of domain-rate samples cannot be padded at the device rate; the compiler rejects the join and suggests the per-join opt-out (C-190).
11. **Even-tap linear-phase FIR:** rejected by the compiler in v1, with a diagnostic suggesting an odd-tap design, so that L stays exact (C-183).
12. **Controller placement:** a bundle source's controllers always ride in their own member stream (C-205), and that member counts towards the 32 members of a bundle, so a source with controllers has at most 31 audio members (186 channels); a wider source with controllers is rejected at compile time (C-08, C-209). A single-stream source's controllers share the audio packet while headers plus controllers stay within the 100-byte header cap (C-137); beyond that the source emits a second stream for them. The compiler checks this from the manifest (C-181).

**The reference mixer's gap rule `[decision]`.** A missing input contributes zero and the output slice is marked degraded.

**Positional delay cannot be seen live.** Timestamps show how long a module took (residence), not whether the content in the slice stamped p is really input p − L. A wrong L silently skews every downstream alignment. The only check is a **probe**: send a marker through the module in test mode and locate it in the output. The probe reports delay as a whole number of domain-rate samples, so it is compared with L exactly (C-105), and reports non-constant delay as such.

**The reference instrument `[code]`.** The POC's echo is a synthesiser (`src/music.rs:287` `Voices`): per voice, fundamental at `pitch` with level `LEVEL × gate × pressure`; second harmonic scaled by timbre; third by articulation; phase restarts at 0 when the gate is 0; each field moves linearly from current to target over the block. Float path uses an add/multiply sine (`:263`); fixed path uses a table (`:311`, to be shipped as a constant). Task 017 ported these by hand into `hadronyche-reference`. This part re-expresses them in the DSL and shows the two agree bit for bit.

**Where outputs live (C-130, C-133).** Specifications this part writes (expect format) go to `doc/design/spec/`; constant changes go to `doc/design/02-constants.md` first. Tool binaries live in the `hadronyche-tools` crate. Fuzzing follows the fuzz budget: each parser 60 s on every change and 1 hour nightly, corpus kept (C-95).

**Constants.** Domain rate 96 kHz, the only rate on the wire (C-01); slice 48 frames = 0.5 ms (C-02); 4-byte samples, f32 or Q1.31 (C-03); 6 channels per stream, 192 bytes per channel per slice, 1 152 bytes of audio, no per-channel id byte (C-04); edge hardware at 48 kHz converts 2:1 through the resampler node (C-80). One minute is 120 000 slices.

**Settled since the ticket split.** L is always a whole number of domain-rate samples and the earlier rational-L rule is withdrawn (C-105 revised): the 96→44.1 and 44.1→96 corpus chains, the probe's fractional measurement method and rational comparison are gone; such ratios are now negative corpus. A resampler node outside an edge module is rejected (C-134). An edge's resampler delay is counted only in D and never in L; the manifest reports the conversion delay separately, on the device-facing port (C-207). A join with a frequency-dependent path is padded on its structural part only and flagged (C-104 revised). The degraded flag of a non-constant-delay output follows its whole-sample structural L (C-104, C-105, C-122). Transport-state gap rule: last known state held, a stopped transport stays where it is and a playing one moves on at its tempo (C-141 revised, C-200); transport state before any has arrived is stopped, playhead at zero, tempo 120 beats per minute (C-184, C-199). Dense-controller initial value (C-158). Controller placement: own member stream for a bundle source, which counts towards the 32-member cap, so a wider source with controllers is rejected at compile time; shared packet or a second stream for a single-stream source, checked by the compiler from the manifest (C-181, C-205, C-209, C-08); tested in 025a-22. Presentation details of the probe (output formats) are decided by this part and recorded in its documentation (C-206). An even-tap linear-phase FIR is rejected with a diagnostic suggesting an odd-tap design (C-183). A join on the device side of an edge module whose paths differ by an odd number of domain-rate samples is rejected, with the per-join opt-out suggested (C-190). Float and fixed variants of a corpus program count as one program; the reference dense controller is the mixer's volume; the L0 baseline scenario is defined by epic 017b (C-192). Earlier: probe result shape; `Marker` defined here and `Module` shared by generated (024) and hand-written (017) modules; cost unit (C-106); class of DSL programs (C-123); degraded flag with latency (C-122).

## Deliverables

Crate `hadronyche-dsl`, directory `conformance/`:

- `corpus/` — at least 40 DSL programs with hand-derived expectations: gains, mixes, explicit delays, lookahead, linear-phase FIR (odd tap counts), block buffering, recursive filters, ramps, table oscillators, resampler chains inside edge modules (96→48, 48→96 and the round trip), latency joins (padded, opted out, and flagged with a frequency-dependent path), sparse-controller instruments, dense-controller consumers, transport readers; in float and fixed variants, which count as one program (C-192). Plus at least 25 negative programs.
- `expect/` — per program: expected L per output (whole domain-rate samples, with a non-constant flag where the delay is frequency-dependent), expected determinism class (exact-fixed or exact-float), expected dependency sets, expected inserted delay or flag per join, for an edge module the conversion delay reported on the device-facing port (C-207), dense-controller initial values, controller placement for a program that emits controllers (shared packet, own member stream or second stream, C-181, C-205), a finite cost bound; for negative programs the expected diagnostic.
- `vectors/` — per positive program (all are exact-class, C-123): input slices and expected output hashes, generated by the compiler (task 024) and checked in.

Crate `hadronyche-tools`:

- `hadronyche-probe` — **latency probe**. Instantiates a module in test mode, feeds a marker (impulse or pseudo-random code) at a known position, locates it in each output by cross-correlation, reports measured structural delay per output as a whole number of domain-rate samples, or non-constant when the delay varies with the marker spectrum. Works on generated and hand-written modules.

CI job:

- `dsl-conformance` — compiles the corpus, compares L, class, dependencies, join delays and flags, edge conversion delays (C-207), controller placement (C-181); runs vectors on the host target; checks executed abstract operations against the bound (target-independent, C-106); runs the probe, gap, memory and reference-equivalence tests.

Crate `hadronyche-reference`: `dsl/echo.hdsl`, `dsl/mixer.hdsl` — the reference echo and mixer as DSL programs, built in CI.

Shapes (names may be refined; `Samples` is the ABI's whole-number type for L, C-105; `Module` is the module interface shared by generated and hand-written modules):

```rust
pub enum Marker { Impulse, PseudoRandom { seed: u64, len: u32 } }
pub enum Delay { Constant(Samples), NonConstant { min: Samples, max: Samples } }
pub struct ProbeResult { pub output: usize, pub delay: Delay, pub confidence: f32 }
pub fn probe<M: Module>(marker: Marker, slices: usize) -> Vec<ProbeResult>;
```

## Dependencies

- **Needs:** the wire layouts (004), for the controller sizes the placement derivations use (025a-22); the conformance vector format (007); hand-written reference modules and the L0 harness (017); the L0 baseline scenario (017b, C-192); the front end, its analyses and diagnostics (023); the generator, ABI, kit and operation-counting instrumentation (024).
- **Provides:** to part 025b, the corpus, the host vectors, the `dsl-conformance` job, the DSL echo and mixer, and the operation counts; DSL versions of the reference modules; with 025b, the verified compiler that the embedded device (026) relies on.

## Acceptance criteria

1. **AC-33, latency.** For every corpus program, the compiler's structural L equals the probe-measured value **exactly**, as a whole number of domain-rate samples, per output (C-105). Outputs flagged non-constant-delay are reported as such by the probe (delay varies with marker spectrum). For an edge module's resampler path, L excludes the resampler delay (C-207); the probe-measured delay equals L plus the conversion delay the manifest reports on the device-facing port.
2. **AC-33, cost (operation count).** For every corpus program, the executed abstract-operation count per slice never exceeds the cost bound (C-106). The time half is part 025b.
3. **Reference equivalence.** The DSL echo and mixer produce the same bits as the hand-written reference modules of task 017, float and fixed, over the L0 baseline scenario (defined by epic 017b, C-192).
4. **Gap behaviour.** For each content type, the generated default matches the specification, including the transport-state rule (a stopped transport stays where it is, a playing one moves on at its tempo, C-141, C-200), the transport state before any has arrived (C-184) and the dense-controller initial value (C-158); an output at p is marked degraded exactly when its input at p − L was gapped (C-122); the reference mixer program contributes zero for a missing input and marks its output degraded.
5. **Memory.** Measured stack never exceeds `stack_bytes`; state size equals `state_bytes`.
6. **Negative corpus.** At least 15 programs that must be rejected (unbounded loop, transcendental call, allocation, cross-rate without the resampler, transport authoring, a resampler node outside an edge module (C-134), a resampler ratio other than 2:1 or 1:2 (C-105), an even-tap linear-phase FIR (C-183), a device-side join of paths differing by an odd number of domain-rate samples (C-190), a source with controllers and more than 31 audio members (C-209)) are rejected with the expected diagnostic.
7. **Latency joins.** For every join program, the manifest reports the inserted delay the hand derivation predicts and L equals the longest path; a join using the opt-out construct inserts no delay; a join with a frequency-dependent path is padded on its structural part only and flagged (C-104); a join on the device side of an edge module whose paths differ by an odd number of domain-rate samples is rejected, and its opt-out variant compiles (C-190).
8. **Controller placement.** For every program that emits controllers, the manifest places them as the hand derivation predicts: in their own member stream for a bundle source (C-205), counted towards the 32 members, with 31 audio members plus controllers compiling (C-209); in the audio packet for a single-stream source while headers plus controllers fit the 100-byte cap (C-137), and in a second stream beyond it (C-181).

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The listed-target set is published in the design docs, with the CI job that verifies each target
- [ ] The cost-bound calibration (operations to time, per target) is recorded
- [ ] Probe and replay tools have user documentation with one worked example each

**Not applicable in this part (C-125):** "Regime stamp fields touched are complete and documented" — this part touches no stamp field (the fingerprint and calibration fields are part 025b). "The listed-target set is published…" and "The cost-bound calibration … is recorded" — both are part 025b deliverables; this part runs on the host only. "Probe and replay tools have user documentation…" applies to the probe here; the replay half is part 025b. Every other line has a subject: code and the `dsl-conformance` job; the expect-file parser; the POC echo, read as a reference and never modified by this part (the frozen POC is written only by task 003, C-130).

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| ~~Structural latency for non-integer rate ratios; probe method for a rational L~~ **Decided (C-105 revised):** L is always a whole number of samples; ratios other than 2:1 and 1:2 are out of v1. No fractional probe method is needed | — | — |
| ~~Is a resampler node outside an edge module rejected?~~ **Decided (C-134):** rejected; tested in 025a-08 | — | — |
| ~~Join padding when one path has non-constant delay~~ **Decided (C-104 revised):** padded on the structural part only, join flagged; tested in 025a-06 | — | — |
| ~~Degraded flag when L is not an integer or the output has non-constant delay~~ **Decided (C-105, C-104, C-122):** L is always whole; a non-constant-delay output follows its structural L | — | — |
| ~~Gap default for read-only transport state~~ **Decided (C-141 revised, C-200, C-184):** last known state held; a stopped transport stays where it is, a playing one moves on at its tempo; output degraded; before any state has arrived, stopped at zero, 120 beats per minute (C-199); tested in 025a-16 | — | — |
| ~~Does the compiler check controller placement and the bundle cap?~~ **Decided (C-181, C-205, C-209, C-08):** yes, from the manifest; a source with controllers and more than 31 audio members is rejected; tested in 025a-22 | — | — |
| Do POC voice windows compare bit for bit? (determines what "equivalence with the POC" can mean) | Task 002 (archaeology) | Tolerance-only comparison in the POC |
| ~~Does an edge module's manifest report the resampler node's delay inside its L, or as a separate conversion delay counted only in the edge's D (C-134, C-170)?~~ **Decided (C-207):** counted only in D, never in L; the manifest reports the conversion delay separately, on the device-facing port; tested in 025a-01, 025a-04, 025a-15 | — | — |
| ~~How does the compiler treat an even-tap linear-phase FIR, whose centre falls between two samples?~~ **Decided (C-183):** rejected in v1 with a diagnostic suggesting an odd-tap design; tested in 025a-21 | — | — |
| ~~Do float and fixed variants count separately towards the "at least 40" corpus programs?~~ **Decided (C-192):** they count as one program; 025a-05 | — | — |
| Refresh interval of sparse keyed controller state (bounds how long a gap affects output) | Wire specification epic 004 | A sparse-keyed gap test cannot bound when normal output returns |
| ~~L0 baseline scenario~~ **Decided (C-192):** defined by epic 017b; 025a-20 | — | — |
| Diagnostic ids of the negative-corpus rejections | Epic 023 | A negative program cannot name its expected diagnostic |
| Definition of an abstract operation and an instrumented build that counts executed ones | Epic 024 | The executed count cannot be compared with the bound |
| Expect-file format; stack measurement method | This part (tickets 025a-01, 025a-17) | The chosen method gives different results on a rerun or misses a case the AC requires |
