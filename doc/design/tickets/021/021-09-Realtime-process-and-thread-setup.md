# 021-09 — Real-time process and thread setup
**Epic:** 021-macOS-backend-and-Core-Audio-edge · **Effort:** S · **Depends on:** 021-01, 021-02, epic 015
## Context
`rt.rs` implements `OsRealtime for MacRealtime { setup_process, setup_thread, measure_wake_latency }` (trait from epic 015) and a capability report.

- Mechanism [memory]: time-constraint thread policy, available **unprivileged**. Check: wake-latency measurement in the self-test.
- Porting trap [memory]: **background throttling** applies unless the process is marked as audio work.
- Real-time scheduling is expected to be available on macOS (not report-only).
- Joining the audio device's workgroup is 021-13 (needs the Core Audio device).
- Strict mode requires real-time scheduling on the data and clock threads and memory locked (C-127); memory locking, the four capability states (C-129) and the strict-mode refusal (C-128) are 021-30. This ticket reports the raw outcomes that 021-30 maps.
- No wake-latency pass threshold exists in the design. The self-test reports the figures; it does not fail on them. A threshold, if any, comes from the L2 run (021-23) and is entered in the constants table by the design owner (open sub-question of the epic).
## Work
- `setup_process`: mark the process as audio/latency-critical work so background throttling does not apply; record success in the capability report.
- `setup_thread(period, computation, constraint)`: apply the time-constraint policy with parameters converted to mach ticks via the timebase ratio (021-02).
- `measure_wake_latency`: on a real-time thread, sleep to an absolute deadline N times (e.g. 10 000) and return percentile statistics of lateness in ns (median, 99.9th, max).
- Capability report struct: time-constraint policy applied (yes/no), audio-work marking (yes/no), workgroup joined (filled by 021-13), wake-latency stats.
## Acceptance criteria
1. In CI, unprivileged, `setup_thread` succeeds and the thread's policy read back is time-constraint with the requested parameters.
2. `measure_wake_latency` returns median, 99.9th percentile and maximum over at least 10 000 wakes; values are reported in the self-test output.
3. The capability report lists each item as applied or not, with the OS error on failure.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
## Out of scope
Workgroup join (021-13). Wake-latency thresholds for certification (measured in 021-23).
