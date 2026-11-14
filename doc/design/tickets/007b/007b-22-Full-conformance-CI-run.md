# 007b-22 — Full conformance CI run
**Epic:** 007b-Scenario-and-reference-vectors · **Effort:** S · **Depends on:** 007a-10, 007a-12, 007a-13, 007a-14, 007b-13, 007b-14, 007b-15, 007b-16, 007b-19, 007b-21, 007b-24

## Context
- Epic criterion 007-AC7 (system criterion AC-1): 100 % of vectors pass in both directions, malformed cases included; zero parser panics and zero hangs within the fuzz budget (C-95).
- Epic criterion 007-AC8 (system criterion AC-23): malformed, wrong-version and foreign-domain inputs produce no crash and no state change; each is counted per input and per reason; a version mismatch produces a named event.
- The core implementation that will be the real `UnderTest` is built in epics 009–014. The epic states that within epic 007 the criterion is met against an adapter around the generator's own table-driven codec (007a-05), and the same run is the entry point those epics must pass.

## Work
1. Add one CI entry point that, given an `UnderTest` implementation, runs all wire (valid both directions including the regime stamp messages, invalid, AC-23, dedup), control (including holder stall, admission details, alignment health, connection alarms, reassembly, stamp distribution, commissioning window and live A change scenarios), timeline, transport (including the gap-semantics vectors of 007b-16) vectors, then the fuzz job, and prints a pass/fail report per kind.
2. Wire it to an adapter around the generator's codec for the wire kinds, as the self-check available in this epic.
3. Document the single command epics 009–014, 025 and 026 must call.

## Acceptance criteria
1. The report lists counts per vector kind and fails on any single failure.
2. Against the generator-codec adapter, 100 % of wire vectors pass in both directions.
3. The job runs unprivileged and is green.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed

## Out of scope
- Making a real core pass (epics 009–014); generated code (epic 025); devices (epic 026).
