# 023-11 — 2:1 / 1:2 resampler node and its whole-sample delay
**Epic:** 023-DSL-compiler-front-end · **Effort:** XS · **Depends on:** 023-09, epic 024a

## Context
- Multi-rate `[decision]`: the domain rate, 96 kHz, is the only rate on the wire (C-01). A resampler node may appear **only inside an edge module**; the compiler rejects it anywhere else (C-134; the check is 023-29). An edge whose hardware runs at 48 kHz converts 2:1 through the kit's resampler node and the wire stays at the domain rate (C-80). The compiler knows the node's delay exactly.
- **Structural latency L is always a whole number of domain-rate samples** (C-105). The kit's resampler node is offered **only in designs whose delay is a whole number of samples: 2:1 and 1:2; other ratios are out of v1** (C-105). The earlier rational-L rule is withdrawn: there is no non-integer ratio, no rational delay and no fractional delay line.
- An edge's conversion delay is counted only in its plumbing delay D and never in L (C-134, C-207; D is defined by C-170 as driver buffer, resampler and converter). The resampler's delay is therefore **not added to L**, which is the module's structural delay in the domain-rate timeline (C-207); the compiler reports it as the conversion delay of the device-facing port it serves (manifest representation in 023-29, analysis in 023-15).
- Slice = 48 frames = 0.5 ms at 96 kHz (C-02); the 48 kHz side has 24 frames per slice (derived).
- Who builds it (C-197, decided): **the arithmetic kit (epic 024a, phase P4, ahead of this epic) builds the fixed 2:1 / 1:2 node once**, with its filter design, coefficient table and delay. This ticket designs no filter: it models the kit's node in the IR and imports its delay, cost and state figures from the kit, without duplicating them. The kit's node is built from kit arithmetic, so it is deterministic (C-123).
- Two kinds of resampler (C-189, C-210): the DSL node is fixed-ratio, 2:1 or 1:2, with whole-sample delay. An edge's steerable rate actuator (ratio varying by parts per million, ±1 000 ppm, trait defined in epic 024a) is a separate component whose delay is part of D; it is not a DSL node and has no place in the closed node set.
- The delay of the kit's design is verified by epic 025's probe suite (falsifier: the probe disagrees with the compiler's figure).

## Work
1. Take the kit's 2:1 (96→48) and 1:2 (48→96) node from epic 024a (C-197) and import its delay in whole domain-rate samples for each direction, its history size and its operation count.
2. Implement `Resample::conversion_delay() -> Samples` (whole domain-rate samples, C-105), `Resample::cost()` (abstract operations per slice, C-106, 24 frames per slice on the 48 kHz side), `Resample::state_bytes()`, and its determinism contribution, each read from the kit's figures, as used by 023-15/16/17/18.
3. Cite the kit's specification (owner epic 024a; falsifier: epic 025's probe disagrees) in the node table instead of writing a second one.

## Acceptance criteria
1. For 96→48 and 48→96, `conversion_delay()` equals the whole number of domain-rate samples documented by the kit epic 024a (unit tests against the kit's own constant, C-197).
2. Constructing a resampler for any ratio other than 2:1 or 1:2 (for example 44 100 → 96 000) is impossible through the node's API; the DSL-level rejection is 023-29.
3. The node table row for the resampler lists its conversion delay (marked as part of D, not of L, C-134, C-207), cost and determinism contribution.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The closed node set is listed with, per node, its latency, cost and determinism contribution (resampler row)

## Out of scope
- Designing and building the resampler filter (epic 024a, C-197); the edge's rate actuator (epic 024a, C-189, C-210); where a resampler may appear and the ratio rejection (023-29); measuring the delay on hardware (epic 025 probe suite); code generation of the filter (epic 024); other ratios (out of v1, C-105).
