# 007a-13 — Bad-input outcome vectors (AC-23)
**Epic:** 007a-Wire-vectors-and-harness · **Effort:** M · **Depends on:** 007a-04, 007a-11

## Context
- Epic criterion 007-AC8 (system criterion AC-23): malformed, wrong-version and foreign-domain inputs produce no crash and no state change; each is counted per input and per reason; a version mismatch produces a named event.
- Required outcomes for bad input `[decision]`:
  - never crash, never change state, bounded time; drop and count per input and per reason;
  - a version mismatch is a named event;
  - a foreign-domain packet (domain field, 16 bits, not the local domain; domain 0 means unconfigured, C-11) is dropped and counted;
  - a packet from a source not pinned at admission is not parsed beyond the filter;
  - reserved fields must be zero in v1: the copy index (C-07), and the authentication scheme and tag (C-12, C-115). The 64-bit counter is the live liveness counter (C-115) and is not a reserved field.
- "Bounded time" is C-164: linear in packet length and at most 10 µs per packet. It is a proposal pending L1, so each vector file carries the bound it was checked with (the C-164 values), and `conformance.md` (007b-23) states it as pending L1.
- A hello with domain 0 is ignored and counted, not refused; ForeignDomain is the refusal for a different non-zero domain (C-173). The hello case is a discovery scenario (007b-01); this ticket covers the media packet drop.
- The `UnderTest` trait and its observation extension (counters, events, state digest, local domain, pinned sources) come from 007a-04; on a real implementation, raw packets are injected with the test-only hook of C-120.

## Work
1. Write scenario-style wire vectors in `vectors/wire/invalid/` that set up a configured state (local domain, pinned sources, an admitted connection), feed one bad input, and expect: the rejection reason, the counter for that reason +1, the state digest unchanged, and for version mismatch the named event.
2. Cover: malformed (one per reason from 007a-11, reusing the bytes), wrong version, foreign domain, unpinned source (expect no parse beyond the filter: the reason is the filter reason, not a later parse reason), each non-zero reserved field.
3. Add a sequence vector of N = 1 000 mixed bad inputs that expects exactly 1 000 counted drops split per reason and an unchanged state digest.
4. Extend the driver of 007a-04 to check these expectations, and to fail if a single input takes longer than the bound recorded in the vector file (C-164: linear in packet length, at most 10 µs per packet).

## Acceptance criteria
1. At least 1 vector each for: wrong version, foreign domain, unpinned source, and each of the 3 reserved fields (copy index, scheme, tag) non-zero.
2. The wrong-version vector expects a named event in addition to the counted drop.
3. The 1 000-input sequence vector expects counters summing to 1 000 and no state change.
4. A fake implementation that changes state on a foreign-domain packet fails the driver; one that counts correctly passes.
5. A control message with a non-zero liveness counter and otherwise valid fields is not counted as a reserved-field drop.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Vector files carry the specification version they were generated from
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Fuzzing itself (007a-15, 007b-20).
- Running against the real core (epics 009–014).
- Confirming the C-164 bound (an L1 measurement).
