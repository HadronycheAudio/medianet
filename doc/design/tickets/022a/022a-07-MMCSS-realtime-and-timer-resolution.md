# 022a-07 — `WinRealtime`: MMCSS registration, timer resolution and wake-latency self-test
**Epic:** 022a-Windows-backend-and-WASAPI-edge · **Effort:** S · **Depends on:** 022a-02, epic 016

## Context
- Part of epic 022a (Windows edge backend), `rt.rs`. Deliverable: `impl OsRealtime for WinRealtime { /* setup_process, setup_thread, measure_wake_latency */ }` (trait from epic 016), with capability report *bounded real-time: absent*.
- Real-time threads on Windows `[memory]`: the MMCSS "Pro Audio" task raises priority; it is **not** hard real-time. A granted MMCSS registration is what epic 022 counts as "real-time scheduling granted" for strict mode (C-127); strict mode itself is 022a-20.
- Bounded real-time is report-only on Windows: deadline overruns are diagnostics only. Any bound on scheduling latency is uncertifiable in v1. Third-party driver interrupt work produces millisecond scheduling spikes user space cannot bound `[memory]`.
- Timer granularity `[memory]`: default 15.6 ms; best waitable timer about 0.5 ms. Sync requests at 50/s (20 ms period, C-36) are fine with a 1 ms timer; the holder accepts 10 to 200 per second (C-112). No spinning ticker on Windows in v1 `[decision]`.
- Ownership is settled in the epic: `rt.rs` owns the timer-resolution request and MMCSS; `power.rs` (022a-09) owns stay-awake and the power-throttling opt-out.
- Thread priorities and pinning (C-177, C-201): the default priorities, clock thread above data thread, never depend on pinning. If the thread layout (epic 016) makes seqlock safety depend on a CPU pin and that pin fails, or cannot be verified (an unverified pin counts as failed), the module refuses to start in every mode, strict or not. This is the one real-time failure that is fatal outside strict mode.
- The self-test is settled in the epic: it is `measure_wake_latency`, run at startup on the data and clock threads after MMCSS registration; it reports 50th, 99th, 99.9th percentiles and maximum as a stamp diagnostic, with no pass threshold.

## Work
1. `setup_process`: request the 1 ms system timer resolution needed for 20 ms sync requests (C-36).
2. `setup_thread`: register the calling thread with MMCSS task "Pro Audio"; keep the handle and unregister on drop. Failure is reported, not fatal (fatal only in strict mode, 022a-20). Report MMCSS registration as a C-129 state per thread. Set the clock thread's priority above the data thread's without relying on any pin (C-177).
3. Pin check (C-177, C-201): when the thread layout makes seqlock safety depend on a CPU pin, apply the pin (thread affinity), read it back, and refuse startup with a named reason in every mode if it failed or cannot be verified.
4. `measure_wake_latency(period, n)`: sleep on a high-resolution waitable timer `n` times, record wake latency against `QpcClock` (022a-02), return percentiles (50th, 99th, 99.9th, max).
5. Startup self-test: run `measure_wake_latency` on the data and clock threads after registration and expose the result for the stamp.
6. Capability report: `bounded_realtime: Absent` always; MMCSS registration state; wake-latency summary.
7. Tests: registration round-trip; pin check with mocked affinity results (applied, failed, unverified) with strict mode both on and off; `measure_wake_latency` returns plausible values on CI (no threshold asserted).

## Acceptance criteria
1. The capability report always says *bounded real-time: absent* (unit test).
2. After `setup_thread`, the thread is registered to MMCSS "Pro Audio" (check via returned task index / handle), and is unregistered on drop; the registration state is *present* or *absent*.
3. `measure_wake_latency(1 ms, 1000)` returns 50th/99th/99.9th/max in ns; on a reference Windows laptop the result is recorded `[measured]`; the startup self-test result is exposed for the stamp.
4. With `setup_process` done, a 20 ms periodic sync loop shows a period error below 2 ms at the 99th percentile on reference hardware.
5. **(C-177, C-201.)** With a layout whose seqlock safety depends on a pin, a failed or unverified pin refuses startup with a named reason whether strict mode is on or off (mocked); the clock thread's priority is above the data thread's with no pin applied.
6. No API in this module busy-waits (code review + test that CPU of an idle 20 ms loop stays below 1 % of a core).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The MMCSS and timer-granularity facts are confirmed `[measured]` or corrected, for 022b-08

## Out of scope
- Stay-awake, power-throttling opt-out and suspend handling (022a-09).
- Memory lock and strict mode (022a-20).
- The idle-sink CPU acceptance check (022a-18).
- The housekeeping restart policy (C-139) and the restart criterion AC-20 (c), owned by the hosted backend, epic 015 (C-144); the panic-injection hook used to test it (C-145) is core.
