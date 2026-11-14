# 024a-03 — ABI `fingerprint_matches`
**Epic:** 024a-Runtime-ABI-and-arithmetic-kit · **Effort:** S · **Depends on:** 024a-01

## Context
Exact-float modules carry a fingerprint: a compile-time input vector and the expected output bits, covering subnormal and NaN-producing paths. Floating-point results can differ across machines through non-IEEE subnormal handling (flush-to-zero / denormals-are-zero, set per thread), NaN payloads and 80-bit intermediate precision on legacy x86 `[decision]`. Floating-point modes are per thread and audio stacks change them, so the runtime evaluates the fingerprint at start **on the processing thread itself**, after its floating-point mode is final. Match ⇒ exact-float eligible; mismatch ⇒ the module runs as bounded-class and the regime stamp says so.

ABI shapes (from 024a-01):
```rust
pub struct Fingerprint { pub inputs: &'static [u32], pub expected: &'static [u32] }
/// Call on the processing thread, after its floating-point mode is final.
pub fn fingerprint_matches<M: Module>() -> bool;
```
`ModuleInfo.fingerprint: Option<&'static Fingerprint>`. Samples are 4 bytes (C-03); a slice is 48 frames (C-02).

Settled by the epic: the vector layout is by input port in `INFO.inputs` order, 48 frames per slice, channel-interleaved 4-byte samples, as many slices as the vector holds; the state is a fresh one from `init`; an exact-float module without a fingerprint is not exact-float eligible. The bounded class arises only from an exact-float fingerprint mismatch and for hand-written modules that do not claim exactness (C-123), so only exact-float modules consult the fingerprint. The word layout for non-audio content (dense, sparse, transport) follows the types of epic 010 (open item owned by epic 010).

## Work
- Implement `fingerprint_matches::<M>()` in `hadronyche-core/src/abi/`:
  - If `M::INFO.fingerprint` is `None`, return `false`. For `ExactFixed` and `Bounded` modules the fingerprint is not consulted (see `effective_class`).
  - Initialise a `M::State` on the stack with `M::init`, feed `inputs` as input slices using the epic's layout rule, run `process`, compare output words bitwise (`u32` equality, not float equality) with `expected`.
  - No allocation; use stack buffers sized by constants.
- Add `pub fn effective_class<M: Module>() -> DeterminismClass` returning `Bounded` when the module is `ExactFloat` and the fingerprint is absent or does not match, otherwise `M::INFO.determinism`.
- Document in the function doc comment that it must be called on the processing thread.

This ticket delivers epic acceptance criterion 024a/AC-5 (ABI side of the fingerprint); the processing-thread test is 024b-17 (024b/AC-4).

## Acceptance criteria
1. A hand-written exact-float test module with a correct fingerprint returns `true`.
2. The same module with one `expected` word flipped by one bit returns `false`, and `effective_class` returns `Bounded`.
3. Comparison is bitwise: a test where expected and actual are both NaN with different payloads returns `false`.
4. The function runs under an allocator that aborts on use (hosted test) without aborting.
5. The input-vector layout rule is written in the doc comment and a test exercises a module with 2 inputs.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Synthesising fingerprint vectors (024b-11).
- Testing a per-thread flush-to-zero change and the regime stamp field (024b-17).
- Whether the vector covers every differing path (owned by task 025).
