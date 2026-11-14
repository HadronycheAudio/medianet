# 017a-01 — Port voices with a constant sine table
**Epic:** 017a-Reference-modules · **Effort:** S · **Depends on:** epic 002

## Context
- The reference modules (sender, echo, mixer) are the permanent test instrument of the runtime: exact-tier verification works because the sender reruns the echo's code and compares bits `[decision]`.
- POC source `[code]`: `music::Voices` (`src/music.rs:287`) plays voices from *parts*. A part is one field of one voice moving linearly to a value over the block — a **sparse keyed controller**, repaired by re-sending current values. Float rendering uses `sine()` (`music.rs:263`), built only from additions and multiplications "so that it gives the same bits on every machine".
- **Hole to close** `[code]`: the fixed-point table is built with libm `f64::sin` on each machine (`music.rs:311`). Rule `[decision]`: no libm in any reference module's signal path. Ship the table as a constant (or generate it with add/multiply only).
- Rule: hand-written Rust first (DSL re-expression is task 025).
- Archaeology (epic 002) must have cleared the ported code before it is cited as an exact-tier baseline.

## Work
1. Create `hadronyche-reference/src/instrument/voices.rs`: port `Voices`, the part-application logic and `sine()` from `music.rs`, for both float and fixed-point rendering.
2. Generate the fixed-point sine table once (offline, with a small generator script or `build.rs`-free test) and commit it as a `const` array in source. Add a test that recomputes it with add/multiply only, or compares against a checked-in hash, so any change is detected.
3. Add a lint/test that no function reachable from rendering calls `f64::sin`, `cos`, `exp`, `powf` or other libm functions (e.g. a grep-based CI test over `instrument/`).
4. Header comment citing the POC source file and line, and the epic 002 finding that cleared it.

## Acceptance criteria
1. The constant table is bit-identical to the table the POC builds on the archaeology reference machine (unit test against a checked-in fixture).
2. `instrument/voices.rs` contains no libm call; the CI check fails if one is added.
3. Rendering 10 s of a fixed set of parts at 96 kHz gives an output hash that is identical in debug and release builds, for float and fixed.
4. The port renders bit-identically to the POC on the fixture parts captured by epic 002 (if epic 002 provides such vectors; otherwise record this in the gaps).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Each ported POC piece cites its source file and the archaeology finding that cleared it

## Out of scope
- Phrase generation (017a-02); echo module wiring (017a-07).
- Cross-machine bit-exactness proof (017a-10).
