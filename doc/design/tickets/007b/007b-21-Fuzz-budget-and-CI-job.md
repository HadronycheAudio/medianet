# 007b-21 — Fuzz budget and CI job
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** S · **Depends on:** 007a-15, 007b-20

## Context
- The fuzz budget is decided (C-95): each parser is fuzzed 60 s on every change and 1 hour nightly, and the corpus is kept. The falsifier recorded with it: a parser bug found later that a longer budget would have caught. It must be stated in `doc/design/spec/conformance.md`.
- The state-machine targets of 007b-20 run under the same budget. The fuzzing tool is the implementing team's choice (C-192).
- No fuzz target needs a privilege, so both jobs run unprivileged; the separate privileged nightly job of C-146 is not used by this epic.
- Epic criterion 007-AC7 (system criterion AC-1) requires zero parser panics and zero hangs within the fuzz budget.

## Work
1. Add a CI job triggered on every change that runs every parser and state-machine target for 60 s each, unprivileged, failing on any panic or hang and saving crashing inputs as artefacts.
2. Add a nightly CI job that runs every target for 1 hour each, with the same failure rules.
3. Keep the corpus between runs: each run starts from the stored corpus and stores the grown corpus back.
4. Pass the budget text (C-95) to 007b-23 for `conformance.md`.

## Acceptance criteria
1. Every target from 007a-15 and 007b-20 runs for 60 s per change and for 1 hour nightly.
2. The job fails when a deliberately panicking stub parser is fuzzed.
3. A second run starts with a corpus at least as large as the first run left.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The fuzz budget (time or executions per target per CI run) is stated in `conformance.md`
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Writing the rest of `conformance.md` (007b-23).
- Choosing the budget (settled by C-95).
