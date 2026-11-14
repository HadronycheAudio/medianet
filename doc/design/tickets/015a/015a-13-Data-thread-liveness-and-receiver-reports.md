# 015a-13 — Data thread liveness and receiver reports
**Epic:** 015a-Hosted-runtime-core · **Effort:** M · **Depends on:** 015a-12, epic 014
## Context
- **L-1** (as defined in the epic): liveness — interrupted, dead, receiver reports and the producer timeout — is executed on the data threads and never passes through housekeeping.
- Consumer side, interrupted: after 8 consecutive missed slices (C-38), 4 ms at the 0.5 ms default slice. A miss is a slice absent at its deadline — missing, or arriving late. A marked gap or a slice on another timeline epoch proves the upstream is alive and resets the count (C-102).
- Late versus missing: at the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span it is reclassified as late (missing −1, late +1) and discarded (C-108). The classification is core's (epic 009); this ticket consumes it.
- Consumer side, dead: 2 s counted from the last slice received with content or a marked gap (C-39, C-111). A slice on another timeline epoch also restarts the dead clock, does not count as a miss, and has its own per-connection counter (C-135; the counter is published by 015a-19).
- A sink-only device has no stream for this module to receive, so its liveness is judged by the producer timeout (C-41) alone; interrupted and dead describe streams a peer receives (C-168).
- The consumer sends one receiver report per bundle every 100 ms (C-40), unicast to the producer; receiver reports are never multicast (C-47). Receiver reports use DSCP CS5 (40, C-13) and fit 1 252 bytes (C-17). Each report carries the consumer's current audition offset A, its own or the one it inherited (C-198).
- Producer side: the producer's data thread stops sending after 2 s without reports (C-41).
- Whether the producer resumes when reports return, or only after re-admission, is still open (control state machines, epic 005). Until it is answered, this ticket follows the epic 012 connection state machine and does not resume on its own.
- **L-7:** a panic in a data thread terminates the process.
## Work
1. Track consecutive missed slices per input with the C-102 rule; transition to interrupted at 8; reset on a received slice, a marked gap or a slice on another epoch.
2. Track the time of the last slice with content, a marked gap or another timeline epoch (C-111, C-135); transition to dead 2 s after it. Count other-epoch slices per input.
3. Emit one receiver report per bundle every 100 ms from the data thread, using the core encoder (epic 014), unicast on a control-class socket, carrying the consumer's current A (C-198).
4. Producer: track last report time per output destination; stop sending to it after 2 s without reports; hand the state change to the connection state machine (epic 012).
5. Count interrupted events and dead transitions for the stamp (015a-19).
## Acceptance criteria
1. Stopping a producer: the consumer marks the input interrupted at the deadline of the 8th consecutive absent slice (4 ms) and dead 2 s ± 1 slice after the last received slice.
2. A producer sending only marked gaps keeps the input neither interrupted nor dead.
3. Slices on another timeline epoch, fed by a test producer built from the core packetiser, reset the miss count, do not trigger interrupted, keep the input from going dead, and increment the other-epoch counter (C-102, C-135; the end-to-end variant with the C-120 epoch hook is in 015b-12).
4. Receiver reports are sent at 10 ± 0.1 per second per bundle over 60 s, to the producer's unicast address only, each carrying the consumer's current A (C-198).
5. Suppressing receiver reports makes the producer stop sending after 2 s ± 1 slice.
6. No message related to liveness crosses the housekeeping queue (code inspection + test).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- Not applicable (C-125): "Every parser touched is fuzzed" — report encoding and decoding are core's (epic 014); "Regime stamp fields touched" — published by 015a-19; "Design documents updated" — no behaviour differs; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — measured in 015b-10.
## Out of scope
- Drop counters and DSCP in reports and stamps (015a-14); health counters in the stamp (015a-19).
- Fail-stop test (015b-05).
- The producer-resume rule (epic 005).
- Acting on a changed A received in reports (015a-24).
