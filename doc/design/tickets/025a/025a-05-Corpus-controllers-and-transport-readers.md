# 025a-05 — Corpus: controller instruments and transport readers
**Epic:** 025a-DSL-conformance-corpus-probe-and-reference-programs · **Effort:** S · **Depends on:** 025a-01
## Context
Part of the conformance corpus in `hadronyche-dsl/conformance/corpus/`, which must reach **at least 40** programs across 025a-02 to 025a-06, in float and fixed variants.

Content domains v1: **audio; dense controllers; sparse keyed controllers; read-only transport state**. Transport authoring is absent (writing transport is a negative-corpus case, 025a-07).

Gap defaults per content type (overridable): audio zero-fill; dense controllers hold last value; sparse keyed controllers rely on state refresh; read-only transport state holds the last known state and advances the playhead as that state dictates: a stopped transport stays where it is, a playing one moves on at its tempo; the output is marked degraded (C-141, C-200). Before any transport state has arrived the state is stopped, playhead at zero, tempo 120 beats per minute (C-184, C-199). Each dense controller declares an initial value in the manifest (default 0), used when a gap precedes any received value (C-158). Degraded marking per output comes from dataflow and follows the content: an output at p is degraded if its input at p − L was gapped (C-122).

Every accepted DSL program is `exact-fixed` or `exact-float` (C-123). L is a whole number of domain-rate samples (C-105).

Still open: the refresh interval of sparse keyed controller state (wire specification epic 004). The programs here do not depend on it; 025a-16 tests the gap rules.

Float and fixed variants of a corpus program count as one program towards the 40 (C-192).
## Work
Add at least 8 programs with `expect/` files and hand derivations:
1. Sparse-controller instruments: a keyed-voice instrument driven by a sparse keyed controller stream, float and fixed.
2. Dense-controller consumers: a gain driven by a dense controller, float and fixed; one of them declares a non-default initial value (C-158), stated in its expect file.
3. Transport readers: a program reading read-only transport state, including the play/stop state, tempo and playhead position, into an output, so 025a-16 can test C-141, C-200 and C-184 with both a stopped and a playing transport.
4. One program overriding a gap default, so 025a-16 can test overrides.
5. After this ticket, count the corpus, float and fixed variants of one program counting as one (C-192); if it is below 40, add further distinct programs (not further variants of existing ones) here until it reaches 40.
## Acceptance criteria
1. At least 8 new programs with `expect/` files and hand derivations.
2. The 025a-01 harness passes on all of them.
3. `corpus/` contains at least 40 positive programs in total, float and fixed variants of one program counting as one (C-192), with at least one float and one fixed variant in each family listed above where the kit allows; the corpus README states the count under that rule.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
- Testing the gap defaults themselves (025a-16); transport authoring rejection (025a-07).
- Controller placement in streams and the bundle cap (C-181, C-205, C-209): 025a-22.
