# 024b-21 — Codegen: frequency-dependent-delay property on outputs
**Epic:** 024b-DSL-code-generator · **Effort:** XS · **Depends on:** 024a-01, 024b-03, 024b-05, epic 023

## Context
Rule (C-189): the ABI keeps the "frequency-dependent delay" property on outputs, for recursive filters. A path with frequency-dependent delay is padded at joins on its structural part only and the join is flagged in the manifest (C-104, 024b-06); this ticket sets the per-output property in `INFO.outputs`. Which nodes have frequency-dependent delay comes from the checked IR analyses (epic 023; an open item owned by epic 023). The field is part of `OutputInfo` in the ABI of part 024a (024a-01); this part changes no ABI type, so if the field is missing there the difference is filed against part 024a. Per-path structure comes from 024b-05.

Epic acceptance criterion 024b/AC-8 (output property part).

## Work
- In codegen, for each output, set the "frequency-dependent delay" property in its `OutputInfo` when any path from an input to that output passes through a node the IR marks as having frequency-dependent delay; clear it otherwise.
- Hand the property to the manifest data written by 024b-12.
- Tests on the corpus's decaying feedback structure (024b-04) and on the gain program.

## Acceptance criteria
1. For the decaying feedback program, the output downstream of the recursive structure carries the property in `INFO` and in the manifest data (C-189).
2. For the gain program, no output carries the property.
3. In a program with two outputs, one downstream of the recursive structure and one not, only the first carries the property.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The `OutputInfo` type (part 024a, 024a-01).
- Identifying frequency-dependent-delay nodes and their structural part (epic 023).
- Join padding and join flags (024b-06); writing the manifest (024b-12).
