# 024a-13 — Kit rate-actuator trait and resampler (separate, variable ratio)
**Epic:** 024a-Runtime-ABI-and-arithmetic-kit · **Effort:** M · **Depends on:** 024a-12

## Context
The epic requires a software rate actuator at edges in `hadronyche-kit/src/resample.rs`. Settled (C-189): there are two kinds of resampler. The DSL resampler node is fixed-ratio, 2:1 or 1:2, with whole-sample delay (C-105, 024a-12); an edge's rate actuator is a separate, steerable resampler whose ratio varies by parts per million around its nominal value to track the hardware crystal. This ticket builds the second kind; it is not a mode of the DSL node. Settled (C-210): the rate-actuator trait, the interface every edge consumes, is defined in this epic together with its implementation, with a ratio range of ±1 000 ppm around the nominal ratio; this ticket defines both. This epic is the single builder of the actuator: every edge (Linux, macOS, Windows, embedded) uses the kit's and builds none of its own, which is why the epic is in phase P4, ahead of the Linux edge, and blocks epics 018, 021, 022a and 026 (C-197). Rules `[decision]`: no allocation, no libm, never blocks. Slice 48 frames (C-02); 6 channels (C-04); 4-byte samples (C-03).

Settled figures that bound this mode:
- The L2 drift-ramp scenario sweeps the injected crystal drift over ±100 ppm at no more than 0.5 ppm/s, and passes only if E is held throughout (C-119). The settled range of ±1 000 ppm (C-210) covers it; the actuator must follow ramps of 0.5 ppm/s with phase continuity.
- The edge rate-loop bandwidth is well below 1 Hz (C-68, pending measurement at L2): ratio updates are small and slow.
- L2 requires zero accumulated slip after a disturbance (C-76).
- Laptop sleep is a departure: both timebase loops are reset and the edge returns through admission (C-124); the actuator restarts from its initial state.
- The actuator's delay is not constant; it is counted only in the edge's plumbing delay D (C-134, C-170, C-189, C-207), never in an output's L; structural latency L stays a whole number of samples (C-105). The ABI's frequency-dependent delay property (C-189) is for recursive filters and does not report it.

The ratio's range is settled (±1 000 ppm, C-210); its format and resolution are an open sub-question of the epic owned by this ticket. Falsifier: ratio quantisation alone produces slip or exceeds E in the L2 drift-ramp scenario (C-76, C-119).

## Work
- Define the rate-actuator trait in `hadronyche-kit` (C-210): the interface every edge (Linux, macOS, Windows, embedded) consumes to set the ratio per slice, process a slice and reset; `no_std`, no allocation; names may be refined.
- Add a separate rate-actuator resampler type to `resample.rs` implementing that trait (not a mode of the DSL node of 024a-12, C-189): ratio set per slice as a fixed-point value around a nominal ratio; choose and document its format and resolution; range ±1 000 ppm around nominal (C-210), which covers the L2 drift ramp (C-119); phase continuity across ratio changes.
- Document the actuator's delay: its nominal delay as a constant of the actuator, and the whole delay counted only in D, never in L (C-134, C-170, C-189, C-207).
- Provide `reset` returning the actuator to its initial state (C-124); it may share code with the node of 024a-12.
- Tests: ratio steps, slow ramps, phase continuity, output count tracking the integrated ratio.

## Acceptance criteria
1. With a constant ratio equal to its nominal value, output equals input delayed by the actuator's documented nominal delay, bit for bit.
2. Over 10⁵ slices with ratio set to nominal + 100 ppm, the produced frame count equals the integrated expectation within 1 frame.
3. A ratio ramp from −100 ppm to +100 ppm at 0.5 ppm/s keeps the produced frame count within 1 frame of the integrated expectation throughout, with no phase discontinuity (C-119).
4. A ratio step mid-stream produces no discontinuity larger than a documented bound on a full-scale sine input `[measured]`.
5. The rate-actuator trait is defined in `hadronyche-kit` and the actuator implements it; with the ratio held at nominal + 1 000 ppm and at nominal − 1 000 ppm for 10⁵ slices each, the produced frame count equals the integrated expectation within 1 frame (C-210).
6. The actuator runs under the aborting allocator and with a compile-time-constant state size; after `reset` its output equals that of a freshly constructed actuator, bit for bit.
7. The DSL resampler node of 024a-12 exposes no variable-ratio constructor or setter (C-189).
8. The trait, the ratio format, resolution, range (±1 000 ppm, C-210) and delay semantics are documented in the module doc, with the open item (owner this ticket, falsifier above) recorded.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The clock-servo and edge rate loop that choose the ratio (other epics; they consume the trait defined here, C-210).
- The fixed-ratio DSL resampler node and its group-delay exactness (024a-12).
