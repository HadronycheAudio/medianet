# 011-01 — Core identity types and no_std build
**Epic:** 011-Gossip-and-discovery-core · **Effort:** S · **Depends on:** none

## Context
Epic 011 implements discovery and clock election in the crate `hadronyche-core` as pure `no_std` state machines: no I/O, no allocation, fixed capacities. The hosted and the embedded profile run the same code; a platform supplies only a transport and a clock. This ticket lays down the module and the shared value types every later ticket of the epic uses, and the CI check that the crate builds for a `no_std` target without an allocator (epic acceptance criterion 11).

Facts this ticket needs:
- C-11 `[decision]`: module id 64 bits, instance nonce 64 bits, domain id 16 bits; domain 0 = unconfigured.
- C-133 `[decision]`: all protocol types, control messages included, live in `hadronyche-core`; harness and tool binaries live in the `hadronyche-tools` crate.
- C-100 `[decision]`: IPv4 only in v1, multicast with TTL 1. A peer address is therefore an IPv4 address and a UDP port.
- Identity `[decision]`: the id is 64 bits from the OS random source plus a 64-bit instance nonce. Devices without a true random source mix a hardware unique identifier with a persistent boot counter; otherwise forty identical units would boot with the same id and nonce. The epic places the `IdSource` implementations in the platform epics 015 and 026; core defines only the trait and a deterministic test source.
- `[code]` The retired POC drew the id as the XOR of wall time, pid and monotonic time, 31 effective bits (`module.rs:1113`). That scheme must not be reproduced.

## Work
1. Create a discovery module in `hadronyche-core` (for example `hadronyche-core/src/discovery/mod.rs`), `#![no_std]`-compatible, no `alloc` import.
2. Define the shapes given by the epic (names may be refined, semantics may not):
   ```rust
   pub struct ModuleId(pub u64);
   pub struct Nonce(pub u64);
   pub struct Domain(pub u16);
   pub struct Addr { pub ip: [u8; 4], pub port: u16 }               // IPv4 only (C-100)
   pub struct Outbox { /* fixed-capacity buffer of outgoing datagrams */ }
   pub trait IdSource { fn draw(&mut self) -> (ModuleId, Nonce); }
   ```
   Derive `Copy`, `Clone`, `Eq`, `PartialEq`, `Debug` (and `Ord`/`Hash` where cheap) so later tickets can compare ids. Add `Domain::UNCONFIGURED = Domain(0)` and `Domain::is_configured()`.
3. `Outbox`: a fixed-capacity buffer of outgoing datagrams (destination: the control multicast group or a unicast `Addr`) that the machines fill and the platform drains. No allocation. Keep it minimal; later tickets add only what they need.
4. Provide a deterministic test `IdSource` (seeded sequence) under `#[cfg(test)]` or a `testing` feature, for use by every later ticket.
5. Add a CI job that builds `hadronyche-core` for a bare-metal `no_std` target with no global allocator (for example `thumbv7em-none-eabihf`) and fails if `alloc` or `std` is linked.

## Acceptance criteria
1. `cargo build -p hadronyche-core --target <no_std target>` succeeds with no allocator; the CI job is red if any `alloc`/`std` item is introduced (verified once by a deliberate failing commit on a branch).
2. `ModuleId` and `Nonce` hold the full 64 bits (a unit test round-trips `u64::MAX` and `0`); `Domain` holds 16 bits.
3. `Domain(0).is_configured()` is false; `Domain(1).is_configured()` is true.
4. The test `IdSource` returns a reproducible sequence of `(ModuleId, Nonce)` pairs for a given seed.
5. `Outbox` capacity is a `const`; pushing past capacity returns an error rather than allocating or panicking.
6. `Addr` holds an IPv4 address and a port only; there is no IPv6 variant (C-100).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] The no_std / no-allocator build job exists and runs on every change to `hadronyche-core`

Not applicable (C-125): "Every parser touched is fuzzed" (no parser here); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "Design documents updated" (types follow the epic exactly, no behaviour differs); "No election path reachable from hello expiry" (no expiry or election code).

## Out of scope
- Platform `IdSource` implementations (OS random, hardware id + boot counter): hosted and device platform epics (015, 026).
- The test-only forced-identity hook (C-120): 011-24.
- Hello encoding/parsing: 011-02, 011-03.
- Peer map: 011-06.
