# 019-18 — Receive buffer overflow scenario
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-07

## Context
v1 has no redundancy: it detects, classifies, conceals and accounts for loss. Loss caused locally by a full socket receive buffer must therefore be classified correctly and be visible.

**AC-37.** Loss induced by shrinking a receive buffer appears in the socket overflow counter, the stamp and the receiver report.

The epic's scenario list has no scenario for AC-37; this ticket adds scenario `rcvbuf-shrink` to binary `hadronyche-l1` (crate `hadronyche-tools`).

Load facts: one 6-channel stream is 2 000 packets/s of about 1 320 bytes on the wire (C-20, C-21); the `two-bundles` load is 64 000 packets/s (C-22).

Classification: a slice whose packet was dropped by the full buffer is absent at the deadline and counted missing (C-108); the per-connection gap-class counters are in the consumer's regime stamp (C-47). Where the socket overflow counter sits in the stamp and in the receiver report is defined by epic 004 (stamp and receiver report layout) and epic 015. Privileged steps run only on the hardware pair; in dry-run they are recorded as skipped (epic 019).

A is computed by the harness with the commissioning rule and recorded (C-148, 019-26) for the `baseline` traffic.

## Work
- Scenario `rcvbuf-shrink`: run `baseline` traffic, then make the consumer's receive socket buffer small enough to overflow (via a runtime option setting `SO_RCVBUF`, if the runtime exposes one; otherwise by lowering `net.core.rmem_max`/`rmem_default` on the receiving host before start, a privileged step). If `baseline` load does not overflow, use the `two-bundles` load.
- Independent ground truth: read the kernel's per-socket drop count (`/proc/net/udp` `drops` column, or `SO_RXQ_OVFL` ancillary data) and `nstat` `UdpRcvbufErrors` delta.
- Report: socket overflow counter as reported by the runtime, kernel ground truth, lost packets in the receiver report (read from the producer host's records, since receiver reports are unicast, C-47), missing slices, and the overflow counter in the stamp.
- Duration: until at least one overflow is seen, at most 10 minutes (C-167).
- Restore system settings at the end.
- Dry-run: `SO_RCVBUF` shrink on loopback (unprivileged when done through the socket option within `rmem_max`).

## Acceptance criteria
1. Dry-run passes in CI and produces a non-zero overflow count.
2. On the hardware pair, the run produces at least 1 lost packet attributed to receive-buffer overflow.
3. The same non-zero count appears in (a) the socket overflow counter, (b) the receiving host's stamp and (c) the receiver report; it matches the kernel's `UdpRcvbufErrors` delta within ± 1 % (AC-37).
4. Settings are restored at the end (checked in the run log).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Flood from the peer host (019-19).
- Wire loss counting (019-20).
