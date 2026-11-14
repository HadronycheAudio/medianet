# 029-11 — Medianet panel
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** 029-08, 029-10

## Context
- `hadronyche-monitor` (epic 029) panel **Medianet**: modules, profiles, holder, election and timeline epochs, alarms, control-degraded marks, active commissioning window.
- Sources: hellos (period 1 s, C-30: module id and nonce, who holds the clock, alarm field, control-degraded mark) and the Identity, Clock and Commissioning groups of each regime stamp (profile ∈ linux-std, macos-std, windows-std, embedded; timeline epoch; commissioning window, C-107).
- Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers.
- **Control-degraded:** the module's housekeeping thread was restarted; live streams are unaffected — the panel must not show it as a stream fault. After 5 panics within 60 s restarts stop: the module stays control-degraded with an alarm and accepts no new connection until an operator restarts it (C-139); the mark for this state is rendered by 029-25.
- Timeline epoch changes only on a timebase discontinuity (including an election won with no locked candidate, C-113, or a holder stall, C-136) and is distinct from the election epoch.
- Loss above the alarm threshold (C-91) is a cable-health alarm. Connection alarm kinds are listed in C-173 and rendered by name in 029-25.
- Open question in the epic, owned by epic 013 with epic 004: where the election epoch is published (hello or stamp). Working assumption: one of them; `NotPublished` otherwise.

## Work
1. `views/medianet.rs` implementing `Panel` (029-10): one row per module (up to 64, C-34) with module id, profile, present / absent, holder marker, alarm, control-degraded mark, commissioning mark, stale-stamp or no-stamp-yet mark and stamp age (029-06).
2. Header: current clock holder, election epoch, timeline epoch, count of modules present.
3. Control-degraded rendered as a neutral note ("control degraded — streams unaffected"), not in an error colour.
4. Produce a text capture with `render_to_text` for the documentation.

## Acceptance criteria
1. A fixture with 3 modules (linux-std holder, macos-std, embedded) renders 3 rows with correct profiles and exactly one holder marker.
2. Election epoch and timeline epoch are shown in separate labelled fields; changing one does not change the other.
3. A control-degraded module is labelled and its streams are not marked faulty.
4. A module stamping an open commissioning window carries the commissioning mark, with the window's expiry if the stamp carries it (C-153).
5. 64 modules fit with scrolling at 80×24.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] A screenshot or text capture of each panel is in the tool's documentation
- [ ] Every new factual claim carries an evidence tag

## Out of scope
- Clock details (029-12), regime stamp details (029-17), presence, departure and id-collision marks (029-24).
