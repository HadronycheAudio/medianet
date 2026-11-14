# 026a-02 — HAL traits and host mocks
**Epic:** 026a-Embedded-HAL-and-host-integration · **Effort:** S · **Depends on:** 026a-01, epic 018, epic 024a

## Context

The one implementation rule `[decision]`: control logic lives in `hadronyche-core` as pure state machines. A device does **not** reimplement hello, election or sync logic; it implements a transport trait (send and receive datagrams, join the discovery group, read time) and drives core. The traits below are the whole hardware surface of a device. Shapes are given by the epic; names may be refined.

Settled facts:

- IPv4 only, multicast with TTL 1 (C-100). `Endpoint` is an IPv4 address and a UDP port.
- `DscpClass` has the three C-13 classes: sync CS6 (48), media and one-way probes EF (46), control CS5 (40).
- Receive-timestamp source values are those of C-131; on a device: mac-hardware, interrupt, or userspace (a clock read after receive, in the poll loop). The kernel value does not occur on a device.
- `DevicePair` is a (frame count, timer-captured device time) pair, the loop-2 input of the two-loop edge core (epic 018).
- The device's hardware surface is five traits. Four are defined here (`Transport`, `DeviceTime`, `AudioOut`, `Identity`) and `Device` is generic over them. The fifth, `RateActuator`, is defined by the arithmetic kit (epic 024a) together with its implementation, with a ratio range of ±1 000 ppm around the nominal ratio, and every edge consumes it (C-210): this crate re-exports it and does not define it. `Device` holds it internally, with the software-ratio implementation, an adapter over the kit's rate actuator (epic 024a, C-197), as the v1 default (026a-09).
- Control messages are binary and bounded: at most 1 252 bytes of UDP payload (C-17). A media packet is about 1 320 bytes on the wire (C-20).

## Work

In `hadronyche-embedded` (no_std, no alloc) define:

```rust
pub trait Transport {
    type Error;
    fn send(&mut self, to: Endpoint, class: DscpClass, packet: &[u8]) -> Result<(), Self::Error>;
    fn recv(&mut self, buf: &mut [u8]) -> Option<Rx>;
    fn join_discovery(&mut self, group: Endpoint) -> Result<(), Self::Error>;
    fn mtu(&self) -> u16;
}
pub struct Rx { pub len: usize, pub from: Endpoint, pub rx_time_ns: i64, pub source: StampSource }
pub enum StampSource { MacHardware, Interrupt, Userspace }   // C-131 values available on a device

pub trait DeviceTime {
    fn now_ns(&self) -> i64;                 // monotonic, 64-bit
    fn declared_jitter_ns(&self) -> u32;
}

pub trait AudioOut {
    fn rate(&self) -> u32;                   // 48_000 in v1
    fn write(&mut self, frames: &[i32]) -> usize;
    fn position(&self) -> DevicePair;        // (frame count, timer-captured device time)
    fn fixed_delay_ns(&self) -> u32;         // converter and interface part of D
}

pub use kit::RateActuator;               // defined by epic 024a (C-210), not here

pub trait Identity {
    fn hardware_unique(&self) -> [u8; 16];
    fn next_boot_counter(&mut self) -> u64;  // persistent
    fn true_random(&mut self) -> Option<[u8; 16]>;
    fn domain_id(&self) -> u16;              // persistent; 0 = unconfigured (C-11, C-173)
}
```

- `Endpoint` (IPv4 address + UDP port), `DscpClass` (CS6, EF, CS5) and `DevicePair`: reuse the core / epic-018 types if they exist; otherwise define them here as stated above.
- `RateActuator` is re-exported from the kit (epic 024a, C-210); it is never defined in this crate. If epic 018 already defines `DevicePair`, re-export rather than duplicate.
- Host mock implementations in a `mock` module behind a `std`/`test` feature: in-memory `Transport` (queue of datagrams with injectable rx time and `StampSource`), settable `DeviceTime` (including injectable jumps, used by the step-detection test of 026a-06), `AudioOut` recording written frames with a scripted `position()`, `Identity` with configurable unique id, in-memory boot counter, settable domain id and switchable true random.
- Rustdoc on each item, citing C-13, C-100 and C-131 where they apply.

## Acceptance criteria

1. The four traits defined here, the re-exported kit `RateActuator` (C-210), and `Rx`, `StampSource`, `Endpoint`, `DscpClass`, `DevicePair` compile for the device target and for the host; `hadronyche-embedded` contains no `trait RateActuator` definition of its own.
2. `Endpoint` cannot hold an IPv6 address (C-100); `DscpClass` has exactly the three C-13 classes; `StampSource` has exactly the three device values of C-131.
3. Mocks pass a smoke test: a datagram sent through mock A is received by mock B with the injected `rx_time_ns` and `source`.
4. Mock `Identity::next_boot_counter` returns strictly increasing values across calls (and across a simulated "reboot" that keeps the backing store).
5. No `alloc` in non-mock code (device build still passes the 026a-01 symbol check).

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] The crate builds for at least one device target and for the host (for tests) in CI
- [ ] Every new factual claim carries an evidence tag

## Out of scope

- `Device` struct, `poll` and `PollStats` (026a-05).
- Identity derivation logic (026a-03).
- The software-ratio `RateActuator` implementation (026a-09).
- Real board implementations (026b-02, 026b-03).
