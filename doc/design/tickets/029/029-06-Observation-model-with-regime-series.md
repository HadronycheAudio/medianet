# 029-06 — Observation model with regime series
**Epic:** 029-Monitor · **Effort:** M · **Depends on:** 029-01, 029-02

## Context
- `hadronyche-monitor` (epic 029) only displays numbers the runtime produced with a stamp. `[decision]`
- The proof of concept aggregated wrongly `[code]`: `stats.rs:324` takes the latest record's loss value across all return streams with no per-stream separation. A real run showed a "lost" counter of about 21.7 million on a run with zero loss `[measured]`. This aggregation must not be reused.
- Rules the model enforces `[decision]`:
  - A change of regime starts a new record set (series); the monitor never averages across it.
  - A measurement involving two endpoints carries both endpoints' profiles (linux-std, macos-std, windows-std, embedded).
- Per-connection summaries (C-47, C-159): producer and consumer each publish, in their own regime stamp, the counters received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159), and the observed DSCP (C-47). They arrive with each stamp: on any regime change and every 10 s (C-18).
- **Missing is not monotonic** (C-108): a slice counted missing at the deadline that arrives while still within the ring span is reclassified (missing −1, late +1). A lower missing count in a later stamp is not a reset and not a new series.
- **Session number** (C-19): one per stream, assigned by the source, bumped on restart and on timeline epoch change. A new session starts a new series for that connection.
- Bundles: a group of narrow aligned streams from one wide source, 6 channels per member at defaults (C-04), up to 32 members, the controller member included (C-08, C-209), with per-member counters. Whether each member is its own connection is an epic open question owned by epic 005 / 004; working assumption: yes.
- Election epoch and timeline epoch are distinct; the timeline epoch changes only on a timebase discontinuity, including an election won with no locked candidate (C-113).
- Open question in the epic, owned by epic 013 / 004: whether the stamp version changes only on a regime change. Working assumption: yes; the regime boundary is therefore taken from the version together with the regime-defining fields, never from a counter change alone.

## Work
1. `model.rs`: `Model` with `fn apply(&mut self, obs: Observation)`.
2. Keys: module id → module state (holding the instance nonce(s) seen, used by 029-24); (module id, connection id) → connection state; (bundle id, member index 0..32) → member state.
3. Each keyed entity holds a list of `Series`, each tagged with the regime identity (stamp version plus the stamp fields that define the regime) and, for connections, the session number. A new stamp whose regime differs from the current series' regime, or a new session number, opens a new `Series`; counters and percentiles are never combined across series. Any derived figure (rates, deltas) is computed within one series only.
4. Deltas of `missing` may be negative within a series (C-108); the model shows the published value and never clamps or treats a decrease as a reset.
5. Connection entries hold producer-side and consumer-side summaries separately, each with its endpoint's profile; no merged "connection loss" value.
6. Staleness: a module whose hello announces a newer stamp version than the model holds (029-02 `Freshness::Stale`) is flagged `stale`; a module with no stamp yet is flagged `no_stamp`. Each stamp keeps its observation time so views can show its age.
7. Keep the last hello per module (presence, holder, alarm, control-degraded, nonce) and its arrival time.
8. Expose read-only query functions used by views: `modules()`, `connections()`, `bundle_members()`, `current_series(key)`, `series_history(key)`.

## Acceptance criteria
1. Two connections from one producer with different `missing` values (0 and 17) are returned as two separate entries with 0 and 17; no API returns a single combined loss value (epic 029-AC-4, unit level).
2. A stamp with path MTU changed to 1 280 bytes (C-06) mid-run opens a new series for that module and its connections; the previous series is retained in history and no figure in the new series includes samples from the old one (epic 029-AC-3, unit level).
3. A stamp carrying a new session number for one connection opens a new series for that connection only.
4. A refresh in which missing drops by 1 and late rises by 1 stays in the same series and shows the new values.
5. Each connection entry exposes both endpoints' profiles.
6. A model fed 64 modules (C-34) and 32 bundle members (C-08) keys all of them distinctly.
7. A hello announcing a newer stamp version marks the module `stale`; arrival of that stamp clears it.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every new factual claim carries an evidence tag
- [ ] The rustdoc of `Model` states which stamp fields define a regime boundary
- Not applicable (C-125): "Regime stamp fields touched are complete and documented" — this ticket adds or changes no stamp field; it reads them.

## Out of scope
- Display-level derivations (clock: 029-07; blanks, failed, injected and commissioning labels: 029-08; connection state and gap classes: 029-09; id collision and departure: 029-24); rendering (029-10 onward).
