# 009-21 — Bounded drain under flood (EC-6, system AC-11 core part)
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-03, 009-04, 009-07, 009-14

## Context
Epic EC-6 (system AC-11, core part): with one input presenting 10 × its admitted packets, the per-cycle work on that input is bounded by C-16 and the fan-in deadline is computed and met in the simulated schedule.

Facts `[decision]`:
- C-16: bounded drain per input per cycle = 2 × the admitted packets per slice for that input (`DrainBudget`, 009-03).
- C-05: default slice = one packet → budget 2 packets per cycle.
- C-02: slice = 0.5 ms = 2 000 slices/s (96 kHz, C-01).
- Source filter: only admitted peers are parsed, by pinned IPv4 source address and port plus stream number (009-04, C-100).
- Packets beyond the budget stay in the socket buffer for the next cycle; if it overflows, the kernel's per-socket overflow counter (read by the backend) counts them (epic Background).
- `[code]` POC behaviour being retired: the data thread drains every input socket completely before looking at its deadline (`module.rs:1669`).

C-156 `[decision]`: the cycle of C-16 is one slice period. The simulated loop runs one cycle per slice period.

## Work
Integration test `hadronyche-core/tests/drain_flood.rs` with a simulated data-thread loop (bounded queues in the test stand in for sockets):
1. Fan-in with 3 inputs, 1 packet per slice each, offset set by aligning on a clean path.
2. Input 2 presents 10 packets per slice period (10 × admitted) for 5 s: a mix of valid duplicates, packets for unadmitted streams, and valid slices.
3. Each cycle: for each input, take at most `DrainBudget::for_input(1).per_cycle() == 2` packets from its queue through `admit` → `Reassembler::push` → `FanIn::insert`; then, if `now >= deadline_ns(p)`, call `on_deadline(p)`.
4. Count packets processed per input per cycle and the simulated time at which each `on_deadline` happens.
5. The test queue has a fixed capacity and counts what it drops, standing for the kernel overflow counter.

## Acceptance criteria
1. In every cycle, packets processed on input 2 ≤ 2 (C-16); total per cycle ≤ 6 across 3 inputs.
2. Every `on_deadline(p)` happens in the cycle whose time first reaches `deadline_ns(p)`; none is skipped or delayed for 5 s of positions (10 000 rounds).
3. Inputs 0 and 1 are `Content` in 100 % of rounds and their counters show `late == 0`, `missing == 0`.
4. Rejected packets of input 2 appear in `rejected[reason]` counts that equal the number of rejected packets actually drained.
5. The queue-overflow count of input 2 equals packets presented − packets drained − packets still queued.
6. The loop body runs under `assert_no_alloc` (009-02) with 0 allocations.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No allocation and no lock anywhere on the receive path (checked by a no-alloc test allocator)
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The real hosted data thread (epic 015) and the L0 harness (epic 017).
- Whether C-16 is enough for legitimate bursts (open sub-question owned by L1 / L2 Windows, tasks 019, 022; falsifier: legitimate bursts exceed it and overflow the socket buffer).
- Socket buffer sizing and the kernel overflow counter (backend).
