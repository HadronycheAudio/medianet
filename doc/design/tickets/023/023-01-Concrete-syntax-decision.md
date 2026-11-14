# 023-01 — Concrete syntax decision
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** none

## Context
- Epic 023 builds the front half of the module-construction DSL compiler, crate `hadronyche-dsl`: language surface, parsing, type and rate checking, static analyses, ending in a checked IR plus a manifest. Code generation is epic 024.
- The epic leaves one open sub-question owned by this task: **concrete syntax — embedded in Rust as macros, or a standalone language.** Falsifier: either form cannot express the reference echo without escape hatches.
- A second open sub-question shares this ticket with 023-29: **how an edge module declares its device-facing side and that side's rate.** Every network port is at the domain rate, 96 kHz (C-01); rate conversion exists only inside edge modules (C-105); an edge with 48 kHz hardware converts 2:1 through the kit's resampler node (C-80). Falsifier: an edge module for 48 kHz hardware cannot be written without an escape hatch.
- The reference echo that must be expressible `[decision]`: a synthesiser instrument (voices driven by sparse keyed controllers, linear ramps over the slice, table sine, phase reset at gate zero) plus a metronome reading read-only transport state (playing, bar, beat, tempo, time code at position p). Epic 025 writes the real program; this ticket only needs sketches.
- Language constraints the syntax must make expressible or impossible `[decision]`:
  - No unbounded loops, no recursion, no allocation, no calls outside the arithmetic kit (no libm, no transcendental functions). Kit: strict f32 operations; Q1.31 and Q0.32 fixed point; shipped deterministic tables.
  - Content types: audio (f32 or Q1.31), dense controllers (one value per tick), sparse keyed controllers (one field of one voice moving linearly to a value over the slice), read-only transport state. **Transport authoring is deferred past v1; the language must not offer it.**
  - Network ports are all at the domain rate (C-01); a non-domain rate appears only on the device-facing side of an edge module, and cross-rate composition goes only through the kit's resampler node, which is offered only at 2:1 and 1:2 (C-105) and only inside an edge module (C-134).
  - Where paths of different latency meet, the compiler pads the shorter path; **an explicit construct opts out per join** (C-104). The grammar must provide that construct.
  - Gap-handling defaults (audio zero-fill; dense controllers hold last value; sparse keyed controllers rely on state refresh; transport state holds the last known state with the playhead advanced as that state dictates, a stopped playhead not advancing, C-141, C-200) must be overridable by DSL constructs.
  - Each dense controller declares an initial value, default 0, carried in the manifest (C-158). The grammar must let an author give it.
  - A single-stream source may always put its controllers in a second stream, and must when headers plus controllers exceed the 100-byte cap (C-181, C-137). The grammar must let an author opt in to the second stream (023-31).
  - Summation order of float operations is defined by DSL semantics; float operations are never reordered.
- Output location (C-130): specifications go in `doc/design/spec/`.

## Work
1. Write a decision record in `doc/design/spec/` (for example `doc/design/spec/023-dsl-syntax.md`) comparing the two options against: expressing the reference echo instrument and metronome without escape hatches; source spans and diagnostics quality; ability to forbid loops/recursion/allocation/out-of-kit calls; fuzzability of the parser (023-AC-9); stability of classification under reformatting (023-AC-5).
2. For each option, write a sketch of the reference echo instrument and the metronome, and of a minimal edge module (48 kHz device input, resampler 2:1, 96 kHz network output).
3. Choose one; record the chosen form with the falsifier above. Include a minimal grammar outline (module header with an edge-module marker, network port declarations with content type / channels / optional bundle layout, device-facing port declarations with their rate, state cells, node expressions, gap-override construct, dense-controller initial value (C-158), second-controller-stream opt-in (C-181), join opt-out construct) that tickets 023-03 and 023-04 implement.
4. Tag every factual claim with an evidence tag (`[decision]`, `[code]`, `[standard]` per C-132, ...).

## Acceptance criteria
1. The record names exactly one chosen concrete syntax and gives at least the five comparison criteria listed in Work step 1.
2. Sketches of the reference echo instrument, the metronome and the edge module exist in the chosen syntax, with zero escape hatches (no raw Rust blocks, no foreign calls).
3. The grammar outline covers: network port declaration (direction, content type, channels, bundle layout), edge-module marker and device-facing port declaration with rate, state cells, the closed node set (arithmetic, delay, table lookup, linear ramp, mix, resampler with its two ratios 2:1 and 1:2, state cells), a gap-override construct, a dense-controller initial value, a second-controller-stream opt-in for a single-stream source (C-181) and a join opt-out construct.
4. The grammar outline contains no loop, recursion, allocation or transport-authoring construct, and no way to give a network port a rate.
5. The record lives under `doc/design/spec/`.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] Decision record reviewed and merged

## Out of scope
- Implementing the lexer (023-03) and parser (023-04).
- Checking edge-module rules (023-29) and join padding (023-28).
- Writing the full reference echo (epic 025).
- `LANGUAGE.md` (023-26).
