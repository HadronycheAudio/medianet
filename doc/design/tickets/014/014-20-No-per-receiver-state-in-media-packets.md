# 014-20 — No per-receiver state in media packets (AC-10)
**Epic:** 014-Measurement-path · **Effort:** XS · **Depends on:** 014-04, epic 004
## Context
Media packets carry **no per-receiver state** `[decision]`, so the same bytes can serve several consumers and the wire stays ready for later fan-out. This retires the POC's in-band round trip `[code]`: the POC's 88-byte stream header carried `t1`, `t2k`, `t2`, `t3`, `ack_t4`, `tx_late`, `rx_lost`, `drift_ppb`, `servo_err`, `path_delay` (`proto.rs:147-185`), overloaded fields by direction, and an `F_ACK` acknowledgement of the receiver's last echo. Epic AC-10: no media packet contains any field written from a receiver report. The session in a media packet is per stream and identical for every consumer (C-19), so it is not per-receiver state.

## Work
1. Add a test that encodes the same media slice twice, once with a `ProducerWatch` / parsed `ReceiverReport` state of consumer A and once of consumer B with very different counters, and asserts the media bytes are identical.
2. Add a source-level check (CI grep or module-visibility rule) that the media encoder of epic 004 does not import the report types of the module `measure`.

## Acceptance criteria
1. The two encodings are byte-identical (AC-10).
2. The dependency check fails if the media encoder references `ReceiverReport`, `MemberCounters`, `DropCounters`, `ConnectionSummary` or `ProducerWatch`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- The media packet layout itself (epic 004).
