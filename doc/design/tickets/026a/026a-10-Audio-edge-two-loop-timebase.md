# 026a-10 — Audio edge two-loop timebase
**Epic:** 026a-Embedded-HAL-and-host-integration · **Effort:** M · **Depends on:** 026a-06, 026a-08, 026a-09, epic 018

## Context

Audio edge contract, same as every edge `[decision]`:

- `emit(p) = T(p) + A` — A is the presentation group's audition offset (C-55).
- `submit(p) = emit(p) − D` — D is the device's declared plumbing delay.

Two-loop timebase from core (epic 018):
- loop 1: device clock → domain line (servo, 026a-06);
- loop 2: device sample counter → device clock, by regression over (sample count, timer-captured time) pairs — on a device, a **timer capture of the audio DMA position** (`AudioOut::position() -> DevicePair`).

The wire is at 96 kHz only (C-01); the device clock is 48 kHz and the edge converts 2:1 through the kit's fixed-ratio resampler node and tracks the device crystal with the kit's separate steerable rate actuator, adapted as `SoftwareRatio` (026a-09, C-80, C-189); both resamplers come from the arithmetic kit (epic 024a) and the device builds none (C-197). A resampler node may appear only inside an edge module, and its conversion delay is part of D (C-134); so is the actuator's delay (C-189). The resampler delay is counted only in D, never in L: L is the module's structural delay in the domain-rate timeline, and the manifest reports the conversion delay separately, on the device-facing port (C-207).

```rust
pub trait AudioOut {
    fn rate(&self) -> u32;                   // 48_000 in v1
    fn write(&mut self, frames: &[i32]) -> usize;
    fn position(&self) -> DevicePair;        // (frame count, timer-captured device time)
    fn fixed_delay_ns(&self) -> u32;         // converter and interface part of D
}
```

D is the sink's total plumbing delay from hand-off to emission: driver buffer, resampler and converter (C-170). Here D = `fixed_delay_ns()` + the fixed node's whole-sample delay + the actuator's `group_delay_frames()` at 48 kHz + any ring/DMA buffering. The list of allowed methods for "D with method" is not yet fixed: open item, owner epic 018 (shared edge core), falsifier "two edges declare D by methods that cannot be compared". This ticket reports the components and labels the method as computed from them.

E is declared per device with the common definition: 99.9th percentile of absolute emission error over a one-hour certification run, maximum beside it (C-73); unknown until a commissioned certification (C-103, C-140). The certification run is 026b-14.

## Work

1. In `Device::poll`: sample `AudioOut::position()`, feed loop 2 (core regression); combine with loop 1 to compute the device-sample-count for `submit(p)`.
2. Steer `SoftwareRatio::set_ratio` from the loop-2 error (core's controller); the fixed 2:1 node is never steered (C-189).
3. Pull slices from the alignment ring for positions due, convert 96 k → 48 k through the fixed node, then the actuator, select the played channels by the manifest channel map (C-04), interleave, `AudioOut::write`.
4. Gate output on `claims_alignment` from 026a-06: when false, output silence (or as core's gap classes dictate) and stop claiming alignment.
5. Compute D and expose it with its components for the stamp (026a-11, 026b-14); report the conversion delay on the edge module's device-facing port in the manifest and keep it out of L (C-207).
6. Bounded work per `poll`.

## Acceptance criteria

1. Host test with mock `AudioOut` whose sample clock runs +30 ppm from the device clock: after convergence, the written-sample position vs `submit(p)` error stays below 1 frame at 48 kHz (20.8 µs) for 60 s.
2. Host test: a click at position p reaches mock `AudioOut::write` such that its scheduled emission time equals `T(p) + A` within 1 frame, given mock D.
3. Host test: when `claims_alignment` goes false, no non-silent frames are written.
4. Reported D equals `fixed_delay_ns` + fixed-node delay + actuator group delay + buffering (each component reported), covering every C-170 component and the actuator (C-189); the edge module's L excludes the conversion delay, which the manifest reports on the device-facing port (C-207).
5. Device build: no alloc, no libm.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] The crate builds for at least one device target and for the host (for tests) in CI
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope

- Real codec driver (026b-03).
- Measuring D and certifying E on hardware (026b-14); cross-sink skew is task 027.
