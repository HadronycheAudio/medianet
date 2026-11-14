# 004-03 — Rejection reason enumeration
**Epic:** 004-Wire-specification · **Effort:** S · **Depends on:** 004-01, 004-02
## Context
Every malformed input must have a required outcome: drop and count under a named reason, or raise a named event. Epic 004 delivers the list of rejection reasons as a closed enumeration. Its starting shape (names may be refined) is:

```rust
// shape, names may be refined
#[repr(u8)]
pub enum Reject {
    TooShort, BadMagic, VersionMismatch, ForeignDomain, UnknownStream,
    UnadmittedSource, BadLength, BadChecksum, BadPacketIndex, WrongEpoch,
    OversizeControl, ReservedNonZero,
}
```

Related rules `[decision]`:
- Rule 12: a version mismatch is a named event, never a silent drop; the version is also carried in the hello so it is detected before a connection exists.
- Rule 13: foreign-domain packets are dropped and counted. Domain id is 16 bits, domain 0 = unconfigured (C-11). C-173: `ForeignDomain` is for a different non-zero domain; a hello with domain 0 is ignored and counted, not refused, so it needs its own counting reason.
- C-17: control messages are at most 1 252 bytes of UDP payload.
- System criterion AC-23: malformed, wrong-version and foreign-domain traffic causes no crash and no state change; each rejection is counted per input and per reason; version mismatch produces a named event.
- The POC's `Header::decode` returned `None` silently on version mismatch `[code]`; that behaviour is retired.
- The design uses "input" for an admitted stream at a consumer: the bounded drain is per input (C-16) and an input can be Demoted (C-109) `[decision]`. Traffic that matches no admitted input (foreign domain, unknown source, bad magic) still needs a counting key; no constant gives it, so this ticket decides it.
- C-164 (proposal, pending L1): processing one bad input is linear in packet length and takes at most 10 µs per packet; the reasons must be decidable within that bound.
- Refusal reasons (the closed set of 18 named in C-185, for example `RequiredEExceeded`, C-103) form a separate enumeration owned by 004-18; they are outcomes of discovery or of a handshake, not packet rejections. `VersionMismatch` and `ForeignDomain` are names in both: as refusals they come from C-185 (`VersionMismatch` raised at discovery), as packet rejections they are counted (`VersionMismatch` also raises a named event, rule 12); the two enumerations keep separate numeric values.
- The crate and module of the Rust `Reject` type are chosen by implementing tasks 009 to 014; this epic specifies the names and numeric values in `wire.md` and the machine-readable file.

## Work
1. Add a "Rejection reasons" section to `doc/design/spec/wire.md`: one row per variant with numeric `u8` value, name, meaning, and which layers can produce it.
2. Assign explicit numeric values (stable across minor versions); reserve a range for future variants and state that the enumeration is closed within a major version.
3. State the counting model: every rejection increments a counter keyed by (input, reason), as AC-23 requires. Use the design's meaning of input (admitted stream at a consumer, C-16, C-109) for traffic that matches one, and define the key for traffic that matches none (per socket, per interface or per source address; decide and tag `[decision]`), with a fixed bound on the number of counters.
4. Distinguish reasons that also raise a named event (at least `VersionMismatch`) from those that only count.
5. Refine names where needed and add any variant the message tickets will need, at least one for a domain-0 packet, which is counted but is not `ForeignDomain` (C-173); keep a "changes from the starting shape" list.
6. Add the enumeration to the machine-readable file (004-02) so the checker can validate references to it.

## Acceptance criteria
1. The section lists at least the 12 starting variants (or a documented rename of each) with a unique `u8` value each.
2. `VersionMismatch` is specified as raising a named event in addition to being counted.
3. The counting key (input, reason) is defined both for traffic matching an admitted input and for traffic matching none, and the counter set has a stated fixed bound.
4. The machine-readable file contains the enumeration and the 004-02 checker's reason-reference check is enabled and passes.
5. The enumeration is declared closed, with a reserved value range for future major versions.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The machine-readable tables parse and round-trip
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Mapping each invalid form of each message to a reason: done in each message ticket and audited in 004-21.
- Implementing counters or events in code (tasks 009 to 014).
