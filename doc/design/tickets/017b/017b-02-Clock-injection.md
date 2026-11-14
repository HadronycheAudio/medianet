# 017b-02 — Clock injection
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** M · **Depends on:** 017b-01, epic 015

## Context
- L0 runs every module in one process on loopback, so all modules share one true clock. Clock injection gives each module a **simulated crystal**: an offset and a drift relative to true time. This replaces the POC's `--sim-offset-ms` and `--sim-drift-ppm` flags (POC `src/main.rs:162`, `:165`) `[code]`.
- Units in the new API: `ClockInjection { offset_ns: i64, drift_ppb: i64 }` (parts per billion; 1 ppm = 1 000 ppb); `step_clock(module, step_ns)` applies an instantaneous step.
- The injected values are the ground truth that L0 assertions compare against (for example AC-8: estimated minus true offset within the lock bound C-62).
- Wall-clock step detection lives in the platform backend that builds each exchange from receive stamps; the servo only ever receives exchanges that passed it (C-121). An injected step must therefore reach the backend's receive stamps, not only the servo. The step threshold is 2 ms (C-67). The injection applies whatever receive-timestamp source is in use (C-131).
- Position on the domain timeline: `position = (network_time − epoch_origin) × rate`, `rate` being the domain rate 96 kHz (C-01).

## Work
1. Implement `Injector::set_clock` and `Injector::step_clock` in the hosted runtime's injector (`hadronyche-std`), so every time read by the target module (media clock, timers, timestamps on sent and received packets, including the receive stamps the platform backend uses) goes through `local = true + offset + drift × (true − t_set)` plus all accumulated steps.
2. `set_clock` replaces the previous offset/drift; `step_clock` adds `step_ns` to the current offset without changing drift.
3. Expose a read accessor for the harness, e.g. `true_offset_ns(module, at_true_ns) -> i64`, so assertions can compute the true offset at any instant.
4. Injection is runtime-only: no effect unless the injector is used; no privileged clock access. Because the step is simulated, step tests run in per-change CI; only a test that steps the real wall clock would belong in the privileged nightly job (C-146), and L0 has none.

## Acceptance criteria
1. With `offset_ns = 5_000_000`, `drift_ppb = 0`, the module's local time differs from true time by exactly 5 000 000 ns at every sample.
2. With `drift_ppb = 50_000` (50 ppm), after 10 s of true time the local clock has advanced by 10 s + 500 000 ns (± 1 ns rounding).
3. `step_clock(m, -2_000_000)` after `set_clock(offset 5 ms)` gives an accessor value of 3 000 000 ns and leaves drift unchanged.
4. Two modules in one process with different injections observe different local times; a module without injection observes true time.
5. `true_offset_ns` matches the injected model in a unit test over 1 000 random (offset, drift, t) triples.
6. A step injected with `step_clock` appears in the receive stamps handed to the platform backend (C-121).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Link, delay, reorder, blackhole injection (017b-03); partition / hello / thread kill (017b-04).
- Recording the injection in the stamp (017b-05).
- Asserting sync accuracy (scenario tickets 017b-10, 017b-11) and step detection itself (platform backend).
