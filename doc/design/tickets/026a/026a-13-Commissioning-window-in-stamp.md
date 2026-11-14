# 026a-13 — Commissioning window in stamp
**Epic:** 026a-Embedded-HAL-and-host-integration · **Effort:** XS · **Depends on:** 026a-11, epic 005

## Context

A commissioning tool may drive test traffic only inside a **declared commissioning window that every module stamps**; outside it the tool is read-only (C-107). The device is a module, so its stamp must show the window.

The stamp is sectioned, multicast on any regime change and refreshed every 10 s (C-18); opening or closing a window is a regime change.

Life cycle (C-153): the tool announces the window on the control group with an expiry time; modules stamp it while open; it closes at expiry or on explicit close. What an open window disqualifies (C-178): ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified while it is open; the commissioning tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results, each recorded with the window it was taken in. The device's stamp therefore carries the window state beside its production figures (026b-11, 026b-12), so a reader can tell which figures were taken while a window was open. The message encoding is not fixed by the constants table: open item, owner epic 005 (control state machines) with the wire specification (epic 004), falsifier "a module cannot tell whether test traffic is inside a window". This ticket implements the device side against whatever core provides and does not define the message.

## Work

1. Feed core's commissioning-window state into the device stamp (the field defined by epic 013 / 005).
2. On window open or close, bump the stamp version and multicast the stamp (C-18).
3. Close the window at its announced expiry even when no close message arrives (C-153).

## Acceptance criteria

1. Host test: opening a window through core's interface changes the device stamp within one poll cycle, bumps the stamp version, and triggers a multicast; closing it does the same.
2. Host test: a window announced with an expiry and never explicitly closed leaves the device stamp at the expiry, with a version bump and multicast (C-153).
3. The window state appears in the record of the hosted collector (026a-12).

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The crate builds for at least one device target and for the host (for tests) in CI

## Out of scope

- The window declaration message and its state machine (epics 004, 005).
- Commissioning tooling.
