# 006-11 — Consumer-side procedures: single input, fan-in, sink
**Epic:** 006-Timeline-epoch-specification · **Effort:** S · **Depends on:** 006-09
## Context
Part of `doc/design/spec/timeline.md` (epic 006), section "Consumer-side procedures". Contributes to system criterion AC-18 (006-AC-5): a fan-in never aligns across timeline epochs; an input on another epoch is treated as a gap until it switches. Provides the fan-in rule to epic 009.

Facts (`[decision]`):
- The timeline epoch (16 bits, C-10) is carried in every slice header and every sync reply.
- A source announces a switch position (lead 64 slices, C-43; re-issued every 16 slices, C-44, in every bundle member) and at that position ends its old session and starts a new one with a bumped session number (C-19) and the new epoch.
- **At a fan-in:** an input that is still on another epoch is treated as a **gap** until it switches. A fan-in never combines samples from two epochs.
- An output at position `p` is degraded if its input at `p − L` was gapped (C-122); L is always a whole number of domain-rate samples (C-105), so `p − L` is a position. So an output that treats an other-epoch input as a gap carries the degraded flag.
- Every other gap rule applies to an other-epoch input as well: transport state is held, a stopped transport staying where it is and a playing one moving on at its tempo (C-141, C-200), and before any has arrived it is stopped, playhead at zero, 120 beats per minute (C-184).
- **At a sink:** the audition offset A applies per epoch; `emit(p) = T(p) + A` uses the line of the epoch the slice carries (T is the line's `time_of`). Alignment rings are sized to A (C-96).
- A re-emitting module is the source of its own output stream: it assigns its own session numbers (C-19) and follows 006-09 for its own switch.
- The map is affine per epoch and never extrapolated across epochs.
- Consumers process positions in order.

## Work
1. **Single-input module** (one input, e.g. a processor): on receiving an announcement, record (epoch, switch position); state how its own output switch (per 006-09, triggered by its own clock thread) relates to the input's switch, and what it does with slices whose epoch differs from both the current and the announced one.
2. **Fan-in** (mixer and similar): define "the module's epoch" (the epoch of its own clock thread); for each input, compare the slice's epoch with it; an input on another epoch contributes a gap until it is observed to switch, and the output slice is flagged degraded (C-122); never combine two epochs in one output slice; transport state from that input is held as on any gap (C-141, C-200), or defaulted if none has arrived (C-184). Specify the output's own switch (it is itself a source of its output stream and follows 006-09).
3. **Sink**: `emit(p) = T_e(p) + A` where `T_e` is the line of the epoch `e` carried by the slice; state what happens to slices of the old epoch still queued after the switch, and to slices whose epoch line is unknown.
4. Give one worked example per consumer type with explicit positions.
5. Refer to 006-21 for the health accounting (interrupted, dead, counters) of an input on another epoch; do not restate it here.

## Acceptance criteria
1. The fan-in rule is stated verbatim: an input on another epoch is treated as a gap until it switches; a fan-in never combines samples from two epochs (system AC-18).
2. The degraded flag of a fan-in output with an other-epoch input is stated with C-122.
3. The sink rule `emit(p) = T(p) + A` with `T` the line of the slice's epoch is stated, with the handling of old-epoch slices queued after the switch.
4. Each of the 3 consumer types has a worked example with numeric positions.
5. The section states how a consumer behaves on an unknown epoch (neither current nor announced).

## Definition of done
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] No new open item without an owner and a falsifier

## Out of scope
- The ordering and loss-survival argument (006-12).
- Health accounting of an other-epoch input (006-21).
- Fan-in implementation (epic 009); the value of the audition offset A (C-55, commissioning).
- Scenario scripts (006-16..18).
