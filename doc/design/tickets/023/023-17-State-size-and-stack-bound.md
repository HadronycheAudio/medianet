# 023-17 — State size and worst-case stack analysis
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-11, 023-12, 023-28

## Context
- `hadronyche-dsl/analysis/memory.rs` — state size and worst-case stack.
- `[decision]`: one `no_std` pure Rust process function per module; it never blocks; **no allocation at process time**; the compiler emits the state size and worst-case stack as constants.
- Manifest fields: `state_bytes: u32`, `stack_bytes: u32`.
- State contributors: `StateCell`s, `Delay(n)` lines (n × 4 bytes per channel, C-03) including delays inserted by join padding (023-28, C-104), lookahead and block buffers, FIR history, resampler history (023-11), hold-last-value cells for dense-controller gap defaults (023-12), voice state for sparse keyed controllers.
- Stack: the front end computes a bound over the IR's temporaries (maximum live values × 4 bytes, C-03) under the evaluation order the code generator will use. How this bound relates to the generated code's real stack is a remaining gap owned by **epic 024** (falsifier: measured stack of generated code exceeds `stack_bytes`). Document the assumption so 024 can uphold or revise it.

## Work
1. `analysis::memory::state_bytes(&Graph) -> u32` with per-node contributions.
2. `analysis::memory::stack_bytes(&Graph) -> u32` via live-value analysis over the canonical evaluation order.
3. Overflow of u32 → diagnostic.
4. Document the stack-model assumption for epic 024, with owner and falsifier.

## Acceptance criteria
1. A 6-channel `Delay(48)` program reports state_bytes = 48 × 6 × 4 = 1 152 plus any documented fixed overhead.
2. A dense-controller input with default gap handling adds exactly the bytes of 1 held value.
3. A padded join adds exactly the bytes of its inserted delay lines; its opted-out twin does not.
4. Adding a node never decreases state_bytes (property test, ≥ 1 000 cases).
5. stack_bytes is > 0 for any non-trivial program and stable under reformatting.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Verifying the stack bound on generated code (epics 024/025).
