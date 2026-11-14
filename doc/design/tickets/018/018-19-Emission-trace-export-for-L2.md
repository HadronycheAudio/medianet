# 018-19 — Emission trace export for L2 verification
**Epic:** 018-Linux-ALSA-edge · **Effort:** S · **Depends on:** 018-14

## Context
- Epic criterion 018-A6 is a **contribution to AC-26, checked at L2 by epic 020**: E ≤ 50 µs (99.9th percentile over one hour, C-70, C-73); restart spread of D ≤ 25 µs over 20 restarts (C-74); round-trip marker offset constant within ±2 samples (C-75); zero slip (offset returns within ±1 sample) after injected loss, a clock handover with 50 ppm crystal difference (C-78), and an announced step (C-76); **MTIE ≤ 10 µs for intervals up to 1 s and ≤ E for any interval** (C-77); E held throughout a drift ramp of ±100 ppm at no more than 0.5 ppm/s (C-119).
- The epic assigns the verification tooling (E statistics, MTIE, D spread) to this epic; epic 020 runs it on hardware and judges pass/fail.
- Fault injection uses the public injection API plus the test-only hooks of C-120 (force a stream onto a given timeline epoch; send a raw packet on a media or control socket), stamped like any injection.
- Fuzz budget for the trace reader: 60 s on every change and 1 hour nightly, corpus kept (C-95).
- Pass criteria mean the target E, 50 µs (C-149), never the E measured in the same run. L2 runs are in strict mode (C-147); A for L2 runs is computed by the harness of epic 020 with the commissioning rule and recorded with the results (C-148).
- L2 test parameters (C-166, proposals pending first runs): no-step tolerance at handover ±1 sample on Linux; comparison windows of 10 s; injected loss 1 % random plus bursts of 8 slices; measurement window after a restart 60 s; restart spread of D is max − min; sample tolerances are measured against the median of the first 60 s; a one-sample slip passes on Linux and is reported as a slip event.
- L2 test parameters completed (C-191, proposals pending first runs): loss bursts are spaced 1 s apart and hit the media path only; the announced step is 5 ms; a 10 s comparison window starts when the injected event ends; the handover is triggered 60 s into the run. Sync loss has no separate scenario (C-202).
- **L2 handover (C-212, C-202):** triggered as a holder loss: the old holder stops answering, the sink passes through about 2 s of holdover (C-37 plus C-33) and the new holder takes over. The run asserts emission error within the target E during holdover and frequency continuity at takeover (rate change within C-61, no phase step); a run in which holdover was not observed is failed, not passed. The rig therefore needs a second holder-eligible Linux node. Per-sink emission error in holdover is an L2 assertion only; the L3 rig cannot observe it (C-220).
- **C-223:** the 50 ppm frequency difference of the handover test is the measured natural difference between the two holders' crystals plus an injected drift, totalling 50 ppm; both parts are recorded in the run report.
- **C-206:** the trace format, file names within the stated directories and the tool's output lines are decided by this epic and recorded in the tool's documentation; only what epic 020 or an operator's script depends on goes into a specification.
- The offline tool lives in the `hadronyche-tools` crate (C-133); results go in `doc/design/reports/` (C-130).
- Open sub-questions owned by epic 020: is E = 50 µs reachable (falsifier: tracking error plus D uncertainty exceed 30 µs); is D repeatable (falsifier: spread above 25 µs over 20 restarts); rate-loop bandwidth value (C-68).

## Work
1. Optional per-period trace from `AlsaSink`: (position, submit_host_ns, emission_error_ns, ratio, clock bound, clock state (locked, holdover, unlocked), residual), written from a non-RT thread via a lock-free ring.
2. Per-start record of D and its components (from 018-11).
3. Offline tool in `hadronyche-tools` (C-133) computing p99.9/max of E, MTIE for windows from 0.5 ms to 1 h, and D restart spread (max − min) from these traces; a mode that reports E over a drift-ramp segment (C-119); offsets measured against the median of the first 60 s, comparison windows of 10 s starting when the injected event ends (C-166, C-191), and one-sample slips reported as slip events (C-166); a handover mode that checks emission error within the target E over the holdover segment and frequency continuity at takeover (rate change within C-61, no phase step), and reports the run as failed when no holdover appears in the trace (C-202, C-212).
4. Scenario scripts for epic 020 built on the public injection API and the C-120 hooks: drop slices with the C-166 pattern (1 % random plus bursts of 8 slices spaced 1 s apart, media path only, C-191), force a stream onto a new timeline epoch (announced step of 5 ms, C-191), and a holder-loss handover 60 s into the run in which the old holder stops answering (C-191, C-212); the handover script refuses to start unless a second holder-eligible Linux node is present on the rig (C-212); it takes the measured natural difference between the two holders' crystals and injects the drift that makes the total 50 ppm, and writes both parts to the run report (C-223); each injection stamped. Scripts check that strict mode is on before a run whose figures are recorded (C-147).
5. Trace-reader fuzz target at the C-95 budget.
6. Document the trace format, file names and tool output lines in the tool's documentation (C-206).

## Acceptance criteria
1. A one-hour simulated run (fake backend) produces a trace whose MTIE computed by the tool matches a reference implementation on a synthetic sequence exactly.
2. Tracing enabled adds no xrun in a 10 min fake-backend run and allocates nothing on the RT path.
3. The D-spread tool reports max − min over 20 synthetic start records correctly (C-166).
6. On a synthetic offset sequence with one one-sample step, the tool reports one slip event against the median of the first 60 s (C-166).
4. Every scenario injection appears in the regime stamp as an injection (C-120).
5. The trace-reader fuzz target runs 60 s per change and 1 hour nightly, corpus committed (C-95).
7. On a synthetic handover trace, the handover mode passes when emission error stays within 50 µs (C-149) over the holdover segment, fails when it exceeds it, and fails a trace with no holdover segment (C-202, C-212); it also fails a trace with a phase step or a rate change above C-61 at takeover.
8. The handover script refuses to start, with a named reason, when the rig has no second holder-eligible Linux node (C-212).
9. Given a measured natural difference (for example 12 ppm), the handover script injects the remainder (38 ppm) so the total is 50 ppm, and the run report records both figures (C-223).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed (trace reader)

## Out of scope
- Running the L2 certification and judging pass/fail (epic 020).
- Providing the rig's second holder-eligible Linux node (epic 020, C-212).
- Implementing the C-120 hooks themselves (hosted runtime).
