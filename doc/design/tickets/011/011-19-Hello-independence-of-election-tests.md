# 011-19 — Hello independence of election: tests
**Epic:** 011-Gossip-and-discovery-core · **Effort:** S · **Depends on:** 011-07, 011-15

## Context
Invariants `[decision]`:
- L-3: gossip is discovery, admission bootstrap and election input only. A follower that receives no sync replies for 1.5 s (C-37) opens an election regardless of hellos.
- L-6: the holder is judged by sync replies only. Hello expiry (C-31, 3 s) never opens an election while replies flow.
- L-1: a peer with a live connection is not removed from the map by hello expiry.

`[code]` POC behaviour retired: `leave()` opened an election when the holder's hello expired (`module.rs:575-585`).

Epic AC 3: AC-20 (b): with the holder's hellos suppressed while sync replies flow, no election is opened. AC-20 (a): with all hellos suppressed for 30 s, no peer with a live connection is removed from the map.

## Work
1. A combined simulation of `Discovery` + `Election` for 5 modules, with switches to suppress hellos per sender and to suppress sync replies (modelled as `ReplySilence` inputs from the clock-thread stub). The harness drains `next_event()` after every input. 011-20 reuses it.
2. The tests listed in the acceptance criteria. Update the epic 005 transition table (or its annotation) to show no edge from hello expiry to an election state.

## Acceptance criteria
1. AC-20 (b): holder's hellos suppressed for 60 s, replies flowing: zero `Opened` events on every module.
2. AC-20 (a): all hellos suppressed for 30 s: every peer marked live is still in every map.
3. L-3: hellos flowing normally, replies suppressed: a follower opens an election 1.5 s (± one tick) after the last reply.
4. Holder's hello expires while it is not marked live and replies still flow: `Left(holder)` may be raised but no `Opened`.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No election path is reachable from hello expiry (shown by test and by inspection of the transition table)
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag

Not applicable (C-125): "Every parser touched is fuzzed" (tests only); "Regime stamp fields touched" and "Holder-eligible bit in the regime stamp" (no stamp field); "No new open item without an owner and a falsifier" (opens no item); "The frozen POC is not modified" (POC cited, not touched).

## Out of scope
- The 24-hour no-rotation run: 011-20. The scenario vectors: 011-21.
