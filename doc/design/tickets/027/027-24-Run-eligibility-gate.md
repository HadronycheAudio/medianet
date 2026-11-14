# 027-24 — Run eligibility gate: strict mode and certification state
**Epic:** 027-L3-cross-sink-rig · **Effort:** XS · **Depends on:** 027-07, 027-22

## Context
- Any run whose numbers enter the constants table as design constants runs in strict mode (C-147). L3 writes C-79 and the device's E back to the table, so every rig run must be strict.
- Strict mode requires real-time scheduling granted on the data and clock threads, and memory locked (C-127); a capability whose state is unverified is treated as absent (C-128). Capability states are present, absent, unverified, not requested (C-129).
- Strict mode is hosted-only: the device stamps real-time scheduling and memory lock as present by construction (C-147). Strict mode is set in the operator policy of the commissioning configuration (C-165).
- Only certification state commissioned gives a numeric E; uncertified and self-monitored sinks declare E as unknown (C-140, C-103). Both sinks must be judged against a numeric E.

## Work
1. In `run.rs` of `hadronyche-l3`: `pub fn check_eligibility(linux: &SinkStamp, device: &SinkStamp) -> Eligibility` returning pass or a list of named reasons: real-time scheduling not present, memory lock not present (unverified counts as absent, C-128), certification state not commissioned (C-140).
2. The runner (027-11) calls it after each stamp fetch at scenario start and after every restart or power cycle (027-12, 027-13); an ineligible run stops before capture, and an ineligible re-check marks the affected restart as failed with the reason.
3. Write the eligibility result into the scenario report (027-07 field).
4. Unit tests on synthetic stamps for each reason.

## Acceptance criteria
1. A Linux stamp with real-time scheduling or memory lock in state absent, unverified or not requested makes the gate fail with the matching reason (C-127, C-128).
2. A stamp with a certification state other than commissioned makes the gate fail with the C-140 reason.
3. A device stamp with both capabilities present and certification state commissioned, and a Linux stamp with the same, passes.
4. Every scenario report shows the eligibility result.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented (strict-mode capability states and certification state, as read)
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — reads `SinkStamp` values parsed by 027-07 and 027-22, adds no parser; "Design documents updated" — implements C-147, C-127, C-128 and C-140 as written; "No new open item" — opens none; "Every new factual claim carries an evidence tag" — no factual claim; "The skew analysis runs in CI on recorded two-channel files" — no skew analysis; "The rig is photographed or diagrammed" — 027-09 documents how strict mode is set; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Granting real-time scheduling or memory lock on the host (rig procedure, 027-09; Linux sink, epic 018).
- Certifying the sinks (epics 020, 026).
