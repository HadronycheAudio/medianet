# 028b-23 — `checklist-windows.md`
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028b-03, 028b-04, 028b-06, 028b-07, epic 022

## Context
Windows machines are edges. Trust statement: "trusted segment per the recommended configuration; Windows and macOS edges are best-effort members of a weaker segment." Baseline: dedicated medianet NIC, OS discovery chatter suppressed, firewall scoped to medianet ports. The runtime uses IPv4 only (C-100), so IPv6 on the medianet interface only produces foreign traffic.

Windows items `[memory]`, to be confirmed by epic 022: allow inbound UDP in the firewall for the medianet ports; exclusive-mode access to the audio device (other software must release it; otherwise "the interface is held": the device cannot be opened in the required mode because another process has it open, C-175); DSCP policy if marking is wanted (classes C-13: 48 / 46 / 40); prevent sleep and power throttling; receive buffer size; silence discovery services on the medianet interface. NIC: coalescing off, EEE checked. Typical Windows D 6 to 20 ms (C-81); E target WASAPI exclusive 1 ms (C-72); a group with a Windows edge needs A of the order of 10 to 25 ms. Settled: sleep is a departure, the edge resets both timebase loops and returns through normal admission (C-124). Audio hardware at 48 kHz is converted 2:1 at the edge (C-80); the conversion delay is part of the edge's D (C-134, C-170), so the checklist records the device's native rate.

Acceptance (epic AC-9): fresh machine + checklist → clean `survey`, no foreign multicast on the medianet interface for 10 minutes.

## Work
- Write `doc/operations/checklist-windows.md` (C-171) with trust statement, baseline, IPv6 disabled on the medianet interface, and all items above, each with how to verify and what the stamp / `survey` shows.
- Execute on a fresh Windows install; attach `survey` output.

## Acceptance criteria
1. Fresh Windows + checklist → `survey` with no findings, including no `net.foreign_multicast` (IPv4 or IPv6) for 10 minutes (epic AC-9).
2. With the DSCP policy item applied, `survey` shows no `dscp.mismatch` for the Windows node.
3. Every item listed in Context is present with verification and expected output; the sleep item states the C-124 consequence.

## Definition of done
- [ ] Each checklist item states how to verify it and what the stamp shows when it is done
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Linux (028b-21), macOS (028b-22).

## Notes
- Open sub-question of the epic (owners epics 021 and 022, then this task): can discovery chatter, including IPv6, be fully suppressed per interface on current Windows? Falsifier: foreign multicast persists after the checklist.
