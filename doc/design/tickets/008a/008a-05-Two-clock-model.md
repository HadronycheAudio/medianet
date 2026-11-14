# 008a-05 — Two-clock model
**Epic:** 008a-Servo-core-and-simulation-harness · **Effort:** S · **Depends on:** 008a-04
## Context
Part of the `sim` module of the `hadronyche-tools` crate (hosted, dev-only) of epic 008. The holder's monotonic clock defines network time `[decision]`; the follower's servo estimates it. The simulation needs both crystals modelled against an ideal reference so that true error can be computed.

Ranges used by the suite: constant drift up to ±100 ppm (case 1); a 50 ppm crystal difference at handover (case 6, C-78); a drift ramp sweeping ±100 ppm at no more than 0.5 ppm/s (case 16, C-119).

## Work
In the `sim` module of `hadronyche-tools`:
1. `ClockModel` mapping ideal time (ns) to local reading (ns) with parameters: initial offset, drift (ppm), drift rate (ppm/s), drift limits for a ramp that sweeps between −100 and +100 ppm and reverses at the limits, thermal wander (a seeded random-walk or filtered-noise process on frequency, amplitude configurable).
2. Two instances per scenario: holder and follower; inverse mapping (local reading → ideal time) for computing true error of published time.
3. Output in integer ns; internal frequency integration must be deterministic (fixed step, documented).

## Acceptance criteria
1. Zero drift, zero wander: local reading equals ideal plus offset exactly over 1 h simulated.
2. Constant drift +100 ppm: after 10 s ideal the reading has advanced 10 s + 1 ms (±1 ns).
3. Drift rate 0.1 ppm/s for 100 s: frequency deviation reaches 10 ppm (±0.01 ppm).
4. Ramp at 0.5 ppm/s between ±100 ppm: frequency reaches +100 ppm, reverses, and never exceeds ±100 ppm or 0.5 ppm/s.
5. Inverse mapping round-trips within 1 ns for all of the above.
6. Same seed, same wander trajectory (digest equal across two runs).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Link delays (008a-06), wall clock (008a-08), handover between holders (008a-16).
