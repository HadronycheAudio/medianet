# 001-12 — Capability-bit definitions
**Epic:** 001-Receive-stamp-probes · **Effort:** S · **Depends on:** 001-02, 001-04, 001-05, 001-06, 001-08, 001-09, 001-10, 001-16

## Context
The regime stamp (a wire object every node publishes) carries capabilities with four states: present, absent, unverified, not requested; "not requested" marks a deliberate configuration choice, as distinct from a denial (C-129). It travels in a dedicated control message split into sections that each fit the control message bound (1 252 bytes of UDP payload, C-17), multicast to the control group on any regime change and refreshed every 10 s, with a unicast request for an immediate fetch; the hello carries only a stamp version number (C-18). The wire encoding and width of the bits belong to the regime stamp specification (task 013). Epic 001 defines the bits for receive timestamp source, drop accounting and don't-fragment, as a table to be copied into that specification.

Settled (C-131): the receive-timestamp source values are kernel (hosted OS kernel stamp), userspace (clock read after receive), mac-hardware (Ethernet MAC timestamp register, embedded) and interrupt (stamped in the receive interrupt, embedded); each is reported with one of the four states of C-129. The clock basis (monotonic or wall clock) is not a source value; it is reported in `REPORT-0a.md` for the backends' step detection (C-121).

Shape given by the epic (names may be refined):

```rust
// shape, names may be refined
pub enum CapabilityState { Present, Absent, Unverified, NotRequested } // C-129
pub struct RxStampCapabilities {          // one state per source value, C-131
    pub kernel: CapabilityState,
    pub userspace: CapabilityState,
    pub mac_hardware: CapabilityState,     // embedded only
    pub interrupt: CapabilityState,        // embedded only
}
pub struct StampCapabilities {
    pub rx_stamp: RxStampCapabilities,
    pub drop_accounting: CapabilityState,
    pub dont_fragment: CapabilityState,
}
```

Known on Linux: `SO_TIMESTAMPNS` gives a `CLOCK_REALTIME` stamp `[kernel-doc]`, so `kernel` is present, on a wall-clock basis; wall-clock step detection is done by the platform backend that builds exchanges, never by the servo (C-121). `SO_RXQ_OVFL` gives per-socket drop accounting `[kernel-doc]`. The network is IPv4 only (C-100), so `dont_fragment` refers to the IPv4 DF bit. macOS and Windows values come from 001-02, 001-04, 001-05, 001-06, 001-08, 001-09, 001-10 and 001-16. Strict mode treats an unverified capability as absent (C-128), so the rule for `Unverified` matters to every strict-mode module.

## Work
- Write a table section `Capability bits` (in `doc/design/reports/REPORT-0a.md` or a file it references, C-130) listing, for each field: name, type, every value with its meaning as settled by C-129 and C-131, and the rule for when a node reports it.
- Give a table of expected values per platform (Linux, macOS at or above its minimum version, macOS below it, Windows ≥ minimum build, Windows below minimum build) with evidence tags (`[kernel-doc]` / `[measured]`); `mac_hardware` and `interrupt` are absent on every hosted platform (C-131).
- State the rule for `Unverified` (for example: a field is `Unverified` until a node's own probe at start-up confirms it) and for `NotRequested` (the option was deliberately not enabled by configuration, C-129) as `[decision]`, proposed to task 013.

## Acceptance criteria
1. Table covers the three fields `rx_stamp` (with its four source values, C-131), `drop_accounting`, `dont_fragment` and all four states (C-129), each with a definition.
2. Per-platform expected-value table has one row each for Linux, macOS at/above its minimum version, macOS below it, Windows at/above minimum build, Windows below it (a "below" row may state that no such version could be obtained); every macOS / Windows cell is backed by a `[measured]` result file path.
3. Table is self-contained so task 013 can copy it without edits; it states no wire encoding.

## Definition of done
- [ ] Regime stamp fields touched are complete and documented
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

Not applicable (C-125): "Tests green in CI, unprivileged", "Every parser touched is fuzzed" and "Probe sources committed" — no code; "Design documents updated" — 001-14; "The frozen POC is not modified" — not touched; "`REPORT-0a.md` delivered before task 004" — 001-13.

## Out of scope
- Wire encoding and width of the bits, and the regime stamp specification itself (task 013).
- Regime stamp transport (settled by C-18).
- Backend code using the bits (tasks 015, 021, 022).
