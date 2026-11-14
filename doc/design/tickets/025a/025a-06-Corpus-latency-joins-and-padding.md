# 025a-06 — Corpus: latency joins and padding
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** S · **Depends on:** 025a-01
## Context
Part of the conformance corpus in `hadronyche-dsl/conformance/corpus/` (at least 40 programs in total across 025a-02 to 025a-05 and this ticket, float and fixed variants, hand-derived expectations).

Settled rule (C-104): where DSL paths of different latency meet at a join, the compiler **pads the shorter path with a whole-sample delay line**, **reports the inserted delay in the manifest**, and the module's L is the **longest path**. An explicit construct **opts out** per join. A path with frequency-dependent delay (a recursive filter) is **padded on its structural part only** and the join is **flagged in the manifest**.

L is always a whole number of domain-rate samples (C-105). Every accepted DSL program is exact-fixed or exact-float (C-123). The expect format of 025a-01 carries `joins` (per join: inserted delay in whole samples, or `opt-out`, and the `flagged` marker).
## Work
Add at least 7 programs with `expect/` files and written hand derivations, float and fixed where the kit allows:
1. A two-input mix where one input passes through a delay of d samples: expected inserted delay d on the other path, L = d.
2. A three-way join with three different path latencies: expected inserted delays per shorter path, L = the longest.
3. A join where one path contains a linear-phase FIR (centre (N − 1) / 2) and the other an explicit delay of a different length.
4. The same program as item 1 with the opt-out construct on the join: expected `opt-out`, no inserted delay, L = the longest path.
5. Two joins in series, so the inserted delays and L compose.
6. A join where one path contains a recursive filter (frequency-dependent delay) and the other an explicit delay: expected padding computed on the recursive path's structural part only, the join `flagged`, L = the longest structural path (C-104).
## Acceptance criteria
1. At least 7 new programs with `expect/` files and hand derivations.
2. The 025a-01 harness passes on all of them, comparing the manifest's inserted delays and L.
3. The opt-out program's manifest reports no inserted delay at that join.
4. Changing one expected inserted delay by 1 sample fails the harness.
5. The recursive-path join is reported flagged in the manifest, with the hand-derived structural padding (C-104).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Any compiler disagreement found is filed against epic 023/024 with an owner, not patched by editing the expectation
## Out of scope
- Probe measurement of these programs (025a-15).
