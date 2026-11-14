# 023-10 — Lowering AST to IR, with kit rejections
**Epic:** 023-DSL-compiler-front-end · **Effort:** M · **Depends on:** 023-08, 023-09

## Context
- Converts the checked AST (023-08) into the dataflow graph IR (023-09).
- 023-AC-1 rejections that belong here, each with a diagnostic (stable code, 023-02) pointing at the source span:
  - an **unbounded loop or recursion** (e.g. mutually referencing user-defined functions/macros, or a definition referencing itself);
  - a **call outside the kit** — any transcendental (sin, cos, exp, log, pow, tanh, ...) or any libm call; kit = strict f32 operations, Q1.31 and Q0.32 fixed point, shipped deterministic tables `[decision]`;
  - **allocation** (any construct that would need heap memory at process time; the process function never allocates);
  - a **linear-phase FIR with an even number of taps** — rejected in v1, with a diagnostic suggesting an odd-tap design, so that L stays exact (C-183).
- Anything outside the deterministic kit is **rejected, never downgraded to the bounded class**: the DSL has no non-deterministic node in v1 (C-123).
- Bounded repetition, if 023-01's grammar allows it (e.g. fixed-count unrolled replication over channels or voices), is unrolled at lowering time so the IR has no loops.
- A sine must be built from the kit: either a shipped deterministic table or additions/multiplications only (POC lesson `[code]`: `src/music.rs:263` builds sine from additions and multiplications "so that it gives the same bits on every machine"; `src/music.rs:311` uses libm and is not strictly deterministic).
- The per-join opt-out construct (C-104) is carried onto the IR join it names.

## Work
1. Lowering pass `ir::lower(typed_ast) -> Result<Graph, Vec<Diagnostic>>`.
2. Call-graph check over user definitions; reject cycles (recursion).
3. Whitelist of kit operations and shipped tables; anything else → "outside the arithmetic kit" diagnostic, with a hint for transcendentals ("use a shipped table").
4. Reject allocation constructs (dynamic-size arrays, growable collections, strings at process time).
5. Unroll bounded repetition; reject repetition whose bound is not a compile-time constant (unbounded loop).
6. Attach source spans to IR nodes; set the opt-out flag on joins the author marked.
7. Reject a linear-phase FIR whose tap count is even → diagnostic citing C-183 and suggesting an odd-tap design.

## Acceptance criteria
1. Direct recursion and 2-step mutual recursion → 1 diagnostic each, naming the cycle.
2. A call to `sin` → 1 diagnostic "outside the arithmetic kit"; same for `exp`, `log`, `pow`.
3. A dynamically sized buffer → 1 allocation diagnostic.
4. Repetition with a non-constant bound → 1 unbounded-loop diagnostic; with constant bound 8 → 8 unrolled copies in the IR.
5. Every IR node of a lowered test program carries a span inside the source.
6. Lowering the same program reformatted (whitespace, comments, reordered independent declarations) yields an identical canonical graph.
7. An opt-out written on a mix lowers to a join with the opt-out flag set; an opt-out on a node that is not a join → 1 diagnostic.
8. A linear-phase FIR of 4 taps → 1 diagnostic citing C-183 whose message suggests an odd-tap design; the same FIR with 5 taps lowers clean.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Rate and transport rejections (023-08); edge-module rules (023-29); join padding itself (023-28); consolidated 023-AC-1 suite (023-23); resampler node semantics (023-11).
