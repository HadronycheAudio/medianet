# 026b-18 — Admission refusal on device
**Epic:** 026b-Reference-device-and-porting-guide · **Effort:** XS · **Depends on:** 026b-03, 026a-07, 026b-11

## Context

Admission checks **buffer memory** and refuses beyond the device's declared limit `[decision]`. Rings are sized to the group's A at the 96 kHz domain rate (C-96, C-01): 23 040 bytes per 6-channel stream at A = 10 ms. The reference device declares one stream at A = 10 ms (026a-07). The unit of subscription is the stream (6 channels, no per-channel id, C-04), so a 7th channel is a second stream. A 100 Mbit port holds about 19 channels at the 70 % ceiling (C-26, C-23), so the second stream fits the port and is refused on memory only.

Epic criterion 026-AC-9 (Admission refusal): a request for a **7th channel** (a second stream, beyond the device's declared buffer memory) is **refused with a named reason**. The reason comes from epic 012's enum, which C-185 completes at 18 reasons: no new reason may be added, and which of the 18 a buffer-memory refusal carries is an open item owned by epic 012.

## Work

1. Bench: board admitted to a 6-channel stream from a Linux producer.
2. From the host, request a second stream from the board.
3. Capture the refusal message. Store the result in `doc/design/reports/` (C-130).

## Acceptance criteria

1. The second-stream request is refused on buffer memory; the refusal carries a named reason from epic 012's enum.
2. The existing 6-channel stream continues with 0 gaps during and after the refusal (checked in the board's per-connection counters, 026b-11).
3. The refusal is visible in the host's admission log.

## Definition of done

- [ ] Every new factual claim carries an evidence tag
- [ ] Result recorded

## Out of scope

- Admission logic and host tests (026a-07).
