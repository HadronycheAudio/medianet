# 017b-03 — Link and hop-delay injection
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** M · **Depends on:** 017b-01, epic 010, epic 015

## Context
- L0 needs deterministic, seeded packet faults between modules in one process. API (in `hadronyche-std/src/inject.rs`, from 017b-01):
```rust
pub struct LinkInjection {
    pub seed: u64,
    pub loss: f64,              // probability per packet
    pub delay_ns: (i64, i64),   // uniform range added per packet
    pub reorder: f64,           // probability a packet is held one slot
    pub drop_above_bytes: Option<usize>,  // MTU blackhole, IPv4 packet size
}
fn set_link(&self, from: ModuleHandle, to: ModuleHandle, inj: LinkInjection);
fn set_hop_delay(&self, module: ModuleHandle, delay_ns: i64);
```
- Faults apply per **packet** (after the packetiser of epic 010), not per slice. A slice is 48 frames = 0.5 ms at 96 kHz, the only rate on the wire (C-01, C-02); 6 channels per stream, 1 152 bytes of audio per slice (C-04).
- The medianet is IPv4 only (C-100). `drop_above_bytes` compares the IPv4 packet size (IP header + UDP header + payload), the quantity a path MTU limits. The 1 280-byte fallback (C-06) leaves 1 252 bytes of UDP payload (C-17).
- Loopback's MTU is 65 536, so loopback never produces small packets by itself `[decision]`; `drop_above_bytes` is the MTU blackhole that lets L0 exercise path-MTU detection and the fallback.
- The injector's decision log is L0's ground truth: AC-8 and AC-15 assert exact equality between the runtime's loss counters and this log, so the log must be complete and deterministic.
- L0 does not use the simulation jitter profile (C-94); delays are seeded uniform ranges.
- **Open sub-question owned by this ticket:** the meaning of "held one slot" for `reorder`. Neither the epic nor the constants table defines it. Choose a definition (for example "delivered immediately after the next packet on the same directed link"), state it in rustdoc and propose it to the constants table through 017b-26. Falsifier: a reorder test passes under one reading and fails under another.

## Work
1. Implement a per-directed-link fault stage in the runtime's send path, driven by a PRNG seeded from `seed` (a fixed, documented algorithm so a seed reproduces exactly the same fault pattern on any machine).
2. Per packet, in this order: drop if IPv4 packet size > `drop_above_bytes`; drop with probability `loss`; add a delay drawn uniformly from `delay_ns`; with probability `reorder`, hold the packet one slot (definition recorded as above).
3. `set_hop_delay(module, delay_ns)`: add a fixed delay to every packet the module emits (used by the diamond scenario on the reference echo).
4. Per-link counters and a decision log exposed to the harness: packets seen, dropped by loss, dropped by blackhole, delayed, reordered, each with packet identity (stream, position, packet index in slice).

## Acceptance criteria
1. Same `seed` and same packet sequence give byte-identical drop/delay/reorder decisions across two runs (test compares decision logs).
2. With `loss = 0.01` over 1 000 000 packets, observed drop rate is within 0.01 ± 0.0005 (more than 5 standard deviations is a failure).
3. With `drop_above_bytes = Some(1280)`, every IPv4 packet of 1 281 bytes or more is dropped and every packet of 1 280 bytes or fewer is not dropped by the blackhole.
4. With `delay_ns = (1_000_000, 2_000_000)`, all observed added delays are within [1 ms, 2 ms].
5. With `reorder = 1.0`, every packet is delivered according to the documented slot definition.
6. `set_hop_delay(m, 3_000_000)` delays every packet from `m` by 3 ms ± scheduling jitter; no other module is affected.
7. Counters equal the decisions in the decision log, and each logged drop names the stream and position it hit.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Clock injection (017b-02); partition, hello suppression, thread kill (017b-04); raw packets (017b-07).
- Scenarios that use these faults (017b-10 to 017b-23).
