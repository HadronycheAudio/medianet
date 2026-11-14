# 019-09 — Pacing spread and switch bursts
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-08

## Context
Two items of epic 019 concern how bursty traffic behaves on a real switch:

- Constant **C-14 intra-slice egress spread** (decided): a source spreads its packets over at most 25 % of the slice period (125 µs at defaults), plus a per-source phase offset derived from the module id. The phase offset is in [0, 75 %) of the period, so offset plus spread stay inside the slice period (C-155). Its falsifier, assigned to L1: "L1 shows queue drops with the spread on, or consumer margins broken by it". The consumer margin is the alignment offset minus the observed lateness.
- Open sub-question: **"Does a small unmanaged switch absorb coherent bursts (32 packets, about 42 KB, per slice period)?"** Owner: this task, then task 028 as a commissioning test. Falsifier: drops with the spread on.

The load is the `two-bundles` load (019-08): 2 × 96 channels = 32 streams, 64 000 packets/s, 676 Mbit/s on one GbE port; each slice period (0.5 ms) every stream emits one packet of about 1 320 bytes (C-20), so 32 packets / about 42 KB arrive per period. Missing and late slices are counted separately (C-108). A is computed by the harness with the commissioning rule and recorded (C-148, 019-26). Every run is a strict-mode run (C-147).

The spread is switched off for the comparison run with the pacing override of the operator policy in the commissioning configuration (C-165).

## Work
- Add a `--spread <on|off>` parameter to `two-bundles` (on = C-14 as decided: 25 % of the slice period plus the per-source phase offset; off = spread and phase offset disabled through the C-165 pacing override in the commissioning configuration used for the run).
- Collect queue drops: switch port drop counters where the switch exposes them (managed switch), the receiving NIC's drop counters (`ethtool -S`) and the socket overflow counter on the consumer.
- Run `two-bundles` 4 times on the hardware pair: {spread on, spread off} × {managed switch, small unmanaged switch from 019-06}. Duration per run: as in 019-08 (1 hour) unless shortened with a reason stated in `l1-results.md`.
- Report: per run, drops (switch, NIC, socket), missing and late slices, lateness p99/max, minimum consumer margin, and with the spread on, each source's phase offset as a fraction of the slice period, checked against [0, 75 %) (C-155).
- Record in `doc/design/reports/l1-results.md` under C-14 whether its falsifier triggered (drops with spread on, or consumer margin broken), tag `[measured]`; record the switch-burst answer under "Switch findings" and hand it to task 028.

## Acceptance criteria
1. Dry-run passes in CI with both `--spread on` and `--spread off`.
2. `l1-results.md` contains a 4-row table (spread × switch) with drops, missing slices, late slices, lateness p99 and minimum consumer margin for each row.
3. With the spread on, every source's phase offset lies in [0, 75 %) of the slice period (C-155).
4. C-14 has a falsifier verdict (triggered / not triggered) tagged `[measured]`, or a stated reason why not.
5. The switch-burst question is answered: if drops occur with the spread on through the unmanaged switch, the falsifier is recorded as triggered and an open item (owner task 028, falsifier) is created.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Each pending-measurement constant in the table above has a measured value tagged `[measured]`, or a stated reason why not
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] Regime stamp fields touched are complete and documented
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The 1-hour zero-late and third-bundle refusal check (019-08).
- The commissioning test itself (task 028).
- Implementing the spread (epic 015) or the commissioning configuration's pacing override.
- Writing C-14 into the constants table (019-22).
