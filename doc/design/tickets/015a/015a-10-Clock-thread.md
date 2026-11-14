# 015a-10 — Clock thread (clock_thread.rs)
**Epic:** 015a-Hosted-runtime-core · **Effort:** M · **Depends on:** 015a-03, 015a-05, 015a-06, epic 008
## Context
- File: `hadronyche-std/src/clock_thread.rs`: deadline-paced sync exchanges driving the core servo (epic 008); publishes through the core seqlock. The time view it publishes carries the clock lock state and bound; these are computed and published by the servo core (epic 008a, C-187), and the clock thread passes them through unchanged.
- One clock thread per process `[decision]`.
- POC defects `[code]`: `src/module.rs:1126` paced clock requests by a 5 ms receive timeout; measured 37 completed exchanges/s at `--sync-hz 50` and 77/s at `--sync-hz 1000` `[measured]` on a kernel with `CONFIG_HZ=250` `[measured]`. Timeouts are not deadlines. `src/module.rs:1518` took `inner.target.lock()` — a mutex shared with housekeeping — every iteration, violating L-8.
- Rules: sync requests are paced by deadline (`ppoll` with a computed timeout or equivalent); the achieved rate must equal the configured rate (50/s by default, C-36) and be reported. The accepted configured range is 10 to 200 requests per second (C-112). Sync uses DSCP CS6 (48, C-13).
- Wall-clock step detection is done in the backend before an exchange is built (C-121): samples with `rx_host_ns = None` never reach the servo and are counted.
- Elections start only on 1.5 s of sync-reply silence (C-37); hello expiry applies to non-live peers only (C-31). The clock thread therefore publishes, lock-free, the host time of the last valid sync reply; housekeeping feeds it to the election state machine (015a-15).
- Peer verification of sync DSCP (C-152): the holder reports in its stamp the DSCP observed on each follower's sync requests. The holder answers sync requests on the clock thread, never on housekeeping, and records there the received TOS per follower (C-188); it publishes the values through the counter board (015a-18). A follower not yet heard from is "unverified".
- Sync requests above the accepted rate (C-204): on the holder, requests beyond 200 per second from one follower (C-112) are dropped before any work is done and counted per follower in the holder's stamp. Built in 015a-23 on this thread's holder path.
- **L-8:** no lock is shared between housekeeping and the data or clock threads; communicate by messages or lock-free publication only. The sync target (holder) is received from housekeeping by a lock-free queue.
- **L-7:** a panic in the clock thread terminates the process (fail-stop).
## Work
1. Loop: compute next request deadline on the host clock; wait with `ppoll` until min(next deadline, socket readable); on deadline send a sync request; on readable, `recv_batch` replies and feed valid exchanges to the core servo.
2. Refuse a configured rate outside 10 to 200 per second at construction (C-112).
3. Publish the servo time view through the core seqlock after each update, including the lock state and bound the servo core supplies (C-187); compute neither here.
4. Receive holder changes from housekeeping through a bounded lock-free channel; no mutex.
5. Publish lock-free: last valid reply time, achieved exchanges/s, sent requests/s, discarded samples.
6. Holder role: answer sync requests on the clock thread, never on housekeeping (C-188); record per follower the DSCP observed on its requests (`tos >> 2`, or none yet), in a table preallocated for 63 followers (C-217), and publish it to the counter board (C-152).
## Acceptance criteria
1. At 50/s configured, sent request rate is 50 ± 1/s over 60 s on loopback with a local responder.
2. A configured rate of 9/s or 201/s is refused with a named error.
3. No mutex or other lock type is acquired in the loop (code review and the 015b-02 check).
4. A sample with `rx_host_ns = None` is not passed to the servo and increments the discard counter.
5. The last-reply time advances with every valid reply and stops advancing when the responder is silenced.
6. Achieved rate is exposed for publication; on a holder, the DSCP observed on each follower's sync requests (48 on loopback) is exposed per follower, and a follower not yet heard reads "unverified" (C-152); with housekeeping stopped, the holder still answers sync requests (C-188).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Every parser touched is fuzzed" — reply decoding is core's (epic 008), no parser here; "Regime stamp fields touched" — published through 015a-18 and 015a-14; "Design documents updated" — no behaviour differs; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — data path only (015b-10).
## Out of scope
- Full-range rate verification (015b-01).
- The holder's per-follower drop of sync requests above 200 per second (015a-23, C-204).
- Servo algorithms (epic 008); the election state machine (epic 011).
- Real-time priority setup (task 016) and strict mode (015b-13).
