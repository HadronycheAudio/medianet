# ADR 07 — Control-plane invariants

## Status

Accepted

## Context

Discovery, elections and admission run on a housekeeping thread that is not real-time. It competes with real-time and busy-polling threads and can be starved; the POC found that under I/O load a writer can be held for seconds `[code]`.

In the POC the control plane's timing gates the data plane `[code]`:

- A peer whose hello expires is removed from the map and its routes are torn down, even if its slices are arriving perfectly.
- Routes are rebuilt when a peer's hello shows a new address, so a wrong or colliding hello redirects a live stream.
- When the holder's hello expires, an election is opened, even while sync replies still flow.
- The clock thread takes a mutex shared with housekeeping on every iteration.
- A synth resets its transport when the gossiped reference changes.

Nothing bounds any of this. A starved housekeeper produces a false expiry, and a false expiry produces an outage on a healthy stream.

## Decision

Eight invariants hold in the runtime.

| ID | Invariant |
|---|---|
| L-1 | An admitted, healthy connection becomes dead only from data-plane evidence: interrupted after 8 consecutive missed slices (C-38), dead after 2 s without data (C-39). Gossip silence alone never tears down a live stream. Reverse direction: the producer stops only after 2 s without receiver reports (C-41) |
| L-2 | All position-deadline control travels in-band in slices: end-of-stream, re-alignment, timeline epoch switch, transport reference change. Guaranteed delivery is not required, because re-issue is free and the event names a position |
| L-3 | Gossip is discovery, admission bootstrap and election input only. A follower that receives no sync replies for 1.5 s (C-37) opens an election regardless of hellos |
| L-4 | Housekeeping admits unknown sources only. It never touches a live data path |
| L-5 | Destinations are pinned at admission and change only by re-admission |
| L-6 | The holder is judged by sync replies only. Hello expiry never opens an election while replies flow |
| L-7 | Fail-stop applies to data and clock threads. Housekeeping is supervised and restarted; the module is marked control-degraded |
| L-8 | No lock is shared between housekeeping and the data or clock threads |

Hello expiry (3 s, C-31) applies to non-live peers only.

## Alternatives considered

| Alternative | Kill-reason |
|---|---|
| Keep gossip-driven liveness with longer timeouts | Does not remove the class of failure; only makes it rarer and slower |
| Make housekeeping real-time | It parses untrusted input and does I/O; putting it at real-time priority moves the starvation problem to the data plane |
| Coordinate re-alignment and go-live over gossip | Gossip is slow (1 s hello, C-30) and unbounded under starvation; in-band announcements reach exactly the affected descendants in position order |
| Gossiped topology version as a re-alignment trigger | Withdrawn for the same reason |
| Fail-stop for every thread | Lets a housekeeping panic kill healthy streams |
| Tighten the election silence below 1.5 s | Holdover error over that span is microseconds, so nothing pushes it down, while a false election costs a handover on every stream |
| Producer keeps sending until the consumer's hello expires | That is gossip-driven liveness in the reverse direction; replaced by receiver reports sent from the consumer's data thread |

## Consequences

Positive

- The false-expiry class of outage is removed outright.
- A wrong or colliding hello cannot hijack a live stream.
- The remaining failure of a starved housekeeper is that new modules join slowly, which is a diagnostic and is stated as one.
- Announcements need no global coordination.

Negative

- The consumer's data thread must send receiver reports (one per bundle every 100 ms, C-40); the producer must track them.
- An address change on a live peer requires re-admission.
- Supervision of housekeeping is extra machinery on hosted profiles and is unsatisfiable on embedded, where it collapses to a whole-device reset.

Neutral

- On a device without an operating system, L-4 and L-8 are not OS-enforced; the implementer must show bounded control handling or the stamp reports it as report-only.

## Falsifier

- AC-20 fails at L0: any stream interruption when hellos are suppressed for 30 s, when the holder's hello is suppressed while replies flow, when the housekeeping thread is killed, or when a hello arrives with a changed address.
- A case is found where housekeeping still owns something that can damage a live stream beyond the four fixed here (destinations, holder expiry, fail-stop scope, transport reference).

## Links

- [01-architecture.md §5 Control-plane invariants](../01-architecture.md#5-control-plane-invariants), [§4.5 In-band announcements](../01-architecture.md#45-in-band-announcements), [§1.3 Per-module runtime](../01-architecture.md#13-per-module-runtime), [§15 Failure semantics](../01-architecture.md#15-failure-semantics)
- [02-constants.md](../02-constants.md): C-30, C-31, C-37, C-38, C-39, C-40, C-41
- [03-acceptance-criteria.md](../03-acceptance-criteria.md): AC-20, AC-21
- Related: [13-Interrupted-and-dead-thresholds.md](13-Interrupted-and-dead-thresholds.md), [08-Two-sided-admission.md](08-Two-sided-admission.md), [15-Crate-split-with-state-machines-in-core.md](15-Crate-split-with-state-machines-in-core.md)
