# 015b-12 — Test-only injection hooks (C-120)
**Epic:** 015b-Hosted-runtime-verification-and-strict-mode · **Effort:** S · **Depends on:** 015a-05, 015a-17, 015a-18
## Context
- C-120: beyond the public injection API, the test-only hooks are: force a module id and nonce; force a stream onto a given timeline epoch; send a raw packet on a media or control socket. Each use is stamped like any injection.
- This ticket provides the hosted side of all three. Users in this epic: the flood test sends from the pinned source with the raw-packet hook on a media socket (015b-03); the changed-address hello uses it on a control socket (015b-04); the epoch hook exercises the C-102 miss-count reset of 015a-13 end to end.
- Panic injection into a data, clock or housekeeping thread is an additional test-only hook (C-145); it reuses this ticket's feature gate and injection record and is built in 015b-14.
## Work
1. `Runtime` configuration: force module id and instance nonce at start (test builds or an explicit test feature only).
2. Data thread: force a given output stream onto a given timeline epoch, through the core packetiser.
3. Socket: send a caller-supplied raw datagram on a module's media socket or the control socket, through `send_to` (same DSCP and DF as the socket).
4. Each hook use increments an injection record on the counter board, so it appears in the stamp.
5. The hooks do not exist in a release build without the test feature.
## Acceptance criteria
1. A runtime started with a forced id and nonce announces exactly those in its hello.
2. A forced epoch change is visible in the stream's packets, and the consumer's miss count resets without an interrupted event (C-102).
3. A raw packet sent through the hook arrives from the module's socket address with the socket's DSCP and DF set.
4. Each use appears in the next stamp as an injection.
5. A release build without the test feature has no hook symbols (build test).
## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- Not applicable (C-125): "Every parser touched is fuzzed" — no parser; "Design documents updated" — follows the epic; "Every new factual claim carries an evidence tag" — no new claim; "Trait surface compiles with a stub backend" — no trait change; "Per-packet receive cost measured" — not a receive-cost test.
## Out of scope
- The public injection API and the injection stamp format (other epics).
- Panic injection (015b-14).
