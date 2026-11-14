# 028b-13 — Decide on-site capture hardware for `certify-group`
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** epic 027

## Context
Open sub-question owned by this task (still open after the design rulings): what on-site capture hardware are operators expected to own for `certify-group`? Falsifier: no affordable device has stable inter-channel skew.

`certify-group` drives a two-channel capture for a pair of sinks (the **L3 procedure**, epic 027); pass = the 99.9th percentile of absolute skew is at most the sum of the two sinks' E values, for 1 hour and across restarts of either, with the maximum reported beside it (C-79). E targets: 50 µs Linux (C-70), 100 µs macOS (C-71), 1 ms Windows (C-72); the pass criterion uses these targets, never the E measured in the same run (C-149). L3 parameters follow C-166 (comparison windows of 10 s; measurement window after a restart 60 s). The tightest pair (two Linux sinks) needs a skew budget of 100 µs, so the capture device's own inter-channel skew must be stable well below that over an hour.

## Work
- Measure inter-channel skew stability of at least two candidate affordable two-channel capture devices (the one used in the L3 rig plus one alternative) using the L3 procedure, over at least 1 hour each.
- Run the measurements in strict mode, since their result enters the constants table (C-147).
- Write a report in `doc/design/reports/` (C-130) with the recommended device list and minimum specification, `[measured]` evidence.
- Propose the device(s) to the product owner for approval and, once approved, record them in the constants table, as for reference hardware (C-172).
- Update the epic's open sub-questions table.

## Acceptance criteria
1. At least two devices measured; each has its inter-channel skew stability over 1 hour reported in µs (p99.9 and max).
2. The report names the recommended device(s) and the minimum specification, or records the falsifier outcome.
3. The approved choice is recorded in the constants table (C-172).

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The `certify-group` command (028b-14).
