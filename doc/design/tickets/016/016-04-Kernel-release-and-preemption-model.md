# 016-04 — Kernel release and preemption model
**Epic:** 016-Real-time-setup-backend · **Effort:** S · **Depends on:** 016-02

## Context
- `ProcessRtReport` carries two plain fields, `kernel_release: String` and `preempt_model: String`, which go into the regime stamp (epic 013).
- Reason: without a real-time kernel, Linux scheduling latency has no hard bound. The edge's emission-error figure E (50 µs target for the Linux reference sink, C-70) is the 99.9th percentile over a one-hour run (C-73). The stamp records the kernel's preemption model so a result can be interpreted `[decision]`.
- The epic fixes the value set, using the kernel's own model names `[kernel-doc]`: `none`, `voluntary`, `full`, `rt`, `dynamic(<active mode>)` for a `PREEMPT_DYNAMIC` kernel (`dynamic(unknown)` when the active mode cannot be read), and `unknown` when nothing can be determined. This ticket owns the detection order.
- Open sub-question owned by task 020 (L2 Linux), not this task: should `PREEMPT_RT` be recommended or required for the 50 µs E target? Falsifier: the target is met on a stock kernel over a one-hour run. This ticket only reports the model.
- Every parser touched is fuzzed (epic definition of done). Fuzz budget (C-95): 60 s on every change and 1 hour nightly; the corpus is kept.

## Work
1. `fn kernel_release() -> String` from `uname(2)` `release`.
2. `fn preempt_model() -> String`: read `/sys/kernel/realtime` (value `1` means `rt`); else parse the version string (`uname` `version` / `/proc/version`) for `PREEMPT_RT`, `PREEMPT_DYNAMIC`, `PREEMPT`; for dynamic kernels, read the current mode from `/sys/kernel/debug/sched/preempt` if readable (the active mode is shown in parentheses). Otherwise `unknown`. Never fails.
3. A pure function `fn parse_preempt(version: &str, realtime_flag: Option<&str>, dynamic: Option<&str>) -> String` holding all parsing, with a `cargo fuzz` target run 60 s on every change and 1 hour nightly, its corpus committed and reused (C-95).
4. Document the detection order beside the value set in the regime-stamp field list of the specification under `doc/design/spec/` (C-130).

## Acceptance criteria
1. `parse_preempt` returns `rt` when `realtime_flag == Some("1")`, regardless of the version string (unit test).
2. For dynamic input `none voluntary (full) lazy` it returns `dynamic(full)` (unit test).
3. For a version string containing `SMP PREEMPT_DYNAMIC` and no readable debugfs it returns `dynamic(unknown)`; for an empty string it returns `unknown` (unit tests).
4. The fuzz target runs 60 s on every change and 1 hour nightly without a crash or panic (C-95), starting from the kept corpus.
5. On the CI runner, `kernel_release()` is non-empty and equals the output of `uname -r`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Whether `PREEMPT_RT` is required (task 020).
- Writing the fields into the stamp (016-13).
