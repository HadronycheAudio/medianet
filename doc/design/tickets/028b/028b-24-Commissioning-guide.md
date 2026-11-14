# 028b-24 — `commissioning-guide.md`
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** M · **Depends on:** 028a-04, 028a-05, 028a-06, 028a-07, 028a-08, 028a-09, 028a-10, 028b-02, 028b-03, 028b-04, 028b-05, 028b-06, 028b-07, 028b-08, 028b-09, 028b-11, 028b-12, 028b-14, 028b-15, 028b-16, 028b-17, 028b-18, 028b-20, 028b-21, 028b-22, 028b-23

## Context
The commissioning guide is the operator's walk-through that turns a set of machines and a switch into a **commissioned medianet**: validate offline, measure on site, propose the numbers the operator must choose, certify what can be certified. Operator policy covered: A per group (no default, C-55; the `default` group for sinks in no named group), strict mode, per-input alignment opt-out and pacing override (C-165), skew budgets for fan-ins that feed no group (C-142, refusal NoSkewBudget, C-185), declared buffer memory checked before any change of A, sinks included (C-211, C-222), required E per group (optional; sinks with unknown E only in groups without one, C-103), trunk capacities per direction (C-117), domain id (16 bits, 0 = unconfigured, C-11). Scale: ≤ 64 modules (C-34), which also bounds followers per holder at 63 (C-114); at least one Linux holder-capable node. Test activity (burst test, soak, marker loop, pairwise capture) happens only inside a commissioning window stamped by every module (C-107). The tool announces the window on the control group with an expiry time; it closes at expiry or on explicit close (C-153). While it is open, ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified; the tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results, each recorded with the window it was taken in (C-178). Changing A on a live medianet: the tool announces the new A to the group's members inside a commissioning window, after the buffer-memory check (C-224, `propose-a --apply`); each sink switches at its own position (C-214), the new A travels upstream one hop per receiver report (C-198) and every fan-in that inherited the old A re-aligns at its own announced position (C-186, C-208). A consumer clamped by an oversized A keeps the cannot-meet-A alarm until a later change of A that fits (C-216). Controller streams count in port and trunk sums at the C-219 cost and the controller member in the bundle cap (C-209). The proposal uses the maximum lateness since the connection went live (C-148). A commissioned sink whose clock is unlocked stays commissioned, its claim suspended until it relocks (C-180). The guide lives in `doc/operations/` (C-171); certificates are stored beside the collected stamps in the deployment's data directory (C-171).

## Work
- Write `doc/operations/commissioning-guide.md` walking through one complete medianet from empty switch to certificate: switch checklist → per-OS checklists → configuration file → `validate` → `survey` → `window open --duration <d>` (an expiry long enough for the steps, at least 24 h if the soak runs) → `burst-test` / `soak` → configure a provisional A (`propose-a --apply` inside the window on a running group, C-224) → close the window and run `propose-a` (lateness figures inside a window are unqualified, C-178) → reopen a window and apply A with `propose-a --apply` (after the buffer-memory check, C-211, C-222; sinks switch at their own positions, C-214; inputs re-align, C-186) → `certify-sink` per sink → `certify-group` → `window close` → `watch` in operation.
- Include the configuration file format with an example, each command's output for the example, and what to do for each refusal/finding/alarm id.
- Perform the walk-through on the lab rig and attach the resulting certificate.

## Acceptance criteria
1. An engineer following only the guide on the lab rig reaches a written group certificate.
2. Every refusal, finding and alarm id emitted by the tool appears in the guide with its remedy.

## Definition of done
- [ ] The commissioning guide walks through one complete medianet from empty switch to certificate
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Tool implementation (all other tickets).

