# 020-28 — Second holder node for the handover
**Epic:** 020-L2-certification-harness · **Effort:** M · **Depends on:** 020-09, 020-14, 020-27, epic 017

## Context
The L2 and L3 handover is triggered as a holder loss: the old holder stops answering, the sinks pass through about 2 s of holdover (C-37 plus C-33) and the new holder takes over; the rig therefore needs a second holder-eligible Linux node (C-212). Who may take over: a holder-eligible module whose time is usable, Locked or in holdover with its alignment claim not lapsed (C-65, C-169); it takes over with frequency continuity and the timeline epoch is kept; the epoch is bumped (C-113) only when no candidate has usable time (C-213).

In the handover scenario (020-17) holder A is the in-process reference module the sink follows; holder B, which takes over, runs on the second node. The runner on the first machine must start B, inject its drift (the 50 ppm difference of C-78: the measured natural difference between the two holders' crystals plus an injected drift, both recorded in the run report, C-223), force its module id to the lowest among the rig's holder-eligible modules so that it wins the election (C-120, C-221), know when B is Locked, and collect B's ground truth and regime stamp: after the takeover B is the holder the sink follows, and the record carries the regime stamps of the sink and of the holder it followed (C-191). Runs whose numbers enter the constants table run in strict mode (C-147), which applies to B as to A (020-27).

## Work
- A `hadronyche-l2 holder` mode, run on the second node: starts a holder-eligible reference module (epic 017 reference source with its injection API, test-only hooks of C-120) on the rig's Medianet, with strict mode set or not as the runner requests (C-147, C-165).
- A control link from the runner to the `holder` mode over the rig's network (address from `holder_node` in `rig.toml`, 020-09), with commands: start, force module id (C-120), inject drift, report clock state, stop, and fetch ground truth and regime stamp. Every injection is stamped (C-120). The control link is the harness's own and is documented in `run.rs`.
- Measurement of the natural difference between the two nodes' crystals before the run (B following A), so that 020-17 sets B's injected drift to make the total difference 50 ppm (C-78); the measured difference and the injected drift are both written to the recording side channel (020-02) and to the certification record (020-13, C-223).
- Report B's clock state to the runner continuously, so the runner knows B has usable time (Locked) before it makes A stop answering (C-213).
- After the run, fetch B's injected ground truth and regime stamp into the recording and the record's `regime_stamps` block (020-13; C-191: the holder the sink followed at the end of the run).
- A CI test running `holder` as a second process on one machine, unprivileged, exercising the control link.

## Acceptance criteria
1. `hadronyche-l2 holder` on the second node starts a holder-eligible reference module; the runner on the first machine starts it, injects drift and stops it over the control link, and each injection appears stamped in the recording (C-120).
2. The runner reads B's clock state and does not report B ready until B is Locked (C-213).
3. The recording and the certification record carry the measured natural crystal difference and B's injected drift, and their sum is the 50 ppm requested by 020-17 (C-78, C-223).
4. B runs with the module id forced by the runner, lower than that of every other holder-eligible module on the rig (C-120, C-221).
5. After a handover run, the record's `regime_stamps` block holds the sink's stamp and B's stamp fetched from the second node (C-191).
6. With strict mode requested, `holder` refuses to start if real-time scheduling or memory lock is absent or unverified on the second node (C-127, C-128).
7. The two-process CI test passes unprivileged without audio hardware.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Regime stamp fields touched are complete and documented — not applicable (C-125): this ticket reads B's regime stamp and adds no regime stamp field

## Out of scope
- The handover scenario and its checks (020-17).
- Building the second node into the reference rig (020-21).
- The election and takeover rules (epic 005) and the reference source with injection (epic 017).
