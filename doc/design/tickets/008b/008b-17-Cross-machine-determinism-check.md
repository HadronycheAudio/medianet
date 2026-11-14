# 008b-17 — Cross-machine determinism check
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** S · **Depends on:** 008b-12, 008b-13, 008b-14, 008b-15, 008b-16, 008b-19, 008b-20
## Context
Epic criterion A10: the simulation is deterministic — same seed, same result, on the machine pair for cross-machine bit-exactness, x86-64 and 64-bit ARM (C-151). The sim `sim` module of the `hadronyche-tools` crate (008a-04) already uses a versioned seeded PRNG, integer-ns simulated time and a trace digest. Risks across machines: floating-point differences (e.g. `TimeView.rate_ppb: f64`, transcendental functions in jitter distributions), parallel test ordering, platform-dependent library code.

## Work
1. A `sim-digests` binary in the `hadronyche-tools` crate (C-133) that runs every named case (1–17 and the studies) at fixed seeds and writes one digest per case to a file.
2. Check in a golden digest file; CI runs the command on an x86-64 runner and a 64-bit ARM runner (C-151) and compares both to the golden file. Record the two runner images in the `hadronyche-tools` README.
3. Remove sources of non-determinism found (e.g. replace platform `libm` calls with a pure-Rust deterministic implementation, avoid fused multiply-add differences, fix iteration orders).
4. Document how to regenerate golden digests when behaviour changes intentionally, and the digest file's format, which this epic decides as a presentation detail of its own tool (C-206).

## Acceptance criteria
1. Two CI jobs, one on x86-64 and one on 64-bit ARM (C-151), produce byte-identical digest files equal to the golden file.
2. Changing one seed changes that case's digest and only that case's digest.
3. Regeneration procedure, digest file format (C-206) and the runner pair documented in the `hadronyche-tools` README.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Adding new cases (each case ticket); determinism within one machine (008a-04, 008a-15).
