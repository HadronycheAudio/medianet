# 021-30 — Strict mode: memory locking and capability states
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-09, 021-13
## Context
Rules [decision]:

- **Strict mode** requires real-time scheduling granted on the data and clock threads, and memory locked. Restricting CPU idle states stays best-effort and is stamped (C-127).
- In strict mode an **unverified** capability is refused: unverified is treated as absent (C-128).
- Every capability in the regime stamp has one of four states: present, absent, unverified, not requested. "Not requested" marks a deliberate configuration choice, as distinct from a denial (C-129).
- Strict mode is set by the operator policy in the commissioning configuration (C-165). It is hosted-only; runs whose numbers enter the constants table run in it (C-147).
- Control isolation (housekeeping cannot delay the data path) is *present* on hosted platforms by construction of the thread model (C-161).
- **Seqlock safety and pinning** [decision, C-177, C-201]: if a thread layout's seqlock safety depends on a CPU pin and that pin fails, or is unverified, the module refuses to start in every mode, strict or not. Default priorities (clock above data) never depend on pinning.

macOS facts [memory]: the time-constraint policy is available unprivileged (021-09); memory can be locked per region but there is no whole-process lock as on Linux; there is no user-level control of CPU idle states. Each is to be confirmed here.
## Work
- In `rt.rs`, lock the memory of the edge's data and clock paths (buffers, rings, thread stacks) at setup; report the result.
- Express each capability of the capability report (time-constraint policy on data and clock threads, memory locked, workgroup joined, audio-work marking, CPU idle-state restriction, control isolation) in the four states of C-129; control isolation is *present* (C-161).
- Read strict mode from the commissioning configuration's operator policy (C-165).
- Whatever the mode, refuse startup with a named failure if the thread layout's seqlock safety depends on a CPU pin that failed or cannot be verified (C-177, C-201); apply the default priorities (clock above data) without relying on any pin.
- Strict mode: refuse startup with a named failure if real-time scheduling on the data or clock thread or memory locking is not *present*, including when it is *unverified* (C-128). CPU idle-state restriction never refuses; it is stamped.
## Acceptance criteria
1. Unit test: each capability maps to exactly one of present, absent, unverified, not requested; a capability disabled by configuration is *not requested*, a denied one is *absent*.
2. Strict mode with memory locking forced to fail (test seam) refuses startup with a named failure; the same with locking forced to *unverified* also refuses.
3. Non-strict mode with the same failures starts and stamps the states.
4. Unit test with a layout whose seqlock safety depends on a pin: a failed pin and an unverified pin (test seam) each refuse startup with a named failure in both strict and non-strict mode (C-177, C-201); the default layout starts without any pin.
5. Manual hardware test on the reference Mac: in strict mode the edge starts, with RT scheduling and memory lock *present* `[measured]`; CPU idle-state restriction stamped.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] CI builds the crate on macOS; hardware tests are a documented manual job
- [ ] Every new factual claim carries an evidence tag
## Out of scope
Applying the time-constraint policy (021-09) and workgroup join (021-13). Stamp assembly (021-20).
