# 017b-10 — Scenario `baseline`: offset, drift and sync-rate invariance
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** M · **Depends on:** 017b-02, 017a-09, 017b-09, epic 008a, epic 014

## Context
- AC-8 (trustworthy smoke), first half: with injected offset and drift, zero content mismatches on the exact tier; estimated minus true offset within the published bound; **no assertion changes when the sync rate is changed from 10 to 200 per second**. The loss half of AC-8 is 017b-11.
- Rules `[decision]`: no assertion depends on sync-rate timing — the POC achieved 37 exchanges/s at a configured 50 and 77/s at 1 000 on a 250 Hz kernel `[measured]` (P-02, P-03), so rates and timeouts are not test oracles. Never assert latency thresholds on loopback; assert content, loss accounting, lock, classification and ground-truth error.
- This ticket is where the L0 baseline scenario is defined (C-192); other epics that cite it refer to this definition.
- **Published bound (settled):** the lock bound, 25 µs (C-62, pending simulation). Assertions cite C-62 by id so they follow any change. The lock state and the published bound are read from the servo core's published time view (epic 008a, C-187).
- **Sync rate range (settled, C-112):** 10 to 200 per second is the accepted range; AC-8 is checked at both ends. Requests beyond 200 per second from one follower are dropped before any work and counted per follower in the holder's stamp (C-204), so at both ends that count must be zero.
- Injected drifts of tens of ppm are legitimate: the slew limits C-60 and C-61 bound the servo's correction term, not the drift estimate it tracks (C-119).
- Clock injection: `ClockInjection { offset_ns, drift_ppb }` per module (017b-02).
- **DSCP observation (settled, C-152, with C-13 and C-47):** the holder reports in its stamp the DSCP observed on each follower's sync requests (CS6, 48), every module reports the DSCP observed on each peer's hellos (control, CS5, 40), and consumers report the observed DSCP of media (EF, 46). The holder answers sync requests and records their DSCP on its clock thread (C-188). The values are read from the stamps, not timed, so they are valid L0 assertions.

## Work
1. Implement scenario `baseline` in `hadronyche-l0`: sender, echo, mixer; inject distinct offsets (e.g. +5 ms, −3 ms, 0) and drifts (e.g. +50 000 ppb, −20 000 ppb, 0) on the three modules.
2. Assertions: exact-tier mismatches = 0 outside excluded windows; for each module, |estimated offset − true offset| ≤ C-62 for every sample after lock (estimates from receiver reports, epic 014, truth from the injector's accessor); lock reached (lock state from the time view, C-187); the holder's per-follower over-rate sync request count is zero (C-204); required families `audio`, `music`, `transport` non-zero; the C-47 counters show zero missing, late and marked-gap slices; the stamps report observed DSCP CS6 on sync requests, CS5 on hellos and EF on media (C-13, C-152).
3. Add a test that runs `baseline` at `--sync-rate 10` and `--sync-rate 200` and checks the set of assertion outcomes is identical.

## Acceptance criteria
1. `hadronyche-l0 baseline --duration 60` exits 0 with the injections above.
2. Exact-tier mismatches = 0; every family has > 0 compared windows.
3. Max |estimated − true offset| after lock ≤ 25 µs (C-62).
4. Runs at sync rates 10/s and 200/s produce identical pass/fail for every assertion (AC-8, C-112), and in both the holder's stamp counts zero over-rate sync requests for every follower (C-204).
5. No assertion in the scenario references a latency or a sync-exchange count.
6. The holder's stamp reports DSCP 48 for every follower's sync requests, every module's stamp reports DSCP 40 for every peer's hellos, and each consumer reports DSCP 46 for its media inputs (C-13, C-152).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Regime stamp fields touched are complete and documented

## Out of scope
- Packet loss and its accounting (017b-11).
- Cross-machine runs (017a-10).
