# 017b-25 — L0 suite as the unprivileged CI gate
**Epic:** 017b-L0-harness-and-scenarios · **Effort:** S · **Depends on:** 017b-10, 017b-11, 017b-12, 017b-13, 017b-14, 017b-15, 017b-16, 017b-17, 017b-18, 017b-19, 017b-20, 017b-21, 017b-22, 017b-23

## Context
- L0 is the regression gate that runs **unprivileged in CI on every change** `[decision]`.
- AC-017.13: the whole L0 suite runs unprivileged in **under 10 minutes of CI time**.
- Unprivileged means non-strict: strict mode needs real-time scheduling and memory locking (C-127) and refuses unverified capabilities (C-128). L0 stamps those capabilities "not requested" (C-129) and never runs in strict mode (C-147). Non-strict does not relax seqlock safety: if a thread layout's seqlock safety depends on a CPU pin and the CI runner fails or cannot verify that pin, the module refuses to start in every mode (C-177, C-201), which fails the job.
- **Privileged tests (settled, C-146):** the few tests that need a privilege (for example stepping the wall clock) run in a separate privileged nightly job; the definition-of-done line "unprivileged" applies to per-change CI. L0 has no such test: clock steps are simulated by the injector (017b-02).
- Scenarios: `baseline`, `diamond`, `loss`, `late-vs-lost`, `gap-classes`, `realign`, `epoch`, `blackhole`, `flood`, `refusal`, `collision`, `malformed`, `partition`, `housekeeping`, `thread-death`. Each is run by `hadronyche-l0 <scenario> --duration <s>` (binary in the `hadronyche-tools` crate, C-133) and exits non-zero on any failed assertion or inert family.
- Fuzzing (C-95) runs in its own job (017b-24), outside this budget.
- L0 is the CI gate for every later task (consumers: epics 019, 020, 025, 027).
- Fuzzing for the reference modules' decoders is epic 017a's job (017a-11); this part's parsers are fuzzed in 017b-24.

## Work
1. CI job running all 15 scenarios (in parallel where the CI runner allows), without root, capabilities or real-time privileges.
2. Choose per-scenario durations so the total wall time is < 10 min; record the durations in one config file.
3. Upload every record file as a CI artefact; fail the job on any non-zero exit.
4. Make the job required on every change.

## Acceptance criteria
1. All 15 scenarios run in CI and pass, unprivileged (job runs as a non-root user with no added capabilities); every record file stamps real-time scheduling and memory locking as "not requested".
2. Total CI time of the L0 job < 10 min, measured over 5 consecutive runs.
3. A deliberately broken change (e.g. a one-bit flip in the mixer) makes the job fail.
4. Record files from every scenario are available as artefacts.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- L1/L2/L3 harnesses (epics 019, 020, 027); the fuzz jobs (017a-11, 017b-24); the privileged nightly job (C-146).
