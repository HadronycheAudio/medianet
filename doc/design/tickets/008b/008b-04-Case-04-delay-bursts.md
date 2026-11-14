# 008b-04 — Case 04: delay bursts
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** XS · **Depends on:** 008a-15
## Context
Simulation case 4: delay bursts do not pull the estimate. The servo uses, per 200 ms window, only the exchange with the smallest round trip (C-66) `[decision]`; bursts raise round trip and should lose that selection. `WIRED_PROFILE` (C-94) already contains 1 % of packets delayed by 1 ms; this case adds longer bursts on top. Slew limits on the correction term while Locked: 10 ppm (C-60), 1 ppm/s (C-61), per C-119; lock bound 25 µs (C-62).

The burst lengths also inform the holdover entry delay, constant C-195: it is at most the 1.5 s election silence threshold (C-37), a hard limit, and its value within that limit is pending simulation, proposed by this epic. Report whether any burst pushes a Locked run into holdover at the configured `holdover_after_ns`.

## Work
1. Named test `case_04_delay_bursts`: after lock under `WIRED_PROFILE`, inject bursts of +100 µs, +1 ms and +10 ms extra delay lasting 0.5 s, 2 s and 10 s (symmetric and one-direction-only).
2. Compare true error during and after bursts with a burst-free run of the same seed.
3. Record each holdover entry caused by a burst, for 008b-05.

## Acceptance criteria
1. For symmetric bursts, |true error| stays ≤ the burst-free run's max |true error| + 1 µs.
2. For any burst, 0 honest-lock violations and C-60/C-61 respected on the correction term while Locked.
3. Bursts long enough to prevent any clean exchange either keep the estimate unchanged or lead to holdover; never a jump.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Coalescing bias (008b-14).
