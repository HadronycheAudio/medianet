# 005-15 — Connection states and inputs
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-02, epic 004

## Context
- Machine 4 of `doc/design/spec/control.md`: connection states, consumer side plus a small producer-side machine. Pure table, no I/O, in `hadronyche-core`.
- Consumer states and arcs (from the epic's diagram):
  - start → Requested; Requested → Refused (admission fails, reason named); Requested → Pending (target not present, visible, no timeout, C-45); Pending → Requested (target appears); Pending → Ended (operator cancels, C-45); Requested → Admitted (both sides accept);
  - Admitted → Locking (an endpoint clock is not usable); Locking → Aligning (both endpoints locked); Admitted → Aligning;
  - Aligning → Live (after the 4 s aligning duration, C-50, offset set, go-live on a slice boundary);
  - Live → Interrupted (8 consecutive slices absent at the deadline, C-38, C-102; 4 ms at the 0.5 ms slice of C-02); Interrupted → Live (slices return, same session); Interrupted → Dead (2 s since the last slice with content, a marked gap or a slice on another timeline epoch, C-39, C-111, C-135);
  - Live → Aligning (ratchet step when persistently late, C-101, taking effect at a position announced one lead ahead, C-215, or in a sink at its own position one lead ahead, unannounced, C-225, or announced re-alignment; the last step goes to A exactly when less than one slice period remains, C-163); Live → Demoted (persistently late with the offset already at A, C-109); Demoted → Requested (re-admission); Live → Ended (end-of-stream marker or disconnect);
  - Dead, Ended, Refused are terminal. That is 11 consumer states.
  - Display (C-154): only Live is satisfied; every other state is shown by name as unsatisfied.
- Further rules: a locking member is present but not live: its inputs to others are declared silent, not gaps, not loss. Go-live is the consumer's local decision, on a slice boundary, flagged on the first slice. Resuming from Interrupted is immediate at the frozen offset. Session numbers are assigned by the source, one per stream, bumped on restart and on timeline epoch change (C-19). A sleeping laptop edge has departed and returns through normal admission (C-124). When an endpoint's holdover claim lapses its clock is Unlocked with time unusable and alignment claims stop (C-162); the reaction is decided in 005-24.
- Producer: Sending → Stopped after 2 s without receiver reports (C-41); receivers send one report per bundle every 100 ms (C-40); reports are unicast, never multicast (C-47). The liveness of a sink-only device is judged by this producer timeout alone; Interrupted and Dead describe streams a peer receives (C-168).
- Constants: C-50 aligning duration 4 s; C-53 ratchet step one slice period, ceiling the presentation group's audition offset A (C-55); C-54 dwell 10 s, expressed as the C-101 test (one late slice in each of 10 consecutive 1 s windows).
- L-2: end-of-stream, re-alignment, timeline epoch switch and transport reference change travel in-band in slices.
- Open sub-question owned by this ticket (epic 005): the producer machine's initial state, entry to Sending, and restart after Stopped. Falsifier: a producer sends before admission or cannot resume after re-admission.

## Work
1. "States and data": for each of the 11 consumer states and the producer states, list the data held (session number (C-19), pinned IPv4 addresses, offset, frozen offset, missed-slice counter, time of the last slice with content, marked gap or other timeline epoch (C-111, C-135), aligning start time, late-window counter for C-101, refuse reason, last receiver-report time).
2. Decide the producer machine's initial state, entry and restart; record it in "Decisions on open sub-questions" with a `[decision]` tag and the falsifier.
3. "Memory bound": bytes per connection and the worst case with 63 peers (C-217; state the assumed connection count).
4. "Input alphabet": admission outcomes (from machine 3), operator cancel, re-admission request; slice with content, slice absent at the deadline, marked gap, slice on another timeline epoch, late arrival of a slice counted missing (C-108), slice with a new session number; in-band markers carried in slices (end-of-stream, re-alignment, timeline epoch switch, transport reference change); disconnect message (epic 004); receiver report; timers: Dead 2 s (C-39), aligning 4 s (C-50), lateness window 1 s (C-101), producer 2 s (C-41); local: endpoint lock state, including the lapse to Unlocked with time unusable (C-162); cannot-meet-A from the alignment stage (C-174, rows in 005-26). There is no Pending timer (C-45).
5. State explicitly that hello and hello expiry are **not** inputs of this machine (L-1, L-5).
6. Define the displayed property `satisfied` as "state is Live" (C-154) and list the unsatisfied name shown for every other state.
7. Mirror into `control.toml` as machines `connection_consumer` and `connection_producer`.

## Acceptance criteria
1. Exactly 11 consumer states (including Demoted) and the decided producer states, each with data and byte sizes.
2. The worst-case memory with 63 peers (C-217) is stated as a number.
3. The alphabet has no hello-derived input and no Pending timer.
4. Interrupted holds the frozen offset used on resume.
5. The producer decision is recorded with its falsifier.
6. `control.toml` declares the same states and inputs as the prose.
7. `satisfied` is true in Live only (C-154).

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Guard definitions for health transitions (005-24) and the transition tables (005-16).
- Regime stamp counters (005-23).
- Admission internals (005-13, 005-14).
- Implementation (epic 012).
