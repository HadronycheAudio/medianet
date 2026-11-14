# 008a-08 — Wall-clock model and stamp step detection
**Epic:** 008a-Servo-core-and-simulation-harness · **Effort:** S · **Depends on:** 008a-01, 008a-04
## Context
In the POC, the receive time is derived from a `CLOCK_REALTIME` kernel stamp `[kernel-doc]`; a wall-clock step between stamp and read corrupts the sample and is clamped to look like zero delay `[code]`. Requirement `[decision]`: a wall-clock step between kernel stamp and read is detected and the sample discarded.

Home of the detection (C-121) `[decision]`: the platform backend that builds each exchange from receive stamps; the servo only ever receives exchanges that passed it. The real backend is wired to sockets by epic 015. The check applies to the receive-timestamp source `kernel` (C-131), whose stamp is `CLOCK_REALTIME`; the sources `userspace`, `mac-hardware` and `interrupt` are not stamped on the wall clock. This ticket writes the detection as a reference function that the backend reuses, and a simulated backend that applies it, so case 9 (008b-10) can show that a wall-clock step does not reach the estimate.

## Work
1. A pure, `no_std`, allocation-free reference function (in `hadronyche-core`, outside the servo module, documented as backend-side per C-121) that converts a `CLOCK_REALTIME` receive stamp into the monotonic timebase using realtime and monotonic readings taken around the read, and returns `None` when the realtime/monotonic relation changed between them. Name and exact signature are for the implementer; document them.
2. In the `sim` module of `hadronyche-tools`: a `WallClockModel` giving `CLOCK_REALTIME` = monotonic + offset, where the offset can step at scheduled simulated instants (positive and negative, any size), including between a packet's kernel stamp and its read.
3. In the `sim` module of `hadronyche-tools`: a simulated backend receive path, with the stamp source as a scenario parameter (C-131 values), that for the `kernel` source calls the function and builds an `Exchange` only from samples that pass; discarded samples are counted and never reach `Servo::feed`.

## Acceptance criteria
1. Unit test: no step → converted stamp equals the monotonic receive time exactly.
2. Unit test: steps of +1 s, −1 s, +1 µs and −1 µs inserted between stamp and read are each detected (`None`).
3. Sim test: a step scheduled between stamp and read of one packet causes exactly that one sample to be discarded and counted.
4. The servo module does not call the function (the check is backend-side, C-121); the function builds for the 008a-01 no_std target.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Case 9 end-to-end assertion on the estimate (008b-10).
- Real socket timestamping and the production backend (epic 015).
