# 027-19 — Admission refusal check
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-11, epic 005

## Context
- Rule: a sink whose E exceeds the group's **required E** is refused at admission with a named reason. The reason is `RequiredEExceeded` (C-103; the same rule refuses a sink whose E is unknown from any group that sets a required E, and admits it to groups that set none).
- E = 99.9th percentile of |actual emission − (T(p)+A)| over 1 h, max beside (C-73). Linux sink target E = 50 µs (C-70); device E declared per device (epic 026). Comparisons use these target values, never an E measured in the same run (C-149); only certification state commissioned gives a numeric E (C-140).
- A connection alarm is raised on refusal (C-173).
- **027-AC-7 / AC-30 (rig part):** with the group's required E set below the device's own declared E, the device is refused at admission with the reason `RequiredEExceeded` on its own figure, whatever the Linux sink's E is (C-194); the Linux sink plays alone when its own E is within the required E.
- The epic's scenario list names this case `admission`.
- How a test sets a group's required E is still open, owner epic 005.
- Settled by C-194: the group's required E is set below the device's own declared E, and the device is refused on its own figure whatever the Linux sink's E is. The test no longer depends on E_device > E_linux. When E_linux is also above the required E, the Linux sink is refused too by the same rule (C-103); that is reported, not asserted.
- L3 rig: Linux sink on capture channel 1, device on channel 2.

## Work
1. `RigControl::set_required_e(group, ns)` by the mechanism epic 005 defines.
2. Scenario `admission`: set required E below the device's own declared E (C-194), using target values (C-70, C-149); where E_device > E_linux, choose it ≥ E_linux so the Linux sink stays admitted. (Re)admit both sinks; capture 60 s.
3. Check: the device's admission result is "refused" with reason `RequiredEExceeded` and a connection alarm is raised (C-173), whatever the Linux sink's E (C-194); channel 2 silent (no markers). If E_linux is within the required E, the Linux sink is admitted and playing (markers present on channel 1); otherwise its admission result is reported.
4. Restore required E and confirm the device is admitted again.

## Acceptance criteria
1. **027-AC-7 / AC-30.** Device refused with reason `RequiredEExceeded` (C-103), recorded in the report together with the connection alarm (C-173).
2. Channel 2 shows zero markers; when E_linux is within the required E, channel 1 shows the expected markers for the whole 60 s.
3. After restoring required E, the device is admitted and p99.9 |skew| is back within `limit_ns`.
4. With E_device ≤ E_linux the scenario still runs (C-194): the device is refused on its own figure, and the Linux sink's admission result is reported with both E values and the required E used.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier (the E_device ≤ E_linux case is settled by C-194)
- Not applicable (C-125): "Every parser touched is fuzzed" — no new parser; "Regime stamp fields touched" — reads stamps only through 027-07/027-22; "Design documents updated" — 027-20; "The skew analysis runs in CI on recorded two-channel files" — the check counts markers, it adds no skew fixture; "The rig is photographed or diagrammed" — 027-09; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Implementing admission control (other epics).
- The unknown-E case of C-103 (tested where admission is implemented).
- Packaging this procedure for operators (epic 028).
