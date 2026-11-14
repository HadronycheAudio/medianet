# 019-12 — Asymmetry reference selection
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-06

## Context
One-way latency figures in the runtime are accurate to half the path asymmetry, which no in-band method can observe; the worst-case asymmetry bound is published with every one-way figure. AC-24 requires that "the asymmetry bound holds against the external check".

Epic 019 owns this open sub-question:

| Question | Owner | Falsifier |
|---|---|---|
| External reference for the asymmetry check (a hardware timestamping NIC pair, a shared pulse, or a swapped-cable method) | This task | No available method resolves below the bound the runtime publishes in `baseline` |

Relevant facts: the runtime's receive timestamps are kernel software stamps (`SO_TIMESTAMPNS`, `CLOCK_REALTIME` basis `[kernel-doc]`); NIC interrupt coalescing delays them in one direction only (measured in 019-11). The follower servo lock bound is at most 25 µs (C-62).

There is no separate fixed value for the asymmetry bound: the runtime publishes a bound with each one-way figure, and the epic tests each figure against its own bound. The bound to compare a method's resolution with is therefore the one the runtime publishes in a `baseline` run on the L1 hardware.

This ticket decides the method; 019-13 implements it as scenario `asymmetry`.

## Work
- Evaluate the three candidate methods on the L1 hardware from 019-06:
  1. **Hardware-timestamping NIC pair** (PTP-capable NICs, `SO_TIMESTAMPING` hardware stamps, or `ptp4l`/`phc2sys` as an independent reference): resolution, availability of the NICs, cost.
  2. **Shared pulse** (one electrical pulse, e.g. GPIO or PPS, wired to both hosts and timestamped by each): resolution, wiring needs.
  3. **Swapped-cable method** (measure, swap the two directions' physical paths, measure again; the difference isolates asymmetry): resolution, applicability through one switch.
- For each, state the expected resolution in µs with an evidence tag, and compare with the asymmetry bound the runtime publishes (read from a `baseline` report).
- Write a decision record (in `doc/design/reports/l1-results.md` under "Asymmetry", or a short design note linked from it): chosen method, rejected methods with reasons, resolution, required hardware.
- If no method resolves below the claimed bound, record the falsifier as triggered and open an item with owner and falsifier.

## Acceptance criteria
1. A decision record exists naming exactly one chosen method and giving its expected resolution in µs with an evidence tag.
2. All 3 candidate methods are assessed with resolution and reason for acceptance/rejection.
3. The chosen method's resolution is below the runtime's published asymmetry bound, or the falsifier is recorded as triggered with a new open item (owner, falsifier).
4. Any hardware the method needs is added to the hardware record in `l1-results.md`.

## Definition of done
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The named hardware pair (hosts, NICs, switch, cables) is recorded
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Implementing and running the `asymmetry` scenario (019-13).
- Improving the runtime's one-way estimate.
