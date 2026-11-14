# 025a-21 — Negative corpus: even-tap linear-phase FIR and device-side joins
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** XS · **Depends on:** 025a-07, epic 023
## Context
Two rules settled after the ticket split add rejection categories to the negative corpus (acceptance criterion 6 of epic 025a):
- An even-tap linear-phase FIR, whose centre falls between two samples, is rejected by the compiler in v1, with a diagnostic suggesting an odd-tap design, so that L stays a whole number of domain-rate samples (C-183, C-105). The positive corpus uses odd tap counts only (025a-03).
- A join on the device side of an edge module: paths that differ by an odd number of domain-rate samples cannot be padded with a whole-sample delay line at the device rate, so the compiler rejects the join and suggests the per-join opt-out (C-190, C-104). An edge whose hardware runs at 48 kHz converts 2:1 through the resampler node (C-80, C-134).

The negative-corpus harness support (a negative program passes only if compilation fails with exactly the expected diagnostic) comes from 025a-07. Diagnostic ids are defined by the epic 023 front end; if an id is missing when this ticket starts, file it against 023 rather than matching on message text.
## Work
1. Add at least 2 negative programs containing an even-tap linear-phase FIR (float and fixed), each expecting the epic 023 diagnostic for C-183; the expect file or program comment records that the diagnostic suggests an odd-tap design.
2. Add at least 2 negative edge-module programs with a join on the device side of the resampler node whose paths differ by an odd number of domain-rate samples, each expecting the epic 023 diagnostic for C-190; the expect file or program comment records that the diagnostic suggests the per-join opt-out.
3. Add, as positive corpus, the same device-side join with the opt-out construct on the join, with an `expect/` file stating `opt-out` and no inserted delay at that join.
4. Add both categories to the negative corpus README and its category count.
## Acceptance criteria
1. At least 4 new negative programs, at least 2 in each of the 2 categories.
2. Every one is rejected with the expected diagnostic; the harness passes.
3. The opt-out variant of the device-side join compiles and passes the 025a-01 harness with no inserted delay at that join, so the rejection is about padding at the device rate, not the join itself.
4. The odd-tap FIR programs of 025a-03 still compile, so the rejection is about the even tap count, not the FIR.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Any missing diagnostic id is filed against epic 023 with an owner
## Out of scope
- Odd-tap FIR programs (025a-03); joins at the domain rate (025a-06); resampler placement and ratio rejections (025a-08); diagnostic wording and implementation (epic 023).
