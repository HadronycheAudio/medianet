# 024b-07 — Codegen: default gap concealment
**Epic:** 024b-DSL-code-generator · **Effort:** S · **Depends on:** 024b-03, 024b-04, epic 009, epic 010, epic 023

## Context
The runtime classifies gaps and delivers each input slice with `InputSlice.gap: bool` ("true: content is the generated default"). Generated code applies gap defaults per content type `[decision]`: audio zero-fill; dense controllers hold last value; sparse keyed controllers rely on state refresh. Each dense controller declares an initial value in the manifest (default 0), used when a gap precedes any received value (C-158). A sparse ramp runs over one slice and ends at its target, so no ramp is in progress across a slice boundary; after a gapped slice the sparse state is the state reached at the end of the previous slice (epic). DSL constructs may override (024b-09).

The gap default for read-only transport state is settled by C-141 and implemented in 024b-08.

Epic acceptance criterion 024b/AC-5 (audio, dense and sparse parts): with an input flagged gap, audio input reads as zeros; a dense controller reads its last value, or its declared initial value if no value was received yet (C-158); sparse state is unchanged. The override part is 024b-09; the transport part is 024b-08.

Slice 48 frames (C-02); 4-byte samples (C-03); 6 channels, 1 152 bytes of audio per slice (C-04).

## Work
- `hadronyche-dsl/src/codegen/gaps.rs`: for each input port by content type, emit code at the top of `process` that, when `gap` is true, substitutes: zeros for audio; the last-received dense value (held in `State`, initialised by `init` to the controller's declared initial value from the manifest, C-158, and updated on every non-gap slice) for dense controllers; no sparse update for sparse keyed controllers.
- State fields for held dense values are counted in `state_bytes`.
- Tests on generated modules from the epic's corpus (024b-04); the dense cases use the mix's volume, the reference dense controller (C-192).

## Acceptance criteria
1. With an audio input flagged gap, the module computes as if all 48 × 6 samples were zero (output equals the zero-input output, bitwise).
2. With a dense controller flagged gap after a slice with value v, the module reads v for the whole slice.
3. With a sparse controller flagged gap after a slice whose ramp ended at target t, the sparse state after the gapped slice equals t for that field, unchanged from before.
4. With a dense controller flagged gap before any value has been received, the module reads the initial value declared in the manifest, and 0 when none is declared (C-158).
5. `state_bytes` increases by the held dense storage and the `const` size assertion still passes.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Gap default for transport state (024b-08).
- DSL overrides (024b-09).
- Degraded marking (024b-10).
- Gap classification (epic 009); emitting the gap slice when a module misses its own deadline (hosted backend, epic 015, C-144).
