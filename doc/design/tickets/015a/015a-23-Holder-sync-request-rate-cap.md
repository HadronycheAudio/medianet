# 015a-23 — Holder sync-request rate cap (C-204)
**Epic:** 015a-Hosted-runtime-core · **Effort:** S · **Depends on:** 015a-10, 015a-18
## Context
- File: `hadronyche-std/src/clock_thread.rs`, holder path. The holder answers sync requests on the clock thread, never on housekeeping (C-188), and records there the DSCP of each follower's requests (C-152, 015a-10).
- C-112: the accepted configured sync request rate is 10 to 200 per second; AC-7 is checked over this range (015b-01).
- C-204: requests beyond 200 per second from one follower are dropped before any work is done and counted per follower in the holder's stamp.
- The follower is identified from the receive metadata (`RxMeta.from`) before the request is decoded; the drop therefore costs no decoding, no servo work and no reply.
- The count reaches the stamp through the counter board (015a-18, L-8). Its wire field is not yet specified (epic 004); the board carries it so it can be stamped as soon as the field exists.
- **L-7:** a panic in the clock thread terminates the process. No allocation on the clock thread after go-live.
## Work
1. On the holder, keep a per-follower request rate tracker, preallocated for 63 followers (the peer map capacity, C-217: at most 64 modules, C-34, less the holder itself), admitting up to 200 requests per second from each follower; the tracking method is documented in the crate docs.
2. For each received request, look up the follower by source address before decoding; a request beyond 200 per second from that follower is dropped with no decoding, no reply and no other work, and the follower's drop counter is incremented.
3. Publish the per-follower drop counters to the counter board (015a-18).
## Acceptance criteria
1. A follower sending 200 requests per second on loopback has none dropped over 60 s and every request answered.
2. A follower sending 400 requests per second on loopback has at most 200 per second answered over 60 s, and every request not answered is counted as dropped against that follower.
3. A flooding follower does not reduce the answered rate of another follower sending 50 per second.
4. A dropped request is neither decoded nor answered (counting decoder and reply-count test).
5. No heap allocation per request after go-live (counting allocator unit test).
6. With 63 followers each sending requests, every follower has its own drop counter and none is refused a tracker slot (C-217).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Every parser touched is fuzzed" — request decoding is core's (epic 008), no parser here; "Design documents updated" — follows the epic; "Every new factual claim carries an evidence tag" — no new claim; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — not a media receive path.
## Out of scope
- Sync rate verification over the C-112 range (015b-01).
- The wire field for the drop count (epic 004, implemented in 013).
