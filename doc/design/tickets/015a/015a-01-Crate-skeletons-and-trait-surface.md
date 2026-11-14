# 015a-01 — Crate skeletons and trait surface
**Epic:** 015a-Hosted-runtime-core · **Effort:** S · **Depends on:** epic 008, epic 009, epic 010
## Context
- Epic 015 builds the hosted (operating-system) half of the runtime for Linux. It drives the pure `no_std` state machines and codecs of `hadronyche-core` against real sockets and threads and contains no protocol logic of its own.
- Two crates are introduced: `hadronyche-std` (hosted common, OS-independent) and `hadronyche-os-linux` (Linux implementation).
- The trait surface defined here is also what the macOS (epic 021) and Windows (epic 022) backends will implement, so it must compile against a stub backend with no Linux dependency.
- IP is IPv4 only in v1 (C-100), so addresses are `SocketAddrV4` throughout.
- Receive-timestamp sources (C-131): kernel (hosted OS kernel stamp), userspace (clock read after receive), mac-hardware (Ethernet MAC timestamp register, embedded), interrupt (stamped in the receive interrupt, embedded). `StampSource` carries all four so the stamp encoding is uniform; Linux produces only `Kernel` and `Userspace`.
- Trait shapes given by the epic (names may be refined):

```rust
/// Host clock H, in nanoseconds, monotonic.
pub trait HostClock: Send + Sync {
    fn now_ns(&self) -> i64;
    fn declared_jitter_ns(&self) -> u32;
}

/// Receive-timestamp source (C-131); Linux produces Kernel or Userspace only.
pub enum StampSource { Kernel, Userspace, MacHardware, Interrupt }

pub struct RxMeta {
    pub len: usize,
    pub from: core::net::SocketAddrV4,
    /// Receive time on the host clock; None if the sample was discarded (wall-clock step).
    pub rx_host_ns: Option<i64>,
    pub source: StampSource,
    /// Cumulative drops on this socket, when the platform provides it.
    pub dropped_total: Option<u32>,
    /// Received TOS byte, for peer-verified DSCP.
    pub tos: Option<u8>,
}

pub trait DatagramSocket {
    /// Receive up to `max` datagrams without blocking. Returns the count.
    fn recv_batch(&mut self, bufs: &mut [RxBuf], meta: &mut [RxMeta], max: usize) -> usize;
    /// Never blocks; a full queue is a counted drop at the sender.
    fn send_to(&mut self, packet: &[u8], to: core::net::SocketAddrV4) -> Result<(), SendError>;
}

pub trait OsNet {
    type Socket: DatagramSocket;
    fn open_media(&self, iface: IfaceId, class: DscpClass, rcvbuf: usize) -> io::Result<Self::Socket>;
    fn open_discovery(&self, iface: IfaceId, group: core::net::SocketAddrV4) -> io::Result<Self::Socket>;
    fn iface_stats(&self, iface: IfaceId) -> IfaceStats;
}
```

- Supporting types, now defined by the epic:
  - `RxBuf` — one preallocated receive buffer, large enough for the largest datagram the socket admits (interface MTU minus IPv4 and UDP headers).
  - `SendError` — `QueueFull` (counted drop at the sender), `MessageTooBig` (would need fragmentation; never fragmented), `Other(errno)`.
  - `IfaceId` — the medianet interface, by kernel interface index.
  - `DscpClass` — `Sync` (48), `Media` (46; also one-way probes), `Control` (40) (C-13).
  - `IfaceStats` — interface MTU, link speed (or unknown), `rx_dropped`, `tx_dropped`.
  - `CapabilityState` — `Present`, `Absent`, `Unverified`, `NotRequested` (C-129), used by 015a-06 and 015b-13.
## Work
1. Create crate `hadronyche-std` with empty modules `process.rs`, `supervisor.rs`, `data_thread.rs`, `clock_thread.rs`, `housekeeping.rs`, `pacing.rs`, `files.rs` (filled by later tickets), and a module holding the trait surface above.
2. Create crate `hadronyche-os-linux` with empty modules `net.rs`, `ifstats.rs`, `clock.rs`.
3. Define the supporting types exactly as listed in Context, and document each in the crate docs.
4. Add a stub backend (e.g. `StubNet`, `StubClock`, in a test module or a `stub` feature) implementing `HostClock`, `DatagramSocket` and `OsNet` with no OS calls, and a test that instantiates it through the generic traits.
5. Wire both crates into the workspace and CI (unprivileged).
## Acceptance criteria
1. Both crates build in CI; `hadronyche-std` has no dependency on `hadronyche-os-linux`.
2. A stub backend implementing `HostClock`, `DatagramSocket` and `OsNet` compiles and a unit test exercises each trait method at least once.
3. `DscpClass` maps `Sync` → 48, `Media` → 46, `Control` → 40 (unit test).
4. `RxMeta` has exactly the fields `len`, `from`, `rx_host_ns`, `source`, `dropped_total`, `tos` (or a documented refinement).
5. `StampSource` has the four C-131 values and `CapabilityState` the four C-129 values.
6. `SendError` has distinct `QueueFull` and `MessageTooBig` variants; `IfaceStats` represents an unknown link speed explicitly.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The trait surface compiles with a stub backend (proves macOS and Windows can implement it)
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — no stamp field; "Per-packet receive cost measured" — no receive path yet (015b-10).
## Out of scope
- Any Linux socket or clock implementation (015a-03 to 015a-07).
- Thread loops (015a-08 onward).
- The macOS and Windows backends themselves (epics 021, 022).
