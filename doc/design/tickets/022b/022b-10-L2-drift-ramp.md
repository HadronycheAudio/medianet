# 022b-10 — L2 Windows: drift ramp
**Epic:** 022b-Windows-edge-L2-certification · **Effort:** S · **Depends on:** 022b-02, epic 020

## Context
- Part of epic 022b (Windows edge L2 certification), L2 certification.
- C-119: the L2 drift-ramp scenario injects crystal drift sweeping ±100 ppm at no more than 0.5 ppm/s; pass is E held throughout. The slew limits C-60 and C-61 bound the servo's correction term, not the drift estimate it tracks.
- Pass is the target E of 1 ms (C-72), never the E measured in the same run (C-149). The run is in strict mode (C-147) with A computed and recorded by the harness (C-148).
- Windows E target 1 ms (C-72), defined as in C-73. Two-loop timebase (epic 018) with rate-loop bandwidth well below 1 Hz (C-68): the ramp checks that the ratio of the edge's rate actuator follows the drift without breaking E. The rate actuator is the steerable resampler, distinct from the fixed 2:1 / 1:2 node, built with its trait by the kit (epic 024a) with a ratio range of ±1 000 ppm around nominal, which covers the ±100 ppm sweep; its delay is part of D (C-189, C-197, C-210).
- The scenario and its injection are provided by the L2 harness (epic 020); injections are stamped (C-120).

## Work
1. With the Windows sink live on the rig (022b-01), run the epic-020 drift-ramp scenario across the full ±100 ppm sweep.
2. Compute E over the run (99.9th percentile and maximum) and its running value during the ramp.

## Acceptance criteria
1. **(022-AC-15.)** E stays at or below 1 ms throughout the sweep; maximum reported beside it.
2. Results recorded `[measured]` in `doc/design/reports/` (C-130) for 022b-07.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The injection tooling (epic 020).
- Zero slip after loss, handover and step (022b-05).
