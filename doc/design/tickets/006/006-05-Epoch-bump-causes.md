# 006-05 — Epoch-bump causes: detector, publisher, event
**Epic:** 006-Timeline-epoch-specification · **Effort:** S · **Depends on:** 006-04, epic 005
## Context
Part of `doc/design/spec/timeline.md` (epic 006), section "Epoch-bump causes". Satisfies 006-AC-2: every bump cause has a detector, a publisher and a resulting event.

Facts (`[decision]`):
- A **timeline epoch** is a 16-bit counter (C-10), distinct from the *election* epoch: a continuous clock handover does not change it; a discontinuity does.
- The epoch is carried in every sync reply and in every slice header.
- **What bumps it:**
  1. a partition merge, for the losing side (the two timebases have diverged);
  2. a new holder that was not locked when it took the clock. When no candidate is locked, the holder-eligible candidate with the lowest module id wins and bumps the timeline epoch (C-113);
  3. a holder that detects its own stall, for example its host was suspended. Detection (C-136): the holder compares its monotonic clock with a clock that keeps running through suspend; a divergence above the step threshold (2 ms, C-67) bumps the timeline epoch;
  4. a step larger than the servo's step threshold (2 ms, C-67), which is never slewed.
- A step of the local wall clock is not cause 4: it is detected in the platform backend that builds each exchange from receive stamps, and the servo only ever receives exchanges that passed that check (C-121).
- **Who announces:** the holder is a clock server, not a stream source, so it cannot announce in-band. It answers sync requests on its clock thread, never on housekeeping (C-188), so the publisher of a bump is the holder's clock thread. Each module's clock thread learns the new epoch and line from sync replies. Each of that module's sources then announces in-band the future position at which it switches (procedure in 006-09).
- Every affected module gets a timebase-discontinuity event carrying the step size (event specified in 006-06).
- Related timing: election window 400 ms (C-33), hello period 1 s (C-30), election on reply silence 1.5 s (C-37), holdover bound growth 1 µs per second (C-64).

Settled since the split: the holder stall threshold (former Q-7) is C-136, above.

Still open, not decided here (the section cites it as an open item):
- Q-6, how the losing side of a partition merge is determined: owner epic 005 (control state machines); falsifier "both sides of a healed partition keep their timeline epoch, or both bump".

## Work
1. Write a table with one row per cause and columns: cause; detector (which module/thread observes it, and on what evidence); publisher (who increments the epoch and puts it in sync replies: the holder's clock thread, C-188); new origin rule (reference to the rule written by 006-04); resulting event at each follower (timebase-discontinuity event, then source switch).
2. For cause 1, cite the losing-side rule from epic 005 if it exists when this ticket runs; otherwise cite Q-6 as open with its owner and falsifier.
3. For cause 2, state the C-113 rule: the winner is the holder-eligible candidate with the lowest module id, it is the detector (it knows it is unlocked) and the publisher.
4. For cause 3, state the C-136 rule: the holder itself is the detector (monotonic clock against a clock that runs through suspend, divergence above 2 ms, C-67) and the publisher.
5. For cause 4, state who measures the step (the follower's servo, on exchanges that passed the backend's wall-clock step check, C-121) and that a step > 2 ms is never slewed, while a step ≤ 2 ms is slewed and does not bump.
6. State explicitly what does **not** bump: a continuous (locked) handover, including a takeover by a candidate that was locked to the old holder and is in holdover when it takes the clock (C-195, C-212; detailed in 006-07) and a local wall-clock step filtered by the backend (C-121).

## Acceptance criteria
1. The table has exactly 4 cause rows, and each row has a non-empty detector, publisher and resulting event (006-AC-2).
2. Cause 2 cites C-113; cause 3 cites C-136 and C-67; cause 4 cites C-67 and C-121.
3. The section states the step threshold 2 ms (C-67) and the rule "> 2 ms bumps, never slewed".
4. The section states that the election epoch and the timeline epoch are independent, with one example where the election epoch changes and the timeline epoch does not.
5. Q-6 appears either as answered (with the source of the answer) or as open with owner and falsifier; the stall threshold cites C-136 and is not listed as open.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The discontinuity event payload (006-06).
- Source-side announcement and switch (006-09).
- Sequence diagrams of the causes (006-13, 006-14, 006-15).
- Election algorithm and servo design (epics 005 and 008).
