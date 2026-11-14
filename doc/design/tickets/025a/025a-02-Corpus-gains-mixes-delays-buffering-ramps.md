# 025a-02 — Corpus: gains, mixes, delays, lookahead, buffering, ramps
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** M · **Depends on:** 025a-01
## Context
The conformance corpus in `hadronyche-dsl/conformance/corpus/` must hold **at least 40** DSL programs with hand-derived expectations, covering: gains, mixes, explicit delays, lookahead, linear-phase FIR, block buffering, recursive filters, ramps, table oscillators, resampler chains inside edge modules (96→48, 48→96 and the round trip; the resampler node exists only at 2:1 and 1:2 and only in edge modules, C-105, C-134), latency joins, sparse-controller instruments, transport readers; in float and fixed variants. The corpus is split across tickets 025a-02 to 025a-06. This ticket covers the simple, constant-delay families.

Rules the hand derivations rely on:
- Structural latency L is always a whole number of **domain-rate samples** (C-105). It is the sum of explicit delays, lookahead, block buffering and linear-phase filter centres. The runtime stamps output at p − L.
- Kit only: strict f32, Q1.31 / Q0.32 fixed point, shipped tables, the resampler node (2:1 and 1:2 only, and only inside an edge module, C-105, C-134). No libm, no transcendental functions; float operations are never reordered.
- Determinism class: every accepted DSL program is `exact-fixed` or `exact-float`; the compiler never assigns `bounded` (C-123).
- Cost bound: a finite upper bound per slice in counted abstract operations (C-106).
- Domain rate 96 kHz (C-01); slice = 48 frames = 0.5 ms (C-02); 4-byte samples (C-03); 6 channels per stream (C-04).

Expect format (from 025a-01): per output `L` (whole samples, with an optional `non-constant` flag), `class`, `deps`, `joins`, `cost_bound`.
## Work
Add at least 14 programs, each in a float and a fixed variant where the kit allows, each with an `expect/` file whose values are derived by hand and the derivation written as a comment in the program:
1. Gains: single-channel gain, 6-channel gain (C-04).
2. Mixes: 2-input and N-input mix of equal-latency inputs; a mix where one output depends on a subset of inputs (exercises `deps`). Mixes of paths with different latency are 025a-06.
3. Explicit delays: 1 sample, 48 samples (one slice), 1 000 samples (spans many slices).
4. Lookahead: a lookahead of k samples (L = k).
5. Block buffering: a block size that is not a multiple of 48 frames, and one that is.
6. Ramps: linear ramp from current to target over the block.
7. A program combining delay + lookahead + block buffering in series, so L is a sum.
## Acceptance criteria
1. At least 14 new programs in `corpus/`, each with a matching `expect/` file.
2. Each program contains a written hand derivation of its L, class, deps and cost bound.
3. The 025a-01 harness passes on all of them (compiler L, class, deps equal the expectations; cost bound finite).
4. At least one program has two outputs with different L and different dependency sets.
5. No expect file states `bounded` (C-123).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Any compiler disagreement found is filed against epic 023/024 with an owner, not patched by editing the expectation
## Out of scope
- Filters and oscillators (025a-03), resamplers (025a-04), controllers and transport (025a-05), latency joins (025a-06), probe measurement of L (025a-15), exact vectors (025a-09).
