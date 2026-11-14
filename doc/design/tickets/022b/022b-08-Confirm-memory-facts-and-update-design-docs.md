# 022b-08 — Confirm `[memory]` facts and update design documents
**Epic:** 022b-Windows-edge-L2-certification · **Effort:** S · **Depends on:** 022b-07, epic 001

## Context
- Part of epic 022b (Windows edge L2 certification). Every Windows platform fact in the epic is `[memory]` — recalled, not verified — until task 001 (receive-stamp probes) and this epic's L2 run confirm it.
- Facts to resolve: QPC host clock; Winsock receive-timestamp control in QPC units on recent Windows 10/11 builds; MMCSS "Pro Audio" not hard real-time; memory lock by `VirtualLock` after raising the working-set minimum; device clock position+QPC; driver-reported latency unreliable; don't-fragment option; DSCP via QoS flow API / `IP_TOS` ignored; no per-socket drop counter; batch receive by loop; timer granularity 15.6 ms default / about 0.5 ms best; UDP connection-reset; firewall first-run prompt; 64 KB default receive buffer; driver interrupt spikes; multicast by interface index; power / modern standby; device period 3 to 10 ms.
- A figure taken from a named, published standard may be cited with its clause and tagged `[standard]` once the citation is checked (C-132); Windows platform behaviour is vendor behaviour, not a standard, and is confirmed `[measured]`.
- Locations (C-130): the evidence appendix is `doc/design/06-evidence-appendix.md`; the constants table is `doc/design/02-constants.md`; results are in `doc/design/reports/`. Nothing is written into the frozen POC tree.
- 022-AC-10: the receive-timestamp claim is replaced by a `[measured]` statement from task 001.

## Work
1. For each fact above, replace `[memory]` with the `[measured]` result (from the measurement tickets of epics 022a and 022b and task 001), with `[standard]` where C-132 applies, or correct it; record each in the evidence appendix.
2. Replace the receive-timestamp claim with task 001's `[measured]` statement.
3. Update design documents where measured behaviour differs, constants table first (C-72, C-74, C-81, C-82 and the C-150 budget split, as measured).

## Acceptance criteria
1. **(022-AC-10.)** The receive-timestamp claim is replaced by a `[measured]` statement citing task 001.
2. No `[memory]` tag remains on any Windows platform fact in the design text of epics 022a and 022b; each resolved fact has an entry in `doc/design/06-evidence-appendix.md`.
3. Every changed constant is changed first in the constants table.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] Every `[memory]` fact in this file is either confirmed (`[measured]`) or corrected
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- New measurements (done in earlier tickets).
