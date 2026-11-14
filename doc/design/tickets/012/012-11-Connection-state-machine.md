# 012-11 — Connection state machine
**Epic:** 012-Admission-handshake · **Effort:** M · **Depends on:** 012-01, epic 005, epic 009

## Context
Each connection in epic 012 follows this life cycle `[decision]` (transition tables come from epic 005, evidence semantics from epic 009):

| From | To | Trigger |
|---|---|---|
| (start) | Requested | request made |
| Requested | Refused(reason) | admission fails, reason named (terminal) |
| Requested | Pending | target not present (visible) |
| Pending | Requested | target appears |
| Pending | (removed) | operator cancels (C-45); the request leaves the desired set |
| Requested | Admitted | both sides accept |
| Admitted | Locking | an endpoint clock is not usable (`ClockUsable(false)`) |
| Locking | Aligning | both endpoints locked (`ClockUsable(true)`) |
| Admitted | Aligning | clocks usable |
| Aligning | Live | offset set (`AligningDone { offset_ns }`), go-live on a slice boundary |
| Live | Interrupted | 8 consecutive missed slices (C-38, 4 ms) |
| Interrupted | Live | slices return, same session (`Resumed`), immediate at the frozen offset |
| Interrupted | Dead | 2 s without data (`SilentFor2s`, C-39) |
| Live | Aligning | ratchet step (`RatchetStep`, C-53: one slice period, ceiling A) or announced re-alignment (`AnnouncedRealign`) |
| Live | Demoted | persistently late with the offset already at A (`Demoted`, C-109) |
| Live | Ended | end-of-stream marker (`EndOfStream`) or disconnect (`Disconnect`) |

Terminal: Dead, Ended, Refused. Demoted is cleared only by re-admission (C-109). Pending has no timeout (C-45).

Rules and settled semantics:
- **L-1:** a live connection becomes dead only from data-plane evidence; gossip silence never tears it down.
- A Locking member is present but not live: its inputs to others are declared silent, not gaps, not loss.
- Go-live is the consumer's local decision, on a slice boundary, flagged on the first slice.
- The evidence arrives already judged by epic 009: a miss is a slice absent at the deadline (missing or late); a marked gap or a slice on another timeline epoch resets the count (C-102). `SilentFor2s` counts from the last slice with content, a marked gap or a slice on another timeline epoch (C-111, C-135). `CannotMeetA` (C-174) is alarm-only: it is in no row of the table and never changes state. `RatchetStep` means persistently late, that is at least one late slice in each of 10 consecutive 1 s windows (C-101, the C-54 dwell); `AligningDone` follows the 4 s aligning duration (C-50). No timer runs in this machine.
- Sleep of an edge is a departure (C-124): `Disconnect` gives Ended; on wake a new request passes Admitted → Locking → Aligning.

Still open (epic open sub-questions, owner epic 005): exits from Demoted other than re-admission (silence, end of stream); who aggregates "both endpoints locked" (each module's lock state is published by the servo core, epic 008a, C-187, and reaches this machine as `ClockUsable` evidence); what a connection does when the source bumps its session (C-19) during Live or Interrupted. This ticket implements only the rows above and treats every other input in Demoted as ignored until epic 005 rules; the table is a `const` so a ruling is a row change.

## Work
- In `hadronyche-core`: `pub fn step(state: ConnState, input: ConnInput) -> Option<ConnState>` where `ConnInput` is either a handshake input (`Accepted`, `Refused(RefuseReason)`, `TargetAbsent`, `TargetAppeared`, `Cancelled`) or a `DataPlaneEvidence`. Returns `None` for inputs not valid in the state (ignored).
- `MissedSlices(n)`: n is the current consecutive-miss count; transition to Interrupted when n ≥ 8 and not before.
- Encode the table as a `const` array so it can be inspected and compared with epic 005's tables.
- Document that no gossip or hello input exists in `ConnInput` for states Live, Interrupted and Demoted (DoD item).

## Acceptance criteria
1. Every row of the table is covered by a unit test; every (state, input) pair not in the table returns `None` (exhaustive test).
2. `MissedSlices(7)` keeps Live; `MissedSlices(8)` gives Interrupted (012-A8, AC-16).
3. Interrupted + `Resumed` → Live in one step (AC-16).
4. Interrupted + `SilentFor2s` → Dead (AC-16).
5. Live + `Demoted` → Demoted; no input except a new admission leaves it (012-A12, C-109).
6. Pending + `Cancelled` → removed; Pending accepts no timeout input (C-45).
7. Dead, Ended, Refused accept no input.
8. The table matches epic 005's transition table (vector or review check).
9. `CannotMeetA` returns `None` in every state (C-174: alarm only).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed — not applicable: no parser
- [ ] Regime stamp fields touched are complete and documented — not applicable: the stamp record is 012-23
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] No transition out of Live or Interrupted is reachable from a gossip input (test and table inspection)
- [ ] Refusal reasons and connection states are documented for operators — not applicable: operator documentation is 012-22

## Out of scope
- Producing the evidence (epic 009). Engine bookkeeping and events (012-12 to 012-15). Producer-side stop (012-15). The cancel API (012-24).
