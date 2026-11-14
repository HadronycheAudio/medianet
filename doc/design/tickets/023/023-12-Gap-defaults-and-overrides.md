# 023-12 — Gap-handling defaults and overrides
**Epic:** 023-DSL-compiler-front-end · **Effort:** S · **Depends on:** 023-10

## Context
- The process function receives inputs at position p **with per-input gap flags**.
- Gap handling `[decision]`: compiler-generated defaults per content type —
  - audio: zero-fill;
  - dense controllers: hold last value;
  - sparse keyed controllers: rely on state refresh;
  — **overridable by DSL constructs** (syntax from 023-01).
- Read-only transport state: the default is decided (C-141): hold the last known state and advance the playhead as that state dictates: a playing transport moves on by the positions elapsed, a stopped one does not advance (C-200). It is implemented in 023-30; this ticket gives it a `GapPolicy` slot like the other defaults.
- **Dense controller with no prior value** (C-158): each dense controller declares an initial value in the manifest (default 0), used when a gap precedes any received value. The hold-last-value cell starts at that value.
- The manifest lists **gap overrides** (manifest contents).
- Degraded marking is separate (023-14, C-122): a gap override changes the content produced, not whether the output is marked degraded.

## Work
1. In the IR, attach to each input a `GapPolicy` (default from content type, or override).
2. Lower the DSL override construct to a `GapPolicy::Override(subgraph)` built from kit nodes only.
3. Hold-last-value requires a state cell, initialised to the input's declared initial value (default 0, C-158); account for it so 023-17 counts its bytes.
4. Lower the initial-value construct of 023-01; reject an initial value outside the controller's value type with a diagnostic.
5. Expose the list of overridden inputs and the initial value of every dense controller input for the manifest (023-19).

## Acceptance criteria
1. An audio input with no override gets zero-fill; a dense controller input gets hold-last (with 1 state cell); a sparse keyed controller input gets state-refresh.
2. An override on an audio input replaces the default; the manifest list of gap overrides contains exactly that input.
3. An override using a non-kit call is rejected with the 023-10 diagnostic.
4. A dense controller input with no declared initial value gets 0; one declaring a value gets that value; in both cases a gap before any received value yields the initial value (unit test on the IR semantics), and the value is exposed for the manifest.
5. A transport-state input without an override is accepted and gets the C-141 policy slot (behaviour tested in 023-30).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] `LANGUAGE.md` covers every construct with one example (gap-override example supplied to 023-26)

## Out of scope
- Degraded marking from dataflow (023-14); generating the gap code (epic 024).
