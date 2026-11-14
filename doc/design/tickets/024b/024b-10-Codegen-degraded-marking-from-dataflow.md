# 024b-10 — Codegen: degraded marking from dataflow
**Epic:** 024b-DSL-code-generator · **Effort:** M · **Depends on:** 024b-03, 024b-05, epic 023

## Context
Rule (C-122): the degraded flag follows the content. An output at position p is degraded if and only if an input it depends on was flagged gap (`InputSlice.gap`) at p − L, where L is that output's structural latency (`OutputInfo.latency`, a whole number of domain-rate samples, C-105). For L = 0 this reduces to "an input flagged gap in this slice". When L is not a whole number of 48-frame slices, the output slice carries content from two input slices, and it is degraded if either contributing input slice was gapped (epic). Dependency sets come from the checked IR analyses (epic 023); per-output L comes from 024b-05 and is the longest path (C-104).

Epic acceptance criterion 024b/AC-6: for a program where output A depends on input 1 only and output B on inputs 1 and 2, a gap on input 2 marks B degraded and not A; for an output with latency L, the flag is set on the output that carries the content of the gapped input, not on the output produced in the same call.

## Work
- `hadronyche-dsl/src/codegen/degraded.rs`: for each output, take its input dependency set from the IR. For L = 0 emit `outputs[k].degraded = inputs[i].gap | inputs[j].gap | ...;`. For L > 0 keep, per input in the set, a fixed-size ring of past gap flags in `State` covering ⌈L / 48⌉ + 1 slices, and set the flag from the entries that contribute to this output slice.
- Ring storage is counted in `state_bytes`; no allocation; constant time per call.
- An output with an empty dependency set is never degraded.
- Document the rule in the generated module docs.

## Acceptance criteria
1. Program: A = f(input 1), B = g(input 1, input 2), L = 0. Gap on input 2 only: B.degraded = true, A.degraded = false.
2. Same program, gap on input 1 only: A and B both degraded. No gaps: neither degraded.
3. An output depending on no input is never degraded, for every combination of gap flags (exhaustive over 2 inputs).
4. An output with L = 96 samples (two slices): a gap on its input in the call for position p marks degraded the output produced two calls later, and not the outputs before it.
5. An output whose L is not a whole number of slices (for example 72 samples): a single gapped input slice marks exactly the two output slices whose content it contributes to.
6. The `const` state-size assertion still passes with the rings included.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Computing dependency sets (epic 023 analyses) and latencies (024b-05).
- Gap concealment (024b-07, 024b-09).
