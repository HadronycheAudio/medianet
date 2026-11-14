# 010-02 — Output position stamping
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** XS · **Depends on:** 010-01
## Context
- Slices are addressed by *position*: network time counted in samples from a domain epoch. There is one sample rate on the wire (C-01, 96 000 Hz), so one timeline per domain. Position is a 64-bit value in the slice header.
- Position stamping rules `[decision]`: sources pace position from network time, not from a local crystal. The runtime stamps output positions and module code cannot change them. A module with structural delay L has its output stamped p − L while it processes input p. Stamps are never restamped per hop.
- C-105 (revised): L is always a whole number of domain-rate samples. The earlier rational-L rule is withdrawn, so there is no numerator/denominator, no rounding and no fractional position; a resampler node exists only inside an edge module (C-134), and an edge's resampler (conversion) delay is counted only in its plumbing delay D, never in L (C-207): L is the module's structural delay in the domain-rate timeline, and the manifest reports the conversion delay separately on the device-facing port.
- C-122 settles the sign convention: stamps denote content position. While a module processes input p it emits the output stamped p − L, whose content came from input p − L; that output is degraded if the input at p − L was gapped. This matches the rule above.
- Settled in the epic: positions count from the domain epoch, so when p < L there is no output position; the function returns none.
- `[code]` In the POC the ticker ran on local monotonic time (`module.rs:1630-1634`); that behaviour is retired.

## Work
- In `hadronyche-core`, `no_std`, no allocation, add `pub const fn output_position(input_position: u64, structural_delay_samples: u64) -> Option<u64>` (names may be refined): `Some(p − L)`, or `None` when p < L (checked subtraction).
- Doc comment states that L is in whole domain-rate samples (C-105), that the runtime, not module code, calls this function, that the result is never changed per hop, and that the stamp denotes content position (C-122).

## Acceptance criteria
1. `output_position(10_000, 0) == Some(10_000)`.
2. `output_position(10_000, 48) == Some(9_952)`.
3. `output_position(u64::MAX, u32::MAX as u64) == Some(u64::MAX - u32::MAX as u64)`.
4. `output_position(10, 48) == None`, without panic.
5. The function can be evaluated in a `const` context and compiles in the `no_std` CI job of 010-01.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the send path
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Sourcing network time (epic 006) and the runtime scheduler that calls this function.
- Computing the degraded flag (C-122): the caller sets it in the slice flags (010-06).
- Stamping in the slice header: done by the encoder (010-06).
- Resampling and edge conversion delay (inside edge modules, C-134; counted in D, never in the L passed here, C-207).
