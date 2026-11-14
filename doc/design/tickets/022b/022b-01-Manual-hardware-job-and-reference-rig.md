# 022b-01 — Documented manual hardware job and reference rig
**Epic:** 022b-Windows-edge-L2-certification · **Effort:** S · **Depends on:** 022a-15, epic 020

## Context
- Part of epic 022b (Windows edge L2 certification). Definition of done: CI builds the crate on Windows; **hardware tests are a documented manual job**.
- The L2 Windows runs (022b-02 to 022b-06, 022b-10) need a reference rig: a Windows laptop as edge with a dedicated medianet NIC, a Linux backbone (clock holder) with a second holder-eligible Linux node for the holder-loss handover (C-78, C-212) whose time is usable (Locked) when the handover is triggered, so it takes over with the timeline epoch kept (C-213), holding the lowest module id among the rig's holder-eligible modules so that it wins among candidates all in holdover (C-221, set by the epic 020 harness), and a Linux reference sink, plus the L2 harness (epic 020) with `WasapiRawCapture` (022a-15).
- Locations are settled: results and the certification record go to `doc/design/reports/` (C-130); the runner and harness binaries live in the `hadronyche-tools` crate and throwaway probes in `probes/` (C-133).
- The reference list is settled in the epic: candidates are the (OS build, device, driver) combinations available on the reference rig; the list holds at least one.
- Commissioning baseline: dedicated medianet NIC, discovery chatter suppressed, firewall scoped to medianet ports `[decision]`. A laptop typically has Wi-Fi up beside the medianet NIC. The medianet is IPv4 only (C-100).
- Any measurement that drives test traffic as commissioning (for example measuring D) runs inside a declared commissioning window that every module stamps (C-107). Life cycle (C-153): the tool announces the window on the control group with an expiry time; modules stamp it while open; it closes at expiry or on explicit close. What an open window disqualifies (C-178): ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified while it is open; the commissioning tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results, each recorded with the window it was taken in.
- Every L2 run whose numbers enter the constants table runs in strict mode (C-147): MMCSS registration granted on the data and clock threads and memory locked (C-127, 022a-20). A run that cannot start in strict mode is not a certification run.
- For every L2 run the harness computes A with the commissioning rule (largest over sinks of maximum lateness + D + headroom) and records it with the results (C-148). Pass criteria use the platform's target E (1 ms on Windows, C-72), never the E measured in the same run (C-149).
- Supported Windows versions are set after task 001 reports; its probes test the current release and the oldest obtainable one (C-138). Candidate combinations are taken from the supported versions.
- Reference hardware is proposed by the implementing team, approved by the product owner and recorded in the constants table (C-172).
- Windows E is dominated by D uncertainty and scheduling spikes (third-party driver interrupt work), not by the clock `[memory]`.

## Work
1. Write the manual job document: rig wiring, OS build / device / driver recorded, power settings, firewall rule, how to open and close the commissioning window (C-153), how to record each D and restart-spread result with the window it was taken in and to take stamp-read production figures only with no window open (C-178), how to start the edge in strict mode (C-147), how the harness computes and records A (C-148), how to run each L2 scenario (for the handover: measure the natural difference between the two holders' crystals, inject the drift that makes the total 50 ppm and record both parts, C-223), where results go (`doc/design/reports/`, 022b-07).
2. List the candidate (OS build, device, driver) combinations available on the rig, within the supported Windows versions (C-138), with exact identifiers; submit the proposed reference hardware for product-owner approval (C-172).
3. A runner entry in `hadronyche-tools` that launches each scenario and collects raw results. Its output format and result file names within `doc/design/reports/` are decided here and recorded in the job document (C-206); only `l2-results-windows.md` is fixed by the design.
4. Dry run on the rig.

## Acceptance criteria
1. The manual job document exists and a second engineer can run a 5-minute smoke scenario from it alone (dry run recorded).
2. At least one candidate (OS build, device, driver) combination is listed with exact identifiers.
3. Results of the dry run land in `doc/design/reports/`.
4. The runner builds from `hadronyche-tools` on Windows.
5. The job document requires strict mode for every certification run and records the A computed by the harness (C-147, C-148).
6. The job document records each D and restart-spread result with the identity of the commissioning window it was taken in, and reads production figures from stamps only while no window is open (C-178).

## Definition of done
- [ ] CI builds the crate on Windows; hardware tests are a documented manual job

## Out of scope
- The certification runs and their thresholds (022b-02 to 022b-06, 022b-10).
