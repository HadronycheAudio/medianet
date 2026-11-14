# 012-21 — Handshake message fuzz target
**Epic:** 012-Admission-handshake · **Effort:** S · **Depends on:** 012-08, 012-13, 012-19, 012-27

## Context
Epic 012 deliverable: a fuzz target for handshake messages. DoD: every parser touched is fuzzed. Messages arrive through `Admission::on_message(now_ns, from: Addr, bytes: &[u8], out)`, which decodes request, accept, refuse, MTU probe and probe ack messages (layouts from epic 004). Control messages are at most 1 252 bytes (C-17), except the probe, which is sent at the intended media packet size (C-116); bundles at most 32 members (C-08) with one session per stream (C-19); addresses are IPv4 (C-100); every message carries the authentication reservation with scheme 0 (C-115); `RefuseReason` is a `u8` with 18 valid values (C-173, C-185). A member list longer than 32 decodes and is refused with `BadMemberList` rather than rejected by the decoder (C-173). Processing one bad input is linear in packet length and at most 10 µs per packet (C-164, a proposal that L1 confirms).

Fuzz budget (C-95): each parser runs 60 s on every change and 1 hour nightly; the corpus is kept. The fuzzing tool is the implementing team's choice (C-192).

## Work
- `fuzz/` target `handshake_decode`: feed arbitrary bytes (up to 1 500) to the 012-08 decoder.
- Target `admission_on_message`: an `Admission<8>` with a few seeded connections in various states (including Live, Interrupted, Demoted); feed a sequence of arbitrary `(from, bytes)` messages, `on_peer_map` calls and ticks; assert no panic, no state change out of Live/Interrupted/Demoted, and `out` never exceeds its capacity. Include `set_control_degraded` toggles (012-27) in the sequence.
- Target `refuse_control_degraded`: feed arbitrary bytes (up to 1 500) to the supervisor's stateless refusal path (012-27, C-185); assert no panic, at most one refuse(`ControlDegraded`) message written, and nothing written for a non-request.
- Seed corpus from the 012-19 vectors' messages; keep the corpus in the repository (C-95, C-130).
- CI: run each target 60 s on every change to the parsers and 1 hour nightly, unprivileged (C-95).

## Acceptance criteria
1. All three targets build and run in CI for 60 s on change and 1 hour nightly without crash.
2. The seed corpus contains at least one valid message of every kind and is kept between runs.
3. The invariant "no transition out of Live or Interrupted from `on_message` or `on_peer_map`" is asserted in the target.
4. The target reports the slowest input per run, so a decode cost that is not linear in length (C-164) shows up in the nightly log.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: no behaviour change
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag — not applicable: fuzz code only, no factual claim
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection)
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Fuzzing hello/gossip parsers (epic 011), regime-stamp parsers (epic 004) or data-plane parsers (epic 010).
