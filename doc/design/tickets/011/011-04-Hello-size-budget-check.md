# 011-04 — Hello size budget check
**Epic:** 011-Gossip-and-discovery-core · **Effort:** XS · **Depends on:** 011-02

## Context
C-17 (decided): a control message is at most 1 252 bytes of UDP payload, the 1 280-byte IPv4 path-MTU fallback (C-06) minus the IPv4 and UDP headers (C-100). The C-17 falsifier is: a hello for a 32-member bundle source does not fit. The hello carries ports and bundles (group id, member list, channel map), which dominate its size. It does not carry the regime stamp, only its version number (C-18), and it carries no per-stream sample rate (C-01). C-08 and C-209: the 32 members include the controller member, since a bundle source's controllers ride in their own member stream (C-205). Epic acceptance criterion 8: a hello for a source with a 32-member bundle encodes in at most 1 252 bytes, or the task reports the constant as falsified.

## Work
1. Add a test that builds a worst-case-but-legal `LocalInfo` for a source with one 32-member bundle, 31 audio members plus the controller member (C-209) (maximum name length, maximum port count with content types, all other fields at their maximum encoded width per epic 004) and encodes it with `Hello::encode`.
2. Assert the encoded length is <= 1 252 and print the actual length and the margin.
3. If the assertion fails, do not raise the bound: record in the design constants table (`doc/design/02-constants.md`, C-130) that C-17 (or the hello content) is falsified, with the measured size tagged `[measured]`, and open an item with an owner and a falsifier.

## Acceptance criteria
1. The test encodes a hello with a 32-member bundle and reports its exact byte length.
2. Either the length is <= 1 252 bytes and the test passes, or the constants table records C-17 as falsified with the `[measured]` length.
3. The worst-case values used (name length, port count, bundle count) are written in the test as named constants.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag

Not applicable (C-125): "Every parser touched is fuzzed" (test of the encoder only); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "The frozen POC is not modified" (test only, no code near the POC); "No election path reachable from hello expiry" (no expiry or election code).

## Out of scope
- The encoder itself: 011-02.
- Whether 63 peers (C-217) with full hellos fit device memory: owned by task 026 (see 011-06 for the footprint const).
