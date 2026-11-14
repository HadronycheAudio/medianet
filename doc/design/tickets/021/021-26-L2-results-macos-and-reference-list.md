# 021-26 — `l2-results-macos.md` and certified reference list
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-20, 021-23, 021-24, 021-25, 021-27, 021-29, 021-30
## Context
Deliverable: document `doc/design/reports/l2-results-macos.md`, the certification record; reports and results live in `doc/design/reports/` (C-130). It provides a certified platform profile and its per-OS checklist inputs to the commissioning tool (epic 028).

- A short reference list of certified OS build / device combinations is published; everything else stamps *unverified*. The list is the reference hardware approved under C-172 (021-31) on the supported macOS versions (C-138) that passes L2.
- Only the certification state *commissioned* gives a numeric E; *uncertified* and *self-monitored* declare E *unknown* (C-103, C-140).
- Uncertifiable in v1 regardless of engineering: a device whose driver misreports latency without a commissioning measurement; aggregate devices; Bluetooth and network outputs; combinations outside the reference list (*unverified*); isolation of the laptop (dual-homed to the internet, emits discovery chatter on the medianet port).
- Trust statement: "Trusted segment per the recommended configuration; Windows and macOS edges are best-effort members of a weaker segment." Commissioning baseline: dedicated medianet NIC, OS discovery chatter suppressed, firewall scoped to medianet ports (checklist by epic 028). Commissioning traffic only inside a declared commissioning window (C-107).
- Report-only on macOS: overflow accounting.
## Work
- Write `doc/design/reports/l2-results-macos.md`: test setup, capability states in strict mode (021-30), OS builds, devices, drivers, results of AC-27 (021-23), L2 common criteria (021-24) and the drift ramp (021-29), D method, cost calibration factor (021-27), wake latency, both stamps (the regime stamps of the sink and of the holder it followed, C-191), uncertifiable list, trust statement.
- Publish the reference list and fill the compiled-in table used by 021-20.
- List per-OS checklist inputs for epic 028 (permission pre-grant from 021-08, power, interface pinning, chatter suppression need); operator checklists live in `doc/operations/` (C-171).
## Acceptance criteria
1. `l2-results-macos.md` contains every AC-27, C-75…C-78 and C-119 result with `[measured]` tags and the raw-data location.
2. The reference list has at least one certified (OS build, device, driver) row; that combination, once *commissioned*, stamps its numeric E (C-140); any combination off the list shows *unverified* with E *unknown*.
3. The 028 inputs section exists.
4. Each certification result carries both stamps: the sink's regime stamp and that of the holder it followed (C-191).
## Definition of done
- [ ] A short reference list of certified OS build / device combinations is published; everything else stamps *unverified*
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
## Out of scope
The commissioning tool and checklist (epic 028).
