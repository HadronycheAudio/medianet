# 023-26 — `LANGUAGE.md` language reference and node table
**Epic:** 023-DSL-compiler-front-end · **Effort:** M · **Depends on:** 023-11, 023-12, 023-15, 023-16, 023-18, 023-24, 023-28, 023-29, 023-30

## Context
- Deliverable: `LANGUAGE.md` — the language reference of `hadronyche-dsl`, shipped in the crate.
- DoD: "`LANGUAGE.md` covers every construct with one example" and "The closed node set is listed with, per node, its latency, cost and determinism contribution".
- Closed node set: arithmetic, delay, table lookup, linear ramp, mix, the resampler node (2:1 and 1:2 only, C-105; edge modules only, C-134), state cells. No unbounded loops, no recursion, no allocation, no libm or transcendental functions; arithmetic kit = strict f32, Q1.31, Q0.32, shipped deterministic tables; float ops never reordered, summation order defined by DSL semantics `[decision]`. The DSL has no non-deterministic node in v1 (C-123).
- Content types: audio f32 / Q1.31 (4-byte samples, C-03), dense controllers, sparse keyed controllers, read-only transport state; no transport authoring in v1.
- Rates: 96 kHz is the only wire rate (C-01); a device-facing side at another rate exists only in edge modules, converted through the resampler (C-80, C-105). Slice 48 frames = 0.5 ms (C-02); 6 channels per stream, no per-channel id byte, channel map in the manifest (C-04); bundles ≤ 32 members, a controller member included, so at most 31 audio members (186 channels) for a source with controllers (C-08, C-209); ≤ 256 streams per source (C-09).
- Latency is always a whole number of domain-rate samples (C-105); a resampler's conversion delay is counted only in the edge's D, never in L, and is reported separately on the device-facing port (C-134, C-207); joins of unequal paths are padded with whole-sample delay lines unless opted out, and a join with a frequency-dependent path is padded on its structural part and flagged (C-104). Cost is in abstract operations per slice (C-106). Degraded marking follows the content (C-122).
- Gap defaults: audio zero-fill; dense controllers hold last value; sparse keyed controllers rely on state refresh; overridable; transport state holds the last known state with the playhead advanced by the positions elapsed while playing and not advanced while stopped, output marked degraded (C-141, C-200, 023-30); each dense controller has an initial value, default 0 (C-158, 023-12); before any transport state has arrived the input reads stopped, playhead at zero, tempo 120 beats per minute (C-184, C-199, 023-30).
- Controllers of a single-stream source: shared with the audio packet while headers plus controllers fit the 100-byte cap, a second stream beyond it; the author may always opt in to the second stream (C-181, C-137, 023-31). A bundle source's controllers always ride in their own member stream (C-205, 023-32).
- Determinism classes: exact-fixed, exact-float; bounded only at runtime or for hand-written modules (C-123).
- Diagnostics: every diagnostic code from 023-02 onwards, including the even-tap linear-phase FIR rejection (C-183, 023-10) the device-side join rejection that suggests the opt-out (C-190, 023-28) and the rejection of a source with controllers wider than 31 audio members (C-209, 023-32).

## Work
1. Write `hadronyche-dsl/LANGUAGE.md`: lexical structure, module and port declarations, bundles, edge modules and device-facing ports, every node, state cells, gap overrides, the join opt-out, transport reads, what is forbidden and the diagnostic code each produces, the manifest fields each construct affects.
2. One example per construct; every example is extracted and run through `check()` in a test (doc test or extracted-snippet test).
3. Node table: per node, latency (023-11, 023-15), cost (023-16), determinism contribution (023-18); move the table started in 023-09 here.
4. Table of diagnostic codes with the rule each names.
5. Evidence tags on factual claims.

## Acceptance criteria
1. A test enumerates every grammar construct, every `ir::Node` variant and every `DiagCode` and fails if any lacks an example, table row or code entry in `LANGUAGE.md`.
2. All examples pass `check()` (or, for documented rejections, fail with the documented diagnostic code).
3. The node table has 7 rows (arithmetic, delay, table lookup, linear ramp, mix, resampler, state cell), each with all 3 columns filled.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] `LANGUAGE.md` covers every construct with one example
- [ ] The closed node set is listed with, per node, its latency, cost and determinism contribution
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Code-generation details (epic 024).
