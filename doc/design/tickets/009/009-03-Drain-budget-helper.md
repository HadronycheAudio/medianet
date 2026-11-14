# 009-03 — Drain budget helper
**Epic:** 009-Gap-and-reassembly-core · **Effort:** XS · **Depends on:** 009-01, 009-02

## Context
Bounded time on the data thread is made real by three things `[decision]`: a bounded drain per input per cycle, a source filter, and a socket buffer sized by the backend. This ticket implements the first.

- C-16 `[decision]`: bounded drain per input per cycle = 2 × the admitted packets per slice for that input.
- C-05 `[decision]`: the default slice is one packet, so the default budget is 2 packets per cycle. Multi-packet slices are an explicit, stamped regime with more packets per slice.
- C-02 `[decision]`: slice length 48 frames = 0.5 ms = 2 000 slices/s at the single domain rate of 96 kHz (C-01).
- Packets beyond the budget are not read in this cycle: they stay in the socket buffer for the next one, and if the buffer overflows the kernel's per-socket overflow counter (read by the backend) counts them `[decision]`. The helper therefore has no overflow counter of its own.
- `[code]` POC behaviour being retired: the data thread drains every input socket completely before looking at its deadline (`module.rs:1669`).

C-156 `[decision]`: the cycle of C-16 is one slice period (0.5 ms at defaults). The helper counts packets per call to `start_cycle`; the caller (the hosted data thread, epic 015) starts one cycle per slice period. Tests use one slice period as the cycle.

## Work
In `hadronyche-core`, module `rx::drain`:
```rust
pub struct DrainBudget { per_cycle: u32 }
impl DrainBudget {
    pub const fn for_input(admitted_packets_per_slice: u16) -> Self; // per_cycle = 2 × admitted
    pub const fn per_cycle(&self) -> u32;
    pub fn start_cycle(&self) -> DrainCycle;                        // countdown for one cycle
}
pub struct DrainCycle { remaining: u32 }
impl DrainCycle { pub fn take(&mut self) -> bool; } // true while budget remains, then false
```
Saturating arithmetic; `admitted_packets_per_slice == 0` yields a budget of 0.

## Acceptance criteria
1. `DrainBudget::for_input(1).per_cycle() == 2` (C-05 default, one packet per slice).
2. `DrainBudget::for_input(8).per_cycle() == 16`.
3. `DrainBudget::for_input(u16::MAX)` does not overflow (`131 070`).
4. A `DrainCycle` from `for_input(3)` returns `true` exactly 6 times, then `false` on every further call.
5. Tests run under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Using the budget in a schedule and the 10 × flood scenario (009-21).
- Sizing the socket buffer and reading the kernel's per-socket overflow counter (the backend's job, not this epic).
- Whether C-16 is large enough for legitimate bursts such as Windows batched capture emission: open sub-question owned by L1 / L2 Windows (tasks 019, 022); falsifier: legitimate bursts exceed it and overflow the socket buffer.
