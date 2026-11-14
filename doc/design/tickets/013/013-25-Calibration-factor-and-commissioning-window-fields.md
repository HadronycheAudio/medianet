# 013-25 — Calibration factor and commissioning window fields
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-05, 013-09

## Context
- Unit of the compiler's cost bound (C-106): **counted abstract operations per slice, identical on every machine. Each target has a calibration factor (time per operation), measured once and recorded in the regime stamp; admission converts with it.** `[decision]` The epic places it in the **Platform** group.
- Commissioning tool activity (C-107): **may drive test traffic only inside a declared commissioning window that every module stamps; read-only outside it.** `[decision]` The epic places the window state in the **Injection** group, beside the injections, because it marks test traffic.
- Commissioning window life cycle (C-153): **announced by the tool on the control group with an expiry time; modules stamp it while open; it closes at expiry or on explicit close.** What it disqualifies (C-178): **ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified while a window is open; the commissioning tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results, each recorded with the window it was taken in.** `[decision]`
- Both are regime fields: a change changes the `StampId` (013-05) and starts a new record set.
- An open window makes `qualified()` false (C-153, C-178); 013-10 reads `commissioning_window_active()` for that.
- The calibration factor's value is measured by the DSL conformance suite (epic 025), which runs a reference workload once per listed target and records time per operation (C-176); its unit and width are fixed with epic 004's byte layout. A target not yet calibrated has no factor.

## Work
1. Add to the Platform section: the calibration factor (time per abstract operation, unit stated in the spec), or "not calibrated".
2. Add to the Injection section: commissioning window open or not and, while open, its expiry time as announced (C-153), with the window's identity as declared by the commissioning tool if it has one.
3. Extend the builder methods `platform` and `injection` and the readers on `Stamp`; provide `Stamp::commissioning_window_active() -> bool`.
4. Extend the fuzz corpus (C-95 budget); add both fields to the field tables of `doc/design/spec/stamp.md` with their source on each of the four profiles.

## Acceptance criteria
1. Round-trip preserves the calibration factor and "not calibrated" as distinct values.
2. Round-trip preserves the window state and expiry; `commissioning_window_active()` is true only while it is open.
3. Opening or closing the window changes `Stamp::id()`; changing the calibration factor changes it.
4. A stamp with an open window and no injection reports `injection_active()` false (the window is not an injection) and `commissioning_window_active()` true, so `qualified()` is false (C-153, through 013-10).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every stamp field has a stated source on each of the four profiles, or is marked absent there
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Design documents updated where behaviour differs, constants table first" (follows C-106, C-107, C-153, C-176 and C-178 as written); "The stamp format is versioned independently of the media format" (adds fields to existing versioned sections through 013-02 framing).

## Out of scope
- Measuring the calibration factor (epic 025, C-176); announcing, expiring and closing the window (epic 028, C-153); the commissioning tool (epic 028); labels in outputs (013-19).
