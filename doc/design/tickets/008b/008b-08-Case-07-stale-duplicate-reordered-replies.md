# 008b-08 — Case 07: stale, duplicate and reordered replies
**Epic:** 008b-Servo-simulation-cases-and-results · **Effort:** S · **Depends on:** 008a-15
## Context
Simulation case 7: stale, duplicate and reordered replies leave the estimate unchanged. The epic defines "unchanged": published values bit-identical to a clean run whenever a window's minimum-round-trip exchange is the same packet, and true error within 1 µs of the clean run otherwise (an extra packet can legitimately win a window).

Rule `[decision]`: each reply echoes the request's own send timestamp plus the holder's receive and send times, so each reply is a self-contained four-timestamp exchange; a stale or duplicate reply is a valid exchange with an honestly large round trip. The servo keeps only the min-round-trip exchange per 200 ms window (C-66).

## Work
1. Named test `case_07_stale_duplicate_reorder`: three runs with identical seed for clocks and base jitter — (a) clean, (b) with 5 % duplication, 5 % reordering, and replies delayed past later requests (stale), (c) with 20 % of each.
2. Compare published views of (b) and (c) against (a).

## Acceptance criteria
1. In runs (b) and (c), whenever the minimum-round-trip exchange of a window is the same packet as in (a), the published `offset_ns`, `rate_ppb` and `bound_ns` are bit-identical to (a).
2. Over the whole run, |true error(b or c) − true error(a)| ≤ 1 µs at every sample after lock.
3. 0 honest-lock violations in (b) and (c).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Loss leading to holdover (008b-05).
