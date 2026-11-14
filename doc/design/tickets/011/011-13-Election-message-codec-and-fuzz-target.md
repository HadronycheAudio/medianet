# 011-13 — Election message codec and fuzz target
**Epic:** 011-Gossip-and-discovery-core · **Effort:** S · **Depends on:** 011-01, epic 004

## Context
The election machine receives election messages as raw bytes (`ElectionInput::Message(&[u8])`) and sends its own through the `Outbox`. The byte layout is owned by epic 004 (election message layout). Constraints:
- C-17 (decided): a control message is at most 1 252 bytes of UDP payload (IPv4, C-100).
- C-11: module id 64 bits, domain 16 bits.
- C-12 / C-115: if the layout carries the authentication reservation, scheme is 0 and tag is zero in v1.
- Election epoch is `u32` (from the epic's `ElectionEvent::Opened { epoch: u32 }`).
- C-164 (proposal, pending L1): processing one bad input is linear in packet length and at most 10 µs per packet.
- C-192 (decided): the fuzzing tool is the implementing team's choice.
- C-95 (decided): each parser is fuzzed 60 s on every change and 1 hour nightly; the corpus is kept.
- Definition of done: every parser touched is fuzzed. Deliverable: a fuzz target for election messages.

## Work
1. In `hadronyche-core`, implement encode and borrowed-view parse for each election message type of epic 004, `no_std`, no allocation, never panicking.
2. Reuse the `Reject` reasons of 011-03 where they apply (foreign domain, unconfigured domain per C-173, malformed, version mismatch). Parse in a single forward pass, so the cost of a bad input is linear in its length (C-164).
3. Add a fuzz target `election_parse` with a round-trip oracle, seeded with golden messages.
4. CI: 60 s on every change to the election codec, 1 hour nightly; the corpus is persisted between runs and crashing inputs are added to it (C-95).

## Acceptance criteria
1. Golden-byte tests match the epic 004 examples for every message type.
2. Round trip `parse(encode(m)) == m` over at least 10 000 random messages.
3. No encoded message exceeds 1 252 bytes.
4. The per-change CI job fuzzes 60 s and the nightly job 1 hour; a 1-hour run finds no panic and no round-trip mismatch; the corpus of one run seeds the next (C-95).
5. A benchmark over malformed inputs of increasing length shows parse time growing at most linearly; the per-packet figure is reported for the L1 check of the 10 µs bound (C-164).
6. Builds in the `no_std`, no-allocator CI job.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag

Not applicable (C-125): "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "Design documents updated" (layout is epic 004's); "No new open item without an owner and a falsifier" (opens no item; layout questions go to epic 004); "No election path reachable from hello expiry" (codec only, no transitions).

## Out of scope
- What the machine does with messages: 011-15 to 011-18.
- Confirming the 10 µs per packet bound on target hardware: L1 (C-164).
