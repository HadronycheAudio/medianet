# 028b-21 — `checklist-linux.md`
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028b-03, 028b-06, 028b-07, epic 020

## Context
Linux hosts are the backbone (clock holder, elections, conformance, commissioning). Baseline for every machine: a **dedicated medianet NIC**, OS **discovery chatter suppressed**, **firewall scoped to medianet ports**. The runtime uses IPv4 only (C-100), so IPv6 on the medianet interface only produces foreign traffic.

Linux items `[memory]`: privileges for real-time scheduling, memory lock and `/dev/cpu_dma_latency`; kernel preemption model recorded. Settled by the rulings:
- Strict mode requires real-time scheduling granted on the data and clock threads and memory locked; restricting CPU idle states (`/dev/cpu_dma_latency`) stays best-effort and is stamped (C-127).
- In strict mode an unverified capability is refused like an absent one (C-128).
- Capabilities are stamped as present, absent, unverified or not requested (C-129); `survey` reports them (028b-07).
- Strict mode is set per node in the commissioning configuration (C-165); the checklist tells the operator to set it on every Linux node that will run strict.
NIC: interrupt coalescing disabled on medianet interfaces (it biases software receive stamps), EEE checked.

Acceptance for checklists (epic AC-9): following it on a fresh machine yields a clean `survey`, i.e. no foreign multicast on the medianet interface for 10 minutes (028b-06).

## Work
- Write `doc/operations/checklist-linux.md` (C-171): dedicated NIC; firewall scoped to medianet ports; IPv6 disabled on the medianet interface; discovery services silenced on the medianet interface; real-time scheduling privilege (required for strict mode); memory lock limit (required for strict mode); access to `/dev/cpu_dma_latency` (best-effort); kernel preemption model recorded; coalescing off; EEE off.
- For each item: how to verify, and what the stamp / `survey` shows when done (capability state `present` for real-time scheduling and memory lock).
- Execute it on a fresh Linux install and attach the `survey` output.

## Acceptance criteria
1. On a fresh Linux machine, following the checklist yields a `survey` with no findings, including no `net.foreign_multicast` (IPv4 or IPv6) over 10 minutes (epic AC-9) and no `capability.*` finding for real-time scheduling or memory lock.
2. Every item has a verification and an expected stamp/tool output.

## Definition of done
- [ ] Each checklist item states how to verify it and what the stamp shows when it is done
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- macOS (028b-22) and Windows (028b-23).
