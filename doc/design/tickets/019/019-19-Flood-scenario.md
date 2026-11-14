# 019-19 — Flood scenario
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-07, 019-18

## Context
Admission limits each endpoint link to 70 % of its rate per direction (C-23, C-117). A misbehaving source may still send more than it was admitted for; the consumer must keep its deadlines and make the excess visible. The consumer drains at most 2 × the admitted packets per slice from one input per cycle (C-16), a cycle being one slice period (C-156), so excess traffic stays in the socket buffer and overflows there.

**AC-11 (L1 part).** One input flooded at 10 × its admitted rate from the other host: no missed deadline on the consumer, excess visible in the overflow counter.

The epic lists scenario `flood` for AC-11 in binary `hadronyche-l1` (crate `hadronyche-tools`).

Rate facts: one 6-channel stream is admitted at 2 000 packets/s, about 21.1 Mbit/s (C-21), about 1 320 bytes per packet on the wire (C-20). 10 × admitted = 20 000 packets/s, about 212 Mbit/s, which still fits on GbE. Slice period 0.5 ms (C-01, C-02, C-05). Alignment uses offset = max lateness over 4 s (C-50) plus headroom (C-51, C-52); a slice absent at `T(p) + offset` is missing, reclassified late if it arrives within the ring span (C-108). Only missing or late slices count toward the interrupted threshold (C-38, C-102).

The flood is generated with the test-only hook "send a raw packet on a media socket" (C-120); each use is stamped like any injection. A is computed by the harness with the commissioning rule and recorded (C-148, 019-26) for the consumer.

## Work
- Scenario `flood`: host `b` consumes `baseline` traffic from host `a` on two inputs (two 6-channel streams), aligned; after alignment, host `a` floods one input at 10 × its admitted rate (20 000 packets/s) through the C-120 raw-packet hook, with the same addressing and DSCP EF (46) as the admitted stream, for 10 minutes (C-167).
- Report: consumer deadline misses (fire time vs `T(p) + offset`), late and missing slices and interrupted events on the non-flooded input, socket overflow counter on the flooded input, packets offered vs accepted, packets drained per cycle on the flooded input against the C-16 bound, and the injection entry in host `a`'s stamp.
- "Missed deadline" is defined as the consumer firing after `T(p) + offset` for any slice; recorded per input.
- Record results in `doc/design/reports/l1-results.md`.

## Acceptance criteria
1. Dry-run passes in CI (flood at 10 × on loopback, shortened).
2. On the hardware pair, the flooded input receives about 20 000 packets/s (10 × 2 000) for the run, and the consumer reports **0 missed deadlines** (AC-11).
3. The excess appears in the overflow counter: overflow count > 0 and of the same order as offered minus drained packets (stated in the report) (AC-11).
4. The non-flooded input reports 0 late slices, 0 missing slices and 0 interrupted events.
5. Packets drained from the flooded input never exceed 2 × its admitted packets per slice in one cycle (C-16).
6. Host `a`'s stamp lists the flood as an injection (C-120).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- AC-11 parts outside L1 (other epics).
- Receive buffer shrink (019-18).
- Implementing input policing, the bounded drain or the test hooks in the runtime (epics 015, 017).
