# 018-01 — Edge module skeleton and traits
**Epic:** 018-Linux-ALSA-edge · **Effort:** XS · **Depends on:** epic 024a

## Context
- Epic 018 builds the first physical edge (Linux ALSA sink and capture) and, with it, a shared **two-loop timebase** in `hadronyche-core`, module `edge/`, that every other edge (macOS epic 021, Windows epic 022, embedded epic 026) reuses.
- Every edge has a host clock H (on Linux `CLOCK_MONOTONIC`) and a device sample counter S (the sound card's crystal). Loop 2 maps S to H by regression over (sample count, host time) pairs supplied by the audio API `[decision]`.
- v1 steers a software resampler on every profile. A **rate-actuator trait** has that one implementation; hardware clock trimming can be added later with nothing on the wire `[decision]`. That implementation is the arithmetic kit's steerable rate actuator, built by epic 024a; this epic builds no resampler (C-197).
- **C-210:** the rate-actuator trait is defined in the kit epic (024a) together with its implementation, with a ratio range of ±1 000 ppm around the nominal ratio; every edge consumes it. This ticket therefore does not define `RateActuator`; it uses the kit's trait as published by epic 024a.
- `edge/` must build `no_std` with no allocation.

## Work
1. Create module `edge/` in crate `hadronyche-core` (`no_std`, no `alloc`).
2. Add the public types exactly as given by the epic (names may be refined, shapes kept); the rate-actuator trait is not among them (C-210):
   ```rust
   /// One observation from the audio API: device frame count at a host time.
   pub struct DevicePair { pub frames: u64, pub host_ns: i64 }

   pub trait DeviceClock {
       fn nominal_rate(&self) -> u32;
       fn declared_jitter_ns(&self) -> u32;
   }
   ```
3. Add empty placeholders for `DeviceLine` and `EdgeTimebase` (filled by 018-02, 018-03, 018-04 and 018-21) so later tickets only add code.
4. Add a CI job that builds `hadronyche-core` with `edge/` for a `no_std` target without the `alloc` crate linked (for example a bare-metal target such as `thumbv7em-none-eabihf`, or `#![no_std]` with no `extern crate alloc`), failing on any allocation dependency.
5. Rustdoc on every item, including the direction of the ratio that `EdgeTimebase` will hand to the kit's actuator: ratio = domain samples consumed per device sample produced (> 1 when the device runs slower than the domain line).

## Acceptance criteria
1. `hadronyche-core` builds with `edge/` on a `no_std` target with no `alloc` crate; the CI job fails if `alloc` or `std` is pulled into `edge/`.
2. A trivial test implementation of `DeviceClock` compiles and is exercised in a unit test (returns `nominal_rate() == 96_000`, the domain rate and the only rate on the wire (C-01)); a test double of the kit's rate-actuator trait (epic 024a, C-210) records `set_ratio(1.0)`. No `RateActuator` trait is defined in `edge/` (C-210).
3. `cargo doc` produces documentation for every item of `edge/` (`DevicePair`, `DeviceClock`, `DeviceLine`, `EdgeTimebase`) with no missing-docs warning.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The `edge/` module builds `no_std` with no allocation
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Regression maths (018-02), ratio and quality (018-03), emission error (018-04).
- The rate-actuator trait and the resampler implementations: the kit's steerable rate actuator and fixed 2:1 / 1:2 node (epic 024a, C-197, C-210).
- The no-libm symbol inspection (018-09).
