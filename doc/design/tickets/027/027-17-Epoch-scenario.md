# 027-17 — Epoch scenario
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-11, epic 005, epic 017, epic 020

## Context
- L3 rig: Linux sink on capture channel 1, embedded device on channel 2; skew = ch2 − ch1, inter-channel skew subtracted. Pass rule (C-79): p99.9 |skew| ≤ E_linux + E_device, max reported.
- Timeline: `position = (network_time − epoch_origin) × rate`. A timeline step is never slewed: it is an announced timeline-epoch change; both sinks restart at the announced position. Announcement lead 64 slices (C-43); re-issue every 16 slices in every bundle member until the named position passes (C-44); slice = 0.5 ms (C-02), so the lead is 32 ms. The source bumps the session number on the timeline epoch change and every consumer sees the same value (C-19); the stamps fetched before and after (027-22) show it.
- A slice on another timeline epoch is not a miss and resets the interrupted count (C-102); it also restarts the dead clock and is counted in its own other-epoch counter (C-135, C-159). So the step must not produce interrupted or dead events; any slice arriving across the step shows in the other-epoch counter.
- **027-AC-5, Epoch:** after an announced timeline step both sinks resume within the limit; neither plays content aligned across the two epochs.
- Two items are still open in the epic: how a test triggers an announced step on demand (owner epic 005; the C-120 hook "force a stream onto a given timeline epoch" is not an announced step), and how the marker content identifies its epoch so the capture can tell pre-step from post-step markers (owner epics 017 and 020). This ticket uses what those epics provide.
- The announced step is 5 ms (C-191, proposal pending first runs).

## Work
1. `RigControl::announce_epoch_step(delta)` — trigger an announced step by the mechanism epic 005 defines; `delta` defaults to 5 ms (C-191).
2. Scenario `epoch`: baseline, step, post-step window. Log the announced position, the session number before and after, and the observed re-issues.
3. Analysis: skew series before and after the step computed separately; check that no channel plays post-epoch content while the other plays pre-epoch content at the same capture instant, using the marker epoch identity provided by `MarkerPlan` (epics 017, 020).
4. Report pre/post p99.9 |skew| and the cross-epoch check result.

## Acceptance criteria
1. **027-AC-5.** Post-step `p999_abs_ns <= limit_ns`.
2. **027-AC-5.** Zero capture instants where channel 1 and channel 2 play markers from different epochs aligned as if they were the same content.
3. Announced position, step size (5 ms by default, C-191), lead (64 slices, C-43), re-issue count observed (every 16 slices, C-44) and the session numbers before and after (C-19, same on both sinks) are in the report.
4. Neither sink's stamp shows a new interrupted or dead event across the step (C-102, C-135); the change in each sink's other-epoch counter is reported (C-159).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] The skew analysis runs in CI on recorded two-channel files
- Not applicable (C-125): "Every parser touched is fuzzed" — no new parser; "Regime stamp fields touched" — reads stamps only through 027-07/027-22; "Design documents updated" — 027-20; "No new open item" — opens none (both dependencies are owned epic sub-questions); "The rig is photographed or diagrammed" — 027-09; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Implementing epoch announcement in the runtime (epic 005 and others).
- Defining the marker epoch identity (epics 017, 020).
