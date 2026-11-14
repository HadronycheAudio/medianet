# 019-15 — Blackhole scenario
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-07, 019-06

## Context
Path MTU handling in the design: a probe pair at admission, sent at the intended media packet size and exempt from the control message bound C-17 (C-116); fallback 1 280 bytes (C-06), which gives 1 252 bytes of UDP payload (IPv4 only, C-100); re-probe when large packets vanish while liveness holds. IP fragmentation is forbidden; every packet has don't-fragment set. A slice larger than the path MTU is carried in the explicit, stamped multi-packet regime (C-05). A path MTU below the probe size silently drops ("blackholes") packets of the intended size unless the probe detects it.

Scenario `blackhole`: "a switch port or interface MTU lowered below the probe size."

**AC-19 (L1 part).** On a real switch with a lowered MTU, the probe pair detects at admission, the connection goes live at 1 280 bytes, and the path MTU appears in the stamp. The epic scopes this to detection at admission: re-probe after a live MTU drop is recorded but is not a pass condition.

Packet size: a default slice fits the 1 252-byte fallback payload, since it carries 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum (C-137); in-band announcements and the measurement extension fit inside that cap or are omitted on a fallback path. A default 6-channel stream therefore cannot trigger the fallback, so the scenario uses a stream whose intended packet clearly exceeds 1 280 bytes, so the probe must fail and the connection must fall back, and it records the fallback connection's multi-packet regime (C-05).

Admission refuses with PathMtuTooSmall only when the probe at the 1 280-byte fallback also fails (C-173); in this scenario the fallback works, so PathMtuTooSmall must not appear.

The `switch` variant needs the managed switch recorded in 019-06. Privileged steps run only on the hardware pair; in dry-run they are recorded as skipped (epic 019).

## Work
- Scenario `blackhole` (A is computed by the harness with the commissioning rule and recorded (C-148, 019-26)):
  - Variant `switch`: set the MTU of the managed switch port facing the consumer to 1 280 (or the smallest value at or above 1 280 the switch accepts, recorded), manual step or switch CLI, recorded in the run log.
  - Variant `interface`: set the receiving host's interface MTU to 1 280 with `ip link set <if> mtu 1280` (privileged; skipped in dry-run).
  - In each variant, start one stream wide enough that its intended packet exceeds 1 280 bytes (channel count recorded), check the admission outcome, run 10 minutes (C-167), restore the MTU.
- Report section: probe size (C-116), probe result at admission, MTU chosen (expected 1 280), slice regime after going live (single or multi-packet, packets per slice), packets lost and missing/late slices after going live, path MTU from both hosts' stamps.
- Recorded only, not a pass condition: lower the MTU after the connection is live and record whether re-probe occurs.
- Record results in `doc/design/reports/l1-results.md` under "Switch findings".

## Acceptance criteria
1. Dry-run passes in CI with MTU changes listed as `Skipped("privileged")`.
2. On the real switch with lowered port MTU, the report shows the probe pair, at the intended media packet size, detected the reduced MTU at admission and the connection went live at 1 280 bytes (AC-19).
3. The stamp of both hosts shows path MTU = 1 280 and the slice regime in force.
4. After going live, 0 packets lost due to size over the run, and no refusal with PathMtuTooSmall is recorded (C-173).
5. The original MTU settings are restored on host and switch at the end, including on abort (verified by `ip link show` in the run log).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] Every new factual claim carries an evidence tag
- [ ] The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- AC-19 parts outside L1 (other epics), including re-probe as a pass condition.
- Implementing probing and fallback (epic 015).
- Multi-packet serialisation timing (019-14).
