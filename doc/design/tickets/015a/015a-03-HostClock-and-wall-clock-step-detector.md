# 015a-03 — HostClock and wall-clock step detector
**Epic:** 015a-Hosted-runtime-core · **Effort:** S · **Depends on:** 015a-01
## Context
- Host clock H is monotonic nanoseconds. On Linux it is `CLOCK_MONOTONIC` (`hadronyche-os-linux/src/clock.rs`).
- `SO_TIMESTAMPNS` delivers `CLOCK_REALTIME` stamps `[kernel-doc]`. The POC derived receive time as `real_ns() − kernel_stamp` (`src/module.rs:1557`, `:1671`) `[code]`, so a wall-clock step between stamp and read corrupted the sample, and `.max(0)` hid it as zero delay.
- Rule `[decision]`: kernel receive timestamps are used with wall-clock step detection: compare the realtime-minus-monotonic difference before and after; if it moved, discard the sample for clock purposes (`RxMeta::rx_host_ns = None`).
- Home of the detection (C-121): the platform backend that builds each exchange from receive stamps — that is, this crate. The servo only ever receives exchanges that passed it; the servo (epic 008) has no step logic of its own.
- Two values remain open as measurements (epic open sub-questions): the value of `declared_jitter_ns()`, and the detector tolerance (how far realtime-minus-monotonic may move without counting as a step; NTP slewing moves it slowly without a step). Any run whose numbers enter the constants table runs in strict mode (C-147), so the measurements themselves are ticket 015b-15, which runs after strict mode exists; this ticket makes both values configurable and builds the measurement probes.
- Where the discarded-sample count is reported in the stamp is not decided (wire specification, epic 004). This ticket exposes the counter only.
- Trait: `pub trait HostClock: Send + Sync { fn now_ns(&self) -> i64; fn declared_jitter_ns(&self) -> u32; }`
## Work
1. Implement `LinuxHostClock: HostClock` with `now_ns()` reading `CLOCK_MONOTONIC`.
2. Implement `declared_jitter_ns()` as a configurable value. Provide the read-jitter probe (10^6 back-to-back reads, p99.9 and maximum) that 015b-15 runs in strict mode.
3. Implement a step detector: `fn rt_minus_mono_ns() -> i64` sampled before and after a receive batch; a helper that converts a realtime kernel stamp to host time using the "before" difference and returns `None` if the "after" difference differs by more than the tolerance.
4. Keep the tolerance configurable, and provide the probe that records how far realtime-minus-monotonic moves across one receive batch while NTP is slewing (run in strict mode by 015b-15).
5. Count discarded samples (a monotonically increasing counter exposed to callers).
6. Unit tests with an injectable realtime source to simulate a ±1 s realtime step and a slow slew.
## Acceptance criteria
1. `now_ns()` is monotonic over 10^6 consecutive calls (never decreases).
2. With a simulated `CLOCK_REALTIME` step of +1 s and −1 s between "before" and "after", the conversion returns `None` and the discard counter increases by 1 per affected sample.
3. With a simulated slew smaller than the tolerance, no sample is discarded; with a simulated step of 1 ms, the sample is discarded.
4. Without a step, a converted stamp lies within [before-read, after-read] on the host clock.
5. No negative delay is clamped to zero anywhere (no `.max(0)` on derived delays).
6. Both probes run (outside strict mode) in CI as smoke tests and print p99.9, maximum and realtime-minus-monotonic movement; the recorded measurements are 015b-15's.
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Regime stamp fields touched" — the discard counter's stamp field is not yet specified (epic 004); "Trait surface compiles with a stub backend" — implements an existing trait, no change; "Per-packet receive cost measured" — no receive path.
## Out of scope
- Using the conversion inside `recv_batch` (015a-06).
- The end-to-end ±1 s step test against the servo (015b-06).
- The stamp field for discarded samples (epic 004).
- The strict-mode measurement of jitter and tolerance (015b-15).
