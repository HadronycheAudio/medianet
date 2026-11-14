# 008a-04 — Sim crate skeleton and seeded determinism
**Epic:** 008a-Servo-core-and-simulation-harness · **Effort:** S · **Depends on:** 008a-01
## Context
Epic 008 drives the servo entirely by a deterministic simulation. The simulation lives in a hosted, dev-only `sim` module of the `hadronyche-tools` crate, the crate that holds harness and tool binaries (C-133); results go to `doc/design/reports/` (C-130). Epic criterion A10: the simulation is deterministic — same seed, same result, on x86-64 and 64-bit ARM (C-151); the cross-machine check is in 008b (008b-17), and this part provides the single-machine determinism of criterion A16.

This ticket creates the crate and the deterministic core everything else plugs into: a simulated time base, a seeded random source, and a scenario description.

## Work
1. Create the `hadronyche-tools` crate if it does not exist yet (hosted, `std` allowed, `publish = false`, not a dependency of any runtime crate) and its `sim` module; `hadronyche-tools` depends on `hadronyche-core`, never the reverse.
2. A seeded PRNG with a fixed, explicitly versioned algorithm (no `thread_rng`, no OS entropy, no hash-map iteration order in results). Sub-streams per model derived from the master seed so adding a model does not perturb others.
3. A simulated time base in integer nanoseconds (`i64`) and a discrete-event queue with deterministic tie-breaking.
4. A `Scenario` struct (seed, duration, model parameters as plain data) and a `run(scenario) -> Trace` entry point returning a serialisable trace.
5. A trace digest (stable hash of the serialised trace) for determinism checks.
6. Choose and document, with the crate, the trace and summary serialisation format and the file names used under `doc/design/reports/`; these presentation details are decided here (C-206), and only what 008b's reports depend on goes into `servo.md`.

## Acceptance criteria
1. Running an empty scenario (event queue with a few synthetic events) twice with seed 1 yields identical digests; with seed 2 yields a different digest.
2. No use of wall-clock time, OS randomness or unordered-collection iteration in the crate (enforced by a `clippy` disallowed-methods/types config on the `sim` module).
3. `hadronyche-tools` is excluded from any `no_std` build and from runtime release artefacts; `hadronyche-core` does not depend on it.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Clock, link, wall-clock, multi-holder models (008a-05, 008a-06, 008a-08, 008a-16).
- Recording metrics (008a-09); closed-loop runner (008a-15); cross-machine check (008b-17).
