# 026a-09 — Software-ratio rate actuator
**Epic:** 026a-Embedded-HAL-and-host-integration · **Effort:** S · **Depends on:** 026a-02, epic 024a

## Context

The wire carries only the 96 kHz domain rate (C-01). The v1 device clock is **48 kHz**: the edge converts 2:1 through the kit's **resampler node** (epic 024a, C-197) and the wire stays at the domain rate (C-80). The node is offered only for 2:1 and 1:2, whose delay is a whole number of samples (C-105). A resampler node may appear only inside an edge module, and the edge's conversion delay is part of its plumbing delay D (C-134, C-170), not of any module's structural latency L. The rate actuator is **not** that node (C-189): the DSL resampler node is fixed-ratio, 2:1 or 1:2, with whole-sample delay; the edge's rate actuator is a separate, steerable resampler whose ratio varies by parts per million around its nominal value to track the hardware crystal, and its delay is part of D. In this ticket the actuator runs at the 48 kHz device rate after the fixed 2:1 node, so its nominal ratio is 1.

**Who builds the resamplers (settled, C-197).** The arithmetic kit (epic 024a) builds both the fixed 2:1 / 1:2 node and the steerable rate actuator, once; every edge, the embedded one included, uses the kit's and builds none of its own. This ticket therefore builds no resampler: `SoftwareRatio` is an adapter that implements `RateActuator` over the kit's steerable rate actuator. The earlier reading, in which this ticket wrote its own steerable resampler in `hadronyche-embedded`, is withdrawn.

**Rate-actuator trait with one v1 implementation: software ratio, over the kit's actuator (C-197).** The trait is defined in the kit epic (024a) together with its implementation, with a ratio range of ±1 000 ppm around the nominal ratio; every edge consumes it (C-210). This crate re-exports it (026a-02). The earlier reading in which epic 018 owned the trait is withdrawn. `Device` holds it internally (it is not one of `Device`'s type parameters). Hardware clock trimming can be added later behind the same trait with nothing on the wire `[decision]`.

```rust
// Shape as defined by the kit (epic 024a, C-210):
pub trait RateActuator { fn set_ratio(&mut self, ratio: f64); fn group_delay_frames(&self) -> u32; }
```

The firmware must contain **no allocator and no libm symbol** (026-AC-11). Kit arithmetic only; no transcendental functions. The cost of `f64` arithmetic on a part without a double-precision FPU is measured in 026b-05; if the `RateActuator` signature, the kit actuator's filter or its arithmetic has to change, the owner is epic 024a (C-197, C-210).

Open sub-question owned by this epic (answered in 026b-05): do the fixed node and the rate actuator fit the device's CPU budget at 2 000 slices/s? Falsifier: it does not fit — shorten the kit actuator's filter (quality cost, epic 024a), change the ratio type, or require hardware clock trimming earlier.

## Work

1. Instantiate the kit's fixed 2:1 resampler node (epic 024a, C-197) in the edge for the 96 kHz → 48 kHz conversion; its ratio is fixed and never steered (C-105, C-189).
2. `SoftwareRatio` in `hadronyche-embedded` implementing the kit's `RateActuator` trait (C-210) as an adapter over the kit's steerable rate actuator (epic 024a, C-197), instantiated at 48 kHz after the node, nominal ratio 1, varied by parts per million through `set_ratio` within the kit's range of ±1 000 ppm around nominal (C-189, C-210). No resampling filter and no rate-actuator trait is written in this crate.
3. `group_delay_frames()` returns the kit actuator's group delay in whole 48 kHz frames; the node's whole-sample delay is reported separately. Both are part of the device's D (C-134, C-170, C-189).
4. Statically sized storage for both kit resamplers (filter history for the 6 channels of one stream), with no allocation.
5. Expose the kit actuator's filter length as a const parameter of the adapter, as far as the kit offers one, so 026b-05 can shorten it; a shorter filter the kit does not offer is requested from epic 024a (C-197).

## Acceptance criteria

1. Host test: 1 kHz sine at 96 kHz in, through the fixed node and the actuator at nominal ratio 1 → 1 kHz sine at 48 kHz out; frame count out = in / 2 ± 1 over 10 s.
2. Host test: steering the actuator's ratio by +10 ppm changes output frame count by +10 ppm ± 1 ppm over 100 s of input; the fixed node's ratio is unchanged (C-189). Steering at +1 000 ppm and −1 000 ppm, the ends of the kit's range, is accepted and changes the output frame count accordingly (C-210).
3. `group_delay_frames()` matches the measured delay of an impulse through the actuator ± 1 frame, and the node's reported delay is a whole number of samples (C-105).
4. Device build: no alloc and no libm symbols (026a-01 check).
5. Both resamplers in the image are the kit's (epic 024a): `hadronyche-embedded` contains no resampling filter of its own (C-197), and `SoftwareRatio` implements the kit's trait, not a local one (C-210).

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] The crate builds for at least one device target and for the host (for tests) in CI
- [ ] Every new factual claim carries an evidence tag

## Out of scope

- Loop 2 steering logic and audio output (026a-10).
- CPU-budget measurement on the board (026b-05).
- Hardware clock trimming (future, not v1).
- Building either resampler: the kit does (epic 024a, C-197).
