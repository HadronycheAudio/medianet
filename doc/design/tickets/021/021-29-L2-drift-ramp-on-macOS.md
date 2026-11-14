# 021-29 — L2 drift-ramp scenario on macOS
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-23, epic 020
## Context
Epic criterion 021-AC2 includes the L2 drift-ramp scenario [decision, C-119]: the injected crystal drift sweeps ±100 ppm at no more than 0.5 ppm/s; pass is E held throughout. The slew limits C-60 and C-61 bound the servo's correction term, not the drift estimate it tracks.

- macOS E target 100 µs (C-71); E is the 99.9th percentile of absolute emission error, maximum beside it (C-73).
- "E held" means the platform target, 100 µs (C-71), never the E measured in the same run (C-149).
- The run is in strict mode (C-147); the harness computes A with the commissioning rule and records it with the results (C-148).
- The drift injector and harness belong to epic 020 (harness code in the `hadronyche-tools` crate, C-133); results go to `doc/design/reports/` (C-130).
## Work
- Run the epic 020 drift-ramp scenario on the reference Mac in strict mode with the macOS edge sink, sweeping the holder's crystal from −100 ppm to +100 ppm and back at 0.5 ppm/s.
- Record emission error over the whole sweep, per 1 s window and overall.
## Acceptance criteria
1. The sweep covers ±100 ppm at no more than 0.5 ppm/s (checked from the injector log).
2. The 99.9th percentile of emission error over the sweep is at most the 100 µs target (C-149), maximum reported beside it.
3. Zero accumulated slip over the sweep.
4. The results record strict mode (C-147) and the A used (C-148).
## Definition of done
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
- [ ] Every new factual claim carries an evidence tag
## Out of scope
The drift injector (epic 020). Other common criteria (021-24).
