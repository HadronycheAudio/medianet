# 026a-05 — Device poll loop and follower discovery
**Epic:** 026a-Embedded-HAL-and-host-integration · **Effort:** M · **Depends on:** 026a-02, 026a-03, 026a-04, epic 011

## Context

The device runs a **cooperative main loop with bounded work per call**. It drives the `hadronyche-core` pure state machines (message and time in, messages and events out); it never reimplements hello, election or sync logic `[decision]`.

```rust
/// The cooperative main loop: bounded work per call.
pub struct Device<T, C, A, I> { /* core state machines, servo, ring, resampler (RateActuator) */ }
impl<T: Transport, C: DeviceTime, A: AudioOut, I: Identity> Device<T, C, A, I> {
    pub fn poll(&mut self) -> PollStats;     // work done this call, its elapsed time, and the running worst case
}
```

Facts:
- **Fixed-capacity peer map: 63 entries** (C-217): a medianet has at most 64 active modules and commissioning refuses more (C-34), and a module holds no entry for itself.
- Control messages are binary, at most 1 252 bytes of UDP payload (C-17). The path-MTU probe pair is exempt: it is sent at the intended media packet size (C-116).
- IPv4 only, multicast with TTL 1 (C-100).
- Control messages carry scheme 0 and a zero tag; the 64-bit message counter is live, as the liveness counter (C-115). Core produces it; the device must not zero it.
- Hello period 1 s (C-30), expiry 3 s for non-live peers (C-31). The hello carries only a stamp version number, not the stamp (C-18).
- **Follower only** `[decision]`: holder eligibility is a capability bit in the hello; no v1 device sets it. When no candidate is locked, the holder-eligible candidate with the lowest module id wins (C-113); the device is never a candidate whatever its id.
- An id seen with two nonces is quarantined by third parties until only one nonce has been seen for 3 s (C-46); the device, as a third party, gets this from core.
- Identity: module id 64 bits, instance nonce 64 bits, domain id 16 bits with 0 = unconfigured (C-11), from 026a-03. A hello with domain 0 is ignored and counted, not refused; a hello from a different non-zero domain is foreign (ForeignDomain) (C-173). Core applies this to received hellos; the device's own hellos carry its stored domain id.
- `Device` is generic over four traits and holds the `RateActuator` internally (software ratio, 026a-09).
- Epic criterion 026-AC-1 (AC-35, part): the device joins as a follower with a 63-entry peer map (C-217); control handling is bounded (worst-case `poll` time measured and stated) or stamped report-only.

## Work

1. Implement `Device::new(transport, time, audio, identity, profile)` and `Device::poll`.
2. `poll` does a bounded amount of work per call: drain at most N received datagrams (N a const), feed each with its `rx_time_ns` to core, run core timers with `DeviceTime::now_ns`, send core's outgoing messages through `Transport::send` with the C-13 DSCP class core assigns.
3. On first poll: `join_discovery(group)`, derive identity (026a-03), start the core discovery map with a **63-entry** fixed-capacity map (C-217) and the election machine in follower-only mode.
4. Hello always carries holder-eligible = 0 and the current stamp version number (C-18).
5. Define `PollStats { datagrams: u16, messages_sent: u16, elapsed_ns: u32, worst_case_ns: u32 }` (or equivalent): `elapsed_ns` this call measured with `DeviceTime`, `worst_case_ns` running maximum.
6. Control datagrams longer than 1 252 bytes are dropped and counted, never parsed — except the path-MTU probe pair (C-116), which is recognised by its header and handled per core.
7. Host integration test: device on mock transport + a hosted core holder simulation.

## Acceptance criteria

1. Host test: device with mock transport joins a simulated medianet with one holder and appears in the holder's map as a follower.
2. Host test: every hello the device emits has the holder-eligible bit clear (checked over at least 1 000 hellos including across re-elections), and carries the stamp version number.
3. Host test (C-113): with no locked candidate and the device forced (C-120 hook) to the lowest module id, a holder-eligible peer wins the election; the device never becomes holder.
4. Host test: the device tracks 63 peers (64 modules counting itself, C-34, C-217); a 64th distinct peer does not grow memory and is handled per core's full-map policy.
5. Host test: hellos received with domain 0 are ignored and counted, and hellos from a different non-zero domain are not admitted to the map (C-173).
6. Host test: two simulated peers sharing a module id with different nonces are quarantined until only one nonce has been seen for 3 s (C-46).
7. `poll` processes at most N datagrams per call (N documented); a burst of 10×N datagrams is drained over ≥10 calls.
8. `PollStats.worst_case_ns` is monotonically non-decreasing and equals the max `elapsed_ns` seen.
9. A 1 253-byte control datagram is dropped and counted; a path-MTU probe at media size is not (C-116). The 64-bit message counter increases on every control message sent (C-115).
10. Control-message parsing paths added here are fuzzed at the C-95 budget. Device build has no alloc / libm symbols.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The crate builds for at least one device target and for the host (for tests) in CI
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope

- Servo and holdover (026a-06); admission (026a-07); media path (026a-08).
- Measuring worst-case `poll` on hardware and deciding bounded vs report-only (026b-06).
- Stamp publication (026a-11).
