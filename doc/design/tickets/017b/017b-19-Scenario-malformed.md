# 017b-19 — Scenario `malformed`
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017b-09, 017b-07, epic 007, epic 010

## Context
- AC-23: **malformed, wrong-version and foreign-domain packets cause no state change** and are **counted per input and reason**; **version mismatch is a named event**.
- **How to inject them (settled, C-120):** the test-only hook `send_raw(from, socket, to, bytes)` (017b-07) sends a raw packet on a module's media or control socket, so the bad packet arrives from a real peer address; the use is stamped like any injection.
- Conformance vectors come from epic 007 and serve as the source of malformed packets; the wire layouts and reject reasons they encode, and so the reasons counted here, are epic 004's (C-193).
- **Version mismatch (settled, C-185):** VersionMismatch is raised at discovery, where it stops a module joining, not as an admission refusal; the named event of AC-23 is that discovery-time event. Domain id is 16 bits, 0 = unconfigured (C-11). IPv4 only (C-100).
- **Domain handling (settled, C-173):** a hello with domain 0 is ignored and counted, not refused; ForeignDomain is the reason for a different non-zero domain.
- The bound on processing one bad input (linear in packet length, at most 10 µs per packet) is a proposal pending L1 (C-164); L0 does not assert it, since it never asserts timing on loopback.

## Work
1. Scenario `malformed`: while sender/echo/mixer run, use `send_raw` from the sender's media socket to the mixer's input to send (a) truncated or corrupted packets (from epic 007 vectors), (b) packets with a wrong protocol version, (c) packets with a foreign domain id; repeat (b) on the control socket; (d) on the control socket, hellos with domain 0 and hellos with a different non-zero domain.
2. Assertions: module state (connection states, alignment, output content) identical to a control run without the bad packets; counters per input per reason equal the number sent; a named version-mismatch event appears; domain-0 hellos are counted and ignored, foreign non-zero-domain hellos are counted under ForeignDomain (C-173).

## Acceptance criteria
1. `hadronyche-l0 malformed --duration 20` exits 0.
2. Exact-tier mismatches = 0 and no connection-state transition is caused by bad packets.
3. Counters: malformed = N₁, wrong-version = N₂, foreign-domain = N₃, exactly the numbers sent, attributed to the right input.
4. A `version-mismatch` event is logged at least once.
5. Every raw send appears in the stamp's injection record.
6. Hellos with domain 0 are counted and ignored, with no refusal; hellos with a different non-zero domain are counted as ForeignDomain; neither changes any connection state (C-173).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Fuzzing parsers (017b-24); the hook itself (017b-07).
