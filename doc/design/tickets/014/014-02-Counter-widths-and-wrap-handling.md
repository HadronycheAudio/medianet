# 014-02 — Counter widths and wrap handling
**Epic:** 014-Measurement-path · **Effort:** S · **Depends on:** 014-01, epic 004
## Context
The receiver report (one per bundle every 100 ms, C-40) carries per-member counters and drop counters. The epic gives the in-memory shape:

```rust
pub struct MemberCounters { pub stream: StreamId, pub session: u32, pub received: u32, pub late: u32,
                            pub missing: u32, pub marked_gap: u32, pub other_epoch: u32, pub too_early: u32,
                            pub duplicates: u32, pub incomplete_reassemblies: u32, pub demoted_rounds: u32,
                            pub interrupted: u16, pub health: InputHealth }
pub struct DropCounters { pub socket_overflow: Option<u32>, pub iface_rx_dropped: Option<u32>,
                          pub iface_rx_missed: Option<u32> }
```

Open sub-question 1 of the epic, owned by **task 004 with this task**: counter semantics (cumulative or per-interval), widths and wrap handling over long runs. Falsifier: a counter wraps within one report interval at the rates below, or a reclassification cannot be expressed in the chosen encoding.

Settled facts:
- Member counters count **slices**, not packets. At the default slice length (C-02) a member produces 2 000 slices per second, 200 per report interval. The default slice is one packet (C-05), so this is also 2 000 packets per second per stream (C-22). A shorter, non-default slice raises the rate in proportion.
- The socket overflow counter is per socket and cumulative (`SO_RXQ_OVFL`, Linux since 2.6.33) `[kernel-doc]`. The highest settled load on one consumer socket is 64 000 packets per second (two 96-channel bundles, C-92). Interface drop counters are reported as deltas `[decision]`.
- Late versus missing (C-108): at the deadline an absent slice is counted missing; if it arrives later within the ring span it is reclassified (missing −1, late +1). A missing count can therefore **decrease** between two reports. A per-interval encoding must be able to carry a negative missing delta; a cumulative encoding with modular deltas carries it naturally.
- Counter set (C-159, completing C-47): the stamp publishes received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events and dead per connection. The producer builds its summary from reports only, so every count except dead travels in the member record. A slice on another timeline epoch counts in other-epoch, never as a miss (C-135). The widths in the struct above are placeholders until this ticket decides them.
- Size (C-17, C-100): the report for 32 members (C-08) with every count and every optional field must fit 1 252 bytes of UDP payload; the chosen widths must allow it. If they cannot, that is a defect returned to epic 004 (no split rule).
- Session (C-19): counters in a member record refer to the session in that record. Whether counters restart at a new session is defined by epic 009, which owns the per-input counters (README remaining gaps).

Derived wrap times at those rates (cumulative counters):

| Width | 2 000 per second | 64 000 per second |
|---|---|---|
| `u16` | about 32.8 s | about 1.0 s |
| `u32` | about 24.8 days | about 18.6 hours |

## Work
1. Together with the owner of epic 004, decide and write down for each counter field: cumulative or per-interval, width, and wrap rule (receiver computes deltas modulo 2^width, or saturates), taking into account that missing can decrease (C-108).
2. Implement `const fn counter_delta(prev: u32, now: u32) -> i64` (modular difference interpreted as signed, so a decrease of missing after reclassification gives a small negative value) and the `u16` equivalent, if the chosen rule is modular.
3. Record the decision in the epic 004 message-layout specification in `doc/design/spec/` (C-130), with an evidence tag `[decision]`, and close open sub-question 1 (owner and falsifier kept).

## Acceptance criteria
1. A written table lists every counter in `MemberCounters` and `DropCounters` with semantics, width, wrap rule and time to wrap at the rate that applies to it (2 000 per second for member counters, 64 000 per second for the socket overflow counter).
2. For every counter, time to wrap is greater than 2 × the report interval (≥ 200 ms), so no counter can wrap twice between two reports; a unit test checks this numerically.
3. Unit tests for the delta helpers cover: no wrap, exactly one wrap (`prev = u32::MAX - 1`, `now = 3` gives `5`), equal values (`0`), and a reclassification decrease (`prev = 10`, `now = 9` gives `-1`).
4. The table covers every C-159 count; a unit test computes the encoded size of a 32-member report with the chosen widths and asserts it is at most 1 252 bytes (C-17, C-100).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Encoding the counters on the wire (014-03).
- Filling the counters from received slices and the reclassification itself (epic 009 provides per-input counters; epic 015 fills reports).
