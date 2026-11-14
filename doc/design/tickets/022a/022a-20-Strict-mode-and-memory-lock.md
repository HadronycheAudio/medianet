# 022a-20 — Strict mode, memory lock and capability states
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** S · **Depends on:** 022a-07, epic 016

## Context
- Part of epic 022a (Windows edge backend), `rt.rs`.
- C-127: strict mode requires real-time scheduling granted on the data and clock threads, and memory locked. Restricting CPU idle states stays best-effort and is stamped.
- C-128: in strict mode an unverified capability is treated as absent, so strict mode is refused.
- C-129: every capability is stamped in one of four states — present, absent, unverified, not requested ("not requested" is a deliberate configuration choice, distinct from a denial).
- The epic maps the Windows mechanisms `[memory]`: real-time scheduling granted = MMCSS "Pro Audio" registration granted (022a-07); memory lock = `VirtualLock` after raising the process working-set minimum. The epic names no Windows mechanism for restricting CPU idle states, so that capability is stamped *absent* when requested and *not requested* otherwise.
- Bounded real-time stays *absent* whatever strict mode says (022a-07).
- Outside strict mode real-time failures are stamped, not fatal, with one exception handled in 022a-07: a failed or unverified pin on which seqlock safety depends refuses startup in every mode (C-177, C-201).
- Strict mode is set by the operator policy in the commissioning configuration (C-165). Strict mode is hosted-only, and every run whose numbers enter the constants table runs in strict mode (C-147), so the L2 runs depend on this gate.

## Work
1. Memory lock in `setup_process`: raise the working-set minimum and lock the memory of the data and clock paths; read back the result; state *present* or *absent* (or *not requested* when configuration disables it).
2. `CapState` per capability (real-time scheduling per thread, memory lock, CPU-idle restriction) exposed for the stamp (022a-17).
3. Strict-mode gate at startup: if strict mode is requested by the operator policy (C-165) and real-time scheduling on the data or clock thread, or memory lock, is anything other than *present*, refuse startup with a named reason naming the missing capability.
4. Unit tests with mocked OS results for every state combination.

## Acceptance criteria
1. **(022-AC-13.)** With strict mode requested, startup succeeds only when MMCSS is granted on both threads and memory is locked; any *absent* or *unverified* required capability refuses startup with a named reason (C-128).
2. Without strict mode, the same failures are stamped and startup continues (the seqlock pin refusal of C-177 and C-201 is tested in 022a-07).
3. Every capability field takes only the four C-129 values; a configuration that disables memory lock shows *not requested*, a denied lock *absent*.
4. On reference hardware, the memory-lock result as a standard user is recorded `[measured]` for 022b-08.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- MMCSS registration and the wake-latency self-test (022a-07).
- Stamp rendering (022a-17).
