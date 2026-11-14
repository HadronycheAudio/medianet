# 009-12 — Fan-in deadline and fixed-time rounds
**Epic:** 009-Gap-and-reassembly-core · **Effort:** M · **Depends on:** 009-07, 009-10, epic 006

## Context
Fan-in alignment `[decision]`:
- A fan-in processes position p at the fixed time `T(p) + offset`, with whatever has arrived. Anything absent is a gap.
- It always fires at the fixed time, never early when all inputs happen to be present; early firing would make output timing data-dependent.
- All aligned inputs of a consumer, including all members of a bundle, share that consumer's one offset. Bundle members have independent gaps. C-08: at most 32 members per bundle, the controller member included (C-209); a bundle source's controllers ride in their own member stream (C-181, C-205), which is one of these inputs.
- An input whose slices carry another timeline epoch is treated as a gap until it switches; no round combines two epochs (system AC-18, epic EC-5).
- A consumer that processed a gap marks its own output slice degraded. Marked gap = slice present as a gap slice or with the degraded flag (upstream alive, loss further up).
- Position invariant: a module's output for position p is derived from its inputs at position p; the runtime stamps output positions; stamps are never restamped per hop.
- C-110 `[decision]`: an aligned multi-packet input is reassembled until the consumer's own deadline for the position; the round for p therefore tells that input's reassembler to drop anything incomplete for p (`expire_before(p + 48)`, 009-07).
- `T(p)` and `Line` come from epic 006. Positions step by 48 per slice (C-02, 0.5 ms at 96 kHz, C-01).

This ticket builds the `FanIn` shell, the deadline and the per-round output. The offset value comes from 009-13; until then tests set it directly. Demoted inputs (C-109) are reported by this round as a gap; the state itself comes from 009-16.

## Work
In `hadronyche-core`, module `rx::fanin`:
```rust
pub struct FanIn<const INPUTS: usize, const DEPTH: usize, const MAX_SLICE: usize> {
    /* offset, rings: [AlignRing<DEPTH, MAX_SLICE>; INPUTS], per-input state */ }
impl<..> FanIn<..> {
    pub fn deadline_ns(&self, position: u64, line: &Line) -> i64;   // T(p) + offset, never earlier
    pub fn insert(&mut self, input: InputIndex, slice: SliceRef<'_>, arrival_ns: i64) -> Inserted;
    pub fn on_deadline(&mut self, position: u64) -> Round<'_>;       // per-input InputAt + flags; sets output degraded
}
pub enum RoundInput<'a> { At(InputAt<'a>), Demoted }
pub struct Round<'a> { pub position: u64, pub inputs: [RoundInput<'a>; INPUTS], pub degraded: bool }
```
The ring capacity const generics `DEPTH` and `MAX_SLICE` are part of the epic's `FanIn` signature.
- `deadline_ns` is the only time source for a round; there is no API that fires a round when all inputs are present.
- `on_deadline(p)` calls `take(p)` on every ring; a Demoted input is reported as `Demoted` whatever its ring holds; `degraded = true` if any input is `MarkedGap`, `OtherEpoch`, `Missing` or `Demoted`.
- Epoch: all rings share the consumer's epoch (`set_epoch` on all); an other-epoch input is a gap for that round.
- `on_deadline` positions must be strictly increasing by 48; anything else is a caller error (debug assert, documented).

## Acceptance criteria
1. With offset 2 ms, `deadline_ns(p, line) == T(p) + 2_000_000` for 1 000 consecutive positions.
2. Fixed-time firing (epic EC-7): in a simulated schedule with 3 inputs, case A (all slices arrive 1.5 ms before the deadline) and case B (two inputs early, the third 1 ns before the deadline) produce `on_deadline` calls at identical simulated times and identical `Round` contents.
3. One input `Missing` at p → that input is `Missing`, others `Content`, `degraded == true`.
4. One input delivers a degraded-flag slice → that input is `MarkedGap`, `degraded == true` (EC-4, marked gap propagates with the degraded flag).
5. All inputs `Content` → `degraded == false`.
6. EC-5: input 1 switches to another epoch at position q; for positions ≥ q, input 1 is `OtherEpoch` in every round until the consumer switches epoch, and no `Round` ever contains `Content` from two epochs (asserted over 1 000 rounds).
7. A 32-member bundle (C-08, controller member included, C-209) as 32 inputs: losing member 5 at p affects only `inputs[5]` (independent gaps), all share one deadline.
8. An input marked Demoted (test hook until 009-16 lands) is reported `Demoted` even when its ring holds content, and `degraded == true`.
9. An aligned multi-packet input whose slice for p is still incomplete at the round for p is `Missing`, and its reassembler has dropped the slot (C-110).
10. `insert` + `on_deadline` run under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] Regime stamp fields touched are complete and documented (degraded flag on the output slice)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Computing the offset (009-13), the ratchet and demotion (009-16).
- Counters and late/missing attribution (009-14); health thresholds (009-15).
- Delaying the degraded flag through a structural latency L (009-23).
- Concealment of gaps (009-19).
- Unaligned inputs (009-18).
- Emitting a gap slice when a module misses its own deadline (C-144), and stamping p − L for structural delay L: the hosted backend (epic 015).
