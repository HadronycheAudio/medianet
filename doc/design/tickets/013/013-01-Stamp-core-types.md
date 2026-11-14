# 013-01 — Stamp core types
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** epic 001

## Context
- The **regime stamp** is a compact, versioned wire object in `hadronyche-core` that states under which conditions every measurement was taken. Every record set the runtime writes is bound to a stamp. `[decision]`
- `hadronyche-core` is `no_std` and does no allocation; every type here must satisfy that.
- Capabilities have **four states: present, absent, unverified, not requested** (C-129); "not requested" marks a deliberate configuration choice, as distinct from a denial. The stamped capabilities are: receive timestamp source (with its own state), real-time scheduling granted (on the data and clock threads, C-127), memory locked, CPU idle states restricted (best-effort and stamped, C-127), drop accounting, DSCP marking, fault injection, holder-eligible, and control isolation (housekeeping cannot delay the data path; present on hosted platforms by construction of the thread model, absent or unverified on a device unless the implementer shows a bound, C-161). `[decision]`
- Capability state value 0 is not valid and is rejected (epic 013, settled). `[decision]`
- Profiles: linux-std, macos-std, windows-std, embedded. `[decision]`
- Receive-timestamp source values (C-131): **kernel** (hosted OS kernel stamp), **userspace** (clock read after receive), **mac-hardware** (Ethernet MAC timestamp register, embedded), **interrupt** (stamped in the receive interrupt, embedded), each reported with one of the four C-129 states. Linux uses kernel receive timestamps on a wall-clock basis via `SO_TIMESTAMPNS` `[kernel-doc]`; macOS and Windows claim kernel receive stamps but this is unverified `[memory]`, so they report "kernel, unverified" until task 001 confirms; embedded uses mac-hardware where available, else interrupt.
- The self-test runs before any number is reported and checks: receive timestamps present and consistent with the monotonic clock; real-time scheduling actually granted; measured wake latency; packets per slice; clock locked within its bound. `[decision]`
- The stamp also carries per-connection summary counters (C-47, C-159); their type `ConnSummary` is defined in 013-23, not here.
- Wire values of the enums belong to the byte layout of epic 004; this ticket proposes them and 013-22 records them once epic 004 confirms.

## Work
In `hadronyche-core` (new module, e.g. `stamp::types`), define these types with the shapes the epic gives (names may be refined, shapes may not grow allocation):

```rust
#[repr(u8)] pub enum CapState { Present = 1, Absent = 2, Unverified = 3, NotRequested = 4 }   // C-129
#[repr(u8)] pub enum Profile { LinuxStd, MacosStd, WindowsStd, Embedded }
#[repr(u8)] pub enum RxStampSource { Kernel, Userspace, MacHardware, Interrupt }   // C-131

pub struct Capabilities {
    pub rx_stamp: RxStampSource, pub rx_stamp_state: CapState,
    pub rt_scheduling: CapState, pub memory_locked: CapState, pub cpu_idle_restricted: CapState,
    pub drop_accounting: CapState, pub dscp_marking: CapState, pub fault_injection: CapState,
    pub holder_eligible: CapState, pub control_isolation: CapState,
}
pub struct StampId(pub u32);   // content-derived over regime fields (013-05)
pub struct SelfTest { pub rx_stamps_consistent: bool, pub rt_granted: bool, pub wake_latency_ns: u32,
                      pub packets_per_slice: u8, pub locked_within_bound: bool }
```

1. Give `Profile` and `RxStampSource` explicit discriminants (they are wire values), document them in a doc comment, and mark them "proposed, pending epic 004".
2. Provide `TryFrom<u8>` for `CapState`, `Profile`, `RxStampSource`; any unassigned byte (including 0 for `CapState`) yields an error, never a panic.
3. Derive `Copy, Clone, PartialEq, Eq, Debug` where meaningful; `StampId` also `Hash`.
4. Add the `Reject` error enum skeleton used by the parser (variants are added by 013-02).

## Acceptance criteria
1. The module compiles under `#![no_std]` with no `alloc` dependency (CI builds `hadronyche-core` for a `no_std` target or with `default-features = false`).
2. `CapState` byte values are exactly 1 = Present, 2 = Absent, 3 = Unverified, 4 = NotRequested; unit tests check all four round-trips and that 0 and 5..=255 are rejected (252 values).
3. `Profile` has exactly 4 variants and `RxStampSource` exactly 4 (C-131); each round-trips through `u8`, every other byte value is rejected.
4. `Capabilities` has exactly 10 fields: 1 `RxStampSource` and 9 `CapState` (the receive-stamp state plus 8 capabilities).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- Discriminant values recorded in the module doc comment so 013-22 can copy them into `doc/design/spec/stamp.md`.
- Not applicable (C-125): "Every parser touched is fuzzed" (only `TryFrom<u8>` conversions, exhaustively tested; the stamp parser is 013-02/013-03); "Regime stamp fields touched are complete and documented" and "Every stamp field has a stated source on each of the four profiles" (no section is encoded here; 013-06 onward); "Design documents updated where behaviour differs, constants table first" (no behaviour differs from the design); "The stamp format is versioned independently of the media format" (no framing here; 013-02).

## Out of scope
- Encoding these types into sections (013-06 onward), the section framing (013-02), `StampId` derivation (013-05), `qualified()` (013-10), `RecordHeader` (013-14), `ConnSummary` (013-23).
- Which source and state each OS really provides (task 001); this ticket only defines the types.
