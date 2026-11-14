# 029-14 — Bundles panel
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-09, 029-10

## Context
- `hadronyche-monitor` (epic 029) panel **Bundles**: per-member counters.
- A **bundle** is a group of narrow aligned streams from one wide source: 6 channels per member at defaults (C-04), up to 32 members, the controller member included (C-08, C-209), with per-member counters. A bundle source's controllers always ride in their own member stream (C-181, C-205): a source with controllers has at most 31 audio members (186 channels) plus the controller member.
- Counters come from the per-connection summaries in regime stamps: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159), and the observed DSCP (C-47). Receiver reports (one per bundle every 100 ms, C-40) are not visible to the monitor.
- Open question in the epic, owned by epic 005 (with epic 004 for the stamp layout): whether each bundle member is its own connection with its own summary. Working assumption: yes. If not, per-member counters do not exist and the panel shows `NotPublished` per member rather than inventing a split.
- Per-member figures are never merged into one bundle total that hides a bad member; a bundle-level total, if shown, is additionally broken down.

## Work
1. `views/bundles.rs` implementing `Panel`: bundle list; for the selected bundle, one row per member (index 0..31) with channels, received, late, missing, marked, interrupted, dead, state; the other C-159 counters (other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds) in a detail row for the selected member.
2. Mark degraded members (a slice computed from a gapped input; the flag lags the input by the module's L, C-122) if the stamp publishes it.
3. Text capture for documentation.

## Acceptance criteria
1. A 32-member bundle fixture renders 32 member rows (with scrolling at 80×24); a fixture of 31 audio members plus the controller member also renders 32 rows, the controller member with its own counters like any other (C-205, C-209).
2. One member with missing 3 and the others 0 shows 3 on exactly that member's row.
3. Members of a bundle whose regime changed show the new-series marker; no counter spans the change.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] A screenshot or text capture of each panel is in the tool's documentation
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Connection-level view (029-13).
