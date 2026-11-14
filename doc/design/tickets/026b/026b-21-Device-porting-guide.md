# 026b-21 — Device porting guide
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** S · **Depends on:** 026b-03, 026b-04, 026b-05, 026b-07

## Context

Deliverable: `docs/device-porting.md` — how to implement the traits of `hadronyche-embedded` on another board. A device's hardware surface is five traits: `Transport`, `DeviceTime`, `AudioOut`, `Identity`, defined by the crate, and `RateActuator`, defined by the arithmetic kit (epic 024a) with a ratio range of ±1 000 ppm around nominal and re-exported by the crate (C-210). A board supplies the four; `Device<T, C, A, I>` holds the `RateActuator` internally with the built-in software-ratio implementation, an adapter over the arithmetic kit's steerable rate actuator (epic 024a, C-197; the epic states this). A port builds no resampler: both come from the kit. The guide covers all five and says which one has a default.

Facts to include:
- One implementation rule `[decision]`: never reimplement hello, election or sync; drive `hadronyche-core` via `Device::poll`.
- Follower only; never set holder-eligible `[decision]`; the device never wins an election, whatever its id (C-113).
- IPv4 only, multicast with TTL 1 (C-100); DSCP classes C-13; control messages ≤ 1 252 bytes except the path-MTU probe pair (C-17, C-116).
- Identity: no PRNG ids without a true random source; mix hardware unique id with a persistent boot counter; id stable, nonce new per boot (C-11, C-46).
- Timestamp source: MAC hardware if available, else rx interrupt; reported with a C-131 value and a C-129 state; `declared_jitter_ns`. Step detection in the backend (C-121).
- `AudioOut`: 48 kHz, the wire stays at 96 kHz (C-80, C-01); timer-captured DMA position; `fixed_delay_ns`. The resampler lives only in the edge and its delay is part of D (C-134, C-170).
- `Identity::domain_id()`: a device with domain 0 is ignored by peers (C-173).
- Strict mode is hosted-only; a device stamps real-time scheduling and memory lock present (C-147); control isolation is absent or unverified unless a bound is shown (C-161).
- Media: whole 6-channel streams, no per-channel id, channel map from the manifest (C-04).
- no_std, no alloc, no libm; symbol check.
- Watchdog: any fault resets the device.
- Memory budget: 63-entry peer map (C-217), one 6-channel stream ring sized to A at 96 kHz (23 040 bytes at 10 ms, C-96), servo, resampler, ≥ 25 % headroom.
- Stamp: sectioned, multicast on change and every 10 s, unicast on request (C-18); E per C-73 or unknown (C-103); per-connection counters (C-47, C-159); lateness percentiles (C-160) and reclaimable latency (C-157); peer DSCP observations (C-152); calibration factor (C-106, C-176); commissioning window (C-107, C-153).
- The "what weakens on no_std" table and the matching stamp fields.

## Work

1. Write `docs/device-porting.md` with sections: overview, each trait (contract, pitfalls, reference implementation pointer), watchdog, identity, memory and CPU budget (with the reference figures from 026b-04 and 026b-05), calibration factor (026b-19), bounded control handling and how to measure it (026b-06), stamp fields to fill, conformance run (026b-08).
2. Name the reference board, its timestamp source and its oscillator (from 026b-01 / 026b-07).

## Acceptance criteria

1. The guide covers all five traits with their signatures, states that `RateActuator` is the kit's trait (C-210) and has a built-in default.
2. The reference board, timestamp source, jitter figure and oscillator are named.
3. Memory and CPU figures from 026b-04 / 026b-05 are quoted with `[measured]` tags.
4. A reviewer who has not worked on the epic can list the steps to port to a new board from the guide alone (review sign-off).

## Definition of done

- [ ] The reference board, its timestamp source and its oscillator are named in the docs
- [ ] Every new factual claim carries an evidence tag
- [ ] The table "what weakens on no_std" is reflected field by field in the device's stamp

## Out of scope

- Design-document and constants-table updates (026b-22).
