# 018-20 — Reference hardware and D method documentation
**Epic:** 018-Linux-ALSA-edge · **Effort:** S · **Depends on:** 018-02, 018-03, 018-11, 018-18

## Context
- The Linux ALSA sink is the reference sink against which every other platform is measured (C-82 at L2, C-79 at L3); certification (epic 020) needs a **named reference hardware configuration (machine, sound device, driver, kernel)**.
- **C-172:** reference hardware is proposed by the implementing team after P0, approved by the product owner, and recorded in the constants table. This ticket makes the proposal and records it once approved (falsifier: the chosen device fails the D repeatability test, C-74, at L2).
- D = driver buffer + resampler group delay + converter, from hand-off to emission; for capture, from acoustic instant to the sample being available (C-170); the conversion delay is part of D (C-134) and never of L, and the manifest reports it separately on the device-facing port (C-207); both resamplers and their filter tables are the kit's, from epic 024a (C-197); driver and resampler parts are self-reported, the analogue part needs an external commissioning measurement, persisted per (device, driver, device rate), written only inside a commissioning window (C-107) and recorded with the window it was taken in (C-178); typical D on Linux about 1 ms (C-81); restart spread ≤ 25 µs over 20 restarts (C-74).
- Constants used by this epic: C-01, C-02, C-03, C-04, C-08, C-17, C-18, C-19, C-38, C-39, C-43, C-47, C-55, C-60, C-61, C-62, C-64, C-65, C-68, C-70, C-73 to C-82, C-95, C-96, C-101 to C-103, C-105 to C-109, C-111, C-119, C-120, C-123 to C-125, C-127 to C-130, C-133 to C-135, C-140, C-144, C-146 to C-149, C-153, C-159, C-160, C-162, C-165, C-166, C-169, C-170 to C-175, C-177, C-178, C-186, C-187, C-191, C-197, C-198, C-201, C-202, C-206, C-207, C-208, C-209, C-210, C-212, C-214.
- Locations (C-130, C-171): specifications in `doc/design/spec/`, reports in `doc/design/reports/`, the constants table `doc/design/02-constants.md`, the architecture specification `doc/design/01-architecture.md`.

## Work
1. After P0, propose the reference hardware configuration (machine model, CPU, sound device and model, driver name and version, kernel version and config, ALSA lib version) to the product owner; once approved, record it in the constants table (C-172) and in the edge design document.
2. Write the D measurement method: each component, how it is obtained, which part is self-reported vs commissioned, persistence and commissioning window (018-11), and how restart spread is measured. The resampler part takes the delays of the kit's default tables (epic 024a, C-197).
3. Update design documents where implementation differs (constants table first): provisional rate-loop bandwidth and residual statistic (018-02), phase-term decision (018-03), emission-error sign convention (018-04).
4. Confirm each open sub-question of the epic is listed with owner and falsifier, and that each epic Definition-of-done line is either covered or marked not applicable with a reason (C-125).

## Acceptance criteria
1. The constants table and the edge design document name one approved reference configuration with all 6 attributes listed in Work step 1 (C-172).
2. The D method section distinguishes self-reported and commissioned parts for all 3 components.
3. The constants table entries listed above are cited by id in the edge design document with their current values; no value is restated differently.
4. The 12 open sub-questions of epic 018 (including the settled resampler row kept for its history) appear with owner and falsifier.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] A named reference hardware configuration (machine, sound device, driver, kernel) is recorded for certification
- [ ] D measurement method documented, including which part is self-reported and which is commissioned
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Approving the reference hardware (product owner, C-172).
- Certification runs (epic 020).
