# 009-15 — Input health: interrupted and dead thresholds
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-14

## Context
Gap contract `[decision]`:
- C-38 Interrupted: 8 consecutive missed slices (4 ms at 0.5 ms per slice, C-02). The module is told; it conceals; resume is immediate at the frozen offset when slices return on the same session.
- C-102 What counts as a miss: only a slice absent at the deadline, that is a round where the input is `Missing` (including a slice that later turns out late, C-108). A marked gap or a slice on another timeline epoch proves the upstream alive and resets the count.
- C-39 Dead: 2 s without data. C-111: the 2 s are counted from the last slice received with content or a marked gap. C-135: a slice on another timeline epoch also restarts the dead clock. A slice with content "received" here is one that reaches the input's ring, on time or late (this epic's reading of C-111); duplicates, too-early slices and rejected packets do not restart the count. End-of-stream event; resources released; re-admission needed.
- Epic EC-4 (system AC-16): interrupted after exactly 8 consecutive misses; dead at 2 s; resume from interrupted is immediate at the frozen offset with no re-alignment.
- `InputHealth { Live, Interrupted, Demoted, Dead }` and `InputCounters::{interrupted, dead}` come from 009-01. The Demoted state is driven by 009-16; this ticket only makes sure Dead also applies to a Demoted input.
- Epic 012 consumes Interrupted, Resumed and Dead as connection-state inputs.
- "Same session" (C-19): the same session number as the slices before the interruption. The check is added by 009-24; this ticket resumes on any returning slice.

## Work
Extend `FanIn`:
```rust
pub enum HealthEvent { Interrupted(InputIndex), Resumed(InputIndex), Demoted(InputIndex), Dead(InputIndex), SessionChanged(InputIndex) }
impl<..> FanIn<..> {
    pub fn health(&self, input: InputIndex) -> InputHealth;
    pub fn check_dead(&mut self, now_ns: i64) -> Option<HealthEvent>;
    pub fn readmit(&mut self, input: InputIndex);   // clears Dead (and Demoted, 009-16); clears the ring
}
// Round gains: pub events: [Option<HealthEvent>; INPUTS]
```
- Per input: consecutive-miss counter incremented by a `Missing` round, reset by `Content`, `MarkedGap` or `OtherEpoch` (C-102); reaching 8 → `Interrupted`, `interrupted += 1`, event emitted once.
- From `Interrupted`, the first `Content`, `MarkedGap` or `OtherEpoch` round → `Live`, `Resumed` event; the offset is unchanged and no aligning phase is entered.
- Last-data timestamp per input, updated by every slice with content or a marked gap that reaches the ring (stored or late, C-111) and by every other-epoch slice (C-135); `now_ns − last_data_ns >= 2_000_000_000` → `Dead`, `dead += 1`, event emitted once; the input's ring is cleared and further slices for it are ignored until `readmit(input)` (the caller also calls `SourceFilter::unpin`, 009-04).
- `stamp_summary` (009-14) reports `interrupted` and `dead`.

## Acceptance criteria
1. 7 consecutive `Missing` rounds then `Content` → health stays `Live`, `interrupted == 0`.
2. 8 consecutive `Missing` rounds → `Interrupted` on the 8th round exactly, `interrupted == 1`, one event.
3. 100 further misses → no further event, `interrupted` stays 1.
4. Slices return → `Resumed` on the first `Content` round; `deadline_ns` for following positions equals the value before the interruption (frozen offset); phase stays `Live`.
5. `MarkedGap` rounds reset the miss count: 7 misses, 1 marked gap, 7 misses → still `Live` (C-102).
6. `OtherEpoch` rounds reset the miss count the same way (C-102).
7. 4 misses whose slices then arrive late (reclassified to late, C-108) followed by 4 more misses → `Interrupted` on the 8th (late arrivals are misses, C-102).
8. No data for 1 999 999 999 ns → not dead; at 2 000 000 000 ns → `Dead`, `dead == 1`, one event; subsequent slices for that input are ignored until `readmit`.
9. A late slice (C-111) or an other-epoch slice (C-135) restarts the 2 s count; a duplicate or a too-early slice does not.
10. Other inputs are unaffected by one input's transitions.
11. Runs under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] Late, missing, marked and rejected counters are all exposed for the receiver report and the stamp
- [ ] Regime stamp fields touched are complete and documented (interrupted events and dead, C-47, C-159)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- What the module does when told (concealment: 009-19; module API: other epics).
- Connection-state machine, re-admission protocol and the connection alarms raised on interrupted and dead (C-173) (epics 005 and 012).
- The session-number check on resume (009-24) and demotion (009-16).
