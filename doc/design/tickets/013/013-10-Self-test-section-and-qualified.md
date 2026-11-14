# 013-10 — Self-test section and qualified()
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-01, 013-02, 013-03, 013-09, 013-25

## Context
- Rule 9: a **self-test** runs before any number is reported: receive timestamps present and consistent with the monotonic clock; real-time scheduling actually granted; measured wake latency; packets per slice equal to the stamped regime's value (one at defaults, C-05); clock locked within its lock bound (at most E/2, 25 µs against the Linux E, C-62, pending simulation). Numbers produced before it passes are labelled **unqualified**. `[decision]`
- The self-test result is a regime section (changes the `StampId`) but is **not one of the eight groups**: a stamp without it is still complete (013-11) and is unqualified.
- Types and signatures from the epic:
  ```rust
  pub struct SelfTest { pub rx_stamps_consistent: bool, pub rt_granted: bool, pub wake_latency_ns: u32,
                        pub packets_per_slice: u8, pub locked_within_bound: bool }
  impl<'b> StampBuilder<'b> { pub fn self_test(&mut self, r: &SelfTest); }
  impl<'a> Stamp<'a> { pub fn qualified(&self) -> bool; } // self-test passed, no injection active, no commissioning window open
  ```
- 013-AC-6: `qualified()` is false while the self-test has not passed, any injection is active, or a commissioning window is open. Injection presence (including the test-only hooks of C-120 and C-145) is read from the Injection section (013-09); the window state from the same section (013-25).
- Commissioning window (C-153, C-178): ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified while a window is open, so an open window makes `qualified()` false. The commissioning tool's own measurements in the window (D, restart spread, group certification, soak) are valid commissioning results, each recorded with the window it was taken in (C-178); `qualified()` describes production figures and is not used to judge them (epic 028).
- Still open: the wake-latency pass limit and the receive-stamp consistency tolerance (owner: a measurement, task 001 then L2). This ticket stores the producer's verdict and does not invent thresholds.

## Work
1. Define the self-test section carrying all five `SelfTest` fields plus an explicit "passed" flag and, if failed, which item(s) failed (needed by strict mode, 013-20).
2. Implement `StampBuilder::self_test` and the reader on `Stamp`.
3. Implement `Stamp::qualified()`: true only if the self-test section is present, passed, `injection_active()` is false and `commissioning_window_active()` (013-25) is false (C-153). A stamp with no self-test section is unqualified.
4. Provide a helper that names each failed item (for messages): `rx_stamps_consistent`, `rt_granted`, `wake_latency_ns`, `packets_per_slice`, `locked_within_bound`.
5. Extend the fuzz corpus; document the section in `doc/design/spec/stamp.md`, citing C-05 and C-62 as the reference values and listing the two open thresholds with their owner.

## Acceptance criteria
1. Truth table test: for self-test {absent, failed, passed} × injection {none, one active} × commissioning window {closed, open}, `qualified()` is true in exactly 1 of the 12 cases (passed, none, closed).
2. Each of the 5 items set to failing individually yields a failed self-test whose failed-item list names exactly that item.
3. Round-trip preserves all 5 fields, including `wake_latency_ns` values 0 and 4 294 967 295.
4. Changing the verdict from failed to passed changes `Stamp::id()`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every stamp field has a stated source on each of the four profiles, or is marked absent there
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Design documents updated where behaviour differs, constants table first" (follows C-153 and C-178 as written); "The stamp format is versioned independently of the media format" (uses 013-02 framing unchanged).

## Out of scope
- Judging the commissioning tool's own results taken in a window (epic 028, C-178).
- Running the self-test on a host (measuring wake latency etc.) — the producer fills `SelfTest`; labelling outputs (013-19); strict-mode refusal (013-20).
- Setting the open thresholds (measurement).
