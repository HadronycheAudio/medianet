# 012-02 — Port budget 70 percent ceiling
**Epic:** 012-Admission-handshake · **Effort:** XS · **Depends on:** 012-01

## Context
Epic 012 admission checks wire bandwidth on both ends: the sender checks its own interface and the receiver checks its own interface, each against an admission ceiling of 70 % of the link rate (C-23) `[decision]`. Wire rate is bit/s on the wire **including headers**. Because all traffic on a medianet is the runtime's own, per-interface sums are enforceable. Trunk links are never computed by the runtime (validated offline by the commissioning tool, with trunk capacity declared per direction, C-117).

Every stream runs at the 96 kHz domain rate (C-01) with 6 channels (C-04); a default packet is about 1 320 bytes on the wire (C-20). Derived reference rates (derived):
- C-21: about 21.1 Mbit/s per stream, about 3.5 Mbit/s per channel.
- C-22: 24 ch source 4 streams, 84 Mbit/s (8 000 packets/s); 48 ch 8 streams, 169 Mbit/s (16 000 packets/s); 96 ch 16 streams, 338 Mbit/s (32 000 packets/s).
- C-24: GbE port wall about 198 channel-copies per port (8 × 24 ch, 4 × 48 ch or 2 × 96 ch).
- C-26: 100 Mbit device port about 19 channels.
- C-92: two 96-channel bundles on one GbE port are 676 Mbit/s and 64 000 packets/s; a third is refused.

A request that would take a port **above** 70 % is refused with `RefuseReason::PortCeiling` (012-A2, AC-12).

## Work
In `hadronyche-core` (admission module):
```rust
pub struct PortBudget { pub link_bits_per_s: u64, pub committed_bits_per_s: u64 }
impl PortBudget {
    pub const CEILING_PERMILLE: u16 = 700;
    pub fn can_admit(&self, extra_bits_per_s: u64) -> bool;
    pub fn commit(&mut self, bits_per_s: u64);
    pub fn release(&mut self, bits_per_s: u64);
}
```
- `can_admit` returns true iff `committed + extra <= link × 700 / 1000`, computed in integer arithmetic without overflow (use `u128` or checked/saturating ops). Exactly 70 % is admitted; anything above is refused.
- `commit`/`release` adjust `committed_bits_per_s` (saturating on release).

How a module learns its link rate on each platform is an open sub-question of the epic owned by tasks 015, 021, 022, 026 (falsifier: a platform cannot report it; the rate becomes operator configuration there). This ticket takes the rate as an input.

## Acceptance criteria
1. GbE (`link = 1_000_000_000`): two admissions of 338 Mbit/s succeed (676 Mbit/s committed, C-92), a third 338 Mbit/s returns `false`.
2. GbE: eight 84 Mbit/s commits succeed (672 Mbit/s), a ninth fails; four 169 Mbit/s succeed (676 Mbit/s), a fifth fails (consistent with C-24).
3. Boundary: with `link = 1_000_000_000`, `committed = 0`, `extra = 700_000_000` → true; `700_000_001` → false.
4. No overflow or panic for `link = u64::MAX`, `extra = u64::MAX` (property test).
5. `release` below zero saturates at 0.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented — not applicable: no stamp field
- [ ] Design documents updated where behaviour differs, constants table first — not applicable: behaviour matches C-23
- [ ] No new open item without an owner and a falsifier — not applicable: no open item is created
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection) — not applicable: no transitions
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Mapping a `false` result to `PortCeiling` inside the handshake (012-12 sender side, 012-13 receiver side).
- Learning the link rate (tasks 015, 021, 022, 026). Trunk feasibility (commissioning tool, epic 028).
