# 019-07 — Baseline scenario and DSCP check
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-04, 019-05, 019-06, 019-26, epics 015, 016, 017

## Context
`baseline` is the reference L1 scenario: **one 6-channel stream each way between the two hosts, for 1 hour**. Several later scenarios (`coalescing`, `stress`, `asymmetry`) repeat it under changed conditions.

Stream facts:
- IPv4 only, multicast with TTL 1 (C-100). The wire carries the domain rate of 96 kHz only (C-01).
- A slice is one packet by default (C-05): 48 frames = 0.5 ms at 96 kHz (C-01, C-02), 4 bytes per sample (C-03), 6 channels per stream (C-04): 192 bytes per channel per slice, 1 152 bytes of audio, no per-channel id byte.
- About 1 320 bytes on the wire including all Ethernet overhead (C-20); 2 000 packets/s and about 21.1 Mbit/s per stream (C-21).
- IP fragmentation is forbidden; every packet has don't-fragment set.

Alignment and lateness: the consumer measures origin-to-arrival lateness for 4 s (C-50), sets its offset to the maximum observed plus headroom (C-51) — the largest of 25 % of that maximum, one slice period, twice the clock bound (C-52) — and fires at `T(p) + offset`. A persistently late input (at least one late slice in each of 10 consecutive 1 s windows, C-101) steps its offset up by one slice period (C-53) up to the ceiling A; at A it becomes Demoted (C-109). An absent slice is counted missing at the deadline and reclassified as late if it arrives within the ring span (C-108). Clock bound is at most 25 µs (C-62).

**The commissioned A.** A is the presentation group's audition offset (C-55): no default, one per presentation group, proposed by commissioning; it is also the skew budget cap to which alignment rings are sized (C-96). For L1 runs the harness computes A with the commissioning rule — the largest over sinks of maximum lateness + D + headroom — and records it with the results (C-148); the computation is 019-26, which passes the value in as `--a`. The run is a strict-mode run (C-147), since AC-14 feeds the C-50 to C-52 falsifier.

DSCP (C-13): sync CS6 (48); media and one-way probes EF (46); control CS5 (40); verified by the receiving peer. The observed DSCP of each media connection is in the receiving peer's regime stamp (C-47); the DSCP observed on each follower's sync requests is in the holder's stamp, and the DSCP observed on each peer's hellos (control) is in every module's stamp (C-152); receiver reports are unicast to the producer and read from the producer host's records.

Acceptance criteria covered here:
- **AC-14.** Zero late slices for 1 hour on a clean medianet at the commissioned A in `baseline`. This also checks the falsifier of C-50 to C-52 (late slices after alignment on a clean network).
- **AC-34.** Receiver reports and the receiving peer's regime stamp show sync 48, media and probes 46, control 40 as received.

## Work
- Scenario `baseline` in `hadronyche-tools`: on each host, start one reference source (6 channels) toward the other host and one reference consumer; parameter `--a <µs>` carrying the A computed by 019-26 (C-148; no default, C-55); duration 3 600 s (dry-run: 5 s).
- Scenario section in the report: per direction, slices expected/received/late/missing/marked gap, ratchet steps, Demoted, lateness p50/p99/max, offset used and A, servo offset p50/p99/max, one-way figure with asymmetry bound.
- DSCP check: the report's `DscpTable` lists, per class (sync, media, one-way probe, control), the DSCP value as received by the peer (media from the receiver reports and the C-47 stamp counters; sync from the holder's stamp and control from each module's hello DSCP, C-152) and the expected value (48, 46, 46, 40); any mismatch is a failed check.
- Verdict: pass iff late slices = 0 in both directions and the DSCP table matches.
- Run once on the hardware pair from 019-06 on a clean wire; attach `report.md` and `stamp.json` to `doc/design/reports/l1-results.md` under a "baseline" section.

## Acceptance criteria
1. `hadronyche-l1 run baseline --dry-run` passes in CI (picked up by job `l1-dry-run`); running without an A is refused with a named error.
2. On the hardware pair, a 1-hour `baseline` run reports **0 late slices** and 0 ratchet steps in each direction (AC-14), with A and its C-148 inputs stated in the report, and both hosts in strict mode (C-147).
3. The report's DSCP table shows, as received, sync = 48, media = 46, one-way probes = 46, control = 40 (AC-34).
4. A dry-run in which the source is configured to mark media with DSCP 0 makes the DSCP check fail with the class name and both values.
5. Both hosts' complete stamps are present in the results.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Results of the hardware run recorded in `l1-results.md` with stamp

## Out of scope
- Coalescing on/off (019-11), stress loads (019-17), 24-hour loss counting (019-20).
- Receive-cost measurement at 2 000 packets/s (019-10 reuses this scenario).
- The jitter profile (019-24) and drain-bound check (019-25), which reuse this scenario's records.
