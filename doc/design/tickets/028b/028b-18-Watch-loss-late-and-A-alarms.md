# 028b-18 — `watch`: loss, late-slice and A alarms
**Epic:** 028b-Commissioning-survey-certification-and-operations · **Effort:** S · **Depends on:** 028b-16, 028b-17

## Context
`watch` raises post-hoc alarms from regime stamps. Producer and consumer publish per-connection summary counters in their stamps (C-47, completed by C-159): received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead and observed DSCP; receiver reports are not visible to this tool. The runtime raises a connection alarm on refusal, interrupted, dead, demoted, cannot-meet-A and control-degraded (C-173). Alarms besides trunks:
- **Loss above threshold**: the `missing` counter rate above C-91 (10 × the clean-wire loss measured at L1, proposal C-167; expected floor about 1.1e-8 per default packet of about 1 320 bytes, C-90) is a cable-health alarm. A missing slice later reclassified as late (missing −1, late +1, C-108) is not a loss.
- **Late slices**: any increase of the `late` counter.
- **Sinks not meeting A (settled, C-101, C-109, C-174)**: two runtime conditions, both raised by the runtime as connection alarms (C-173) and read here from the stamps, since 10 consecutive 1 s windows cannot be judged from a 10 s stamp refresh (C-18):
  - **cannot meet A**: maximum lateness + D + headroom exceeds A in each of 10 consecutive 1 s windows (C-174), the lateness figures using the stamp's rolling 60 s window (C-182). A consumer that received an A larger than its buffer memory, a sink included (C-222), keeps the deepest ring it can hold as its ceiling, raises this alarm and keeps its streams running (C-211); the alarm stays raised until a later change of A that fits, and the consumer's receiver report carries the A it was given, not the clamped value (C-216);
  - **Demoted**: the input was persistently late (at least one late slice in each of 10 consecutive 1 s windows, C-101) with its offset already at the ceiling A, so it is treated as gapped, counted separately (the `demoted rounds` counter, C-159), and cleared only by re-admission (C-109).

## Work
- Alarm `loss.above_threshold` per connection/link, using C-91 once it has a value (before that, report against the C-90 floor as information).
- Alarm `slices.late` per connection with count and window.
- Alarm `sink.a_not_met` per sink with an input under a cannot-meet-A or demoted connection alarm (C-173, C-174), naming group, A, the sink, the input and which condition; it clears when the runtime clears the connection alarm (for Demoted, on re-admission; for a clamped consumer, after a later change of A that fits, C-216).
- Fixture tests for each raise and clear.

## Acceptance criteria
1. A fixture with a `missing` rate above C-91 on one link raises `loss.above_threshold` naming it; a missing slice reclassified as late does not.
2. A fixture with late counts raises `slices.late`.
3. A fixture where a sink's input enters Demoted raises `sink.a_not_met`; re-admission clears it. A fixture with a cannot-meet-A connection alarm raises `sink.a_not_met` naming that condition; for a clamped sink (C-222) the alarm stays raised until a fixture step applies an A that fits (C-216).
4. No alarm on a clean fixture.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Regime stamp fields touched are complete and documented

## Out of scope
- Trunk alarm (028b-17). Measuring C-91 (028b-16).

## Notes
- Which stamp field carries the runtime's connection alarms (C-173), including demoted and cannot-meet-A, is for epic 013 to specify (part of the remaining stamp-field gap). The `demoted rounds` counter itself is settled (C-159).
