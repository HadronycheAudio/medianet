# 005-09 — Election states and inputs
**Epic:** 005-Control-state-machines-specification · **Effort:** S · **Depends on:** 005-02, 005-08, epic 004

## Context
- Machine 2 of `doc/design/spec/control.md`: clock election. Pure table, no I/O, in `hadronyche-core`.
- Facts:
  - the holder's monotonic clock defines network time; one holder per medianet;
  - candidates: holder-eligible capability bit, usable time (Locked, or in holdover with the alignment claim not lapsed, C-213, C-65, C-169), host not under I/O pressure; embedded, macOS and Windows nodes do not set the bit in v1;
  - with no candidate with usable time, the lowest eligible module id wins and bumps the timeline epoch (C-113); this is the cold-start case and the only election that bumps the timeline epoch (C-213);
  - trigger: 1.5 s without sync replies at a follower (C-37, L-3, L-6); **not** hello expiry; also the holder comes under pressure, or an operator forces it;
  - no rotation: the holder changes only on loss or pressure;
  - election window 400 ms (C-33); deterministic winner (005-08): among several candidates with usable time, a Locked candidate is preferred over one in holdover, and among equals the lowest module id wins (C-221);
  - merge: higher election epoch wins, rank not compared across epochs;
  - handover to a candidate with usable time is frequency-continuous and keeps the timeline epoch (C-213); if no candidate has usable time, the new holder bumps the timeline epoch (announced discontinuity, `TimelineEpochChanged`; every stream's session number is bumped, C-19);
  - a holder that detects its own stall bumps the timeline epoch; stall detection is settled (C-136): the holder compares its monotonic clock with a clock that keeps running through suspend, and a divergence above the 2 ms step threshold (C-67) is a stall; the comparison runs on the clock thread and reaches the machine as a local input;
  - a module whose holdover claim lapses is Unlocked with time unusable (C-162) and leaves the set of candidates with usable time;
  - the holder answers sync requests on the clock thread, never on housekeeping, and records there the DSCP observed on each follower's requests, reported in its stamp (C-188, C-152);
  - the holdover entry delay is at most C-37 = 1.5 s, a hard limit, so a follower is always in holdover with a growing bound when it calls an election; the value within that limit is pending simulation (epic 008b) (C-195);
  - a holder serves at most 64 followers (C-35); the 65th is refused (`FollowerRefused`, `RefuseReason::FollowerCap`); with at most 64 modules the real maximum is 63, so the cap is exercised with synthetic followers (C-114); the value 64 is pending measurement at L1 (task 019) and is carried by id;
  - followers send sync requests at 50 per second (C-36); the holder accepts 10 to 200 per second per follower (C-112); requests beyond 200 per second from one follower are dropped before any work is done and counted per follower in the holder's stamp (C-204);
  - listen-before-claim 1.5 s (C-32).
- `[code]` POC behaviour to retire: rotation after 2 mandates (`MANDATES = 2`, `module.rs:62`); an election opened by `leave()` when the holder's hello expires (`module.rs:575-585`); text messages (`"HADRO/1"`).
- Still open, not owned here: the I/O-pressure eligibility rule's inheritance (task 002; falsifier: the POC experiment data behind it is not intact); the definition of the Locked state (the candidacy rule is settled by C-213) (servo core, epic 008a, which publishes the lock state and bound through the published time view, C-187; C-62 pending simulation); the operator-force message (epic 004).

## Work
1. In `control.md`, Machine 2 "States and data": define states (at least Listening, Follower, Electing (window open), Holder, Holder-handing-over) with data held: election epoch, timeline epoch, current holder id, candidate set (≤ 64; per candidate its module id and clock class, Locked or in holdover with the claim not lapsed, for the C-221 order), last sync reply time, follower list (≤ 64, C-35) with the DSCP observed on each follower's sync requests (C-152) and the count of that follower's dropped sync requests (C-204), own eligibility (bit, usable time per C-213, pressure), rate for frequency-continuous handover.
2. "Memory bound": bytes per state and the worst case with 63 peers (C-217) / 64 followers.
3. "Input alphabet": sync request (with the DSCP observed on receipt, C-152), sync reply, hello (election epoch and candidacy fields, including the clock class read by C-221), election/claim messages and operator force as named by epic 004; timers: reply-silence 1.5 s (C-37), election window 400 ms (C-33), listen-before-claim 1.5 s (C-32), sync request tick 20 ms (50/s, C-36); local: lock state change from the published time view (C-187; including entry into holdover and the lapse to Unlocked, C-162, which together decide usable time, C-213), I/O pressure on/off, own stall detected (C-136).
4. State that the sync-request and sync-reply rows run on the clock thread, never on housekeeping (C-188). State the sync-request acceptance rule at the holder: requests at 10 to 200 per second per follower are served (C-112); requests beyond 200 per second from one follower are dropped before any work is done and counted in that follower's dropped-request counter, published in the holder's stamp (C-204). Write the over-rate test as a guard on the sync-request input.
5. Add a note that the I/O-pressure rule is pending task 002 and the Locked state is defined by the servo core, epic 008a (C-187); usable time is C-213.
6. Mirror states and inputs into `control.toml` under machine `election`.

## Acceptance criteria
1. The hello-expiry timer is **not** an input of the election machine (L-6); the reply-silence timer of 1.5 s is.
2. Each state lists its data with sizes; worst-case memory with 63 peers (C-217) is one stated number of bytes.
3. The alphabet contains every trigger: reply silence (1.5 s), holder pressure, operator force, own stall.
4. No input or state implements rotation (no mandate counter).
5. The holder's accepted request range is 10 to 200 per second (C-112); the follower-list data includes a per-follower dropped-request counter (C-204).
6. `control.toml` declares the same states and inputs as the prose.

## Definition of done
- [ ] Every new factual claim carries an evidence tag
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The transition table (005-10) and model check (005-11).
- Clock servo and lock detection internals (other epics).
