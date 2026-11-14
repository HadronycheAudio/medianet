# 026b-22 — Design docs and constants update
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** XS · **Depends on:** 026b-01, 026b-04, 026b-05, 026b-06, 026b-07, 026b-15, 026b-14, 026b-19, 026b-13

## Context

Epic 026's definition of done requires design documents to be updated where behaviour differs, **constants table first** (`doc/design/02-constants.md`, C-130), and no open item left without an owner and a falsifier.

Already settled in the constants table and not to be re-decided here: stamp transport and the 10 s refresh (C-18), the alignment-claim margin rule (C-65), the definition of E (C-73, C-103, C-140), capability states (C-129), timestamp-source values (C-131), the device's ring sizing (C-96), strict mode on a device (C-147), control isolation (C-161), the margin with E unknown (C-169), sink liveness (C-168), resampler placement and D (C-134, C-170, C-207), the home of the rate-actuator trait (C-210), what an open commissioning window disqualifies (C-178), certification while unlocked (C-180), the stamp's 60 s figure window (C-182), the holdover entry delay (C-195, pending simulation), and a changed A on the device (C-186, C-198, C-211).

Items to reflect:
- Embedded devices are followers only; consequence for users: **a medianet needs at least one holder-capable (Linux) node, or there is no audio** `[decision]`.
- Device clock 48 kHz; the wire stays at 96 kHz (C-80, C-01).
- Measured outcomes of the open sub-questions owned by this epic: resampler CPU fit and `f64` cost (026b-05), bounded control handling (026b-06), timestamp source and jitter / reachability of the C-62 lock bound (026b-07), DSCP, don't-fragment and received-DSCP support (026b-02), reference board (026b-01), which is recorded in the constants table once the product owner approves it (C-172).
- Memory-fit figure (026b-04); filter length if shortened (026b-05); calibration factor of the device target (026b-19).
- New stamp fields of the embedded profile (026a-04), per-connection counters (026b-11), commissioning window (026a-13), and alignment figures and peer DSCP observations (026b-12).
- Device E value: produced by 026b-14, confirmed by task 027 (L3), falsifier: measured skew exceeds the declared sum.
- Collector record format and file names: decided by this epic in 026a-12 and recorded in the collector's documentation (C-206).
- Open items owned elsewhere, each recorded with owner and falsifier: D methods (epic 018), which of the 18 refusal reasons (C-185) a memory refusal carries (epic 012), commissioning-window message encoding (epic 005 with 004), how a domain id reaches a device (commissioning tooling with epic 005), how device alarms reach an operator (epic 013 with 005).

## Work

1. Update the constants table first with any new or changed figures measured in this epic (e.g. filter length, memory figure if it is promoted to a constant) and their evidence tags, and record the approved reference board (C-172).
2. Update the relevant design documents (clock, admission, regime stamp, roles) where device behaviour differs.
3. Add the user-facing note about the holder-capable node to the operator guide in `doc/operations/` (C-171).
4. Check every open item created during the epic has an owner and a falsifier.

## Acceptance criteria

1. The constants table lists every new constant introduced by tickets 026a-01 to 026a-13 and 026b-01 to 026b-22, each with an evidence tag; none duplicates or contradicts an existing C-id.
2. The "at least one holder-capable node" rule appears in the operator guide in `doc/operations/` (C-171).
3. The approved reference board is recorded in the constants table (C-172).
4. Each open sub-question of the epic has a recorded answer or an owner and falsifier.

## Definition of done

- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope

- The porting guide (026b-21).
