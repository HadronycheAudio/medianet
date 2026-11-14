# 014-04 — Receiver report parser
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-03, epic 004
## Context
The producer receives receiver reports on the reverse unicast port (epic 012) and reads them as a borrowed view, without copying or allocating. Report content: `at_position: u64`, up to 32 members (C-08) of `MemberCounters { stream, session, received, late, missing, marked_gap, other_epoch, too_early, duplicates, incomplete_reassemblies, demoted_rounds, interrupted, health }` (the C-159 counts, 014-03) (stream number 16 bits and session 32 bits, C-10; one session per stream, C-19; health includes Demoted, C-109), `DropCounters { socket_overflow, iface_rx_dropped, iface_rx_missed }` (each `Option<u32>`; absent on platforms with no counter), and the observed DSCP of received media (`Option<u8>`; expected 46 for media, C-13). A valid report is at most 1 252 bytes (C-17, C-100).

Signatures from the epic (names may be refined):
```rust
pub struct ReceiverReport<'a> { /* borrowed view */ }
impl<'a> ReceiverReport<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Reject>;
    pub fn at_position(&self) -> u64;
    pub fn members(&self) -> impl Iterator<Item = MemberCounters> + '_;
    pub fn drops(&self) -> DropCounters;
    pub fn observed_dscp(&self) -> Option<u8>;
}
```
`Reject` is the rejection type of epic 004.

## Work
1. Implement `ReceiverReport::parse`: check message type, length, member count ≤ 32, that the length matches the member count exactly, presence flags, health values, and that any reserved bits are zero per epic 004.
2. Implement `at_position()`, `members()`, `drops()`, `observed_dscp()` reading from the borrowed bytes.
3. No allocation, no panics, no `unsafe`. Work on a bad input stays linear in its length (C-164; checked in 014-25).

## Acceptance criteria
1. Round trip: for 0, 1, 16 and 32 members, with each `Option` field present and absent and with members carrying different sessions, `parse(encode_report(...))` returns the same values field by field.
2. `parse` returns `Err` (never panics) for: empty input, truncated input at every byte length shorter than a valid report, a member count of 33, trailing bytes, an unknown health value, and a wrong message type.
3. `members()` yields exactly the encoded member count.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed (fuzz target delivered in 014-05)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Shared vectors and the fuzz target (014-05).
- Reverse-port message dispatch and counting of unknown types (014-12).
- Ignoring member records of a stale session (014-22).
