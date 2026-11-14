# 005-17 — Invariant preservation analysis
**Epic:** 005-Control-state-machines-specification · **Effort:** M · **Depends on:** 005-07, 005-10, 005-14, 005-16

## Context
- 005-AC-2 of epic 005: for each invariant L-1 to L-8, the document lists the transitions that could violate it and shows they do not; in particular no transition out of Live, Interrupted or Demoted is triggered by hello expiry (L-1), no election is opened by hello expiry (L-6), and no transition changes a pinned destination (L-5). L-7 and L-8 are shown to hold by construction, and the emitter of `ControlDegraded` is named.
- Invariants:
  - L-1 An admitted, healthy connection becomes dead only from data-plane evidence: Interrupted after 8 consecutive missed slices, Dead after 2 s. Gossip silence alone never tears down a live stream. Reverse direction: the producer stops only after 2 s without receiver reports.
  - L-2 All position-deadline control (end-of-stream, re-alignment, timeline epoch switch, transport reference change) travels in-band in slices.
  - L-3 Gossip is discovery, admission bootstrap and election input only. A follower that receives no sync replies for 1.5 s opens an election regardless of hellos.
  - L-4 Housekeeping admits unknown sources only and never touches a live data path.
  - L-5 Destinations are pinned at admission and change only by re-admission. A hello with a new address never redirects a live stream.
  - L-6 The holder is judged by sync replies only. Hello expiry never opens an election while replies flow.
  - L-7 Fail-stop applies to data and clock threads. Housekeeping is supervised and restarted; the module is marked control-degraded.
  - L-8 No lock is shared between housekeeping and the data or clock threads.
- Settled in the epic: L-7 and L-8 have no table row; the machines discharge them by construction (each owns its state, takes no lock, performs no I/O), and `ControlEvent::ControlDegraded` is emitted by the housekeeping supervisor when it restarts housekeeping. The supervisor policy is settled (C-139): backoff 100 ms to 10 s; after 5 panics within 60 s restarts stop, the module stays control-degraded with an alarm, live streams continue, and no new connection is accepted until an operator restart. On hosted platforms the control-isolation capability holds by construction of the thread model; on a device it is absent or unverified unless the implementer shows a bound (C-161). The restart behaviour itself and AC-20 (c) belong to epic 015 (C-144). Once restarts stop, the supervisor keeps a minimal refusal path that answers new admission requests with `RefuseReason::ControlDegraded` (C-185); it touches no live data path (L-4).
- The holder answers sync requests on the clock thread, never on housekeeping, and records the observed DSCP there (C-188).
- Tables: `doc/design/spec/control.toml` machines `discovery`, `election`, `admission`, `connection_consumer`, `connection_producer`; each row carries the L-ids it preserves.

## Work
1. Write the "Invariant preservation" section of `control.md`: for each L-n, list by (machine, state, input) every row that could violate it, and argue per row why it does not.
2. Add mechanical checks to the checker of 005-03 (or a sibling test) over `control.toml`:
   - L-1: no row with state Live, Interrupted or Demoted and a hello / hello-expiry input has a different next state;
   - L-6: no election row with a hello-expiry input emits `ElectionOpened`;
   - L-5: no row after Admitted modifies the pinned-address field (rows must declare written fields);
   - L-2: end-of-stream, re-alignment, timeline epoch switch, transport reference change inputs are of slice-marker kind only.
3. For L-7 and L-8, write the by-construction argument: which machine runs on housekeeping versus data/clock threads (the election's sync-request and sync-reply rows on the clock thread, C-188), that machines share no lock, and the supervisor rule that emits `ControlDegraded` under the C-139 policy and answers new requests with the `ControlDegraded` refusal (C-185) without touching a live data path (L-4); show that when restarts stop, no row of the connection machine on the data side depends on housekeeping (L-1, C-139: live streams continue), and state the C-161 capability this gives on hosted platforms.

## Acceptance criteria
1. Each of the 8 invariants has its own subsection with a list of candidate rows (possibly empty, stated as such) and an argument.
2. The 4 mechanical checks of Work step 2 run in CI and pass; a mutation fixture for each (one violating row) makes it fail.
3. The L-7 subsection names the housekeeping supervisor as the emitter of `ControlDegraded` (event and refusal reason) and cites C-139, C-161 and C-185; the L-8 subsection places sync handling on the clock thread (C-188).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Writing the per-machine tables (005-07, 005-10, 005-14, 005-16).
- Thread supervision implementation (other epics).
