# 029-13 — Connections panel
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-08, 029-09, 029-10

## Context
- `hadronyche-monitor` (epic 029) panel **Connections**: state, offset, late / missing / marked / interrupted counts, dead, path MTU, DSCP as observed.
- Connection states: Requested, Refused (with reason, for example RequiredEExceeded, C-103), Pending (target not present; no timeout, C-45), Admitted, Locking, Aligning, Live, Interrupted, Dead, Ended. Health state Demoted (C-109) is shown beside the state. Only Live is satisfied; Pending, Locking, Aligning, Interrupted and Demoted are shown by name as unsatisfied (C-154).
- Gap classes: marked (upstream alive, loss further up), late (after deadline; remedy is budget), missing (unknown), dead (classified). Interrupted after 8 consecutive missed slices (C-38, C-102); dead after 2 s without content or marked gap (C-39, C-111). Missing can fall by one when a slice is reclassified late (C-108). A slice on another timeline epoch is counted as other-epoch, not as a miss (C-135).
- Per-connection summaries come from both endpoints' stamps: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159), and the observed DSCP (C-47). Interrupted and dead describe the receiving side; a sink-only device's liveness is judged by the producer timeout (C-41, C-168). Each endpoint's figures are shown separately with its profile; there is **no** last-writer-wins aggregate (the POC bug at `stats.rs:324` `[code]`).
- One-way figures are blank until lock and carry the bound afterwards (029-08). Lateness percentiles (50th, 99th, 99.9th, maximum, C-160) and reclaimable latency (C-157) come from the stamp's alignment group; both are over a rolling 60 s window, and the maximum since the connection went live is published beside them (C-182). Path MTU per connection and configured DSCP classes come from the Network group of the stamp; observed DSCP from the C-47 summary.

## Work
1. `views/connections.rs` implementing `Panel`: one row per (connection, endpoint side) with producer, consumer, both profiles, session number, state (with refusal reason; every state other than Live, and Demoted health, marked "unsatisfied" beside its name, C-154; Pending with its age), health, offset, lateness percentiles and reclaimable latency labelled as over the last 60 s, the maximum since live beside them (C-182), the eleven C-159 counters (the counters other than received, late, missing, marked, interrupted and dead may sit in a detail row for the selected connection), path MTU, DSCP configured vs observed, one-way latency via `Shown`, stamp age.
2. Highlight rows whose DSCP observed differs from configured.
3. Show a "new series" marker on rows whose regime or session changed (029-06) and never display a figure spanning the change.
4. Text capture for documentation.

## Acceptance criteria
1. Two return streams with missing 0 and 17 render as two rows showing 0 and 17 (epic 029-AC-4, render level).
2. Before lock the one-way column shows a blank marker (not 0); after lock it shows value ± bound.
3. A Refused connection shows its reason text (RequiredEExceeded in the fixture).
4. Pending, Locking, Aligning and Interrupted connections and a Demoted input each show their name and "unsatisfied"; a Live connection does not (C-154).
5. Path MTU 1 280 after a fallback is shown with the new-series marker.
6. DSCP mismatch row is highlighted.
7. A fixture with non-zero other-epoch, too-early, duplicates, incomplete reassemblies and demoted rounds shows each value under its own label (C-159).
8. Percentiles and reclaimable latency carry a "60 s" window label, and the maximum since live is shown beside them under its own label, distinct from the 60 s maximum (C-182).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] A screenshot or text capture of each panel is in the tool's documentation
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Bundle member counters (029-14); end-to-end gap-class test (029-21); alarm names (029-25).
