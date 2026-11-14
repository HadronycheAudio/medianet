# 029-17 — Regime panel
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-08, 029-10

## Context
- `hadronyche-monitor` (epic 029) panel **Regime**: the full regime stamp of a selected module, with **absent** and **unverified** capabilities highlighted.
- Stamp groups and fields:
  - Identity: protocol version, build identity, configuration, domain, profile (linux-std, macos-std, windows-std, embedded)
  - Capabilities, each in one of four states: present, absent, unverified, not requested (C-129). "Not requested" is a deliberate configuration choice, distinct from a denial. Fields: receive timestamp source (value kernel, userspace, mac-hardware or interrupt, C-131), real-time scheduling granted, memory locked, drop accounting, DSCP marking, fault injection, holder-eligible, control isolation (C-161)
  - Network: interface, interface MTU, path MTU per connection, packets per slice, pacing mode, DSCP classes and peer-observed values (sync-request DSCP per follower at the holder, hello DSCP per peer, C-152), coalescing state
  - Clock: discipline mode, lock state and bound (published by the servo core, epic 008a, C-187), timeline epoch; at the holder, sync requests dropped above 200 per second, counted per follower (C-204)
  - Alignment: offset, lateness percentiles (50th, 99th, 99.9th, maximum, C-160) and reclaimable latency (C-157), both over a rolling 60 s window, maximum lateness since the connection went live (C-182), A, required E
  - Platform: OS build, audio device, driver, cost calibration factor (time per abstract operation, C-106), CPU idle-state restriction (best-effort, C-127)
  - Drops: per-socket overflow, interface drop counters, measurement-buffer overflow
  - Injection: any active injection, always stamped, including the test-only hooks of C-120 and C-145
  - Commissioning: declared commissioning window, with its expiry (C-107, C-153); while open, ordinary production measurements are unqualified, commissioning results are not (C-178)
  - Connections: per connection, session number (C-19), the counters received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159), and the observed DSCP (C-47)
- An injected run must be visibly labelled as injected; so must a run inside a commissioning window.
- Stamps arrive on any regime change and every 10 s (C-18).

## Work
1. `views/regime.rs` implementing `Panel`: all groups for the selected module, field by field, scrollable.
2. Capability absent → error style; unverified → warning style; not requested → neutral style with the label "not requested"; present → normal. The receive timestamp source shows its value and its state.
3. Non-zero drop counters highlighted; active injection and open commissioning window each shown in a banner at the top of the panel.
4. Show stamp version, stamp age and whether it is stale or not yet received (029-06).
5. Text capture for documentation.

## Acceptance criteria
1. A fixture stamp renders every field listed above (test asserts each field label is present in the text capture).
2. "memory locked: absent", "drop accounting: unverified" and "real-time scheduling granted: not requested" render in error, warning and neutral style respectively.
3. "receive timestamp source: kernel (present)" renders with both value and state.
4. A stamp with an active injection shows the injection banner; one with an open commissioning window shows the commissioning banner.
5. Measurement-buffer overflow 4 is highlighted.
6. A holder stamp with dropped sync requests 12 for one follower and 0 for another shows each count against its follower (C-204).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] A screenshot or text capture of each panel is in the tool's documentation
- [ ] Every new factual claim carries an evidence tag
- Not applicable (C-125): "Regime stamp fields touched are complete and documented" — this ticket displays stamp fields and adds or changes none; their definition and documentation belong to epic 013.

## Out of scope
- Changing stamp contents (epic 013).
