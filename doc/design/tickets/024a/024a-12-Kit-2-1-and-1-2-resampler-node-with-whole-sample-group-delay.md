# 024a-12 — Kit 2:1 and 1:2 resampler node with whole-sample group delay
**Epic:** 024a-Runtime-ABI-and-arithmetic-kit · **Effort:** S · **Depends on:** 024a-07, 024a-08, 024a-11

## Context
`hadronyche-kit/src/resample.rs` provides the DSL resampler node, which is fixed-ratio, with a constant group delay (C-189). Rules `[decision]`: no allocation, no libm, never blocks; the module cannot set positions. Structural latency L is always a whole number of domain-rate samples, and the node is offered only in designs whose delay is a whole number of samples: 2:1 and 1:2; other ratios are out of v1 (C-105). The node may appear only inside an edge module; the compiler rejects it anywhere else (C-134). An edge whose hardware runs at 48 kHz converts 2:1 through this node and the wire stays at 96 kHz (C-01, C-80); 1:2 is the playback direction. The conversion delay is counted only in the edge's plumbing delay D (driver buffer, resampler and converter, C-170), never in any output's L (C-134, C-207); L is the module's structural delay in the domain-rate timeline, and the manifest reports the conversion delay separately, on the device-facing port (C-207). This node is the only 2:1 / 1:2 node: every edge (Linux, macOS, Windows, embedded) uses it and builds none of its own (C-197). Samples are 4 bytes, f32 or Q1.31 (C-03); slice 48 frames at 96 kHz, so 24 frames on the 48 kHz side (C-01, C-02); 6 channels (C-04).

Epic acceptance criterion 024a/AC-3: for 2:1 and 1:2, the probe-measured group delay is a whole number of domain-rate samples and equals the kit's `group_delay` constant (C-105); no other ratio is offered. This ticket provides the node and its constant; 024b-05 makes the compiler report it as the edge's conversion delay, in the manifest on the device-facing port and separately from L (C-207).

## Work
- `pub struct Resampler<const CH: usize>` for the two directions (or one type per direction) with state in a fixed-size array, using the coefficient tables of 024a-11.
- `pub const fn group_delay(direction) -> Latency` returning the delay as a whole number of domain-rate samples (`hadronyche_core::abi::Latency`, C-105), as the single source of the constant; its documentation states that it is a conversion delay counted only in D and never added to any output's L (C-207). There is no constructor for any other ratio.
- `process(&mut self, input: &[..], output: &mut [..]) -> usize` handling 48-frame domain-rate slices and 24-frame 48 kHz slices.
- `reset(&mut self)` returning to the initial state (used when an edge leaves on sleep and re-enters through admission, C-124).
- Fixed-point path (Q1.31, exact-fixed) required; float path if the report of 024a-11 keeps it.
- Probe test: feed an impulse (and a band-limited step), locate the response peak, and check the measured delay equals `group_delay` exactly for 2:1 and 1:2.

## Acceptance criteria
1. For 2:1 and 1:2, the probe-measured group delay is a whole number of domain-rate samples and equals `group_delay` exactly (C-105).
2. Over 1000 slices, the output frame count is exactly twice (2:1) or half (1:2) the input frame count.
3. `Resampler` state size is a compile-time constant (`core::mem::size_of` in a `const` assertion), and the node runs under the aborting allocator.
4. Output bits for a committed input vector match a golden file on the hosted target for each direction.
5. After `reset`, the output for a given input equals the output of a freshly constructed node, bit for bit.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The edge rate actuator, a separate steerable resampler, not a mode of this node (024a-13, C-189).
- Compiler reporting of the conversion delay on the device-facing port (C-207) and rejection of the node outside an edge module (024b-05, epic 023).
- Cross-target identity (024a-14).
- Any ratio other than 2:1 and 1:2 (out of v1, C-105); the earlier rational-L handling (fractional phase, rational delay, a non-integral test ratio) is withdrawn.
