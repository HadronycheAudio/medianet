# 014-12 — Reverse-port message classifier
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-04, 014-08, epic 004
## Context
The v1 message set on the reverse unicast port is **exactly two** kinds `[decision]`: the receiver report (DSCP 40) and the one-way probe request / reply (DSCP 46). Nothing else travels on this path in v1: receiver reports are never multicast and there is no query message (C-47), and the regime stamp has its own control message (C-18). Definition of done of the epic, and epic AC-13: "The message set on the reverse port is closed: any other message type is rejected and counted." Parsers are fuzzed with the C-95 budget: 60 s on every change and 1 hour nightly, the corpus kept.

## Work
1. Implement `classify_reverse(bytes: &[u8]) -> Result<ReverseMessage<'_>, Reject>` with variants `Report(ReceiverReport)`, `ProbeRequest`, `ProbeReply` (names may be refined).
2. Add a counter struct `ReverseRejects { unknown_type: u32, malformed: u32 }` incremented by the classifier caller (or by a `&mut` argument); no allocation, no locking.
3. Any message type other than the three is rejected with the reject reason epic 004 specifies for an unknown type (C-193) and counted as `unknown_type`; parse failures of known types are counted as `malformed`.
4. Add a fuzz target `reverse_classify` (with the team's chosen fuzzing tool, C-192), run 60 s on every change and 1 hour nightly, corpus kept (C-95).

## Acceptance criteria
1. Each valid report, probe request and probe reply vector is classified to the right variant.
2. For every possible type value other than the three known ones, `classify_reverse` returns `Err` and `unknown_type` increments by exactly 1 (epic AC-13).
3. A truncated known message increments `malformed`, not `unknown_type`.
4. The fuzz target `reverse_classify` runs at the C-95 budget with zero panics.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] The message set on the reverse port is closed: any other message type is rejected and counted
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Socket receive loop and exposing the counters in telemetry (epic 015).
