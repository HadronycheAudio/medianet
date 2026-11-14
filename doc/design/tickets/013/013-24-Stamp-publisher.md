# 013-24 — Stamp publication schedule
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-05, 013-11, 013-13, epic 004, epic 005, epic 015 (stamp message sending)

## Context
- Regime stamp transport (C-18): **a dedicated control message split into sections that each fit C-17. Multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch. The hello carries only a stamp version number.** `[decision]`
- Control message bound (C-17): at most 1 252 bytes of UDP payload. IP is IPv4 only, multicast with TTL 1 (C-100). DSCP for control traffic is CS5 (40) (C-13). Authentication reservation in v1 (C-115): scheme 0 and tag zero; the 64-bit counter is live. Message and header layouts are epic 004's; the hello is sent by epic 005.
- Owner of sending (C-144): the hosted backend, epic 015, sends the regime stamp message this epic defines. This ticket decides **when** a stamp goes out and **what** goes out (the split messages and their destination: control group or a unicast requester); epic 015 puts them on the wire with IPv4, TTL 1 and DSCP 40.
- Every node publishes its own stamp, including its per-connection summaries (C-47), so third parties can observe stream health; there is no other query message. Devices publish through epic 026 with the same rule; this ticket covers hosted nodes in `hadronyche-std`.
- The `StampId` covers regime fields only (013-05): a regime change produces a new id and an immediate multicast; a counter-only change waits for the next 10 s refresh.
- Oversize sections are split by the rule of 013-13.
- How the hello's stamp version number relates to the `StampId`, and whether it changes on a counter refresh, is an open sub-question owned by epic 004 with epic 005. This ticket exposes the current `StampId` and a change notification; it does not define the hello field.

## Work
In `hadronyche-std`:
1. A publication schedule owned by the process that holds the current stamp, rebuilt by its producers through `StampBuilder`. It does no socket I/O: it emits (destination, message bytes) items that epic 015 sends (C-144).
2. On every new `StampId`: emit the stamp at once for the control multicast group, as messages ≤ 1 252 bytes (split per 013-13).
3. Every 10 s: emit the current stamp, with current counters, whether or not it changed.
4. On a unicast stamp request (delivered by epic 015): emit the current stamp at once for that requester only.
5. Expose the current `StampId` and a change notification for the hello (epic 005).
6. Refuse to emit an incomplete stamp (013-11).

## Acceptance criteria
1. A regime change is emitted for the control group within one scheduling pass of the new id (test with a simulated clock asserts it comes before the next refresh tick).
2. Over 60 s of simulated time with no change, the stamp is emitted 6 times (±1), each carrying the latest counters.
3. A unicast request yields the current stamp addressed to the requester only.
4. Every emitted message is ≤ 1 252 bytes of UDP payload (C-17).
5. An incomplete stamp is not emitted and the error names the missing groups.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier
- [ ] Design documents updated where behaviour differs, constants table first
- The split of duties with epic 015 (C-144) is written into `doc/design/spec/stamp.md` with the publish and request flow (013-22).
- Not applicable (C-125): "Every parser touched is fuzzed" (emits messages only; request parsing is epic 004's message parser); "Regime stamp fields touched are complete and documented" and "Every stamp field has a stated source on each of the four profiles" (no field added); "The stamp format is versioned independently of the media format" (no format change).

## Out of scope
- Sending the messages, socket options (TTL 1, DSCP 40) and receiving unicast requests on the socket (epic 015, C-144); the device-side publisher (epic 026); the message layouts (epic 004); the hello (epic 005); receiving stamps (013-17).
