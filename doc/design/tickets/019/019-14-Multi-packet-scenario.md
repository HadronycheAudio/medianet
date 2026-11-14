# 019-14 — Multi-packet scenario
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-07

## Context
A slice is one packet by default (C-05): 48 frames = 0.5 ms at 96 kHz (C-01, C-02), 4 bytes per sample (C-03), 6 channels per stream (C-04): 192 bytes per channel per slice, 1 152 bytes of audio, no per-channel id byte. About 1 320 bytes on the wire (C-20). When a slice is larger than the path MTU it is carried in several packets; this multi-packet regime is explicit and stamped (C-05). IP fragmentation is forbidden; every packet has don't-fragment set, so the runtime splits slices itself. IPv4 only (C-100).

Reassembly deadline: for an aligned multi-packet input it is the consumer's own deadline for that position (C-110); 2 slice periods applies only to arrival-driven inputs (C-15). A slice not complete at the deadline is counted missing, and reclassified as late if it completes within the ring span (C-108).

Scenario `multi-packet`: "a slice forced larger than the path MTU; records first-packet and slice-complete arrival." The epic settles the method: a stream wider than the default 6 channels (C-04 gives 6 only "at defaults"). For example 24 channels give 48 × 24 × 4 = 4 608 bytes of audio per slice, which needs several packets at a 1 500-byte MTU. The number of packets per slice actually used is read from the stamp.

AC-24 (part): **in the multi-packet regime, first-packet and slice-complete latency differ by the serialisation time** — about 12 µs per 1 500-byte packet at 1 Gbit/s (1 500 × 8 bit / 1 Gbit/s = 12 µs).

The tolerance of that comparison is ±5 µs (C-167). Whether the runtime accepts a wider stream is an epic 015 matter; if it does not, the ticket falls back to lowering the interface MTU and says so.

Fuzzing is settled by C-95: each parser 60 s on every change and 1 hour nightly; the corpus is kept.

## Work
- Scenario `multi-packet`: one stream each way, aligned consumer, with a channel count giving N packets per slice; run N = 2 and N = 4 (channel counts chosen from the header size reported by the runtime, recorded). A is computed by the harness with the commissioning rule and recorded (C-148, 019-26). Duration 10 minutes per run (C-167).
- Record, per slice, first-packet arrival and slice-complete arrival (from the runtime's record file; extend the report parser if needed, with fuzzing).
- Report section: distribution of (slice-complete − first-packet) per slice, compared with expected (N − 1) × serialisation time of the packets actually sent (about 12 µs per 1 500-byte packet at 1 Gbit/s); slices missing because incomplete at the deadline (C-110); confirm 0 IP fragments (`nstat` `IpFragCreates`/`IpReasmReqds` deltas = 0 on both hosts).
- Record results in `doc/design/reports/l1-results.md` with stamp, tagged `[measured]`.

## Acceptance criteria
1. Dry-run passes in CI.
2. On the hardware pair, the median (slice-complete − first-packet) difference equals (N − 1) × serialisation time within ±5 µs (C-167), for N = 2 and N = 4 (AC-24).
3. IP fragment counters on both hosts do not increase during the runs.
4. Both hosts' stamps show the path MTU in force and the multi-packet regime with its packets per slice (C-05).
5. Any parser extension has a fuzz target run 60 s on every change and 1 hour nightly, with its corpus kept (C-95).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Path MTU probing and blackhole detection (019-15).
- Implementing slice splitting in the runtime (epic 015).
