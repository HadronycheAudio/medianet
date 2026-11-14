# 008b-09 — Case 08: achieved request rate
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** S · **Depends on:** 008a-03, 008a-15
## Context
Simulation case 8: achieved request rate equals the configured rate within 2 % over the whole accepted range, 10 to 200 per second (C-112). Epic criterion A6 (system AC-7).

POC: pacing by a 5 ms socket receive timeout gave 37 exchanges/s at a configured 50 and 77 at a configured 1 000 on a `CONFIG_HZ=250` kernel `[measured]` (P-02, P-03). Rule `[decision]`: requests are paced by deadline; the achieved rate is reported. Default rate 50 per second (C-36).

## Work
1. Named test `case_08_request_rate`: configured rates 10, 20, 50, 100, 150 and 200 per second; 60 s each, with a simulated scheduler that wakes the follower late by a jitter modelled on a `CONFIG_HZ=250` timer (4 ms tick granularity) and random preemption.
2. Compare achieved (counted requests and the pacer's reported rate) to configured.

## Acceptance criteria
1. Achieved rate within 2 % of configured at every tested rate.
2. Reported rate equals counted rate within 0.1 %.
3. The test fails if pacing is switched to a receive-timeout scheme (demonstrated once by a local variant; not merged).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Real kernel timers and sockets (epic 015).
