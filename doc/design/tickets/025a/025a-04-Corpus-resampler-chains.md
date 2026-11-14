# 025a-04 — Corpus: resampler chains
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** S · **Depends on:** 025a-01
## Context
Part of the conformance corpus in `hadronyche-dsl/conformance/corpus/` (at least 40 programs in total across 025a-02 to 025a-06, float and fixed variants).

Settled rules:
- The wire carries one rate only, the domain rate of 96 kHz; an edge whose hardware runs at another rate converts at the edge (C-01). An edge with 48 kHz hardware converts 2:1 through the kit's resampler node (C-80).
- Structural latency L is always a whole number of domain-rate samples (C-105). The kit's resampler node is offered only at 2:1 and 1:2, whose delay is a whole number of samples; other ratios, 96↔44.1 kHz among them, are out of v1 (C-105). The earlier rational-L rule is withdrawn, so the former 96→44.1 and 44.1→96 chains are removed from this ticket; a resampler at such a ratio is a negative-corpus case (025a-08).
- A resampler node may appear only inside an edge module; the compiler rejects it anywhere else (C-134). Every program in this ticket is an edge module; a non-edge resampler is a negative-corpus case (025a-08).
- An edge's resampler delay is counted only in its plumbing delay D and never in L; L is the module's structural delay in the domain-rate timeline, and the manifest reports the conversion delay separately, on the device-facing port (C-207, C-134, C-170). The expect file therefore states L without the resampler delay and the conversion delay in the `conversion_delay` field (025a-01), in whole samples.
- Slice = 48 frames = 0.5 ms at 96 kHz (C-01, C-02).
- Open sub-question owned by this epic: **is the resampler node exact-fixed in its fixed-point form?** (falsifier: bits differ across targets). Resolved in 025b-09; this ticket only records the compiler's assigned class.
## Work
1. Add at least 6 programs with `expect/` files and hand derivations, all written as edge modules: 96→48 and 48→96, each float and fixed; a round trip 96→48→96; a resampler followed by a FIR.
2. For each, derive by hand the conversion delay from the resampler node's documented delay, as a whole number of samples, and L from the module's other structural delays only, as a whole number of domain-rate samples (C-207).
3. Record the class the compiler assigns to the fixed-point resampler programs.
## Acceptance criteria
1. At least 6 new programs with `expect/` files and hand derivations.
2. The 025a-01 harness passes on all of them.
3. Every expect file states L as a whole number of domain-rate samples excluding the resampler delay (C-105, C-207), and the conversion delay on each device-facing port; the harness checks both against the manifest.
4. No program uses a ratio other than 2:1 or 1:2, and none places a resampler node outside an edge module (C-105, C-134).
5. A program that changes rate without the resampler node is not in this ticket (it is negative corpus, 025a-07).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Probe-measured delay for these programs (025a-15); cross-target bit-exactness of the fixed resampler (025b-09); the expect parser (025a-01); rejection of other ratios and of non-edge resamplers (025a-08).
