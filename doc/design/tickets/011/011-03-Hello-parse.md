# 011-03 — Hello parse
**Epic:** 011-Gossip-and-discovery-core · **Effort:** M · **Depends on:** 011-02, epic 004

## Context
The receive side of the hello (see 011-02 for its content, which includes the regime stamp version, C-18, and the authentication reservation with scheme 0, zero tag and live counter, C-115). `Hello` is a *borrowed view* over a received buffer: parsing does not copy and does not allocate. Parsing is where the domain and version filters apply.

Rules `[decision]`:
- A foreign-domain hello (a non-zero domain != my domain) is dropped and counted.
- A hello carrying domain 0 is ignored and counted under its own reason `UnconfiguredDomain`, whatever my domain; it is not a foreign-domain refusal (C-173).
- A version mismatch is counted and is a named event; the version is in the hello precisely so it is detected before any connection. VersionMismatch is raised at discovery, not at admission (C-185).
- Malformed input (truncated, out-of-range counts, lengths past the buffer, size > 1 252 bytes per C-17) is dropped and counted. A bundle with more than 32 members (C-08), the controller member included (C-209), is an out-of-range count. Refusing a connection for an unknown or duplicate member is admission's BadMemberList (C-173, epic 012), not a parse reject.
- C-164 (proposal, pending L1): processing one bad input is linear in packet length and at most 10 µs per packet.
- Domain is 16 bits; domain 0 = unconfigured (C-11).
- The epic defines `Reject` as `ForeignDomain`, `UnconfiguredDomain`, `Malformed`, `VersionMismatch { peer: ModuleId, theirs: u8 }`.

Epic acceptance criterion 10 (AC-23): foreign-domain, domain-0, malformed and wrong-version hellos cause no state change and are counted per reason; a version mismatch produces a named event. This ticket supplies the reason; 011-08 does the counting and the event in `Discovery`.

## Work
1. Implement, with the epic's signature:
   ```rust
   impl<'a> Hello<'a> {
       pub fn parse(bytes: &'a [u8], my_domain: Domain) -> Result<Self, Reject>;
   }
   ```
2. Define `Reject` as in the epic. Parse order: header check, then domain, then version, then body, so that a wrong-version hello still yields the peer id and their version.
3. Accessors on `Hello` for every field listed in 011-02 (module id, nonce, domain, version, name, capability bits incl. holder-eligible, ports, bundles, election epoch, believed holder, timeline epoch, transport role, alarm, liveness counter, regime stamp version). Collections are exposed as iterators over the borrowed buffer.
4. No panics on any input: all index arithmetic checked.
5. Single forward pass over the buffer, no backtracking, so the cost of a bad input is linear in its length (C-164).

## Acceptance criteria
1. Round-trip: `parse(encode(info))` returns every field equal to `info`, over at least 10 000 random `LocalInfo` values.
2. A hello with non-zero domain D parsed with `my_domain != D` returns `Err(Reject::ForeignDomain)`; a hello with domain 0 returns `Err(Reject::UnconfiguredDomain)` for any `my_domain`, including 0 (C-173).
3. A hello with a different protocol version returns `Err(Reject::VersionMismatch { peer, theirs })` carrying the sender's id and version.
4. Each truncation of a valid hello to lengths 0..len-1 returns `Err(Reject::Malformed)` (or `ForeignDomain`/`VersionMismatch` if the field is already readable), never a panic.
5. Input longer than 1 252 bytes returns `Err(Reject::Malformed)`; a bundle declaring 33 members, counting a controller member, returns `Err(Reject::Malformed)` (C-08, C-209).
6. A benchmark over malformed inputs of increasing length shows parse time growing at most linearly; the per-packet figure is reported for the L1 check of the 10 µs bound (C-164).
7. Builds in the `no_std`, no-allocator CI job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed (fuzz target delivered in 011-05)
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

Not applicable (C-125): "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (reads the stamp version only, no stamp field); "Design documents updated" (behaviour follows the epic and the epic 004 layout); "No election path reachable from hello expiry" (no expiry or election code).

## Out of scope
- Counting rejects and raising `MapEvent::ForeignDomain` / `MapEvent::VersionMismatch`: 011-08.
- The fuzz target: 011-05.
- Strict-mode handling of domain 0 at startup: 011-09.
- Confirming the 10 µs per packet bound on target hardware: L1 (C-164).
- The BadMemberList admission refusal (C-173): epic 012.
