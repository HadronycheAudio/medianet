# 004-07 — Timeline epoch wrap analysis
**Epic:** 004-Wire-specification · **Effort:** XS · **Depends on:** 004-06
## Context
The timeline epoch is 16 bits (C-10) and appears in the media packet header and in every sync reply. Position is network time counted in samples at the domain rate from a domain epoch; there is one timeline per domain because the wire carries one rate only (C-01).

Events that bump the timeline epoch `[decision]`:
- the in-band "timeline epoch switch" announcement, which names a future position;
- an election in which no candidate has usable time: the holder-eligible candidate with the lowest module id wins and bumps the timeline epoch (C-113). Usable time is Locked, or holdover with the alignment claim not lapsed (C-213, C-65, C-169); a candidate with usable time takes over with frequency continuity and keeps the epoch, so an election alone does not bump it;
- a holder stall: the holder's monotonic clock diverges from a clock that keeps running through suspend by more than the step threshold (2 ms, C-67), and the holder bumps the timeline epoch (C-136).

Each bump also bumps the session of every stream (C-19). No constant gives a bump rate. The election timers bound how often elections can follow each other: listen-before-claim 1.5 s (C-32), election window 400 ms (C-33), election on reply silence 1.5 s (C-37).

Open sub-question owned by epic 004:

| Question | Owner | Falsifier |
|---|---|---|
| Does a 16-bit timeline epoch wrap within a plausible uptime? | This task; epic 005 confirms the bump rate | An epoch-bump rate that wraps it in under a year of operation |

## Work
1. Start from the three bump events above (epoch switch announcement; election in which no candidate has usable time, C-113, C-213; holder stall, C-136). Give two rates: the pathological ceiling implied by the election timers C-32, C-33 and C-37 (back-to-back elections with no candidate having usable time; an election won by a candidate with usable time keeps the epoch, C-213), and a plausible worst-case operating rate, each tagged `[decision]` or `[memory]` as appropriate. Ask epic 005 to confirm the plausible rate.
2. Compute time-to-wrap for 65 536 values at both rates.
3. If time-to-wrap ≥ 1 year: record the answer in `wire.md` beside the epoch field. If < 1 year: report C-10 epoch width as falsified, specify the wrap-safe comparison rule (serial-number arithmetic) or propose a wider field, and flag the change for the constants table.
4. Specify the comparison rule receivers use for "current" vs "stale" epoch either way.

## Acceptance criteria
1. `wire.md` states both bump rates with their sources (C-113, C-213, C-136, C-32, C-33, C-37), the time-to-wrap for each, and the verdict against the 1-year falsifier.
2. A comparison rule for epochs (including behaviour at wrap) is specified.
3. If falsified, the falsification and proposed replacement width are recorded as an open item with owner and falsifier.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Epoch-switch semantics and state machines (epics 005 and 006).
- The epoch switch announcement layout (004-09).
