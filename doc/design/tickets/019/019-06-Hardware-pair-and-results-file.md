# 019-06 — Hardware pair and results file
**Epic:** 019-L1-two-machine-harness · **Effort:** XS · **Depends on:** 019-03

## Context
Epic 019 delivers a measurements file `doc/design/reports/l1-results.md` (reports and results live in `doc/design/reports/`, C-130; the constants table is `doc/design/02-constants.md`), with, for each constant it measures or checks, the measured value, the hardware, and the stamp. Its definition of done requires "The named hardware pair (hosts, NICs, switch, cables) is recorded".

Constants L1 must measure or check and write back to the constants table:

| Constant | Currently | Measured how |
|---|---|---|
| C-93 receive cost per packet | 2 to 4 µs, guessed `[memory]` | Data-thread time in batch receive divided by packets, at 2 000, 32 000 and 64 000 packets/s |
| C-92 two-bundle capacity | pending-measurement | `two-bundles` (AC-13) |
| C-90 loss floor | about 1.1e-8 per default packet (about 1 320 bytes on the wire), from memory `[memory]` | `loss-soak`: 24 h at 64 000 packets/s, aggregate over all streams (C-118) |
| C-91 loss alarm threshold | rule: 10 × the measured clean-wire loss (C-167); value unset | Applied to the clean-network loss measured in `loss-soak` |
| C-94 wired jitter profile | provisional, synthetic | One-way delay distribution from `baseline` and `two-bundles` |
| C-35 followers per holder | 64, decided (owner L1, task 019) | `serving`: serving lateness against follower count (C-167) |
| C-14 egress spread | at most 25 % of the slice period (125 µs), decided | Falsifier check: queue drops and consumer margin with spread on and off |
| C-16 bounded drain | 2 × admitted packets per slice, decided | Falsifier check: largest legitimate burst per input per cycle |
| Coalescing bias | unknown | `coalescing` |
| C-164 bound on processing one bad input | linear in packet length, at most 10 µs per packet, proposal pending L1 | `bad-input` (019-27) |

The L1 topology is two Linux hosts on one wire through one switch, IPv4 only (C-100). The epic lists the hardware the scenarios need: two Linux hosts with GbE NICs; at least one NIC whose interrupt coalescing can be changed with `ethtool -C`; a managed switch whose port MTU can be lowered and whose drop counters can be read (`blackhole`, queue drops); a small unmanaged switch (the burst question: does it absorb 32 packets, about 42 KB, per slice period?); cables; and whatever the asymmetry reference chosen in 019-12 needs.

## Work
- Choose and record the L1 hardware pair: for each host, the machine model, CPU, RAM, kernel; for each NIC, model, driver, firmware, link speed (GbE), whether coalescing can be changed; the managed and unmanaged switch models; the cable type and length; the IPv4 addressing used.
- Create `doc/design/reports/l1-results.md` with:
  - a "Hardware" section holding the above;
  - one section per row of the constants table above, each with fields *measured value*, *hardware*, *stamp* (link to or excerpt of the `stamp.json` from 019-03), *evidence tag*, initially "pending";
  - "Asymmetry", "Switch findings", "Redundancy recommendation" and "Acceptance summary" sections, initially "pending".
- Record for each host that strict mode (C-127) is attainable: real-time scheduling and memory locking granted, CPU idle-state restriction available or not, with C-129 states. Every run whose numbers enter the constants table runs in strict mode (C-147), and under strict mode an unverified capability is refused (C-128), so a host on which strict mode cannot be granted is not used for the pair. Record also whether any CPU pin the runtime's thread layout requests succeeds and is verified on each host: if the layout's seqlock safety depends on that pin and it fails or is unverified, the module refuses to start in every mode (C-177, C-201), so such a host is not used either.
- Every hardware fact is tagged (`[measured]` when read from the machine, e.g. `ethtool -i`).

## Acceptance criteria
1. `doc/design/reports/l1-results.md` exists and contains a Hardware section naming 2 hosts, their NICs (model, driver, firmware, coalescing changeable or not), 1 managed switch and 1 unmanaged switch (model each) and the cables.
2. It contains one section per constant row: C-93, C-92, C-90, C-91, C-94, C-35, C-14, C-16, coalescing bias, C-164 (10 sections), each with the fields measured value, hardware, stamp, plus an "A used" field for the A computed per C-148.
4. Both hosts show real-time scheduling and memory lock as present (C-127, C-129), so strict mode can be granted (C-147), and the reference modules start on both hosts with no pin failure (C-177, C-201).
3. The recorded NIC driver and firmware match the `ethtool -i` output captured in a `stamp.json` from a real `noop` run on the pair.

## Definition of done
- [ ] The named hardware pair (hosts, NICs, switch, cables) is recorded
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Filling in measured values (scenario tickets 019-07 to 019-21, 019-24, 019-25, 019-27).
- Asymmetry reference hardware (added by 019-12).
- Updating the constants table (019-22).
