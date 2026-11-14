# 006-12 — Announcement ordering and loss survival
**Epic:** 006-Timeline-epoch-specification · **Effort:** S · **Depends on:** 006-11
## Context
Part of `doc/design/spec/timeline.md` (epic 006), section "Announcement ordering and loss". Satisfies 006-AC-3 and 006-AC-4.

Facts (`[decision]`):
- An announcement names a future position, with a lead of 64 slices (C-43; 32 ms at defaults, slice = 48 frames = 0.5 ms at 96 kHz, C-02), and is re-issued every 16 slices (C-44) until the named position passes, in every bundle member.
- The lead is **flat** (C-43): it is the same 64 slices at every hop and is not scaled with graph depth. Each module learns a new epoch from its own clock thread and announces its own switch at its own current position + 64 slices, so the lead does not shrink along a chain.
- C-43 is decided, with the falsifier "a descendant misses an announcement despite four re-issues". The former open sub-question "does the 64-slice lead hold in a deep graph?" is therefore settled; L0 (task 017) confirms it empirically.
- Consumers process positions in order, so an announcement is always processed before the position it names; the lead exists only to survive loss (four transmissions).
- An input that is still on another epoch is treated as a gap at a fan-in until it is observed to switch.
- Header cap (C-137): a default slice carries at most 100 bytes for all headers and the checksum; in-band announcements fit inside that cap or are omitted on a fallback path.

## Work
1. **Ordering argument (006-AC-3):** show that an announcement carried in slice position `q < p` (named position `p`) is processed before position `p` at every descendant, using only in-order position processing along each edge and induction on graph depth, with no assumption about gossip or any side channel. State that a re-emitting module (single input or fan-in) announces its own switch in its own output with the full flat lead.
2. **Loss survival (006-AC-4):** the 4 transmissions are at 64, 48, 32, 16 slices before `p`; the announcement survives the loss of any 3 consecutive carrying slices. State the residual case: all 4 lost; outcome: the input is on another epoch and is treated as a gap until it is observed to switch (no misalignment, only a gap).
3. **Fallback path (C-137):** state that on a path where announcements are omitted, the consumer receives none of the 4 transmissions; the outcome is the residual case: the input is on another epoch and is treated as a gap until it is observed to switch. State that this is a gap, never a misalignment.
4. **Deep graph:** state, for a chain of depth `d`, the lead each hop has: 64 slices at every hop under the flat rule (C-43). Cite C-43's falsifier and task 017 as its confirmation.

## Acceptance criteria
1. The ordering argument is written as an induction over depth and contains no reference to gossip (006-AC-3).
2. The section states that loss of 3 consecutive carrying slices is survived and gives the residual case (4 lost) with its outcome "treated as a gap until it is observed to switch" (006-AC-4).
3. The fallback-path case (announcements omitted, C-137) is stated with the same outcome as the residual case.
4. The deep-graph rule states the lead at depth `d` for `d = 1, 2, 5` (64 slices each) and cites C-43.
5. The section cites C-43's falsifier and L0 task 017 as the confirmation; the deep-graph question is not listed as open.

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- Running the L0 confirmation (task 017).
- Scenario scripts, including the lost-announcement scenario (006-17).
- Conformance vectors themselves (epic 007).
