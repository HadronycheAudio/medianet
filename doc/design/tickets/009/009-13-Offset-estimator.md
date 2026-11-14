# 009-13 — Offset estimator (aligning state)
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-12, epic 008

## Context
Offset `[decision]`: during the aligning state the consumer measures origin-to-arrival lateness per input (arrival time minus `T(p)`), which already includes every upstream hop. Offset = maximum observed plus headroom. Frozen once live. A shorter path leaves the offset safe; reclaimable latency is reported. Parameters validated by simulation in task 008.

- C-157 `[decision]`: reclaimable latency = offset − (current maximum lateness + headroom), when positive (0 otherwise), where headroom is the C-52 headroom of that maximum; published in the stamp's alignment group (publication: 009-25).
- C-182 `[decision]`: the "current maximum lateness" of C-157 (and the maximum used by the cannot-meet-A test, C-174, 009-26) is taken over a rolling 60 s window; the maximum since the connection went live is reported beside it.

Constants `[decision]`:
- C-50: aligning duration 4 s (8 000 slice periods at 0.5 ms, C-02).
- C-51: offset estimator = maximum observed lateness plus headroom (percentiles are reported, not used).
- C-52: headroom = the largest of 25 % of the observed maximum, one slice period, twice the published clock bound.
- The published clock bound comes from epic 008 (`TimeView`), published by the servo core, epic 008a (C-187).
- All aligned inputs, including all bundle members, share the consumer's one offset.

One point remains an open sub-question of the epic; this ticket implements the stated assumption behind a clearly named function so the answer is a local change:
- An aligned input that yields no lateness sample during the 4 s (owner: control state machines, epic 005). Assumed: `finish_aligning` refuses and the consumer stays Aligning. Falsifier: a consumer never leaves Aligning while one of its inputs is silent.

## Work
Extend `FanIn` (009-12):
```rust
pub enum Phase { Aligning, Live }
impl<..> FanIn<..> {
    pub fn observe_lateness(&mut self, input: InputIndex, lateness_ns: i64); // aligning: track max; live: rolling 60 s max and max since live
    pub fn finish_aligning(&mut self, clock_bound_ns: i64, slice_ns: i64) -> Result<i64, NoSample>; // max + headroom; freezes offset
    pub fn phase(&self) -> Phase;
    pub fn reclaimable_ns(&self, clock_bound_ns: i64, slice_ns: i64) -> i64; // C-157 over the rolling 60 s window (C-182)
    pub fn advance_window(&mut self, now_ns: i64);  // slides the rolling 60 s window by whole seconds (C-182)
    pub fn rolling_max_ns(&self) -> i64;            // max lateness over the last 60 s (C-182), read by 009-25 and 009-26
    pub fn max_since_live_ns(&self) -> i64;         // reported beside it (C-182)
}
```
- Live lateness is kept as 60 one-second maxima in a fixed ring (no allocation); `advance_window` drops seconds older than 60 s, driven by the same 1 s window clock the ratchet uses (009-16). Publishing the stamp does not reset anything.
- The maximum since live starts when `finish_aligning` switches to `Live` and is never reset by stamp publication.
- During `Aligning`, rounds are not produced (`on_deadline` is not called by the scheduler; documented).
- `finish_aligning` computes `max + max(max / 4, slice_ns, 2 × clock_bound_ns)` over all aligned inputs, stores it as the offset, switches to `Live`. The caller calls it after C-50 = 4 s.
- After `Live`, `observe_lateness` never changes the offset (only the ratchet, 009-16, can).
- If an aligned input produced no lateness sample, `finish_aligning` returns `Err(NoSample)` naming the input and the phase stays `Aligning` (assumption above).

## Acceptance criteria
1. Max 2 ms, slice 0.5 ms, clock bound 0.1 ms → headroom max(0.5, 0.5, 0.2) ms = 0.5 ms → offset 2.5 ms.
2. Max 4 ms, slice 0.5 ms, clock bound 0.1 ms → headroom 1 ms (25 %) → offset 5 ms.
3. Max 1 ms, slice 0.5 ms, clock bound 1 ms → headroom 2 ms (2 × clock bound) → offset 3 ms.
4. The maximum is taken across all inputs (input 0 max 1 ms, input 3 max 3 ms → uses 3 ms).
5. After `finish_aligning`, observing lateness of 10 ms does not change `deadline_ns`.
6. Live, path shortens so the rolling 60 s max lateness is 0.5 ms with offset 2.5 ms, slice 0.5 ms, clock bound 0.1 ms (headroom 0.5 ms) → `reclaimable_ns(100_000, 500_000) == 1_500_000` (C-157); with a rolling max of 2.5 ms the result is 0, never negative.
7. Rolling window (C-182): a 3 ms sample at t = 0 then only 1 ms samples → `rolling_max_ns` is 3 ms until 60 s have elapsed and 1 ms after; `max_since_live_ns` stays 3 ms throughout; the result does not depend on when (or whether) a stamp was published.
8. An aligned input with no sample → `Err(NoSample)` naming it; phase stays `Aligning`.
9. Runs under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Ratchet and demotion (009-16).
- Whether maximum-plus-headroom over 4 s is sufficient: open sub-question owned by task 008, then L1; falsifier: late slices after alignment on a clean network. Note it in the code doc; do not change the rule here.
- Driving the 4 s timer (the scheduler, epics 015 / 017) and the Aligning state transitions (epic 005).
