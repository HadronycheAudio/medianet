# 028b-20 — `checklist-switch.md`
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028b-03, 028b-04, 028b-15, 028b-16, 028b-19

## Context
Operator checklist for the switch fabric. Recommended configuration: wired, isolated LAN with no internet traffic; a single non-blocking switch is the simple case (all links are endpoint links, so online endpoint admission is complete); with several switches, trunks are invisible to endpoints and must be declared.

Items to cover, each with how to verify it and what the stamp/tool shows when done:
- Isolation: no internet uplink, only medianet traffic. The runtime uses IPv4 only, multicast with TTL 1 (C-100), so multicast never needs to cross a router.
- QoS: sync CS6 (48), media/probes EF (46), control CS5 (40) (C-13); per-family settings from 028b-19 for the families recorded in the constants table (C-172); unmanaged switch: marking does nothing, and the tool says so.
- Energy-efficient Ethernet off on medianet ports.
- Burst handling: `burst-test` with two 96-channel bundles (32 packets ≈ 42 KB per 0.5 ms slice; runtime pacing ≤ 25 % of slice, C-14) shows no drops. Run inside a commissioning window (C-107).
- MTU: path MTU probed at admission at the intended media packet size (C-116), fallback 1 280 bytes (C-06); `survey` lists connections not at interface MTU.
- Port speeds: any port above about 198 channel-copies runs 10 GbE (C-24, C-25); 100 Mbit ports hold about 19 channels (C-26).
- Trunks declared in configuration with their real rate per direction (C-117).
- Loss soak: 24 hours at the two-bundle load, aggregate loss against the floor (about 58 expected, C-90, C-118) and the threshold (C-91, 10 × the clean-wire loss measured at L1, C-167). Above 10 × the floor (about 580 losses in 24 h) reopens the redundancy decision; between 1 × and 10 × is investigated as a cabling or host problem (C-179).

## Work
- Write `checklist-switch.md`, one item per line with "Verify:" and "Stamp/tool shows:". It lives in `doc/operations/` (C-171).

## Acceptance criteria
1. Every item listed in Context is present, each with a verification step and the expected tool/stamp output.
2. Following it on the lab switch yields no `dscp.*`, `mtu.*` or `nic.eee_enabled` finding from `survey` and zero drops from `burst-test`.

## Definition of done
- [ ] Each checklist item states how to verify it and what the stamp shows when it is done
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Host NIC settings (028b-21 to 028b-23).
