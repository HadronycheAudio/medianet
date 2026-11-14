# 023-28 — Join padding pass and per-join opt-out
**Epic:** 023-DSL-compiler-front-end · **Effort:** M · **Depends on:** 023-11, 023-13, 023-15

## Context
- Rule (C-104, decided): **where DSL paths of different latency meet at a join, the compiler pads the shorter path with a whole-sample delay line, reports the inserted delay in the manifest, and the module's L is the longest path; an explicit construct opts out per join. A path with frequency-dependent delay is padded on its structural part only and the join is flagged in the manifest.**
- A join is an IR node with more than one data input (typically `Mix`, also arithmetic on two signals); 023-09 marks joins and 023-10 sets the opt-out flag from the construct of 023-01.
- Path latencies at every join come from `analysis::latency::join_paths` (023-15). L is always a whole number of domain-rate samples (C-105), so every padding amount is whole and fits `Delay(n)`.
- Inserted delays are ordinary `Delay(n)` nodes marked as inserted; they count in state size (023-17) and cost (023-16) and do not change dependency sets (023-14).
- The former open sub-question on fractional differences is closed: with whole-sample L (C-105) and whole-sample delay lines (C-104) it cannot arise. One residual case remains inside an edge module: a join running on the 48 kHz device side pads in 48 kHz samples (each 2 domain-rate samples), so a difference of an odd number of domain-rate samples there cannot be padded by a delay line at that join. Rule (C-190, decided): **the compiler rejects such a join and suggests the per-join opt-out**, so no inexact padding is ever emitted.
- 023-AC-3: for every corpus join of unequal paths the compiler inserts exactly the hand-derived delay, lists it in the manifest and reports the longest path as L; with the opt-out no delay is inserted and the manifest records the opt-out.

## Work
1. `ir::pad_joins(&mut Graph, &JoinPaths) -> Result<Vec<JoinPadding>, Vec<Diagnostic>>`: for each join without opt-out, insert on each shorter input a `Delay` equal to (longest − its path latency), expressed in samples at that input's rate; for a path marked frequency-dependent by 023-15, use its structural latency only.
2. Process joins in topological order and recompute downstream path latencies after each insertion, so nested joins are padded once and exactly.
3. A join with at least one frequency-dependent input path: pad as in step 1 and set the join's `frequency_dependent` flag in its `JoinPadding` record (C-104).
4. Opted-out joins: no insertion; recorded as `JoinPadding::OptedOut { join, path_latencies }`.
5. Supply `JoinPadding` records to the manifest (023-19).
6. A 48 kHz device-side join without the opt-out whose path difference is an odd number of domain-rate samples → diagnostic naming the join, citing C-190 and suggesting the per-join opt-out.

## Acceptance criteria
1. Every corpus join of unequal paths: inserted delays equal the 023-13 sidecar values exactly; output L after padding equals the longest path.
2. After padding, `join_paths` reports equal latency on every input of every non-opted-out join.
3. An opted-out join: 0 inserted nodes; one `OptedOut` record listing each path latency.
4. Two nested joins: each shorter path padded once; running the pass twice inserts nothing the second time.
5. The corpus join fed by a recursive-filter path: inserted delay equals the hand-derived structural difference, and its manifest record carries the frequency-dependent flag; joins without such a path are not flagged.
6. A 48 kHz device-side join whose paths differ by an odd number of domain-rate samples → 1 diagnostic citing C-190 whose message suggests the per-join opt-out; with the opt-out → accepted, no delay inserted, `OptedOut` record (C-190).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The opt-out syntax (023-01, 023-04) and its lowering (023-10); latency computation (023-15); generating the delay code (epic 024).
