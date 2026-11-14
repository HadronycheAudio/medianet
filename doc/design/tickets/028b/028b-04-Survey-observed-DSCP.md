# 028b-04 — `survey`: DSCP as observed by peers
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028b-02

## Context
Switch QoS uses three DSCP classes (C-13): sync **CS6 (48)**, media and one-way probes **EF (46)**, control **CS5 (40)**. Marking is verified by the receiving peer and reported. On an unmanaged switch marking does nothing, and the tool must say so.

Settled: producer and consumer each publish per-connection summary counters in their own regime stamp, and these include the **observed DSCP** of the media (C-47). For the other classes (C-152): the holder reports in its stamp the DSCP observed on each follower's sync requests, and every module reports the DSCP observed on each peer's hellos (control). Receiver reports are not visible to this tool and there is no query message, so the stamps are the only source, and they cover all three classes.

## Work
- Finding `dscp.mismatch`: compare the observed DSCP with the expected value: media per connection from the receiver's stamp (46, C-47); sync per follower from the holder's stamp (48, C-152); control per peer pair from the hello observations (40, C-152). Report mismatches naming the sender, the observer, the class, expected and observed.
- Finding `dscp.unknown`: no observed DSCP available for a connection, follower or peer.
- Notice `switch.unmanaged`: when the configuration or operator flag `--unmanaged-switch` says the switch is unmanaged, print "marking has no effect on an unmanaged switch" once, and downgrade DSCP findings to informational.

## Acceptance criteria
1. A fixture where sync traffic arrives with DSCP 0 yields `dscp.mismatch` (expected 48, observed 0).
2. Correct marking for all three classes yields no DSCP finding.
3. With the unmanaged-switch flag set, the notice appears and the run exits 0 if DSCP findings are the only ones.
4. A fixture where one peer's hellos arrive with DSCP 0 at another module yields `dscp.mismatch` for the control class (expected 40).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented

## Out of scope
- Switch configuration recommendations (028b-19, 028b-20).
