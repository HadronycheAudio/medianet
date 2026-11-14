# 025a-18 — Reference echo as a DSL program
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** M · **Depends on:** 025a-10, epic 017, epic 024
## Context
Deliverable: crate `hadronyche-reference`, file `dsl/echo.hdsl` — the reference echo re-expressed as a DSL program, built in CI.

The reference instrument `[code]`: the POC's echo is a synthesiser (`src/music.rs:287` `Voices`). Per voice:
- fundamental at `pitch` with level `LEVEL × gate × pressure`;
- second harmonic scaled by timbre;
- third harmonic scaled by articulation;
- phase restarts at 0 when the gate is 0;
- each field moves linearly from current to target over the block.
The float path uses an add/multiply sine (`:263`); the fixed path uses a table (`:311`, to be shipped as a constant). Epic 017 ported these by hand into `hadronyche-reference`; this ticket writes the DSL version.

Kit rules: strict f32, Q1.31 / Q0.32 fixed point, shipped tables; no libm, no transcendental functions; float operations never reordered. Voice fields arrive as a sparse keyed controller stream (content domain v1). The echo runs at the domain rate and has no resampler (a resampler node may appear only inside an edge module, C-134). Every accepted DSL program is exact-fixed or exact-float (C-123); L is a whole number of domain-rate samples (C-105).

Constants: domain rate 96 kHz, the only rate on the wire (C-01); slice = 48 frames = 0.5 ms (C-02); 4-byte samples (C-03); 6 channels per stream, 192 bytes per channel per slice (C-04).

Note: whether POC voice windows compare bit for bit is an open question owned by **task 002 (archaeology)** (falsifier: tolerance-only comparison in the POC). It does not block this ticket: equivalence here is against the epic 017 hand-written module, not the POC.
## Work
1. Write `hadronyche-reference/dsl/echo.hdsl` in float and fixed variants (two programs, or one with a variant switch), matching the 017 hand-written module's operation order exactly (float operations are never reordered).
2. Ship the fixed-path sine table as a constant table in the kit form.
3. Add an `expect/` entry (L, class, deps, cost bound) and add both variants to the conformance corpus run.
4. Build both in the `dsl-conformance` CI job.
## Acceptance criteria
1. Both variants compile in CI with no diagnostics.
2. The compiler assigns `exact-float` to the float variant and `exact-fixed` to the fixed variant.
3. The expect entries pass the 025a-01 harness and the 025a-15 probe check.
4. No transcendental call: the float path is the add/multiply sine, the fixed path the shipped table.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Bit equivalence with the 017 module (025a-20); mixer (025a-19); replay test (025b-05).
