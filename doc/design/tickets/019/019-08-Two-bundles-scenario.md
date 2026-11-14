# 019-08 — Two-bundles scenario
**Epic:** 019-L1-two-machine-harness · **Effort:** M · **Depends on:** 019-07, epics 015, 016, 017

## Context
`two-bundles` is the C-92 test, the epic's capacity test on one GbE port, and owns the open sub-question "Does the consumer keep its deadlines at 64 000 packets/s?" (falsifier: any late slice — then tighten the port rule C-25 or the per-stream channel count; this is also the C-93 falsifier "measured cost makes C-92 fail").

Facts:
- A wide source emits a bundle of narrow aligned streams. 96 channels = 16 streams = 338 Mbit/s = 32 000 packets/s (C-22). A bundle source's controllers always ride in their own member stream, which counts towards the 32-member cap (C-181, C-205, C-209, C-08); the reference wide sources here carry no controllers, so each bundle is exactly 16 audio members and the load is 32 streams.
- Per stream: 6 channels (C-04; 192 bytes per channel per slice, 1 152 bytes of audio), 2 000 packets/s, about 21.1 Mbit/s (C-21), about 1 320 bytes per packet on the wire (C-20), one packet per slice of 0.5 ms (C-05, C-01, C-02). IPv4 only (C-100).
- Each endpoint link is admitted to 70 % of its rate (C-23), per direction (C-117). One GbE port therefore carries about 198 channel-copies (C-24): two 96-channel bundles (192), not three (288).
- A coherent bundle burst is 32 packets, about 42 KB, per slice period.
- Alignment: lateness measured for 4 s (C-50), offset = max observed + headroom (C-51), headroom = max(25 % of max, one slice period, 2 × clock bound) (C-52); fire at `T(p) + offset`; a persistently late input (C-101) ratchets by one slice period up to A (C-53) and is Demoted at A (C-109). Missing and late are counted separately, with late reclassification (C-108).
- **The commissioned A** is the presentation group's audition offset (C-55, no default), also the skew budget cap to which rings are sized (C-96). For L1 the harness computes it with the commissioning rule (largest over sinks of maximum lateness + D + headroom) and records it with the results (C-148, 019-26); the scenario receives it as `--a`. The run is a strict-mode run (C-147).
- A refused admission raises a connection alarm (C-173).

**AC-13 (two bundles, C-92).** One Linux consumer on one GbE port aligns 2 × 96 channels — 32 streams, 64 000 packets/s, 676 Mbit/s — for 1 hour with **zero late slices** at the commissioned A; a third bundle is **refused at admission** with a named reason.

## Work
- Scenario `two-bundles`: host `a` runs two reference wide sources of 96 channels each (16 streams each, no controller member, C-205); host `b` runs one consumer aligning all 32 streams on one GbE port. Parameters: `--a <µs>` (the A computed by 019-26, C-148; required), `--bundles <1|2>` (default 2; 1 is used by 019-10). Duration 3 600 s (dry-run: 5 s, reduced stream count allowed only in dry-run).
- After alignment is established, the harness asks host `a` to start a third 96-channel bundle toward the same consumer port and records the admission outcome and reason.
- Report section: total packets/s and Mbit/s measured on the consumer port, per-stream received/late/missing, ratchet steps, Demoted inputs, lateness p50/p99/max, offset used and A, admitted channel-copies per direction, third-bundle admission result, reason string and the connection alarm raised on refusal (C-173).
- Verdict: pass iff 0 late slices over 1 hour and the third bundle is refused with a named reason (not a timeout or generic error).
- Run on the hardware pair; record in `doc/design/reports/l1-results.md` under C-92 ("two-bundles"), and answer the open sub-question there (yes / no with data).

## Acceptance criteria
1. Dry-run passes in CI; running without an A is refused with a named error.
2. On the hardware pair, the consumer receives 32 streams at a measured 64 000 ± 1 % packets/s and about 676 Mbit/s for 3 600 s with **0 late slices** and 0 ratchet steps (AC-13).
3. The third bundle (which would bring the port to 288 channel-copies in one direction against a ceiling of about 198) is refused at admission; the report shows the named refusal reason and the connection alarm raised on refusal (AC-13, C-173).
4. If any slice is late, the report says so and `l1-results.md` records the C-92 and C-93 falsifiers as triggered, with a new open item (owner, falsifier) to tighten the port rule C-25 or the per-stream channel count.
5. Both hosts' complete stamps are present.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Results of the hardware run recorded in `l1-results.md` with stamp

## Out of scope
- Egress spread on/off and the unmanaged-switch burst question (019-09).
- Receive-cost per packet (019-10), which reuses this scenario at 32 000 and 64 000 packets/s.
- The 24-hour loss soak at this load (019-20).
- Implementing admission itself (epic 015).
