# 009-01 — Rx module scaffold and shared types
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** epic 004

## Context
Epic 009 implements the receive side of the data plane in the crate `hadronyche-core`: packet validation and source filtering, copy-index dedup, multi-packet reassembly, a per-input alignment ring, fixed-time fan-in, the gap classes, the degraded flag, the input health states and default concealment. All of it is pure `no_std` code with no sockets and no allocation; every capacity is a const generic `[decision]`.

This ticket creates the module and the small shared types every later ticket of the epic uses, and proves the module builds for a `no_std` target without an allocator.

Facts used here `[decision]`:
- Gap classes (three): `MarkedGap` (slice present as a gap slice or with the degraded flag; upstream alive, loss further up), `Late` (slice arrives after the deadline; discarded, counted separately; reclassified from missing per C-108), `Missing` (slice absent at the deadline, unknown cause).
- Input health states: `Live`; `Interrupted` (8 consecutive missed slices = 4 ms, C-38, miss as defined by C-102); `Demoted` (persistently late with the offset at A; treated as gapped, counted separately, cleared only by re-admission, C-109); `Dead` (2 s since the last slice with content, a marked gap or another timeline epoch, C-39, C-111, C-135). Dead is a health state, not a gap class.
- Counters the consumer publishes per connection in its regime stamp (C-47, completed by C-159): received, late, missing, marked gap, other-epoch (C-135), too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead. Counters kept for the receiver report (epic 014) and the epic's definition of done but not in the C-159 list: rejected per rejection reason, and packets missing from dropped multi-packet slices (EC-9).
- Rejection reasons, content types and the packet/slice layouts come from epic 004.

## Work
1. In `hadronyche-core`, add a module `rx` (proposed layout: `rx/mod.rs` plus one submodule per later ticket: `filter`, `dedup`, `reassembly`, `drain`, `align`, `fanin`, `degraded`, `conceal`, `counters`). The crate (or at least `rx`) is `#![no_std]` and does not depend on `alloc`.
2. Define the shared types:
   ```rust
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub struct InputIndex(pub u8);           // index of an input inside one consumer
   pub enum GapClass { MarkedGap, Late, Missing }
   pub enum InputHealth { Live, Interrupted, Demoted, Dead }
   pub struct InputCounters {
       pub received: u64, pub late: u64, pub missing: u64, pub marked: u64,   // C-47 gap classes
       pub other_epoch: u64, pub demoted: u64, pub too_early: u64,             // C-159
       pub duplicate: u64,                  // C-159, EC-8
       pub incomplete_reassemblies: u64,    // C-159
       pub reassembly_missing_packets: u64, // EC-9, receiver report only
       pub interrupted: u32, pub dead: u32, // C-47 events
       pub rejected: [u32; REJECT_REASONS],
   }
   ```
   `REJECT_REASONS` is the number of rejection reasons defined by epic 004; `Reject` is re-exported (or wrapped) from epic 004's rejection reason type, and `Reject as usize` (or an equivalent `index()`) indexes `rejected`.
3. `InputCounters::new()` is a `const fn` returning all zeros.
4. Add a CI job that builds `hadronyche-core` for a bare-metal `no_std` target with no global allocator (for example `thumbv7em-none-eabihf`), unprivileged.

## Acceptance criteria
1. `cargo build -p hadronyche-core --target <no_std target>` succeeds in CI with no `alloc` crate linked and no `#[global_allocator]` (epic EC-10, compile part).
2. `InputCounters::new()` is usable in a `const` context and every field reads 0.
3. `rejected` has exactly `REJECT_REASONS` entries, and a unit test maps every epic 004 rejection reason to a distinct index in `0..REJECT_REASONS`.
4. `GapClass` has exactly three variants and `InputHealth` exactly four (checked by an exhaustive `match` in a test).
5. `size_of::<InputCounters>()` is reported by a test (documentation value; no target).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Late, missing, marked and rejected counters are all exposed for the receiver report and the stamp
- [ ] `rx` module and the `no_std` build job merged

## Out of scope
- Any parsing, filtering, reassembly or alignment logic (009-04 to 009-16).
- Counter update rules (009-14) and health transitions (009-15, 009-16, 009-24).
- The no-alloc test allocator (009-02).
- The packet, slice and regime stamp layouts themselves (epic 004).
