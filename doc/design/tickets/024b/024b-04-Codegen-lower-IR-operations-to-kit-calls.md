# 024b-04 — Codegen: lower IR operations to kit calls
**Epic:** 024b-DSL-code-generator · **Effort:** M · **Depends on:** 024a-07, 024a-08, 024a-09, 024a-10, 024a-12, 024b-02, 024b-03

## Context
The process function body is lowered from checked IR (epic 023) into calls to `hadronyche-kit` only. Rules `[decision]`:
- No libm, no transcendental functions; only the kit: strict f32 operations; Q1.31 and Q0.32 fixed point; shipped deterministic tables; the 2:1 / 1:2 resampler node (C-105).
- Float operations are never reordered; no fused multiply-add contraction; summation order is defined by DSL semantics; no auto-vectorisation that changes results.
- Content domains v1: audio; dense controllers; sparse keyed controllers (one field of one voice moving linearly to a value over the slice); read-only transport state. No transport authoring.
- Position invariant: output for p is derived from inputs at p.
- Slice 48 frames at 96 kHz (C-01, C-02); 4-byte samples, f32 or Q1.31, channel-interleaved (C-03); 6 channels, 1 152 bytes of audio per slice, no per-channel id byte (C-04); bundles up to 32 members, the controller member included (C-08, C-209).
- The resampler node may appear only inside an edge module, and the compiler rejects it anywhere else (C-134); it is offered only at 2:1 and 1:2 (C-105). The front end (epic 023) performs the rejection, as it does every compile-time rejection (C-218); how the DSL declares an edge module and its hardware port is an open item owned by epic 023.
- No non-deterministic node exists in the DSL v1 (C-123).
- Subnormal handling follows the decision of 024b-02.

## Work
- In `emit.rs` (or a `lower.rs` sibling under `codegen/`): lowering of each IR operation to kit calls (`float::add/mul/sum_left/sine`, `fixed::*`, `ramp::*`, `resample::Resampler`), keeping DSL-defined order (explicit temporaries; no reassociation).
- Byte (de)serialisation of 4-byte channel-interleaved samples from `InputSlice.audio` / into `OutputSlice.audio`; dense controllers from `dense`; sparse parts via ramps; transport read-only.
- Reject (compile error with a named reason) any IR operation with no kit lowering.
- Define the epic's shared test corpus, reused by the later tickets of this part: a gain, a mix of two inputs, a ramped sparse controller, a fixed-point sine oscillator, a decaying feedback structure, an edge module with a 2:1 resampler, and a program with a padded join (the join program is completed in 024b-06). Each has an exact-fixed variant and, where it uses floats, an exact-float variant; the two variants count as one program (C-192). The mix carries a volume, the reference dense controller (C-192).
- Tests: the corpus generated, compiled and run, checked against reference outputs.

## Acceptance criteria
1. Each corpus program compiles `no_std` and produces outputs equal to a reference evaluator's bits on the hosted target for 100 slices.
2. Generated source calls no function outside `hadronyche-kit` and `core` (grep/AST test).
3. A sum of three terms in the DSL is emitted as `((a + b) + c)` in DSL order (snapshot test).
4. An IR operation without kit lowering produces a named error, not a panic.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Reporting latency (024b-05), gap defaults (024b-07, 024b-08, 024b-09), degraded marking (024b-10), fingerprint (024b-11).
