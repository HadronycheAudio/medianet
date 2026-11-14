# 024b-09 — Codegen: DSL gap overrides
**Epic:** 024b-DSL-code-generator · **Effort:** S · **Depends on:** 024b-07, epic 023

## Context
Generated gap defaults per content type `[decision]`: audio zero-fill; dense controllers hold last value; sparse keyed controllers rely on state refresh. "DSL constructs may override." Epic acceptance criterion 024b/AC-5 (override part): "An override declared in the DSL replaces the default."

The override construct is parsed and checked by the front end (epic 023) and arrives in the checked IR; its syntax is an open item owned by epic 023.

## Work
- In `hadronyche-dsl/src/codegen/gaps.rs`: when the IR carries an override for an input, emit the override's concealment instead of the default (e.g. a constant value, hold-last for audio, a DSL expression evaluated with kit operations).
- Overrides obey the same rules as all generated code: no allocation, kit only, defined evaluation order.
- Tests: one program per content type with an override, checked against the default behaviour.

## Acceptance criteria
1. An audio input with an override "hold last slice" reads the previous slice's samples when flagged gap, not zeros.
2. A dense controller with an override "constant c" reads c when flagged gap, not the last value.
3. Inputs without an override keep the defaults of 024b-07 (regression test).
4. An override expression that would need a non-kit operation is rejected with a named error.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Parsing the override syntax (epic 023).
- Default concealment (024b-07).
