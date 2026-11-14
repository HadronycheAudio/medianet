# 011-06 — Fixed 63-entry peer map
**Epic:** 011-Gossip-and-discovery-core · **Effort:** S · **Depends on:** 011-03

## Context
Discovery keeps a fixed-capacity map of peers learned from hellos. It is the data that epic 012 (admission handshake) resolves against: addresses, ports, bundles, capability bits. It is owned by the housekeeping thread; per L-8 `[decision]`, no lock is shared between housekeeping and the data or clock threads — anything exported to other threads goes through messages or a lock-free publication.

Constants:
- C-34 (decided): at most 64 active modules per medianet; commissioning refuses more.
- C-217 (decided): the peer map holds 63 peers: a medianet has at most 64 modules and a module does not hold an entry for itself.
- C-100 (decided): addresses are IPv4 (`Addr { ip: [u8; 4], port: u16 }` from 011-01).
- C-18 (decided): the hello carries the sender's regime stamp version; the map keeps it so that a change can be detected (011-23).
- Epic acceptance criterion 9: a 64th peer (a 65th module counting the receiver) is not inserted; `MapFull` is raised.
- Deliverable: a `const` that reports the map's memory footprint.

Open sub-question (owner: task 026, not this ticket): does a 63-entry map (C-217) with full hellos fit the reference device's memory? Falsifier: it does not; C-34 or the hello content must shrink. This ticket only provides the number task 026 needs.

## Work
1. Implement with the epic's shape:
   ```rust
   pub const MAX_PEERS: usize = 63;   // C-217
   pub struct PeerMap { /* [Option<Peer>; MAX_PEERS] */ }
   ```
   `Peer` stores an owned copy (fixed-size arrays) of the hello fields: id, nonce, address (`Addr`), name, capability bits (incl. holder-eligible), ports with content types, bundles, election epoch, believed holder, timeline epoch, transport role, alarm, liveness counter, regime stamp version, plus bookkeeping: last-hello time (ns), DSCP observed on the last hello (C-152, filled by 011-25), live flag, quarantine flag and the time the second nonce was last seen, last liveness-counter change time.
2. Operations: `get(id)`, `iter()`, `upsert(now_ns, addr, &Hello) -> Result<Slot, MapFull>`, `remove(id)`, `len()`. No allocation. The module's own id is never inserted (C-217); its own hellos are not peers.
3. `pub const PEER_MAP_BYTES: usize = core::mem::size_of::<PeerMap>();` and a test that prints it.
4. Define the `MapEvent` enum here, as given by the epic:
   ```rust
   pub enum MapEvent {
       Joined(ModuleId), Left(ModuleId), Collision(ModuleId), Quarantined(ModuleId),
       NameTaken, ModuleCap, VersionMismatch { peer: ModuleId, theirs: u8 }, ForeignDomain,
       MapFull, Stalled(ModuleId), StampVersionChanged { peer: ModuleId, version: u32 },
   }
   ```
   The width of `version` follows the epic 004 layout.

## Acceptance criteria
1. 63 distinct ids insert successfully; the 64th returns `MapFull` and `len()` stays 63 with all 63 previous entries unchanged (epic AC 9, C-217).
2. After removing one entry, a new id inserts into the freed slot.
3. Upserting an existing id updates its fields without consuming a new slot.
4. `PEER_MAP_BYTES` is a compile-time `const` and its value is reported in the test log and in the ticket's PR description.
5. Builds in the `no_std`, no-allocator CI job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

Not applicable (C-125): "Every parser touched is fuzzed" (consumes parsed views, no parser); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (stores the stamp version only, no stamp field); "Design documents updated" (the footprint number is recorded by 011-22); "No election path reachable from hello expiry" (no expiry or election code).

## Out of scope
- When entries are added, expired or quarantined (the `Discovery` machine): 011-07, 011-10.
- Commissioning's refusal of more than 64 modules: not part of this epic. The newcomer's own ModuleCap refusal at discovery (C-185): 011-26.
- Deciding whether the footprint fits the device: task 026.
