# 010-19 — Packetiser round-trip fuzz target
**Epic:** 010-Packetiser-and-bundles-core · **Effort:** S · **Depends on:** 010-08, 010-09, 010-13, epic 009
## Context
- The epic requires a fuzz target that round-trips arbitrary slices through the packetiser and the reassembler of epic 009 (epic 009 is a declared dependency of the epic).
- Definition of done: every parser touched is fuzzed.
- C-95: each parser is fuzzed 60 s on every change and 1 hour nightly; the corpus is kept.
- The receiving handler gets a slice whole or not at all. Multi-packet slices are split into ordered packets each with slice identity, index, count and its own send timestamp; no packet exceeds the path MTU; IP fragmentation is forbidden.
- C-100: IPv4 only; UDP payload ≤ path MTU − 28 bytes. C-06: path-MTU fallback 1 280 bytes. C-07: copy index always 0 in v1.

## Work
- Add a fuzz target (project's fuzz harness) that derives from the fuzz input: path MTU (1 280..=1 500 and others), frames, channel count, flags (gap, degraded), content messages, pending and relayed announcements, and a send-timestamp sequence.
- Encode with `Packetiser`/`SliceWriter` (or `gap_slice`), feed the packets to the epic 009 reassembler in order and in a fuzz-chosen permutation, and compare the reassembled slice with the input.
- Wire it into CI: 60 s on every change, 1 hour in the nightly job; keep the corpus in the repository (C-95).

## Acceptance criteria
1. Reassembled position, frames, flags, content and announcements equal the input for every generated case.
2. Every packet length ≤ path MTU − 28 bytes (C-100) is asserted inside the target.
3. Each packet's decoded send timestamp equals the value the closure returned for that index.
4. CI runs the target 60 s on every change and 1 hour nightly without a crash, unprivileged (epic AC-13).
5. The corpus is committed and reused by both runs.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The reassembler itself and its own fuzzing (epic 009).
