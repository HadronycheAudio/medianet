# 023-16 — Cost bound analysis
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-11, 023-13, 023-28

## Context
- `hadronyche-dsl/analysis/cost.rs` — operation-count bound per slice per output and total.
- `[decision]`: with no unbounded loops, an operation-count upper bound per slice is derivable. Admission uses it, so CPU stops being a mere diagnostic for DSL modules.
- Unit (C-106, decided): **counted abstract operations per slice, identical on every machine.** Each target has a calibration factor (time per operation), measured once and recorded in the regime stamp; admission converts with it. Measuring the factor (a reference workload run once per listed target, C-176) and checking measured cost against the bound are epic 025's; this ticket defines the per-node operation counts.
- Slice = 48 frames = 0.5 ms at 96 kHz (C-02), the only wire rate (C-01). Inside an edge module the device-facing side at 48 kHz has 24 frames per slice (derived).
- Delays inserted by join padding (023-28, C-104) count like author-written delays.
- Manifest fields: `cost_ops_per_slice: u64` (total) and per-output bounds.
- 023-AC-4 (contributes to AC-33): for the 023-13 corpus the bound is finite, and **monotone under adding nodes**.

## Work
1. Per-node cost function (abstract operations per frame × frames per slice at the node's rate); resampler cost from 023-11. Count one operation per kit arithmetic op, table lookup, ramp step, mix term, delay read/write; document the table.
2. Per-output bound = sum over nodes in the output's cone; total = sum over all nodes counted once, after padding.
3. Overflow-safe arithmetic (checked → diagnostic).
4. Fill the cost column of the node table; document the unit with reference to C-106.

## Acceptance criteria
1. Every corpus program yields a finite total and per-output bound (no overflow).
2. Monotonicity property test: for ≥ 1 000 random graph extensions (adding one node of any kind to a corpus program), the new total ≥ the old total.
3. Pure gain on 6 channels at 96 kHz gives total = 48 × 6 × (operations of one multiply) per slice.
4. A corpus program with a padded join costs exactly the inserted delays' operations more than the same program with the opt-out.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The closed node set is listed with, per node, its latency, cost and determinism contribution (cost column)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Measuring the per-target calibration factor (epic 025, C-176) and checking measured cost (epic 025); admission use of the figure (epic 012).
