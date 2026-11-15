# 015b — Hosted runtime verification and strict mode

**Phase:** P4 · **Scope:** M (15 tickets, about 14 days) · **Depends on:** 015a, 016 · **Blocks:** 017, 019

## Goal

Prove the Linux hosted runtime built in 015a against its isolation and robustness criteria, add strict mode with the capability states and the test-only hooks, take the measurements whose numbers enter the constants table in strict mode, and update the design documents for both parts.

This is part b of epic 015, split under C-143. It depends on part a (015a), which delivers the crates `hadronyche-std` and `hadronyche-os-linux`, the thread model, the sockets, the counter board and the regime stamp publication. A reference elsewhere to "epic 015" means both parts.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules exchange *slices* — all content of one stream for one stretch of media time — addressed by *position* on a domain-wide timeline at the domain rate of 96 000 Hz (C-01). Linux hosts are the backbone (clock holder, elections, conformance, commissioning). The runtime owns *when and whether* content moves; module code owns *what* it is.

**What 015a provides.** A `Runtime` (`process.rs`) with one housekeeping thread and one clock thread per process and one data thread per module; supervised housekeeping (C-139 restart policy); IPv4 sockets with `recvmmsg`, `SO_TIMESTAMPNS`, `SO_RXQ_OVFL`, `IP_RECVTOS`, don't-fragment and DSCP; a `CLOCK_MONOTONIC` host clock with a wall-clock step detector and an injectable realtime source; a pacing executor; a single-writer file service; a lock-free counter board read by housekeeping when it builds the regime stamp; per-connection health counters; the task 016 real-time hook, whose per-thread result `Runtime` keeps; per-socket counts of the receive-timestamp source used. The trait surface compiles with a stub backend.

**Thread rules being verified `[decision]`.**

- **L-1:** liveness — interrupted, dead, receiver reports and the producer timeout — is executed on the data threads and never passes through housekeeping.
- **L-4:** housekeeping admits unknown sources only; it never touches a live data path.
- **L-5:** destinations are pinned at admission and change only by re-admission. A hello carrying a new address never redirects a live stream.
- **L-7:** a panic in a data or clock thread terminates the process (fail-stop). A panic in housekeeping is restarted, live streams continue, and the module is marked control-degraded. Restart policy (C-139): exponential backoff from 100 ms to 10 s; after 5 panics within 60 s restarts stop, the module stays control-degraded with an alarm, its live streams continue, and it accepts no new connection: the housekeeping supervisor keeps a minimal refusal path that answers with ControlDegraded (C-185). The housekeeping-restart criterion AC-20 (c) belongs to the hosted backend (C-144).
- **L-8:** no lock is shared between housekeeping and the data or clock threads; they communicate by messages or lock-free publication only. The crossing paths are: housekeeping → data/clock queues, the clock seqlock, the clock thread's last-reply time, the counter board, and the file service queues.

**Figures used by the tests.** Bounded drain (C-16): per input per slice period (C-156), at most 2 × the admitted packets per slice. Socket buffer: declared rate × A, where A is the presentation group's audition offset (C-55, C-96). Interrupted after 8 consecutive slices absent at their deadline (C-38, C-102), 4 ms at 0.5 ms slices; dead 2 s after the last slice with content, a marked gap or another timeline epoch (C-39, C-111, C-135); receiver report every 100 ms per bundle (C-40); producer stops after 2 s without reports (C-41). Elections start only on 1.5 s of sync-reply silence (C-37); hello expiry (3 s) applies to non-live peers only (C-31). Sync: 50 requests/s by default (C-36), accepted range 10 to 200 per second, AC-7 checked over this range (C-112); the holder answers sync requests on its clock thread, never on housekeeping (C-188), and drops requests beyond 200 per second from one follower before any work, counting them per follower in its stamp (C-204). Fallback path MTU 1 280 bytes, 1 252 bytes of UDP payload (C-06, C-100); IPv4 only, TTL 1. Two-bundle load (C-92): 32 packets per 0.5 ms slice, 64 000 packets/s; a default packet carries 1 152 bytes of audio (C-04) and is about 1 320 bytes on the wire (C-20). Receive cost guessed at 2 to 4 µs per packet, 13 to 26 % of the slice period for two bundles (C-93) `[memory]`.

**POC defects the tests must not reproduce `[code]`.** `src/module.rs:1518`: the clock thread took a mutex shared with housekeeping every iteration. `:1669`: every input was drained completely before the deadline check. `:1126`: sync paced by a 5 ms receive timeout gave 36.1 exchanges/s at `--sync-hz 50` `[measured]` on `CONFIG_HZ=250` `[measured]` (P-02, P-03). `:1557`, `:1671`: receive time derived as `real_ns() − kernel_stamp`, with `.max(0)` hiding wall-clock steps; `SO_TIMESTAMPNS` stamps are `CLOCK_REALTIME` `[kernel-doc]`. `:1639`: `routes = r.clone()` allocated on the data thread. `:1354`: four modules called `File::create` on one metrics path and corrupted it `[measured]`.

**Strict mode and capability states.** Strict mode requires real-time scheduling granted on the data and clock threads and memory locked; restricting CPU idle states stays best-effort and is stamped (C-127). In strict mode an unverified capability is treated as absent, so the runtime refuses to start (C-128). Every capability is stamped in one of four states: present, absent, unverified, not requested — "not requested" marks a deliberate configuration choice, distinct from a denial (C-129). The receive-timestamp source is one of kernel, userspace, mac-hardware or interrupt, each with a C-129 state (C-131); on Linux only kernel and userspace occur. The control-isolation capability (housekeeping cannot delay the data path) is present on hosted platforms by construction of the thread model (C-161). Strict mode is operator policy set in the commissioning configuration (C-165), and it is hosted-only: a device stamps real-time scheduling and memory lock as present by construction (C-147). If a thread layout's seqlock safety depends on a CPU pin and that pin fails, or is unverified, the module refuses to start in every mode, strict or not; default priorities (clock above data) never depend on pinning (C-177, C-201). Task 016 performs the real-time setup; this part maps its result to C-129 states, enforces C-128, C-177 and C-201 and publishes the states.

**Measurements run in strict mode (C-147).** Any run whose numbers enter the constants table runs in strict mode; L0 does not. The recorded runs of this epic — `declared_jitter_ns()` and the step-detector tolerance (015b-15), the achieved sync rates (015b-01) and the per-packet receive cost (015b-10) — therefore run in strict mode on the development machine with task 016 bound. Per-change CI tests run outside strict mode.

**Test-only injection hooks (C-120, C-145).** Beyond the public injection API, tests may force a module id and nonce, force a stream onto a given timeline epoch, send a raw packet on a media or control socket (C-120), and inject a panic into a data, clock or housekeeping thread (C-145). Each use is stamped like any injection. The flood and changed-address tests use the raw-packet hook so that injected packets come from the pinned source; the restart and fail-stop tests use the panic hook.

**Privileged tests (C-146).** The few tests that need a privilege — here the real `CLOCK_REALTIME` step, which needs `CAP_SYS_TIME` and is not virtualised by time namespaces `[kernel-doc]` — run in a separate privileged nightly job. The definition-of-done line "unprivileged" applies to per-change CI. The don't-fragment capture runs unprivileged inside a user and network namespace.

**Where outputs live.** Reports and results go to `doc/design/reports/`, values to the constants table `doc/design/02-constants.md`, evidence to `doc/design/06-evidence-appendix.md` (C-130); the architecture specification is `doc/design/01-architecture.md` (C-171). Harness and tool binaries (scripted peer, DF checker) go in the `hadronyche-tools` crate (C-133). Nothing is written into the frozen POC tree. Report file names within `doc/design/reports/` and the output formats of the scripted peer and the DF checker are decided in this epic and recorded in their documentation (C-206).

## Deliverables

In crate `hadronyche-std`:

- Strict mode and capability states: mapping of the task 016 hook result to C-129 states; strict-mode refusal with a named reason (C-127, C-128); refusal in every mode when seqlock safety depends on a failed or unverified CPU pin (C-177, C-201); receive-timestamp source states (C-131); control isolation stamped present (C-161); strict mode read from the commissioning configuration (C-165).
- Test-only hooks behind a test feature: forced id and nonce, forced timeline epoch, raw packet send (C-120), panic injection (C-145); each use recorded as an injection on the counter board.
- Asserting allocator, exported for the L0 harness (017).
- Static check that no lock type is shared across thread classes, run in CI.

In crate `hadronyche-tools`:

- Scripted peer for the housekeeping isolation tests; DF capture checker.

Tests and scenarios: flood isolation, housekeeping isolation (AC-20 a–d, with the C-139 restart stop and the C-185 ControlDegraded refusal), fail-stop, wall-clock step (unprivileged and privileged nightly variants), don't-fragment capture, one writer per file, no allocation after go-live, achieved sync rate and the holder's drop of requests above 200 per second (C-204).

Reports in `doc/design/reports/` (strict mode): host clock jitter and step tolerance, achieved sync rates, per-packet receive cost. Updates to the constants table and the design documents for both parts.

## Dependencies

- **Needs:** the running runtime, sockets, clock, counter board and stamp publication of 015a; real-time thread setup (016) for strict mode and the strict-mode measurement runs.
- **Provides:** the asserting allocator to the L0 harness (017); the per-packet receive cost report to the L1 harness (019); measured values to the constants table.

## Acceptance criteria

1. **015b-A1 · Flood isolation (AC-11).** One input flooded at 10 × its admitted rate from its pinned source (raw-packet hook, C-120): zero late slices on other inputs, no missed deadline on the module; the excess appears in the socket overflow counter.
2. **015b-A2 · Housekeeping cannot damage a live stream (AC-20).** (a) hellos suppressed for 30 s: live streams continue. (b) holder hello suppressed while sync replies flow: no election (elections start only on 1.5 s of reply silence, C-37). (c) housekeeping thread killed with the panic hook (C-145): restarted after the 100 ms backoff (C-139), streams unaffected, module marked control-degraded; on a holder, sync replies continue through the restart (C-188); after 5 panics within 60 s restarts stop, live streams continue and a new connection request is refused with ControlDegraded (C-139, C-185). (d) a hello with a changed address: no redirect. (e) a static check or lint shows no lock type shared between housekeeping and data or clock threads.
3. **015b-A3 · Fail-stop (AC-21).** Killing a data or clock thread (panic hook, C-145) terminates the process; peers report interrupted after 8 consecutive slices absent at their deadline (4 ms, C-38, C-102) and dead 2 s after the last slice with content, a marked gap or another timeline epoch (C-111, C-135); the producer stops after 2 s without reports (C-41).
4. **015b-A4 · Achieved sync rate (AC-7).** Within 2 % of the configured rate over the whole accepted range of 10 to 200 requests per second (C-112), and reported; the recorded run is in strict mode (C-147). The POC value (36.1/s at 50, P-02) must not reproduce. A follower requesting above 200 per second has the excess dropped by the holder and counted per follower in the holder's stamp (C-204).
5. **015b-A5 · One writer per file (AC-31).** Four modules in one process writing metrics produce one parseable file; no line fusion.
6. **015b-A6 · No allocation on the data thread** after go-live, checked with an asserting allocator in a loopback scenario of this epic; the L0 harness (017) reuses the same allocator.
7. **015b-A7 · Wall-clock step.** Stepping `CLOCK_REALTIME` by ±1 s during a run changes no servo estimate; discarded samples are counted. The injected-source variant runs in per-change CI; the real step runs in the privileged nightly job (C-146).
8. **015b-A8 · Don't-fragment.** Every packet leaves with don't-fragment set (verified by capture).
9. **015b-A9 · Strict mode (C-127 to C-129, C-161, C-165, C-177, C-201).** In strict mode, a process whose data or clock threads lack real-time scheduling, or whose memory is not locked, or for which either capability is unverified, refuses to start with a named reason; outside strict mode it starts and stamps each capability as present, absent, unverified or not requested; the CPU idle restriction is stamped in every mode; control isolation is stamped present; strict mode is set from the commissioning configuration; a layout whose seqlock safety depends on a CPU pin refuses to start in every mode when that pin is failed or unverified, and the default layout starts without any pin.
10. **015b-A10 · Measurements in strict mode (C-147).** The reports for `declared_jitter_ns()`, the step-detector tolerance, the achieved sync rates and the per-packet receive cost each show a strict-mode run.
11. **015b-A11 · Test-only hooks (C-120, C-145).** Each hook works as specified, each use appears in the stamp as an injection, and a release build without the test feature has no hook symbol.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The trait surface compiles with a stub backend (proves macOS and Windows can implement it)
- [ ] Per-packet receive cost measured on the development machine and recorded for task 019

**Not applicable in this part (C-125):** "Every parser touched is fuzzed" — this part adds no parser (the control-message and sysfs parsers are 015a's; the DF checker reads pcap through an existing library). "The trait surface compiles with a stub backend" — this part changes no trait; the check is 015a-01, 015a-17 and 015a-21. Every other line has a subject: the stamp fields are the injection and capability-state fields; "Tests green in CI, unprivileged" applies to per-change CI (C-146), with the real realtime step of 015b-A7 in the privileged nightly job and the capture of 015b-A8 unprivileged inside a user and network namespace.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| Receive cost per packet: guessed 2 to 4 µs, 13 to 26 % of the slice period for two bundles (C-93) `[memory]`; this part records the development-machine figure | Task 019 (L1) | Two 96-channel bundles miss deadlines on one GbE port (C-92) |
| Value of `declared_jitter_ns()` and the step-detector tolerance | Measurement in this part (015b-15), in strict mode (C-147), recorded in the constants table; the jitter is confirmed at L2 | Measured read jitter exceeds the declared value; a slewed but unstepped clock discards samples, or a 1 ms step passes undetected |
| Does the CI host allow unprivileged user namespaces (needed for the capture of 015b-A8)? | A check on the CI host | The namespace cannot be created and the capture job cannot run unprivileged |
