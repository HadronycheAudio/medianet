# 019-27 — Bad-input processing bound
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-07, 019-19, epic 015, epic 017

## Context
C-164 proposes a bound on processing one bad input: **linear in packet length and at most 10 µs per packet**, pending L1. Epic 019 is L1, so it measures the bound and writes it back with the other L1 constants (019-22).

Means and facts:
- Malformed packets are sent with the test-only hook "send a raw packet on a media socket" (C-120); each use is stamped like any injection.
- Slice period 0.5 ms (C-01, C-02); a default packet carries 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum, within 1 252 bytes of UDP payload (C-137).
- The consumer's data-thread time per rejected packet must come from the hosted runtime's instrumentation (open, owner epic 015, as for C-93 in 019-10). If it is absent, this ticket records the reason and stops.
- The run's number enters the constants table, so it is a strict-mode run (C-147); A is computed by the harness with the commissioning rule and recorded (C-148, 019-26).

## Work
- Scenario `bad-input`: host `b` consumes `baseline` traffic on two aligned inputs; host `a` sends malformed packets to one input through the C-120 raw-packet hook, at packet lengths spanning the minimum header size to the largest the path MTU allows (lengths recorded), at a rate that keeps the input within its admitted rate. 10 minutes (C-167).
- Malformed variants: truncated header, bad checksum, unknown stream number, length field disagreeing with the datagram. The variant list and counts are recorded.
- Report section `bad_input`: per variant and length, packets sent, packets rejected, data-thread time per rejected packet (median, 99.9th percentile, maximum), a linear fit of time against length, and late, missing and interrupted counts on the other input.
- Record in `doc/design/reports/l1-results.md` under C-164: maximum time per bad packet, the fit, whether the proposal holds (linear and at most 10 µs), hardware, stamp, tag `[measured]`.

## Acceptance criteria
1. Dry-run passes in CI with every malformed variant sent and counted as rejected.
2. On the hardware pair, `l1-results.md` states the maximum data-thread time per bad packet and the linear fit against packet length, tagged `[measured]`, and a verdict on C-164 (holds / does not hold), or a stated reason why not.
3. The non-attacked input reports 0 late slices, 0 missing slices and 0 interrupted events during the run.
4. Host `a`'s stamp lists the malformed packets as injections (C-120).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Each pending-measurement constant in the table above has a measured value tagged `[measured]`, or a stated reason why not
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Fuzzing the runtime's parsers (C-95; not this epic).
- Implementing input validation or its instrumentation (epic 015).
- Writing C-164 to the constants table (019-22).
