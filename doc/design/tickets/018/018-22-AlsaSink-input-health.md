# 018-22 — AlsaSink input health: missing, late, interrupted, dead, Demoted
**Epic:** 018-Linux-ALSA-edge · **Effort:** S · **Depends on:** 018-13, epic 009

## Context
- The sink takes position p at a deadline fixed by A and D (018-13); absent slices are zero-filled. The gap classes and the health state machine are epic 009's; this ticket applies the settled rules in the sink.
- **Late versus missing (C-108):** at the deadline an absent slice is counted missing; if it arrives later while its position is still within the ring span it is reclassified as late (missing −1, late +1) and discarded.
- **Interrupted (C-38, C-102):** after 8 consecutive missed slices (4 ms). Only slices absent at the deadline count as misses (missing, or arriving late); a marked gap or a slice on another timeline epoch proves the upstream alive and resets the count.
- **Dead (C-39, C-111):** 2 s counted from the last slice received with content or a marked gap.
- **Other epoch (C-135):** a slice on another timeline epoch restarts the dead clock, does not count as a miss, and has its own counter.
- **Counters (C-159):** received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead. Too-early, duplicate and incomplete-reassembly classification comes from the alignment ring and reassembly of epic 009; this ticket counts them per connection.
- **Alarms (C-173):** a connection alarm is raised on interrupted, dead and Demoted, through the alarm path of epic 015.
- **Ceiling of a sink (C-226):** A − D, since the sink hands position p to the interface at T(p) + A − D; the offset never grows beyond it.
- **Demoted (C-101, C-109):** an input persistently late (at least one late slice in each of 10 consecutive 1 s windows) with its offset already at the ceiling enters Demoted: treated as gapped, counted separately, cleared only by re-admission.
- **Ratchet in a sink (C-225, with C-53, C-163, C-214):** below the ceiling, a persistently late input steps the sink's offset by one slice instead (to the ceiling exactly when less than one slice remains). The sink has no output to announce in, so it takes the step at its own position one lead (C-43: 64 slices) ahead, as it does for a changed A; it uses epic 009's ratchet (009-16) and carries its `announce` nowhere.

## Work
1. Classify each position at its deadline as received, marked gap or missing; reclassify a later arrival within the ring span as late and discard it (C-108).
2. Interrupted counter per C-38 and C-102; dead timer per C-39 and C-111; other-epoch slices per C-135 (restart the dead clock, reset the miss count, own counter).
3. Count too-early, duplicates and incomplete reassemblies as reported by the epic 009 ring (C-159).
4. Persistently-late detector over 1 s windows (C-101), through epic 009's ratchet (009-16): on trigger below the ceiling, step the offset by one slice at the sink's own position one lead ahead, with no announcement (C-225); on trigger at the ceiling, Demoted per C-109, zero-fill, separate counter, cleared only by re-admission.
5. Raise a connection alarm on entering interrupted, dead and Demoted (C-173).
6. Expose the full C-159 counter set, including demoted rounds, for the stamp (018-23).

## Acceptance criteria
1. **Epic criterion 018-A9 (sink part):** a slice arriving after its deadline but within the ring span moves one count from missing to late and is not played.
2. 8 consecutive missing slices → Interrupted; 7 missing then a marked gap then 7 missing → not Interrupted; a slice on another epoch also resets the count.
3. Marked gaps only, for 3 s → not Dead; no slice and no marked gap for 2 s → Dead.
4. With the offset below the ceiling, one late slice in each of 10 consecutive 1 s windows → the offset grows by one slice from the position one lead (64 slices) ahead of the sink's position at the trigger, positions before it keep the old offset, no announcement is emitted and the input stays Live (C-225); with the offset at the ceiling, the same pattern → Demoted; 9 windows then a clean window → neither; Demoted stays until re-admission.
5. Slices on another timeline epoch only, for 3 s → not Dead, not Interrupted, and the other-epoch counter equals the number received (C-135).
6. Entering interrupted, dead and Demoted each raises one connection alarm on the fake alarm path (C-173).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- The alignment ring and generic health state machine (epic 009).
- Publishing the counters (018-23).
