# 023-08 — Type and rate checker
**Epic:** 023-DSL-compiler-front-end · **Effort:** M · **Depends on:** 023-04, 023-06, 023-07

## Context
- `hadronyche-dsl/types/` checks an AST against content types, rates and channel counts (model in 023-06, bundle layouts in 023-07).
- Rules `[decision]`:
  - **Every network port is at the domain rate, 96 kHz** (C-01). A different rate exists only on the device-facing side of an edge module; those rules are checked in 023-29.
  - **Cross-rate composition goes only through the kit's resampler node**, offered only at 2:1 and 1:2 (C-105) and only inside an edge module (C-134). Any other connection between different rates is rejected (023-AC-1).
  - Read-only transport state (playing, bar, beat, tempo, time code at position p) is input only; **transport authoring is deferred past v1; the language must not offer it** (023-AC-1).
  - A port wider than 32 × 6 = 192 channels, or an explicit layout with more than 32 members or a member wider than 6 channels, is rejected (023-AC-1; C-04, C-08). A source with controllers has at most 31 audio members, 186 channels, because its controller member counts towards the 32 (C-209); that rejection is in 023-32.
  - Samples are 4 bytes, f32 or Q1.31 (C-03). Arithmetic kit: strict f32; Q1.31 and Q0.32 fixed point; mixing f32 and fixed point requires an explicit conversion.
  - At most 256 streams per source (C-09).
- Every rejection carries a diagnostic with a span and a stable code (023-02).

## Work
1. Name resolution (ports, state cells, nodes) with diagnostics for undefined/duplicate names.
2. Type inference/checking of every expression: content type, sample format, rate, channel count.
3. Rate check on every edge; the resampler node is the only node whose input and output rates may differ.
4. Port width check via `default_layout`/`validate_explicit` from 023-07; stream count check against C-09.
5. Transport state: reject any write or output of transport state type.
6. Output: a typed AST (or annotation table) consumed by lowering (023-10) and by the edge-module check (023-29).

## Acceptance criteria
1. Inside an edge module, a 96 kHz network output fed directly from a 48 kHz device input → exactly 1 diagnostic naming both rates and suggesting the resampler node; the same graph with a resampler 48→96 in between checks clean.
2. A program writing transport state → 1 diagnostic naming transport authoring as unavailable in v1.
3. A 200-channel port → 1 diagnostic citing the 192-channel (32 × 6) limit.
4. Adding an f32 value to a Q1.31 value without conversion → 1 type diagnostic.
5. A module needing 257 streams → 1 diagnostic citing C-09.
6. Undefined name → diagnostic with the name's span.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Edge-module rules: network port rate, resampler placement, device-facing ports (023-29); loop/recursion/allocation/out-of-kit rejection on the IR (023-10); resampler delay (023-11); the consolidated 023-AC-1 suite through `check()` (023-23).
