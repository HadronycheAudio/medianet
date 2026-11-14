# 025a-22 — Corpus: controller placement and the bundle cap
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** S · **Depends on:** 025a-05, 025a-07, epic 004, epic 023
## Context
Three rules settled after the ticket split make controller placement a compile-time promise of the DSL compiler, so this suite must test it:
- A bundle source's controllers always ride in their own member stream (C-205, C-181).
- That controller member counts towards the bundle cap of 32 members (C-08): a source with controllers has at most 31 audio members (186 channels at 6 channels per stream, C-04). A wider source with controllers is rejected at compile time (C-209).
- A single-stream source's controllers share the audio packet while headers plus controllers stay within the 100-byte cap (C-137); beyond that the source emits a second stream for them. The compiler checks this from the manifest (C-181).

The hand derivations of the single-stream cases use the controller and header sizes of the wire specification (epic 004) and the 100-byte cap; they invent no size. The negative-corpus harness support (a negative program passes only if compilation fails with exactly the expected diagnostic) comes from 025a-07. Diagnostic ids are defined by the epic 023 front end; if an id is missing when this ticket starts, file it against 023 rather than matching on message text.

Every accepted DSL program is exact-fixed or exact-float (C-123); L is a whole number of domain-rate samples (C-105).
## Work
1. Extend the expect format and parser of 025a-01 with a `controllers` field for a program that emits controllers: `shared` (in the audio packet), `own-member` (bundle source, with the member count including the controller member) or `second-stream`; the parser rejects `shared` or `second-stream` on a bundle source (C-205). Update the format's README and its `doc/design/spec/` page.
2. Extend the harness to compare the field with the controller placement and member count the manifest reports.
3. Add positive programs with `expect/` files and hand derivations: a bundle source with controllers and 31 audio members (`own-member`, 32 members, C-209); a smaller bundle source with controllers (`own-member`); a single-stream source whose headers plus controllers fit the 100-byte cap (`shared`); a single-stream source whose controllers exceed it (`second-stream`) (C-181, C-137).
4. Add at least 2 negative programs: a source with controllers and 32 audio members, and one wider still, each expecting the epic 023 diagnostic for C-209.
5. Add the category to the negative corpus README and its category count.
## Acceptance criteria
1. At least 4 new positive programs and at least 2 new negative programs, each with an `expect/` file and a hand derivation.
2. The 025a-01 harness passes on all of them; changing one expected `controllers` value fails it, naming the program and the field.
3. The 31-audio-member source with controllers compiles with 32 members, its controllers in their own member stream; the 32-audio-member and wider sources with controllers are rejected with the expected diagnostic (C-209, C-08).
4. The single-stream programs place their controllers in the audio packet and in a second stream exactly as the hand derivation from the 100-byte cap predicts (C-181, C-137).
5. No bundle-source program shares its controllers with an audio member (C-205).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed (the extended expect parser keeps the 025a-01 fuzz target, C-95)
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Any missing diagnostic id is filed against epic 023 with an owner
- [ ] Any compiler disagreement found is filed against epic 023/024 with an owner, not patched by editing the expectation
## Out of scope
- The wire layout of controllers and the member stream (epic 004); the bundle runtime (other epics); controller gap defaults (025a-16).
