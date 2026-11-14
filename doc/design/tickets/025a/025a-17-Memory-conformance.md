# 025a-17 — Memory conformance: stack and state size
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** S · **Depends on:** 025a-10
## Context
Acceptance criterion 10 (memory): **measured stack never exceeds `stack_bytes`; state size equals `state_bytes`.**

The compiler (epic 024) emits, per generated module, the state size and worst-case stack as constants (`state_bytes`, `stack_bytes`). The process function `process(state, inputs at position p with per-input gap flags, position) -> outputs` never blocks and performs **no allocation**.

The stack measurement method is not given by the epic or the constants table; it is owned by this ticket, which chooses it and documents it (falsifier: the method reports different high-water marks on reruns of the same program).
## Work
1. A test in `dsl-conformance` that, for every positive corpus program, checks `size_of` the generated state type (or the allocated state block) equals `state_bytes`.
2. Measure stack use of `process` per program, e.g. by running it on a dedicated thread whose stack is pre-filled with a pattern (stack painting) and finding the high-water mark, with a documented measurement overhead.
3. Assert measured stack ≤ `stack_bytes` for every program.
4. Assert no allocation during `process` (e.g. a counting global allocator in the test).
## Acceptance criteria
1. Every positive corpus program: state size equals `state_bytes` exactly.
2. Every positive corpus program: measured stack ≤ `stack_bytes`.
3. Zero allocations observed during `process` for every program.
4. A test module with an artificially lowered `stack_bytes` fails the check.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Stack measurement on non-host targets (025b-06 may reuse this); cost (025b-10).
