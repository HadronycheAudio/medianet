# 023-14 — Dataflow analysis for degraded marking
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-10

## Context
- `hadronyche-dsl/analysis/dataflow.rs` — per-output input dependency sets.
- Rule (C-122, decided): **the degraded flag follows the content — an output at position p is degraded if an input it depends on was gapped at position p − L**, where L is that output's structural latency (023-15). The code generator (epic 024) applies the rule using the dependency sets from this ticket and L from 023-15.
- 023-AC-6: for a two-input, two-output program where output A depends on input 1 only and output B on both, the dependency sets are exactly {1} and {1, 2}.
- Dependencies through state cells and delays still count (an input feeding a state cell that an output reads is a dependency).

## Work
1. `analysis::dataflow::dependencies(&Graph) -> Vec<(PortId, BTreeSet<PortId>)>`, one entry per output.
2. Traverse backwards from each output through all node kinds including `StateCell`, `Delay` (author-written and padding-inserted), `Resample`.
3. Decide and document whether gap-override subgraphs (023-12) contribute dependencies (an override that ignores the input still depends on its gap flag).
4. Hand the sets, with each output's L, to the IR handed to epic 024; document the C-122 rule in the module docs.

## Acceptance criteria
1. 023-AC-6 program: output A → {1}, output B → {1, 2}, exactly.
2. An output fed only by a constant → empty set.
3. An input reaching an output only through a `Delay(48)` → included.
4. Sets are identical for a reformatted copy of the program and unchanged by join padding (023-28).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Mapping p − L onto slice boundaries and emitting degraded flags at runtime (epics 024/025).
