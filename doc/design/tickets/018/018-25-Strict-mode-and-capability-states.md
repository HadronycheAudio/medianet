# 018-25 — Strict mode and capability states for the ALSA edge
**Epic:** 018-Linux-ALSA-edge · **Effort:** S · **Depends on:** 018-12, 018-16, 018-18, epic 015, epic 016

## Context
- **C-165:** the operator policy in the commissioning configuration sets strict mode.
- **C-127:** strict mode requires real-time scheduling granted on the data and clock threads, and memory locked. Restricting CPU idle states stays best-effort and is stamped.
- **C-128:** in strict mode an unverified capability is refused: unverified is treated as absent.
- **C-129:** each capability is stamped in one of four states: present, absent, unverified, not requested. "Not requested" marks a deliberate configuration choice, as distinct from a denial.
- **C-147:** any run whose numbers enter the constants table runs in strict mode, so every L2 run of this sink and capture (epic 020) is strict. Strict mode is hosted-only.
- **C-146:** tests that need a privilege run in the separate privileged nightly job; "unprivileged" in the definition of done applies to per-change CI.
- Epic criterion 018-A8 (best effort, strict mode not set) stays: the module runs unprivileged and stamps what it lacks (018-12).
- **C-177, C-201:** if a thread layout's seqlock safety depends on a CPU pin and that pin fails, or is unverified, the module refuses to start in every mode, strict or not. Default priorities (clock above data) never depend on pinning.
- Real-time setup itself (scheduling, memory lock, probing what was granted) is epic 016.

## Work
1. Read the strict-mode flag from the operator policy (C-165) for `AlsaSink` and `AlsaCapture`.
2. At start, obtain from epic 016 the state of real-time scheduling on the data thread and on the clock thread, memory lock and CPU idle restriction, each in one of the four states of C-129.
3. In strict mode, refuse to start with a named reason naming each required capability that is absent or unverified (C-127, C-128); CPU idle restriction never causes a refusal and is only stamped.
4. In every mode, strict or not, refuse to start with a named reason when the thread layout's seqlock safety depends on a CPU pin that failed or is unverified (C-177, C-201).
5. Out of strict mode, keep the best-effort behaviour of 018-12 for everything else; a capability the configuration does not ask for is stamped "not requested".
6. Feed the capability set to the stamp fields (018-18).

## Acceptance criteria
1. **Epic criterion 018-A11:** with strict mode set and memory lock absent, the sink refuses to start with a named reason naming memory lock; the same with real-time scheduling unverified on the clock thread (C-128); the same for the capture.
2. With strict mode set and CPU idle restriction absent, both modules start and the stamp shows it as absent (C-127).
3. A table-driven test maps each probe outcome to one of the four states of C-129, including "not requested" for a capability disabled in configuration.
4. A privileged test in the nightly privileged job (C-146) starts the sink in strict mode with all required capabilities present.
5. With strict mode not set and a fake thread layout whose seqlock safety depends on a CPU pin, a failed pin and an unverified pin each refuse start with a named reason; with the default layout, whose priorities do not depend on pinning, a failed pin does not (C-177, C-201).

## Definition of done
- [ ] Tests green in CI, unprivileged (the privileged strict-mode test runs in the privileged nightly job, C-146)
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Real-time setup and capability probing (epic 016).
- The commissioning configuration format (commissioning tooling).
- Strict mode on devices, which stamp real-time scheduling and memory lock as present by construction (C-147).
