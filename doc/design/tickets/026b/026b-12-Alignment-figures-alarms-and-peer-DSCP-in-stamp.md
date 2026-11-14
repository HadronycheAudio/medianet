# 026b-12 — Alignment figures, alarms and peer DSCP in stamp
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** S · **Depends on:** 026a-08, 026a-10, 026a-11, 026b-02, epics 009, 013

## Context

Rules settled after the last refinement add consumer-side figures that every module publishes in its regime stamp. The device is a consumer with aligned inputs, so its stamp must carry them:

- Lateness percentiles in the stamp: 50th, 99th, 99.9th and the maximum (C-160).
- Window of these figures (C-182): the percentiles, the reclaimable latency and the cannot-meet-A alarm use a rolling 60 s window; the maximum since the connection went live is reported beside them.
- While a commissioning window is open these production figures are unqualified (C-178); the stamp carries the window state beside them (026a-13).
- Reclaimable latency: offset − (current maximum lateness + headroom), when positive; published in the stamp's alignment group (C-157). Headroom is the C-52 rule.
- Peer verification of sync and control DSCP (C-152): the holder reports the DSCP it observes on each follower's sync requests; every module reports the DSCP it observes on each peer's hellos. The device reports, per peer in its 63-entry map (C-34), the DSCP of that peer's hellos; the holder's report on the device's sync requests is what verifies the device's own marking.
- Alarms (C-173): a connection alarm is raised on refusal, interrupted, dead, demoted, cannot-meet-A and control-degraded. "Cannot meet A" is maximum lateness + D + headroom exceeding A in each of 10 consecutive 1 s windows (C-174); D is the device's declared plumbing delay (C-170, 026a-10). The control-degraded alarm and the ControlDegraded refusal (C-185) arise when housekeeping has stopped restarting (C-139); on the device any fault is a whole-device reset, so neither arises here.

Core computes lateness, headroom and the alarm conditions (epic 009); the stamp fields are those of epic 013. Reading received DSCP depends on the network stack (026b-02): when `rx_dscp_observation` is not present, the per-peer DSCP fields carry that C-129 state instead of a value. How a device's alarms reach an operator, given it has no files or local log, is not settled: open item, owner epic 013 with epic 005, falsifier "a device alarm is raised but no operator can see it". This ticket raises the alarms through core's interface and publishes whatever stamp field epic 013 defines for them.

The stamp is sectioned, multicast on change and every 10 s, unicast on request (C-18); every section fits 1 252 bytes (C-17).

## Work

1. Per admitted input, take core's lateness percentiles (C-160) and reclaimable latency (C-157) over the rolling 60 s window, plus the maximum since the connection went live (C-182), and publish them in the stamp's alignment group (026a-11); fixed-size storage, no allocation.
2. Per peer in the 63-entry peer map (C-217), record the DSCP observed on its hellos (C-152) when `rx_dscp_observation` is present; otherwise publish the C-129 state.
3. Feed the device's declared D to core's cannot-meet-A detection (C-174) and raise the C-173 connection alarms through core's alarm interface; publish them as epic 013 defines.
4. Check that the stamp with 64 peers' DSCP entries still splits into sections of at most 1 252 bytes.

## Acceptance criteria

1. Host test: a scripted lateness distribution yields the expected 50th, 99th, 99.9th percentiles and maximum in the decoded stamp (C-160). A lateness spike older than 60 s has left the percentiles and the reclaimable latency but is still the reported maximum since the connection went live (C-182).
2. Host test: reclaimable latency equals offset − (maximum lateness + headroom) when positive and is absent or zero otherwise, as epic 013 encodes it (C-157).
3. Host test: with 63 simulated peers sending hellos at known DSCP values, the decoded stamp lists each peer's observed DSCP; with `rx_dscp_observation` absent, each entry carries that C-129 state (C-152).
4. Host test: maximum lateness (over the rolling 60 s window, C-182) + D + headroom above A in 10 consecutive 1 s windows raises cannot-meet-A; 9 windows do not (C-174). Refusal, interrupted, dead and demoted each raise a connection alarm (C-173).
5. Every encoded section is at most 1 252 bytes with 64 peers present (C-17).
6. On the board, the holder's stamp shows the DSCP it observed on the device's sync requests (C-152), and the collector (026a-12) records the device's new fields.
7. Any new stamp parser path is fuzzed at the C-95 budget; device build has no alloc / libm symbols.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The crate builds for at least one device target and for the host (for tests) in CI
- [ ] No new open item without an owner and a falsifier

## Out of scope

- Computing lateness, headroom and alarm conditions (epic 009, core).
- Per-connection counters (026b-11).
- The channel by which alarms reach an operator (epic 013 with 005).
