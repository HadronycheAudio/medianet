# 005-24 — Connection health guards and demotion
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-15

## Context
- Machine 4 (connection states) of `doc/design/spec/control.md`; states and inputs from 005-15. The transition table (005-16) needs exact guards for the health transitions; the constants table now fixes them:
  - **Miss count for Interrupted (C-38, C-102):** 8 consecutive slices absent at the deadline (missing, or arriving late); a marked gap or a slice on another timeline epoch proves the upstream alive and resets the count. At the 0.5 ms slice (C-02) that is 4 ms.
  - **Late versus missing (C-108):** an absent slice is counted missing at the deadline; if it arrives later within the ring span it is reclassified as late (missing −1, late +1) and discarded.
  - **Dead (C-39, C-111, C-135):** 2 s counted from the last slice received with content, a marked gap, or on another timeline epoch (an other-epoch slice restarts the dead clock, does not count as a miss, and has its own counter).
  - **Aligning (C-50):** lasts 4 s; then the offset is set and go-live happens on the next slice boundary.
  - **Ratchet (C-53, C-54, C-101):** "persistently late" is at least one late slice in each of 10 consecutive 1 s windows (the 10 s dwell of C-54); it triggers a step of one slice period, ceiling A (C-55, C-96); when the gap between offset and A is less than one slice period, the step goes to A exactly (C-163). A step is a re-alignment like any other: it takes effect at a position the fan-in announces one lead (64 slices, C-43) ahead in its own output, never silently on the next round (C-215). A sink has no output to announce in, so it takes the step at its own position one lead (C-43) ahead, as it does for a changed A (C-225, C-214).
  - **Demotion (C-109):** an input persistently late with the offset already at A enters Demoted: treated as gapped, counted separately, cleared only by re-admission.
  - **Session (C-19):** assigned by the source per stream, bumped on restart and on timeline epoch change; a timeline epoch switch is announced in-band (L-2).
  - **Sleep (C-124):** a sleeping laptop edge has departed; it returns through normal admission.
  - **Clock lapse (C-162):** after the holdover claim lapses an endpoint's clock is Unlocked with time unusable; one-way figures and alignment claims stop.
- Open sub-questions owned by this ticket (epic 005):
  - consumer reaction to slices carrying a new session number (source restart) in Live or Interrupted; falsifier: a restarted source resumes at a stale offset or is never re-aligned;
  - exits from Locking, Aligning and Demoted when data stops or an end-of-stream marker arrives; falsifier: the totality check finds a pair with no defensible row, or a connection stays in one of these states forever;
  - reaction of a post-admission connection (Aligning, Live, Interrupted, Demoted) when an endpoint clock becomes unusable (C-162); falsifier: a connection keeps claiming alignment with an endpoint whose time is unusable, or has no row for the input.

## Work
1. Write each guard above in `control.md` (Machine 4) as a predicate over the state data of 005-15, with its C-id, and in the `guard` syntax of `control.toml`. For the ratchet, write the effect separately from the guard: when the guard holds, the new offset is recorded with a switch position one lead (C-43) ahead, announced in-band in the fan-in's output (C-215, L-2), and applied only at that position; in a sink the same position one lead ahead is used with no announcement (C-225).
2. Decide the three open sub-questions; record them in "Decisions on open sub-questions" with a `[decision]` tag and their falsifiers.
3. Give for each guard a boundary table (for example 7 versus 8 misses; a marked gap at the 7th; 1.999 s versus 2 s since the last content; an other-epoch slice at 1.999 s restarting the dead clock (C-135); 9 versus 10 late windows; offset one slice below A versus at A; offset half a slice below A stepping to A exactly (C-163)).

## Acceptance criteria
1. Each of the 6 guards (miss count, late reclassification, Dead, Aligning end, ratchet, demotion) is written once, cites its C-id, and is used verbatim by 005-16.
2. The boundary tables have at least 2 cases per guard with the expected transition.
3. All three open sub-questions have a recorded decision with a falsifier, or are listed in "Open items" with owner and falsifier.
4. No guard uses a hello-derived input (L-1).
5. The Dead guard restarts on an other-epoch slice (C-135) and the ratchet guard never leaves the offset above A (C-163).
6. The ratchet's boundary table shows the new offset applied at the announced position, 64 slices after the guard holds, and not before (C-215); for a sink it shows the offset applied at its own position 64 slices after the guard holds, not before, with no announcement output (C-225).

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier
- [ ] Design documents updated where behaviour differs, constants table first

## Out of scope
- Writing the full transition tables (005-16).
- Data-plane counting and ring sizing (other epics).
