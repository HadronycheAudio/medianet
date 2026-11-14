# 023-18 — Determinism class assignment
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-10, 023-11, epic 013

## Context
- `hadronyche-dsl/analysis/determinism.rs` — class assignment.
- Classes `[decision]`: `pub enum DeterminismClass { ExactFixed, ExactFloat, Bounded }`.

  | Class | Scope | Verification |
  |---|---|---|
  | exact-fixed | Bit-exact everywhere, including between host and device | Exact tier: record and replay |
  | exact-float | Bit-exact only where a startup fingerprint matches | Exact tier: record and replay |
  | bounded | Everything else | Tolerance tier |

- Where bounded arises (C-123, decided): **only from an exact-float fingerprint mismatch at runtime, and for hand-written (non-DSL) modules that do not claim exactness. The DSL has no non-deterministic node in v1.** So the static classifier assigns a DSL program `ExactFixed` (all fixed point, Q1.31 / Q0.32, with shipped deterministic tables) or `ExactFloat` (any f32 node), never `Bounded`. Anything outside the kit is rejected by 023-10, not downgraded. `Bounded` stays in the enum for hand-written modules (023-25) and the runtime downgrade.
- The startup fingerprint (compile-time vector incl. subnormal and NaN paths, evaluated on the processing thread) is emitted and evaluated in epics 024 and 025; **this task only classifies**. A fingerprint mismatch at runtime downgrades the module to bounded and the regime stamp says so (stamp fields are from epic 013).
- 023-AC-5: an all-fixed-point program is exact-fixed; adding one f32 node makes it exact-float; no corpus program is bounded; the classification is stable under reformatting.

## Work
1. Per-node determinism contribution (ExactFixed or ExactFloat); module class = ExactFloat if any node is ExactFloat, else ExactFixed.
2. Include resampler (023-11) and gap-override subgraphs (023-12).
3. Fill the determinism column of the node table.
4. Map `DeterminismClass` to the epic 013 regime stamp field; document the mapping, including that `Bounded` is never produced by the DSL classifier.

## Acceptance criteria
1. An all-Q1.31 corpus program → `ExactFixed`.
2. The same program with one f32 multiply added → `ExactFloat`.
3. Every corpus program and every expressiveness program (023-24) is `ExactFixed` or `ExactFloat`; a test asserts the classifier has no code path returning `Bounded`.
4. Class unchanged across 3 reformatted variants of each corpus program.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] The closed node set is listed with, per node, its latency, cost and determinism contribution (determinism column)

## Out of scope
- Fingerprint emission (epic 024) and evaluation (epic 025); the class of hand-written modules (023-25).
