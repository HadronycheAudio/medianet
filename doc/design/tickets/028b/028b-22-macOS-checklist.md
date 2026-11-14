# 028b-22 — `checklist-macos.md`
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028b-03, 028b-06, 028b-07, epic 021

## Context
macOS machines are edges. Trust statement: "trusted segment per the recommended configuration; Windows and macOS edges are best-effort members of a weaker segment." Baseline: dedicated medianet NIC, OS discovery chatter suppressed, firewall scoped to medianet ports. The runtime uses IPv4 only (C-100), so IPv6 on the medianet interface only produces foreign traffic.

macOS items `[memory]`, to be confirmed by epic 021: pin to the medianet interface (Wi-Fi is default-routed); grant the local-network permission; prevent sleep; exclude aggregate, Bluetooth and network audio devices; silence discovery services on the medianet interface. NIC: coalescing off, EEE checked. Typical macOS D a few ms (C-81); E target 100 µs (C-71). Settled: sleep is a departure, the edge resets both timebase loops and returns through normal admission (Locking, Aligning) (C-124), so preventing sleep avoids a re-admission during a show. Audio hardware at 48 kHz is converted 2:1 at the edge (C-80); the conversion delay is part of the edge's D (C-134, C-170), so the checklist records the device's native rate.

Acceptance (epic AC-9): fresh machine + checklist → clean `survey`, no foreign multicast on the medianet interface for 10 minutes.

## Work
- Write `doc/operations/checklist-macos.md` (C-171) with the trust statement, baseline, IPv6 disabled on the medianet interface, and all items above, each with how to verify and what the stamp / `survey` shows.
- Execute on a fresh macOS install; attach `survey` output.

## Acceptance criteria
1. Fresh macOS + checklist → `survey` with no findings, including no `net.foreign_multicast` (IPv4 or IPv6) for 10 minutes (epic AC-9).
2. Every item listed in Context is present with verification and expected output; the sleep item states the C-124 consequence.

## Definition of done
- [ ] Each checklist item states how to verify it and what the stamp shows when it is done
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Linux (028b-21), Windows (028b-23).

## Notes
- Open sub-question of the epic (owners epics 021 and 022, then this task): can discovery chatter, including IPv6, be fully suppressed per interface on current macOS? Falsifier: foreign multicast persists after the checklist. If it persists, record it in the open sub-questions table with the epic 021 findings.
