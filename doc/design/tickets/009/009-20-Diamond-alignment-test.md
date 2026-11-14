# 009-20 — Diamond alignment test (EC-2, system AC-10)
**Epic:** 009-Gap-and-reassembly-core · **Effort:** S · **Depends on:** 009-07, 009-13, 009-14, 009-19, epic 006

## Context
Epic EC-2 (system AC-10): in a diamond (a source feeds a fan-in directly and through one processor with injected per-hop delay), the fan-in output equals the offline reference bit for bit. The epic defines the offline reference: the same bit-exact test processor and the reference mixer (009-19) applied to the undelayed source slices, position by position.

Facts `[decision]`:
- Position invariant: a module's output for position p is derived from its inputs at position p; stamps denote content position and are never restamped per hop.
- The fan-in processes position p at `T(p) + offset` (009-12). Offset = maximum lateness observed during the 4 s aligning state (C-50) + headroom = max(25 % of max, one slice period, 2 × clock bound) (C-51, C-52).
- Alignment is sample-exact by position whatever the clock error.
- Slice = 48 frames = 0.5 ms at 96 kHz (C-01, C-02); default 6 channels × 192 bytes = 1 152 bytes of audio per slice (C-04).
- C-94 (provisional, synthetic): wired jitter profile for simulation = base one-way delay 100 µs, exponential jitter with 20 µs mean, 1 % of packets in bursts delayed by 1 ms. Task 019 (L1) replaces it with the measured profile; the test takes the profile as a parameter so the swap is a data change.
- `[code]` POC behaviour being retired: the mixer keeps only the latest block per input and mixes on arrival with no alignment (`mixer.rs:101-124`).

## Work
Integration test `hadronyche-core/tests/diamond.rs`, fully simulated time (no sockets, no sleeps). Arrival times are given directly as `arrival_ns`; in the real system the backend supplies them, with their source (kernel, userspace, mac-hardware or interrupt) stamped per C-131.
1. Source emits deterministic Q1.31 slices (e.g. a counter pattern) for 10 s of positions.
2. Path A: direct to fan-in input 0, one hop with the C-94 profile. Path B: through a test processor (a bit-exact integer gain or XOR with a constant), two hops each with the C-94 profile, plus injected processing delays of 1 ms and 1.5 ms. Jitter from a seeded PRNG.
3. All packets go through `Reassembler` → `FanIn::insert`; lateness is fed to `observe_lateness`; `finish_aligning` after 4 s; rounds via `on_deadline` at `deadline_ns`; output = the reference mixer of 009-19.
4. Offline reference: for every position p after live, mix(source(p), processor(source(p))) computed directly, with no delays.
5. Run the scenario with a simulated clock error between source and fan-in (e.g. +50 ppm) in a second case.

## Acceptance criteria
1. For every position from the end of aligning to 10 s (12 000 slices), fan-in output equals the offline reference bit for bit, in both cases (no clock error, +50 ppm).
2. `late == 0`, `missing == 0`, `degraded == false` for every round after live (the C-94 bursts seen during aligning are covered by the offset).
3. Control: a naive "latest block per input" mixer on the same inputs differs from the reference (shows the test is sensitive).
4. Test is deterministic (seeded) and runs in under 10 s in CI.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Real network delay or sockets (L0 harness, epic 017).
- Measuring the jitter profile (task 019).
- Late/missing classification under excess delay (covered in 009-14).
