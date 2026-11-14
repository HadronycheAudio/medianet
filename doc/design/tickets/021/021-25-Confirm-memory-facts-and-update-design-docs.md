# 021-25 — Confirm `[memory]` facts and update design documents
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-05, 021-08, 021-21, 021-22, 021-23, 021-24, 021-30, epic 001
## Context
Evidence warning: every macOS platform fact in epic 021 is `[memory]` until epic 001's receive-stamp probes and this epic's L2 run confirm it. Epic criterion 021-AC9: this ticket replaces the receive-timestamp row of the epic's platform mapping with a `[measured]` statement citing task 001.

`[memory]` facts to resolve: mach absolute time needs timebase conversion and stops during sleep; `SO_TIMESTAMP_MONOTONIC` kernel stamps; time-constraint policy unprivileged and workgroup join; callback supplies sample time, host time and rate scalar; D property composition; dedicated IPv4 don't-fragment option; `IP_TOS` DSCP; no per-socket drop counter; Wi-Fi multicast default-route trap; local-network permission prompt; lid-close sleep; background throttling; other apps changing sample rate; Bluetooth/AirPlay latency.

Constants this epic may change if measurements disagree: C-71 (E target), C-74 (restart spread), C-81 (typical D), C-150 (macOS E budget split). Only strict-mode runs may change them (C-147).
## Work
- For each fact, cite the ticket result that confirms it (`[measured]`), or a checked clause of a named published standard (`[standard]`, C-132) where one applies, or write the correction.
- Include the memory-locking and CPU idle-state facts confirmed in 021-30.
- Replace the receive-timestamp row and text in the epic 021 document with epic 001's `[measured]` statement.
- Update design documents wherever behaviour differs, constants table first (e.g. C-71, C-74, C-81 if measured values change targets).
## Acceptance criteria
1. 0 `[memory]` tags remain on the macOS platform facts of epic 021; each is `[measured]` or `[standard]` with its source, or corrected.
2. The receive-timestamp row and text cite epic 001's measurement.
3. Every changed constant is updated in the constants table before any other document.
## Definition of done
- [ ] Every `[memory]` fact in this file is either confirmed (`[measured]`) or corrected
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
## Out of scope
New measurements (done in 021-05 … 021-24 and 021-27 … 021-30).
