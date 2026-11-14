# 025b — Replay, cross-target fingerprint and cost calibration

**Phase:** P5 · **Scope:** L (13 tickets, about 17 days) · **Depends on:** 007, 013, 015, 017, 024, 025a · **Blocks:** 026

## Goal

Prove that the DSL compiler's exactness and cost promises hold beyond the host, and keep proving it in CI: exact-class modules are bit-identical across listed targets, the fingerprint gates exact-float correctly and catches every known divergence mechanism, and measured time stays within the cost bound converted with each target's calibration factor. Deliver the **record-and-replay** verifier for exact-class modules, the **cross-target fingerprint suite**, the **per-target calibration factors** and their regime stamp field, and the published **listed-target set**.

This is the second part of epic 025, split under C-143 because the whole epic exceeded 40 days by ticket tally. It depends on part **025a**, which delivers the conformance corpus, host vectors, the CI job `dsl-conformance`, the latency probe and the DSL reference echo and mixer. A reference elsewhere to "epic 025" means both parts.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules exchange *slices* — all content of one stream for one stretch of media time — addressed by *position* on a domain-wide timeline. Every stream on the wire is at the domain rate, 96 kHz (C-01); an edge whose hardware runs at another rate converts at the edge. Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers. The module-construction DSL is one of the three product deliverables.

**Why this suite exists `[decision]`.** The proof of concept's bit-exact guarantee worked because its sender reran the receiver's exact code. That is impossible for arbitrary user modules — unless the module's code is generated, deterministic and replayable. The DSL revives the exact tier for **user** modules, and moves trust from each module author to the compiler. The compiler therefore needs its own conformance suite.

**Verification tiers.**

| Tier | Proves | Mechanism |
|---|---|---|
| Exact | exact-class modules produce identical bits | Record input slices and output hashes; replay the same generated code offline; compare |
| Tolerance | bounded-class modules are within stated epsilons | Analyzer windows and per-voice projections |

**The compiler's promises under test.** The compiler (front end 023, generator, ABI and kit 024) implements every promise below, including the fingerprint; this epic tests them and files any defect against 023 or 024. It never patches an expectation to match the compiler.

1. **Process function:** `process(state, inputs at position p with per-input gap flags, position) -> outputs`; never blocks; **no allocation**; state size and worst-case stack emitted as constants (`state_bytes`, `stack_bytes`).
2. **No libm, no transcendental functions**; only the kit (strict f32, Q1.31 / Q0.32 fixed point, shipped tables, the resampler node at 2:1 and 1:2 only, C-105). Float operations never reordered.
3. **Cost bound:** an upper bound per slice in **counted abstract operations**, identical on every machine. Each target has a calibration factor (time per abstract operation), measured once and recorded in the regime stamp; admission converts the bound to time with it (C-106). The factor is obtained by running a reference workload once per listed target and recording time per operation (C-176). The operation-count check is part 025a; the time check and the factors are this part.
4. **Structural latency L**, a compile-time constant, **always a whole number of domain-rate samples** (C-105). The runtime stamps output at p − L. Checked by the probe in part 025a.
5. **Multi-rate** only through the kit's resampler node, at 2:1 or 1:2 (C-105), and only inside an edge module (C-134). An edge whose hardware runs at 48 kHz converts 2:1 through this node (C-80); the embedded device relies on host-to-device exactness of its fixed-point form. The node is fixed-ratio with whole-sample delay; an edge's steerable rate actuator, whose ratio varies by parts per million to track the hardware crystal, is a separate resampler whose delay is part of D (C-189). The kit, epic 024a, builds both, including the rate-actuator trait (C-197, C-210); this part tests only the node's exactness. Conversion delay is never counted in L: L is the module's structural delay in the domain-rate timeline, and the manifest reports the conversion delay separately on the device-facing port (C-134, C-207).
6. **Determinism classes:**
   - **exact-fixed** — bit-exact everywhere, including between a host and a device;
   - **exact-float** — bit-exact only where the startup **fingerprint** matches;
   - **bounded** — a runtime outcome, never a compiler assignment: an exact-float module whose fingerprint mismatched, or a hand-written (non-DSL) module that does not claim exactness. The DSL has no non-deterministic node in v1, so every accepted DSL program is exact-fixed or exact-float (C-123).
7. **Fingerprint:** a compile-time vector (including subnormal and NaN paths) evaluated at start **on the processing thread itself**, because floating-point modes are per thread and audio stacks change them. Match ⇒ exact-float eligible. Mismatch ⇒ the module runs bounded-class and the regime stamp says so. **The listed-target set is whatever CI verified; nothing else claims exactness.**
8. **Gap defaults and degraded marking:** the degraded flag follows the content; an output at position p is degraded if its input at p − L was gapped (C-122), so recorded degraded marks sit at the shifted positions.

**Exact tier and loss `[decision]`.** Windows of 1 024 samples touched by loss or by a degraded slice are excluded from exact comparison and counted; the count is part of the replay outcome. The expected clean-network loss floor (C-90) is far too low to produce loss in a one-minute run, so the replay acceptance test injects loss.

**Cross-target pair.** The machine pair for cross-machine bit-exactness is x86-64 and 64-bit ARM (C-151); the listed-target candidates are those two plus one 32-bit device target, which epic 026 names. Timing on an emulator gives no calibration factor (C-106).

**Regime stamp.** The stamp is a dedicated control message split into sections that each fit the control message bound; it is multicast to the control group on any regime change, refreshed every 10 s, and fetched immediately by a unicast request; the hello carries only a stamp version number (C-18). The stamp message is defined by epic 013 and sent by the hosted backend, epic 015 (C-144). This part touches two stamp subjects: the fingerprint result and effective determinism class of a module, and the per-target cost calibration factor (C-106, C-176).

**Recorder hand-off.** The runtime recorder writes through the single-writer file service, which the hosted backend, epic 015, provides (C-144).

**Strict mode.** C-147 requires strict mode for runs whose numbers enter the constants table; the calibration factors are recorded in `doc/design/reports/` and in the stamp, and the calibration report states whether its run was in strict mode (C-127).

**Where outputs live (C-130, C-133).** Specifications this part writes (recording format, listed-target set, fingerprint coverage map, stamp fields) go to `doc/design/spec/`; measured results (cost calibration, resampler exactness) go to `doc/design/reports/`; constant changes go to `doc/design/02-constants.md` first. Tool binaries live in the `hadronyche-tools` crate. Fuzzing follows the fuzz budget: each parser 60 s on every change and 1 hour nightly, corpus kept (C-95).

**Constants.** Domain rate 96 kHz, the only rate on the wire (C-01); slice 48 frames = 0.5 ms (C-02); 4-byte samples, f32 or Q1.31 (C-03); 6 channels per stream, 192 bytes per channel per slice, 1 152 bytes of audio, no per-channel id byte (C-04); edge hardware at 48 kHz converts 2:1 through the resampler node (C-80). One minute is 120 000 slices.

**Settled since the ticket split.** Single-writer file service and stamp sending belong to epic 015, the stamp message to epic 013 (C-144). Calibration procedure: a reference workload once per listed target (C-176). Cross-machine pair x86-64 and 64-bit ARM (C-151). The resampler exists only at 2:1 and 1:2 (C-105 revised), so the former 147/320 long-run exactness vectors are dropped. Two kinds of resampler: the fixed 2:1 / 1:2 node tested here and the steerable rate actuator, both built by the kit epic 024a (C-189, C-197, C-210); resampler delay is counted only in D, never in L (C-134, C-207). Fuzzing tool is the implementing team's choice (C-192). Presentation details of `hadronyche-replay` (output line format, documentation file name) are decided by this part and recorded in the tool's documentation; only what another epic or an operator's script depends on goes into `doc/design/spec/` (C-206). Earlier: replay outcome carries the excluded-window count; cost unit and calibration (C-106); class of DSL programs (C-123); degraded flag with latency (C-122); stamp transport (C-18).

## Deliverables

Crate `hadronyche-tools`:

- `hadronyche-replay` — **record and replay**. Reads a recording of the slices delivered to a module (with gap flags) and the hashes of what it emitted; reruns the module's code offline; reports the excluded-window count and the first differing slice.

Crate `hadronyche-std`: `record.rs` — the runtime-side recorder (input slices and output hashes for one module), written through the single-writer file service of epic 015 (C-144).

CI jobs:

- `dsl-cross-target` — runs exact vectors and the fingerprint on every listed target (x86-64, AArch64, one 32-bit device target under emulation or on hardware).
- `dsl-cost` — measures time per slice on each calibrated target against the bound converted with that target's calibration factor (C-106).

Also: the fingerprint mutation suite and its coverage map; the per-target calibration factors from the reference workload (C-176) and their regime stamp field; the published listed-target set.

Shapes (names may be refined; `Module` is the module interface shared by generated and hand-written modules):

```rust
pub enum ReplayOutcome {
    Identical { slices: u64, excluded_windows: u64 },
    Differs { first_slice: u64, output: usize, excluded_windows: u64 },
}
pub fn replay<M: Module>(recording: &Recording) -> ReplayOutcome;
```

## Dependencies

- **Needs:** part 025a (corpus, host vectors, `dsl-conformance`, DSL echo, operation counts); the conformance vector format (007); the regime stamp message and its fields (013); the hosted backend's single-writer file service and stamp sending (015, C-144); the L0 harness (017); the generator, ABI, kit and fingerprint (024).
- **Provides:** the listed-target set for exact-float; the per-target cost calibration factors that the regime stamp carries and admission uses (C-106, C-176); the verified compiler that the embedded device (026) relies on for host-to-device exactness.

## Acceptance criteria

1. **AC-33, cost (time).** On every listed target, measured time per slice never exceeds the bound multiplied by that target's recorded calibration factor (C-106), for every corpus program.
2. **AC-32, exact-fixed.** Every exact-fixed program is bit-identical on every listed target.
3. **AC-32, exact-float.** Every exact-float program is bit-identical on every target where the fingerprint matches.
4. **AC-32, fallback.** With a deliberately altered floating-point mode on the processing thread, the fingerprint mismatches, the module runs bounded-class (C-123), and the regime stamp records it (C-18).
5. **Fingerprint coverage.** Mutation test: for each known divergence mechanism (flush-to-zero, denormals-are-zero, contraction, extended precision), a build or mode exhibiting it is caught by the fingerprint. A target that passes the fingerprint and differs on any vector is a **failed suite**.
6. **Replay.** A one-minute L0 recording of the DSL echo under injected loss replays identically outside excluded windows, with the excluded-window count reported; a deliberately corrupted recording is reported at the correct slice.
7. **Calibration in the stamp.** On each listed target, the regime stamp carries that target's recorded calibration factor (C-106, C-176).

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

**Not applicable in this part (C-125):** none fully. "Probe and replay tools have user documentation…" applies to the replay tool here; the probe half is part 025a. Every other line has a subject: code and the two CI jobs; the recording parser; the fingerprint-result and calibration-factor stamp fields; the listed-target set and calibration report; the frozen POC, which this part never touches.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| ~~Cost-bound calibration per target (operations to nanoseconds)~~ **Decided (C-106, C-176):** the bound is in counted abstract operations; each target has one time-per-operation factor from a reference workload run once per listed target, recorded in the regime stamp. The values are a deliverable (definition of done) | — | — |
| ~~Home of the single-writer file service used by `record.rs`~~ **Decided (C-144):** epic 015 | — | — |
| Fingerprint vector coverage | This part (025b-08) | A target passes the fingerprint but differs on a conformance vector |
| Is the resampler node exact-fixed in its fixed-point form? | This part (025b-09) | Bits differ across targets |
| Which 32-bit device target is listed, and is it available on hardware in CI? Timing on an emulator gives no calibration factor (C-106) | Embedded device epic 026 | No 32-bit target can be both vector-checked and timed in CI |
| Regime stamp field names for fingerprint result, effective class and calibration factor | Regime stamp epic 013 | Ticket 025b-07 or 025b-12 cannot name the field it asserts on |
| L0 injected-loss rate and pattern | Epic 017 | The replay test cannot state its input |
| Recording format and output hash (reusing 007 where possible); excluded-window alignment (timeline or recording-relative); how to build the contraction and extended-precision mutants; content of the calibration reference workload | This part (tickets 025b-01, 025b-03, 025b-08, 025b-10) | The chosen method gives different results on a rerun or misses a case the AC requires |
