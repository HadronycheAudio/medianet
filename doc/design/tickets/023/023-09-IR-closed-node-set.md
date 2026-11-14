# 023-09 — IR: dataflow graph with the closed node set
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-06

## Context
- `hadronyche-dsl/ir/` — a dataflow graph IR: nodes from a **closed set** (arithmetic, delay, table lookup, linear ramp, mix, the resampler node, state cells), **no unbounded loops, no recursion** `[decision]`.
- The IR is what the analyses (latency, cost, memory, determinism, dataflow) and the join-padding pass (023-28, C-104) run on, and what the code generator (epic 024) consumes. Code generated from it must be one `no_std`, pure Rust process function `process(state, inputs at position p with per-input gap flags, position) -> outputs` that never blocks and **does not allocate at process time**.
- Arithmetic kit: strict f32 operations; Q1.31 and Q0.32 fixed point; shipped deterministic tables. No libm, no transcendental functions. Float operations are never reordered; summation order is defined by DSL semantics.
- The DSL has no non-deterministic node in v1 (C-123): every node in the set is deterministic.
- DoD: "The closed node set is listed with, per node, its latency, cost and determinism contribution" — this ticket creates the table skeleton; latency/cost/determinism columns are filled by 023-15, 023-16, 023-18.

## Work
1. `ir::Node` enum with exactly: `Arith(op, format)` (f32 / Q1.31 / Q0.32 ops), `Delay(n)`, `TableLookup(table_id)`, `LinearRamp`, `Mix(order)` (ordered summation), `Resample(Down2 | Up2)` (the kit's fixed 2:1 or 1:2 node only, C-105, C-197; the edge's steerable rate actuator is not a node, C-189), `StateCell`; plus `Input(port)`, `Output(port)`, `Const` as graph terminals. Each `Delay` records whether it was written by the author or inserted by join padding (023-28).
2. `ir::Graph`: nodes, typed edges (content type, rate, channels), node spans back to source; joins (nodes with more than one data input) are identifiable and carry the author's opt-out flag (C-104).
3. Invariant check `Graph::validate()`: acyclic except through `StateCell`/`Delay` (feedback must pass through state), every node type in the closed set.
4. Graph is canonical (node order independent of source formatting) so analyses are stable under reformatting (023-AC-5).
5. Start the node table document (a section to be moved into `LANGUAGE.md` by 023-26) with one row per node.

## Acceptance criteria
1. `Node` has exactly the 7 node kinds listed plus the 3 terminals; adding a variant requires updating the node table test (a test enumerates variants against the table).
2. `validate()` rejects a combinational cycle not passing through `StateCell` or `Delay` (unit test) and accepts the same cycle through a `Delay(1)`.
3. Mix preserves operand order: two graphs with operands in different order are distinct.
4. A join with the opt-out flag and the same join without it are distinct graphs.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The closed node set is listed with, per node, its latency, cost and determinism contribution (rows created here; columns completed in 023-15, 023-16, 023-18)

## Out of scope
- Lowering from AST (023-10); resampler internals (023-11); join padding (023-28); analyses (023-14 to 023-18).
