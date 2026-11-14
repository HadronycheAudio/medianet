# 022b-07 — `l2-results-windows.md` and certified reference list
**Epic:** 022b-Windows-edge-L2-certification · **Effort:** S · **Depends on:** 022a-08, 022a-09, 022a-10, 022a-18, 022a-19, 022a-20, 022a-21, 022b-02, 022b-03, 022b-04, 022b-05, 022b-06, 022b-09, 022b-10

## Context
- Part of epic 022b (Windows edge L2 certification). Deliverable: document `doc/design/reports/l2-results-windows.md` (C-130), the certification record.
- Definition of done: a short reference list of certified OS build / device / driver combinations is published; everything else stamps *unverified* and declares E *unknown* (C-103).
- The reference list is settled in the epic: a table in the certification record plus a data file in the crate read by the stamp builder (022a-17); candidates are the combinations on the reference rig; at least one combination.
- Uncertifiable in v1 regardless of engineering: any bound on scheduling latency; a device whose driver misreports latency, without the commissioning measurement; any combination outside the short reference list (*unverified*); isolation of the laptop (dual-homed, emits discovery chatter).
- Targets recorded against: E ≤ 1 ms (C-72, C-73); D restart spread ≤ 0.5 ms over 20 restarts (C-74); typical D 6–20 ms (C-81); cross-sink skew ≤ 1.05 ms (C-82, statistic as in C-79); no capture gaps at commissioned A (C-108, C-101, C-109); zero slip (C-76), including E held through holdover in the holder-loss handover, frequency continuity at takeover within C-61 (C-202, C-212) and the timeline epoch kept (C-213), with both parts of the 50 ppm difference, measured natural and injected (C-223); E held through the drift ramp (C-119); idle sink CPU < 0.1 core; firewall failure < 5 s; exclusive-mode refusal → *uncertified*; return from sleep through Locking and Aligning (C-124); strict-mode refusal (C-127, C-128); step check (C-121); peer verification of sync and control DSCP (C-152). Pass criteria use the target E, never the E measured in the same run (C-149); every run whose numbers enter the constants table ran in strict mode (C-147) with A computed and recorded by the harness (C-148).
- Only the certification state *commissioned* gives a numeric E (C-140). Reference hardware is proposed by the implementing team, approved by the product owner and recorded in the constants table (C-172). Operator checklists and the guide live in `doc/operations/` (C-171); epic 028 writes them from the inputs handed over here.
- Provides a certified platform profile and per-OS checklist inputs to the commissioning tool (epic 028), including the edge configuration key for the commissioned D.
- Trust statement: "Trusted segment per the recommended configuration; Windows and macOS edges are best-effort members of a weaker segment."

## Work
1. Write `doc/design/reports/l2-results-windows.md`: rig, combinations, each criterion with measured value, pass/fail, evidence tag `[measured]`; answers to the sub-questions owned by this epic (D restart spread, batched capture, DSCP without administrator rights, firewall query versus probe); the trust statement; both stamps, that is the regime stamps of the Windows sink and of the holder it followed (C-191).
2. Publish the reference list of certified combinations and wire it into the stamp's lookup data (022a-17), after product-owner approval of the reference hardware, and record the approved hardware in the constants table (C-172).
3. Hand checklist inputs (firewall, DSCP policy route, power, dedicated NIC, D configuration key) to epic 028.

## Acceptance criteria
1. `l2-results-windows.md` lists a measured value or a test result and a verdict for every one of 022-AC-2 to 022-AC-16 and every part of 022-AC-1.
2. The reference list has at least one certified combination with exact OS build, device and driver identifiers; the stamp of a non-listed combination says *unverified* with E *unknown* (test from 022a-17 re-run with the published list).
3. Epic-028 inputs are delivered.
4. The record carries both stamps: the regime stamps of the Windows sink and of the holder it followed (C-191).

## Definition of done
- [ ] A short reference list of certified OS build / device / driver combinations is published; everything else stamps *unverified*
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Rewriting the design text's `[memory]` facts (022b-08).
