# 024b-01 — Decide output form: Rust source versus object code
**Epic:** 024b-DSL-code-generator · **Effort:** S · **Depends on:** 024a-08

## Context
The epic's open sub-question "Output form: Rust source (auditable) versus direct object code" is owned by this task, with falsifier: "Rust source cannot guarantee evaluation order under optimisation". The epic's deliverables assume Rust source (`emit.rs` — IR → Rust source implementing `Module`). Rules the output must respect `[decision]`: float operations are never reordered; no fused multiply-add contraction; summation order defined by DSL semantics; no auto-vectorisation that changes results; no libm.

## Work
- Build, as a throwaway probe in `probes/` (C-133), a test corpus of Rust functions in the style the generator will emit (strict left-to-right sums, products, `(a*b)+c` patterns, loops over 48 frames × 6 channels) using kit helpers.
- Compile at `opt-level=3` with LTO on the three CI targets (024a-06); compare result bits with `opt-level=0`; inspect assembly for FMA instructions (`vfmadd*`, `fmadd`, `vfma`) and reassociated reductions.
- Write a decision note in `doc/design/reports/` (C-130): Rust source retained or not, with the evidence `[measured]`, and the compiler flags required (e.g. no `-C target-feature` that enables FMA contraction, no fast-math).
- Add a permanent CI check (assembly scan for FMA opcodes on generated-style code), as a binary in the `hadronyche-tools` crate (C-133), if Rust source is retained.

## Acceptance criteria
1. Result bits at `opt-level=0` and `opt-level=3` + LTO are identical on all three CI targets for the corpus.
2. Assembly of the corpus contains no FMA instruction on any target; the grep check is in CI.
3. The decision note states the outcome, the falsifier, the evidence tagged `[measured]`, and the required compiler settings.
4. If the falsifier is triggered, the note records the alternative and a new open item with owner and falsifier.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Implementing the emitter (024b-03, 024b-04).
