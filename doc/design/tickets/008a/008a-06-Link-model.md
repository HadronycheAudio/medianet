# 008a-06 — Link model
**Epic:** 008a-Servo-core-and-simulation-harness · **Effort:** M · **Depends on:** 008a-04
## Context
Part of the `sim` module of the `hadronyche-tools` crate of epic 008. Medianets are wired, isolated IPv4 LANs (C-100) whose only traffic is this runtime's `[decision]`. Each sync exchange is a request (follower → holder) and a reply (holder → follower) carrying four timestamps t1..t4 `[decision]`. The link model decides when, whether and how often each packet arrives.

The suite needs: the C-94 wired jitter profile (base one-way delay 100 µs, exponential jitter with 20 µs mean, 1 % of packets in bursts delayed by 1 ms), other jitter distributions for case 1, longer bursts (case 4), asymmetry (case 3: bias must be exactly half the asymmetry), loss, duplication, reordering (case 7), and interrupt-coalescing delay (open sub-question: does coalescing bias survive the minimum-round-trip filter?).

## Work
In the `sim` module of `hadronyche-tools`, a `LinkModel` per direction with seeded randomness:
1. Base delay (ns), separately per direction so asymmetry = forward − reverse base delay.
2. Pluggable jitter distribution trait with at least: constant, uniform, exponential, log-normal, and an empirical histogram loaded from data (the form L1 will deliver).
3. Bursts: a burst process adding extra delay of configurable size, with either a per-packet fraction (C-94: 1 % of packets, +1 ms) or Markov on/off episodes of configurable duration.
4. Loss probability; duplication probability (duplicate delivered after a configurable extra delay); reordering (packet held back past later ones).
5. Interrupt-coalescing delay: arrivals released only at coalescing ticks of configurable period, applied to the receive side, toggleable.
6. All parameters in a plain-data `LinkConfig` embedded in `Scenario`.

## Acceptance criteria
1. With constant jitter, delivered delay equals base delay exactly in each direction.
2. Each jitter distribution's sample mean and 99th percentile over 10^6 samples match the configured analytic values within 1 %.
3. Per-packet bursts at 1 % and +1 ms: burst fraction 0.010 ± 0.001 over 10^6 packets, each burst packet delayed by exactly 1 ms more than its unburst delay.
4. Loss 10 %: delivered fraction 0.90 ± 0.005 over 10^5 packets; duplication 5 %: duplicate fraction 0.05 ± 0.005.
5. With reordering enabled at 5 %, at least one packet is delivered after a later-sent one, and a test asserts the reorder fraction 0.05 ± 0.01.
6. Coalescing at period P: every arrival time is a multiple of P (after phase offset); disabling it restores the uncoalesced arrival times exactly.
7. Same seed, same delivery sequence.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The `WIRED_PROFILE` preset and its documentation (008a-07).
- The coalescing bias study (008b-14).
