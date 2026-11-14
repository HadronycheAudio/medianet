# 009-26 — "Cannot meet A" test
**Epic:** 009-Gap-and-reassembly-core · **Effort:** XS · **Depends on:** 009-13, 009-16

## Context
A consumer whose lateness budget no longer fits the presentation group's audition offset A must raise an alarm `[decision]`:

- C-174: "cannot meet A" = maximum lateness + D + headroom exceeds A in each of 10 consecutive 1 s windows.
- C-182: the maximum lateness of the test is taken over a rolling 60 s window (the same window as the stamp's percentiles and reclaimable latency), evaluated at the close of each 1 s window.
- C-170: D is a sink's total plumbing delay from hand-off to emission; the caller supplies it (0 for a fan-in that is not a sink). An edge's resampler delay is part of D (C-134).
- C-52: headroom = largest of 25 % of the maximum, one slice period, twice the published clock bound.
- A is the presentation group's (C-55), or for a fan-in outside any group the inherited A or configured budget (C-142).
- C-173: a connection alarm is raised on cannot-meet-A; raising it is the connection state's (epic 012). This ticket provides the test and the event.
- C-211: a change of A deeper than the fan-in's compiled ring that reaches it anyway leaves the fan-in at the deepest ring it can hold as its ceiling and raises the cannot-meet-A alarm (009-29). `CannotMeetA` is therefore emitted at once.
- C-216: the cannot-meet-A alarm stays raised until a later change of A that fits, so `MeetsAAgain` is withheld until such a change; after it, the window test decides as usual.
- C-222: the clamp of C-211, and C-216 with it, applies to any consumer that buffers to meet A, sinks included. `set_ceiling_short` is therefore not fan-in-specific: a sink epic calls it when a change of the sink's own A does not fit its buffer.
- The 10 consecutive 1 s window machinery already exists for the ratchet (C-101, 009-16); this ticket reuses its window clock.

## Work
Extend `FanIn`:
```rust
impl<..> FanIn<..> {
    pub fn check_meet_a(&mut self, now_ns: i64, a_ns: i64, d_ns: i64,
                        clock_bound_ns: i64, slice_ns: i64) -> Option<HealthEvent>; // CannotMeetA
}
// HealthEvent gains: CannotMeetA, MeetsAAgain
```
- At each 1 s window close: the maximum lateness over the rolling 60 s window (C-182) over all aligned live inputs, kept as 60 one-second maxima in a fixed ring as in 009-13; one bit "max + D + headroom(max) > A".
- 10 consecutive set bits → `CannotMeetA`, emitted once; the first clear window after it → `MeetsAAgain`, emitted once.
- Unaligned (009-18) and Demoted inputs (009-16) contribute no lateness.
- `pub fn set_ceiling_short(&mut self, short: bool) -> Option<HealthEvent>` (called by 009-29 for a fan-in and by the sink epics for a sink, C-211, C-216, C-222): `true` emits `CannotMeetA` at once if not already raised and holds it; `false` releases the hold, after which the window test above decides.

## Acceptance criteria
1. A = 5 ms, D = 1 ms, slice 0.5 ms, clock bound 0.1 ms, rolling 60 s maximum 3.5 ms (headroom 0.875 ms, total 5.375 ms) at the close of each of 10 consecutive windows → one `CannotMeetA` at the close of the 10th window.
2. Same values except a rolling maximum of 2 ms (total 3.5 ms) → no event.
3. A rolling maximum giving exactly A (not above) clears the bit.
4. A single 3.5 ms sample followed only by 2 ms samples (C-182): the bit stays set while the sample is within the 60 s window, so one `CannotMeetA` at the close of the 10th window; one `MeetsAAgain` at the first window close after the sample has left the 60 s window; no repeated events.
5. C-211, C-216: `set_ceiling_short(true)` with clean lateness → one `CannotMeetA` at once and no `MeetsAAgain` over 120 s of clean windows; after `set_ceiling_short(false)` and a clean window → one `MeetsAAgain`.
6. Runs under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Raising and displaying the connection alarm (epic 012, C-173).
- Measuring D (the sink backend) and choosing A (commissioning, C-55).
