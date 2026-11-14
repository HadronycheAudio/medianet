# 025a-08 — Negative corpus: resampler placement and ratio
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** XS · **Depends on:** 025a-07, epic 023
## Context
Two rules settled after the ticket split add rejection categories to the negative corpus (acceptance criterion 11 of epic 025a):
- A resampler node may appear only inside an edge module; the compiler rejects it anywhere else (C-134).
- The kit's resampler node is offered only in designs whose delay is a whole number of samples, that is 2:1 and 1:2; other ratios are out of v1 (C-105). A program asking for 96→44.1 kHz conversion (ratio 147/320), or its inverse, must be rejected. The earlier rational-L rule, under which such chains were positive corpus, is withdrawn.

The negative-corpus harness support (a negative program passes only if compilation fails with exactly the expected diagnostic) comes from 025a-07. Diagnostic ids are defined by the epic 023 front end; if an id is missing when this ticket starts, file it against 023 rather than matching on message text.
## Work
1. Add at least 3 negative programs that place a resampler node in a non-edge module (for example a plain processing module, a mixer and an instrument), each expecting the epic 023 diagnostic for C-134.
2. Add at least 3 negative edge-module programs that request a resampler ratio other than 2:1 or 1:2 (96→44.1, 44.1→96 and one further ratio), each expecting the epic 023 diagnostic for C-105.
3. Add both categories to the negative corpus README and its category count.
## Acceptance criteria
1. At least 6 new negative programs, at least 3 in each of the 2 categories.
2. Every one is rejected with the expected diagnostic; the harness passes.
3. The same resampler node at 2:1 inside an edge module compiles (positive corpus, 025a-04), so the rejection is about placement and ratio, not the node itself.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Any missing diagnostic id is filed against epic 023 with an owner
## Out of scope
- The five original rejection categories (025a-07); positive resampler programs (025a-04); diagnostic wording and implementation (epic 023).
