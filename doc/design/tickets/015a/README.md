# Epic 015a — Hosted runtime core: tickets

## Summary
Epic 015a is part a of epic 015 (split under C-143). It builds the hosted (operating-system) half of the Hadronyche runtime for Linux: crate `hadronyche-std` (process `Runtime`, housekeeping supervisor with the C-139 restart policy, per-module data thread, clock thread, housekeeping thread with regime stamp publication, pacing executor, single-writer file service, lock-free counter board) and crate `hadronyche-os-linux` (IPv4 sockets with `recvmmsg`, `SO_TIMESTAMPNS`, `SO_RXQ_OVFL`, `IP_RECVTOS`, don't-fragment and DSCP; interface statistics; `CLOCK_MONOTONIC` host clock with wall-clock step detection and a suspend-inclusive read). It drives the pure `no_std` state machines of `hadronyche-core` (epics 008 to 014), contains no protocol logic, and carries the duties C-144 assigns to the hosted backend: the gap slice and processing-overrun event on a missed own deadline, housekeeping restart, the single-writer file service and sending the regime stamp. Part b (015b) verifies this runtime, adds strict mode and the test-only hooks, and takes the strict-mode measurements; it depends on this part.

## Tickets
| Id | Title | Effort | Depends on |
|---|---|---|---|
| 015a-01 | Crate skeletons and trait surface | S | epic 008, epic 009, epic 010 |
| 015a-02 | recvmmsg control-message spike on target kernels | S | 015a-01 |
| 015a-03 | HostClock and wall-clock step detector | S | 015a-01 |
| 015a-04 | Linux socket setup (open_media, open_discovery) | S | 015a-01 |
| 015a-05 | Non-blocking send_to with counted drops | XS | 015a-04 |
| 015a-06 | Batch receive with control messages (recv_batch) | M | 015a-02, 015a-03, 015a-04 |
| 015a-07 | Interface statistics (ifstats.rs) | S | 015a-01 |
| 015a-08 | Single-writer file service (files.rs) | S | 015a-01 |
| 015a-09 | Pacing executor (pacing.rs) | S | 015a-01, 015a-05, epic 010 |
| 015a-10 | Clock thread (clock_thread.rs) | M | 015a-03, 015a-05, 015a-06, epic 008 |
| 015a-11 | Data thread receive side | M | 015a-06, 015a-10, epic 009, epic 010, epic 012 |
| 015a-12 | Data thread process callback and paced send | S | 015a-09, 015a-11 |
| 015a-13 | Data thread liveness and receiver reports | M | 015a-12, epic 014 |
| 015a-14 | Drop accounting and peer-verified DSCP in stamp and report | S | 015a-07, 015a-13, 015a-18, epic 013 |
| 015a-15 | Housekeeping thread (housekeeping.rs) | M | 015a-04, 015a-06, 015a-08, 015a-10, epic 011, epic 012 |
| 015a-16 | Housekeeping supervisor (supervisor.rs) | S | 015a-15 |
| 015a-17 | Runtime process assembly (process.rs) | M | 015a-08, 015a-10, 015a-13, 015a-14, 015a-16, 015a-18, 015a-19 |
| 015a-18 | Counter board and regime stamp publication (C-18) | M | 015a-15, epic 013 |
| 015a-19 | Per-connection stream-health counters in the stamp (C-47, C-159, C-160, C-157) | M | 015a-13, 015a-18 |
| 015a-20 | Own-deadline miss: gap slice and processing-overrun event (C-144) | S | 015a-12, 015a-18, epic 010 |
| 015a-21 | Holder stall detector (C-136) | S | 015a-03, 015a-10 |
| 015a-22 | Control-degraded refusal path (C-185) | S | 015a-15, 015a-16, epic 012 |
| 015a-23 | Holder sync-request rate cap (C-204) — new | S | 015a-10, 015a-18 |
| 015a-24 | Changed A carried upstream and fan-in re-alignment (C-198) — new | S | 015a-11, 015a-13, epic 009, epic 014 |

Renumbered from epic 015: 015-01 to 015-10 keep their numbers; 015-12 → 015a-11, 015-13 → 015a-12, 015-14 → 015a-13, 015-15 → 015a-14, 015-16 → 015a-15, 015-17 → 015a-16, 015-18 → 015a-17, 015-29 → 015a-18, 015-30 → 015a-19, new 015-33 → 015a-20, new 015-35 → 015a-21; 015a-22, 015a-23 and 015a-24 were added after the split.

Changes in the last refinement: no ticket added or deleted and no effort changed. 015a-24 applies C-214 (a sink switches to a changed A at its own position one lead ahead, announcing nothing), C-215 (a ratchet step is announced one lead ahead in the fan-in's output, never applied silently) and C-216 (a clamped A keeps the cannot-meet-A alarm raised until an A that fits; reports carry the given A, not the clamped value), with Work steps and Acceptance criteria; 015a-19 notes that the clamp alarm is held by C-216, not by the 60 s window. 015a-23 sizes its per-follower tracker to 63 followers (C-217), with an Acceptance criterion; 015a-10 and 015a-14 size their per-follower and per-peer DSCP tables to 63 (C-217). C-213 has no subject here (election is core's, epic 011).

Previous refinement: no ticket added or deleted and no effort changed. 015a-18 gains the commissioning window in the stamp (C-153, C-178: production latency, loss and alignment figures unqualified while a window is open), with a Work step and an Acceptance criterion. 015a-19 states the rolling 60 s window of the lateness figures and reclaimable latency, with the maximum since live beside them (C-182); its Acceptance criteria are corrected and one added. 015a-24 states that each fan-in picks and announces its own re-alignment position one lead ahead (C-208) and keeps streaming with the deepest ring it can hold and the cannot-meet-A alarm when a new A exceeds its buffer memory (C-211). 015a-11 and the epic count the controller member in the 32-member bundle cap (C-209, C-205).

Refinement before that: 015a-23 added (C-204: the holder drops sync requests beyond 200 per second from one follower before any work and counts them per follower in its stamp; no ticket covered it; 015a-10 and 015a-18 cross-reference it). 015a-24 added (C-198: the receiver report carries the consumer's current A; a fan-in that inherited the old A re-aligns, C-186; no ticket covered acting on it). 015a-13 corrected: its receiver reports carry the current A. 015a-11 notes that the inherited A arrives in receiver reports. 015a-17 corrected for C-201: an unverified pin on which seqlock safety depends counts as failed and refuses start. No effort changed and no ticket was deleted.

Earlier still: 015a-22 added (C-185: the housekeeping supervisor answers admission requests with ControlDegraded once restarts stop; 015a-16 had assumed no answer, and its Work and Acceptance criteria are corrected). 015a-17 corrected for C-177: a failed pin on which seqlock safety depends refuses start in every mode, not only in strict mode (015b-13). 015a-10 cites C-188 (sync requests answered on the clock thread) and C-187 (the lock state and bound in the time view come from the servo core). 015a-11 notes C-142 and NoSkewBudget for a fan-in outside any group. No effort changed and no ticket was deleted.

Before that: 015a-20 and 015a-21 added (C-144 and C-136 created work no ticket covered). 015a-19 raised from S to M (C-159 completes the counter set; C-160 percentiles and C-157 reclaimable latency added). The measurements formerly in 015a-03 moved to 015b-15 (C-147: recorded runs are in strict mode). No ticket was deleted: the withdrawn rational-L rule (C-105 revised) had produced no work in this epic, and no other ruling removed work.

## Effort tally
- XS: 1 (0.25 day)
- S: 15 (15 days)
- M: 8 (20 days)
- Total: 24 tickets, about 35 days — epic scope L (C-126)

## Coverage
| Epic item | Tickets |
|---|---|
| `process.rs` — Runtime (incl. C-177 refusal on a failed pin, C-201 unverified counts as failed) | 015a-17 |
| `supervisor.rs` (C-139 restart policy, C-173 alarm, C-185 ControlDegraded refusal path) | 015a-16, 015a-22 |
| `data_thread.rs` (incl. A in receiver reports and fan-in re-alignment, C-198, C-186, at a self-picked announced position, C-208; A beyond buffer memory, C-211, held alarm and given A reported, C-216; sink switch at its own position, C-214; ratchet step announced, C-215; controller member in the bundle cap, C-209) | 015a-11, 015a-12, 015a-13, 015a-14, 015a-19, 015a-20, 015a-24 |
| `clock_thread.rs` (incl. holder answers sync requests, C-188, and their DSCP, C-152; time view with lock state and bound from 008a, C-187; holder stall, C-136; per-follower sync-request cap, C-204; per-follower tables sized to 63 peers, C-217) | 015a-10, 015a-21, 015a-23 |
| `housekeeping.rs` (incl. regime stamp publication; commissioning window in the stamp, C-153, C-178) | 015a-15, 015a-18 |
| `pacing.rs` (C-155 pacing fit) | 015a-09 |
| `files.rs` | 015a-08 |
| `net.rs` | 015a-04, 015a-05, 015a-06 |
| `ifstats.rs` | 015a-07 |
| `clock.rs` (incl. suspend-inclusive read) | 015a-03, 015a-21 |
| Trait shapes and supporting types | 015a-01, 015a-21 |
| C-144 duties: gap slice and overrun event; restart; file service; stamp sending | 015a-20; 015a-16 (scenario test in 015b-04); 015a-08; 015a-18 |
| 015a-A1 — Drop accounting (AC-37) | 015a-06, 015a-14, 015a-18 |
| 015a-A2 — Peer-verified DSCP (AC-34, C-152; per-follower sync drops, C-204) | 015a-04, 015a-10, 015a-14, 015a-23 |
| 015a-A3 — Regime stamp transport (C-18; commissioning window, C-178) | 015a-18 |
| 015a-A4 — Stream health in the stamp (C-47, C-159, C-160, C-157, C-135; 60 s window, C-182) | 015a-13, 015a-19 |
| 015a-A5 — Own deadline missed (C-144) | 015a-12, 015a-20 |
| 015a-A6 — Holder stall (C-136) | 015a-21 |
| 015a-A7 — Housekeeping restart policy (C-139, C-185) | 015a-16, 015a-22 |
| DoD — Tests green in CI, unprivileged (per-change CI, C-146) | every ticket |
| DoD — Every parser touched is fuzzed (C-95) | 015a-06, 015a-07 |
| DoD — Regime stamp fields documented | 015a-07, 015a-14, 015a-18, 015a-19, 015a-20, 015a-23 |
| DoD — Design documents updated, constants table first | 015a-01, 015a-02, 015a-03, 015a-15; consolidated in 015b-11 |
| DoD — No new open item without owner and falsifier | every ticket |
| DoD — Frozen POC not modified | every ticket |
| DoD — Evidence tags | 015a-01 to 015a-04, 015a-06 to 015a-11, 015a-13 to 015a-15, 015a-18, 015a-21 |
| DoD — Trait surface compiles with a stub backend | 015a-01, 015a-17, 015a-21 |
| DoD — Per-packet receive cost recorded for task 019 | not applicable in this part (C-125): 015b-10 |
| DoD applicability (C-125) | every ticket lists its not-applicable lines with reasons |
| Open sub-question: recvmmsg per-message control messages | 015a-02 |
| Open sub-question: queue capacities | 015a-15 |
| Open sub-questions owned elsewhere (coalescing: 019; stamp fields: 004; producer resume: 005; metrics schema: 019; jitter and tolerance: 015b-15; kernel list: 001) | noted in 015a-02, 015a-03, 015a-05, 015a-08, 015a-13, 015a-16, 015a-18, 015a-20, 015a-21 |

## Remaining gaps
1. **Kernel support list.** The spike runs on the current and oldest obtainable kernels meanwhile (C-138). Affects 015a-02, 015a-06. Answer: task 001's report (epic 001).
2. **Value of `declared_jitter_ns()`.** Affects 015a-03, 015a-10. Answer: a measurement, 015b-15 in strict mode (C-147), recorded in the constants table, confirmed at L2.
3. **Tolerance of the wall-clock step detector.** Affects 015a-03, 015a-06. Answer: a measurement, 015b-15 in strict mode (realtime-minus-monotonic movement under NTP slewing).
4. **Stamp fields for host counters not named by C-47 or C-159**: discarded clock samples, sender-side queue drops, the control-degraded mark (stamp field or hello flag), the processing-overrun event, holder stalls, per-follower sync requests dropped above 200 per second (C-204 places them in the holder's stamp). Affects 015a-03, 015a-05, 015a-16, 015a-18, 015a-20, 015a-21, 015a-23. Answer: wire specification, epic 004 (implemented in 013).
5. **Capacities of the housekeeping queues.** Affects 015a-15. Answer: this epic (015a-15), recorded in the constants table.
6. **Producer resume after receiver reports return** (after the C-41 timeout), or only after re-admission. Affects 015a-13. Answer: control state machines, epic 005.
7. **Metrics record schema** (framing is settled: one record per line). Affects 015a-08. Answer: the L1 harness, epic 019, as its consumer.
8. **Unprivileged user namespaces on the CI host**, needed for the forced-MTU test of 015a-05. Answer: a check on the CI host.

## Settled since the split
Fourteen former gaps of epic 015 are closed; one more is narrowed. Settled for this part:
- Where a sink switches to a changed A — C-214: at its own position, one lead (C-43) ahead, as a fan-in does (C-208); it announces nothing. A ratchet step — C-215: a re-alignment announced one lead ahead in the fan-in's own output, never silent on the next round. A clamped A in detail — C-216: the cannot-meet-A alarm stays raised until a later change of A that fits, and the receiver report carries the given A, not the clamped value. All three applied in 015a-24 (which assumed the sink's switch and left the clamp alarm's release and the reported A unstated); 015a-19 noted.
- Peer map capacity — C-217: 63 peers (64 modules, C-34, less the module itself). The per-follower tables of 015a-10 and 015a-23 and the per-peer hello DSCP of 015a-14, preallocated with no stated size, are sized to 63.
- Commissioning window in production stamps — C-153 and C-178: modules stamp an open window; while it is open, production latency, loss and alignment figures are unqualified. Applied in 015a-18 (window state and fields are core's, epic 013).
- Window of the stamp's lateness figures — C-182: rolling 60 s for percentiles, reclaimable latency and cannot-meet-A, maximum since live beside them. 015a-19 corrected.
- Re-alignment position after a changed A — C-208: each fan-in picks its own, one lead (C-43) ahead, and announces it in its own output. A larger than a fan-in's buffer memory — C-211: deepest ring as ceiling, cannot-meet-A alarm, streams keep running; the pre-check is commissioning's (028). Both applied in 015a-24.
- Controller member and the bundle cap — C-209 (with C-205 and revised C-08): the controller member is one of the 32 members. Stated in the epic and 015a-11; the drain bound and batch size (64 for 32 members) are unchanged.
- Sync requests above the accepted rate — C-204: requests beyond 200 per second from one follower (C-112) are dropped before any work and counted per follower in the holder's stamp. New ticket 015a-23; its wire field joins gap 4.
- How a changed A reaches a fan-in that inherited it (C-142) — C-198: the receiver report carries the consumer's current A, one hop per report; each fan-in that inherited the old value re-aligns (C-186). 015a-13 corrected; 015a-11 noted; new ticket 015a-24.
- An unverified pin — C-201: treated as failed for C-177; the runtime refuses to start in every mode. 015a-17 corrected.
- Holder sync-request answering (provisionally placed on the clock thread from L-8 and C-152) — C-188: on the clock thread, never on housekeeping, which is where the DSCP of each follower's requests is recorded. Applied in 015a-10, 015a-14.
- Refusal while control-degraded (015a-16 had assumed an admission request gets no answer) — C-185: the housekeeping supervisor keeps a minimal refusal path answering ControlDegraded. 015a-16 corrected; new ticket 015a-22. C-185's NoSkewBudget (C-142) noted for socket sizing in 015a-11; its discovery-time reasons are core's.
- Failed CPU pin (015a-17 had left all enforcement on the hook result to strict mode, 015b-13) — C-177: if seqlock safety depends on a pin that fails, the runtime refuses to start in every mode; default priorities never depend on pinning. 015a-17 corrected.
- Publisher of the clock lock state and bound — C-187: the servo core (epic 008a), in the published time view; the clock thread passes it through (015a-10). This epic reports no one-way figures, so no work is added.
- Former 1, task 016 missing from the dependency list — epic change: 016 added; called through a no-op-default hook.
- Former 3, L-1 cited but undefined — epic change: L-1 defined as applied here.
- Former 4, skew budget for socket buffers — C-96: buffer = declared rate × A.
- Former 6, undefined trait types — epic change: supporting types defined; `StampSource` has the four C-131 values.
- Former 10, sync and control TOS into a per-bundle report — C-47 and C-152: each observer publishes observed DSCP in its own stamp; the holder reports DSCP on each follower's sync requests, every module the DSCP on each peer's hellos.
- Former 11, how flowing sync replies suppress an election — C-37 and C-31.
- Former 15, interface statistics period and counters — C-18 plus epic change.
- Former 16, how the path MTU is found — C-116, C-06, C-100.
- Former 20, `recvmmsg` batch size — C-16 plus epic change: batch size equals the drain bound; the drain cycle is one slice period (C-156).
- Housekeeping restart policy (formerly gap 6) — C-139: backoff 100 ms to 10 s; after 5 panics within 60 s restarts stop, the module stays control-degraded with an alarm, live streams continue, no new connection until operator restart. Implemented in 015a-16.
- Kernel support list (formerly gap 1, design owner) — narrowed by C-138: set after task 001 reports; probes cover current and oldest obtainable releases. Now gap 1, owned by epic 001.
- New rules applied without a former gap: C-135 (other-epoch slice restarts the dead clock, own counter), C-136 (holder stall, 015a-21), C-137 (header cap), C-144 (duties of this epic, 015a-20), C-155 (pacing fit), C-157, C-159, C-160 (stamp counters and figures, 015a-19), C-165 (pacing override is operator policy), C-168 (sink-only liveness), C-171 (architecture document location), C-173 (control-degraded alarm).
- Settled for part b (see 015b's index): panic-injection hook (C-145), privileged real step (C-146), strict-mode measurements (C-147), control isolation (C-161), strict mode in commissioning configuration (C-165).
- Not applicable to this epic: C-104 and C-105 as revised (L a whole number of samples; rational-L withdrawn) and C-134 as revised, C-189, C-190, C-207 (resamplers, D and L) — no rational-L handling, rounding of L or fractional delay line existed here, so no ticket was deleted. C-35's owner change concerns the L1 follower-cap test (019), not this epic. Also without a subject here: C-213 (who may take over as holder, with the epoch kept or bumped per C-113: election state machine, epic 011; this epic only feeds it the last-reply time), C-179 (loss and redundancy, 019), C-180 (certification), C-181 (controllers in the packet, compiler and packetiser), C-183 (compiler), C-141, C-184, C-199, C-200 (transport state, module side), C-169 and C-195 (holdover, servo epics), C-191, C-194, C-202, C-203, C-212 (ladder tests), C-192, C-193 (vectors, L0), C-196 (specification epics only), C-197 and C-210 (kit epic 024a), C-206 (no tool built here; the metrics record schema stays with its consumer, 019).
