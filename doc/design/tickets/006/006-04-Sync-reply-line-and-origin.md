# 006-04 — Sync reply contents and choice of epoch origin
**Epic:** 006-Timeline-epoch-specification · **Effort:** S · **Depends on:** 006-02, 006-03, epic 004
## Context
Part of `doc/design/spec/timeline.md` (epic 006), section "Sync reply and line reconstruction". Every follower must reconstruct the same line from the holder's sync replies. A *line* is the affine map of one timeline epoch: `position = (network_time − epoch_origin) × 96 000 Hz` (`[decision]`), represented as `Line { epoch: TimelineEpoch, origin_ns: i64 }`; the rate is the domain rate, the only rate on the wire (C-01), so it is not carried.

Facts (`[decision]`):
- There is one timeline for the whole domain, at 96 000 Hz (C-01). There is no line at any other rate; edges convert at the edge (C-80).
- The holder is a clock server, not a stream source. It answers sync requests on its clock thread, never on housekeeping (C-188), so the epoch and origin in a reply come from the clock thread. Each module's clock thread learns the new epoch (and the new line) from sync replies.
- The timeline epoch (16 bits, C-10) is carried in every sync reply.
- When no candidate is locked, the holder-eligible candidate with the lowest module id wins the election and bumps the timeline epoch (C-113).
- The sync reply fields are defined by epic 004; this ticket states what they must carry.

It answers the epic's open sub-question:

| Id | Question | Owner | Falsifier |
|---|---|---|---|
| Q-3 | How is `epoch_origin` chosen at cold start and after a bump? | This ticket | Two modules reconstruct different lines from the same sync replies |

## Work
1. List what a sync reply must carry for a follower to reconstruct the line: the timeline epoch number, the epoch origin (`origin_ns`, i64 ns of network time), and the holder identity. State why each is needed (identity: to discard replies from a former holder; epoch: to detect a bump; origin: to compute positions).
2. State that one reply gives one `Line`, valid for every stream of the module, since all streams are at the domain rate (C-01).
3. Specify the choice of `epoch_origin`:
   - at cold start (first holder of a domain);
   - after each bump cause (partition merge, unlocked takeover per C-113, holder stall, step > 2 ms C-67).
   The rule must be a function of data in the sync reply only, so two modules cannot derive different lines. Consider (and decide, `[decision]`) whether the new origin must keep positions monotonic across the bump or may restart.
4. Specify that a reply whose epoch is older than the module's current epoch (serial comparison from 006-03) is ignored, and that a reply from a non-current holder is ignored.
5. List the field requirements as a request to epic 004 (a short table: field, width, meaning), without defining the wire layout.

## Acceptance criteria
1. The section lists exactly which sync reply fields are required (epoch, origin, holder identity, plus any other the rule needs) with widths: epoch 16 bits, origin 64 bits signed; no rate field.
2. The origin rule is written for cold start and for each of the 4 bump causes.
3. A worked example shows two followers receiving the same two sync replies (before and after a bump) and computing the same `Line` and the same position for one network time.
4. Q-3 is closed, or left open with owner and falsifier.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Wire layout of the sync reply (epic 004).
- Who detects each bump cause and what is published (006-05).
- Servo and election behaviour (epics 005 and 008).
