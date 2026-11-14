# 020-17 — Handover 50 ppm scenario
**Epic:** 020-L2-certification-harness · **Effort:** M · **Depends on:** 020-14, 020-16, 020-28

## Context
One L2 disturbance is **a clock handover between two holders whose crystals differ by 50 ppm (C-78)** `[decision]`. The servo requirement is a *frequency-continuous* handover between holders `[decision]`.

**AC-5 (hardware part)** of the epic: during the 50 ppm handover the steered ratio changes by at most **1 ppm/s (C-61)** and the marker offset shows **no step**. The handover is also one of the three disturbances in **AC-26**: zero slip after it, within ±1 sample on Linux (C-76; 1 sample = 10.4 µs at 96 kHz, C-01).

Other servo limits `[decision]`: slew at most 10 ppm from the holder's rate (C-60); published time continuous while locked (zero phase step); lock bound at most 25 µs (C-62).

Settled (C-212): the rig needs a second holder-eligible Linux node. Holder A is the in-process reference module (epic 017) that the sink follows; holder B, which takes over, runs on the second node through the `holder` mode of 020-28. Their 50 ppm crystal difference (C-78) is produced with the injection API (injected drift on B), set so that the injected drift plus the measured natural difference between the two nodes' crystals is 50 ppm; both parts are recorded in the run report, that is the certification record (C-223, 020-13). Per-sink emission error in holdover is an L2 assertion only, which this run makes (C-212, C-220).

The C-60 and C-61 limits bound the servo's correction term, not the drift estimate it tracks (C-119). The AC-5 bound applies to the steered ratio itself.

Settled (C-166): the no-step tolerance at the handover is ±1 sample on Linux and the platform restart spread elsewhere; offsets are compared over 10 s windows; a one-sample slip passes on Linux and is reported as a slip event. Settled (C-191): the handover is triggered 60 s into the run, and the 10 s window after it starts when the handover ends; an embedded device declares its own restart spread, measured at L3, so that tolerance is not this harness's to measure. Settled (C-202): L2 has no separate sync-loss scenario; this run gains one assertion: emission error stays within the target E (C-149) while the sink's clock is in holdover between holders. Settled (C-212): the handover is triggered as a holder loss: the old holder stops answering, the sink passes through about 2 s of holdover (election on reply silence 1.5 s, C-37, plus the 400 ms election window, C-33) and the new holder takes over. The run asserts emission error within the target E during holdover (C-202) and frequency continuity at takeover, that is a rate change within C-61 and no phase step; a run in which holdover was not observed is failed, not passed. Settled (C-213): a holder-eligible module may take over only if its time is usable (Locked, or in holdover with its alignment claim not lapsed, C-65, C-169); such a candidate takes over with frequency continuity and the timeline epoch is kept; the epoch is bumped (C-113) only when no candidate has usable time. B is therefore Locked when A stops answering, and an epoch bump at the handover means the run did not exercise a frequency-continuous takeover. Settled (C-221): among several candidates with usable time a Locked candidate is preferred over one in holdover, and among equals the lowest module id wins. After A stops answering B, like every other holder-eligible module on the rig, passes into holdover, so B is given the lowest module id among the rig's holder-eligible modules (forced-identity hook, C-120) to make it the winner. The holdover entry delay itself is pending simulation (epic 008b, C-195) and expected at or below C-37, so the sink is already in holdover when the election starts; the run therefore checks that holdover occurred, not its exact length. The means of making an in-process holder stop answering is an injection interface of epic 017 (the election rules belong to the control state machines, epic 005).

## Work
- Scenario `handover-50ppm` in `run.rs`: reference holder A in-process and reference holder B, holder-eligible, on the rig's second Linux node (020-28, C-212); before the run, measure the natural difference between the two nodes' crystals and inject drift on B so that the total difference is +50 ppm (sign configurable, C-78), recording both; B is given the lowest module id among the rig's holder-eligible modules (C-120, C-221); the sink follows A; the run does not trigger the loss until B reports Locked (usable time, C-213) and refuses to start if it does not; 60 s into a 1-hour run (C-191), A is made to stop answering through the epic 017 injection interface, so that the sink enters holdover and B takes over after the election (holder loss, C-212). Optionally a second holder loss later in the run (B stops answering, A takes over, A being Locked by then, C-213) exercises the reverse direction.
- Record the steered ratio at full resolution around the handover.
- Record the sink's clock state through the run (read access from epic 018) and the start and end of each interval in which the sink's clock is in holdover between holders.
- Checks:
  - frequency continuity at takeover (C-212): maximum rate of change of the steered ratio ≤ 1 ppm/s (C-61) through the holdover and the takeover by B, using 020-07;
  - epoch kept (C-213): the timeline epoch of the sink's stream is the same before and after the takeover; an epoch bump (C-113) fails the run;
  - no step in the marker offset at the handover: the offset's change across the handover is within ±1 sample on Linux, the platform restart spread elsewhere (C-166), and the offset over the 10 s window after, starting when the handover ends (C-191), equals the offset over the 10 s window before within the C-76 tolerance (C-166);
  - correction term within 10 ppm (C-60) and 1 ppm/s (C-61) throughout (C-119), using 020-07;
  - holdover (C-202): the emission error of every marker emitted while the sink's clock is in holdover between holders is within the platform target E (C-149), using 020-05;
  - holdover observed (C-212): if the recorded clock state shows no holdover interval between the loss of A and the takeover by B, the run fails.
- Report each check in the record (020-13) as the AC-5 result.

## Acceptance criteria
1. `hadronyche-l2 run handover-50ppm` runs 1 hour, with B holding the lowest module id among the rig's holder-eligible modules (C-221), triggers the handover 60 s into the run (C-191, checked in the recording), and reports maximum ratio rate of change, offset change across the handover, the C-76 verdict, and the holdover interval (start, end, length) with the largest emission error in it against the target E (C-202); the recording shows A stopping answering 60 s in and B, on the second Linux node, taking over after the holdover (C-212), with B Locked at the loss and the timeline epoch unchanged across the takeover (C-213).
2. On a simulated servo that steps its ratio instantly by 50 ppm, the scenario reports a rate of change above 1 ppm/s and fails AC-5.
3. On a simulated servo ramping at 0.5 ppm/s, the scenario passes the ratio check.
4. On a synthetic recording with a marked holdover interval, a marker in that interval whose emission error exceeds the target E fails the holdover check (C-202), while the same error outside the interval does not; a recording with no holdover interval fails the run (C-212), and is never reported as passed or as not exercised.
5. The 50 ppm difference (injected drift on B plus the measured natural difference between the two nodes' crystals, C-78) is verified in the recording within 0.1 ppm, and the certification record states both parts (C-223).
6. On a synthetic recording in which the epoch changes at the takeover, the run fails the epoch check (C-213); a run in which B is not Locked when the loss is due is refused, not run (C-213).
7. A shortened CI version passes unprivileged without audio hardware, with B in a second process (020-28).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Handover logic in the servo (epics 017, 018) and the election (epic 005).
- The second node's `holder` mode and its control link (020-28).
- Real certification run (020-22).
- Drift sweep (020-18).
