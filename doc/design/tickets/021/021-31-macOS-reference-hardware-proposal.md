# 021-31 — macOS reference hardware proposal (new, C-172)
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** XS · **Depends on:** epic 001
## Context
The L2 runs, the commissioning measurement of D and the calibration factor all run on "the reference Mac" and "the reference device" (021-08, 021-22, 021-23, 021-24, 021-27, 021-29), and the certified reference list (021-26) is built from them.

- Reference hardware [decision, C-172]: proposed by the implementing team (after P0 for audio hardware), approved by the product owner, recorded in the constants table.
- Supported OS versions [decision, C-138]: set after the receive-stamp probe task (001) reports; the probes test the current release of each OS and the oldest obtainable one.
- The macOS E target applies to built-in or bus-attached devices (C-71); the epic asks for which device classes (built-in, bus-attached, USB class) E is reachable, so the proposal should let 021-23 test each class.
- Excluded regardless: aggregate devices, Bluetooth and network outputs (021-11).
- The constants table is `doc/design/02-constants.md`; reports go in `doc/design/reports/` (C-130).
## Work
- Propose the macOS reference hardware: Mac model(s) with CPU architecture, the audio device(s) per device class with transport, and driver (built-in or named third-party with version).
- Pair each with the supported macOS versions from task 001's report (C-138).
- Write the proposal as a short note in `doc/design/reports/` and submit it to the product owner.
- Once approved, record the rows in the constants table (C-172) and point 021-08, 021-22, 021-23, 021-24, 021-27 and 021-29 at them.
## Acceptance criteria
1. The proposal note names, per row, Mac model, CPU architecture, audio device, device class, driver and version, and macOS version(s).
2. At least one row uses a built-in or bus-attached device, so the C-71 target can be tested.
3. The product owner's approval is recorded, and the approved rows are in `doc/design/02-constants.md`.
## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
## Out of scope
Supported switch families (proposed in P7, C-172; not this epic). Running L2 (021-23, 021-24, 021-29). Publishing the certified reference list (021-26).
