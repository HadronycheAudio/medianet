# 009-04 — Source filter
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-01, 009-02, epic 004

## Context
Receive-side enforcement `[decision]`: only admitted peers are parsed. A peer is identified by the source address pinned at admission plus the stream number. The filter is the first thing a received datagram meets on the data thread, before reassembly and alignment. Together with the per-cycle drain bound (C-16, 009-03) it keeps data-thread time bounded.

- C-100 `[decision]`: IPv4 only in v1. A source address is an IPv4 address and a UDP port; there is no IPv6 case.
- C-10 `[decision]`: the stream number is 16 bits.
- Packet layout, stream number field and the list of rejection reasons come from epic 004.
- Rejected packets are counted per reason in `InputCounters::rejected[REJECT_REASONS]` (type from 009-01).
- C-08 `[decision]`: at most 32 members per bundle, the controller member included (C-209); the filter capacity `N` is a const generic and is not tied to C-08.

## Work
In `hadronyche-core`, module `rx::filter`:
```rust
pub struct Addr { pub ip: [u8; 4], pub port: u16 }   // IPv4 only (C-100), unless epic 004 provides the type
pub struct SourceFilter<const N: usize> { /* pinned (addr, stream) pairs -> InputIndex */ }
impl<const N: usize> SourceFilter<N> {
    pub const fn new() -> Self;
    pub fn pin(&mut self, addr: Addr, stream: StreamId, input: InputIndex) -> Result<(), Full>;
    pub fn unpin(&mut self, input: InputIndex);           // used on Dead (re-admission needed)
    pub fn admit(&self, from: &Addr, pkt: &[u8]) -> Result<InputIndex, Reject>;
}
```
`admit` order: (1) structural validation of the header with epic 004's parser (length, version, fields), (2) lookup of `(from, stream)` among pinned pairs. Unknown source address or unknown stream for a known address returns the matching epic 004 rejection reason. The payload beyond the header is not touched.

## Acceptance criteria
1. A valid packet from a pinned `(addr, stream)` returns its `InputIndex`.
2. The same packet from a different source address (other IPv4 address, or same address with another port) returns `Err(Reject)` with the "unknown source" reason of epic 004; from the pinned address with a stream number that is not pinned, returns the "unknown stream" reason (or the closest epic 004 reason).
3. Truncated packets of every length 0 .. header length − 1 are rejected without panic.
4. `pin` on a full filter (`N` entries) returns `Err(Full)`; after `unpin`, a packet from the removed pair is rejected.
5. Lookup is bounded: a test with `N = 32` pinned pairs shows `admit` performs at most 32 comparisons (or uses a fixed-size index).
6. All tests run under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed (fuzz target delivered in 009-09)
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Incrementing `InputCounters::rejected` from a fan-in context (009-14); this ticket returns the reason only.
- Socket creation, buffer sizing, kernel overflow counter, observed DSCP (backend).
- Admission control itself (deciding who gets pinned): other epics.
- Conformance vectors (009-08) and fuzzing (009-09).
