# 029-01 — Crate skeleton and observation types
**Epic:** 029-Monitor · **Effort:** S · **Depends on:** epic 013, epic 014, epic 015

## Context
- Epic 029 builds `hadronyche-monitor`, an **optional** (v1) terminal monitor that shows the live state of a Hadronyche Medianet. It is strictly a **read-only consumer** of what the runtime already publishes: regime stamps (with per-connection summaries, C-47, C-159) and hellos. It adds no measurement of its own and can never affect a stream. `[decision]`
- Rule `[decision]`: numbers are produced by the runtime with a stamp; tools only display them. The monitor writes no measurement files.
- Tool binaries live in the `hadronyche-tools` crate; all protocol types (stamps, control messages, hellos) live in `hadronyche-core` (C-133). The monitor defines only read-only views over them.
- Everything the monitor knows enters through one interface given by the epic:

```rust
/// Everything the monitor knows comes through this; it has no other input.
pub enum Observation {
    Hello(HelloView),
    Stamp(StampView),
}
pub trait ObservationSource { fn next(&mut self) -> Option<Observation>; }
```

- Receiver reports are not an observation: they are unicast from consumer to producer, never multicast, and there is no query message (C-40, C-47). Stream health comes only from the per-connection summaries in stamps. The epic dropped the former `ReceiverReport(ReportView)` variant for this reason.
- Inputs come from other epics: the regime stamp wire object (epic 013), the receiver report format (epic 014, only for the meaning of the gap-class counters the stamp summaries publish), the hosted discovery socket and stamp collector (epic 015).
- Whether the monitor is built in v1 at all is an open question owned by Product, decided at the start of P7 (falsifier: operators manage with `survey` and `watch` alone). This ticket should not start before that decision is "yes".

## Work
1. Add binary target `hadronyche-monitor` to crate `hadronyche-tools` (C-133); create the crate only if no earlier epic has. Allowed extra dependency: `ratatui` (added in 029-10, not here).
2. Module layout (empty stubs where later tickets fill them): `collect.rs`, `model.rs`, `views/`.
3. Define in a module `observation` (re-exported from the crate root) the `Observation` enum and `ObservationSource` trait exactly as above.
4. Define `HelloView` and `StampView` as read-only projections of the `hadronyche-core` hello (epic 015) and regime stamp (epic 013) types (no copies of wire layout; construct from the decoded upstream types).
   - `HelloView`: module id and instance nonce (C-11), presence, clock holder, alarm field, control-degraded mark, stamp version number (C-18).
   - `StampView`: Identity (protocol version, build identity, configuration, domain, profile ∈ {linux-std, macos-std, windows-std, embedded}); Capabilities, each in one of four states present / absent / unverified / not requested (C-129): receive timestamp source with its value kernel / userspace / mac-hardware / interrupt (C-131), real-time scheduling granted, memory locked, drop accounting, DSCP marking, fault injection, holder-eligible, control isolation (C-161); Network (interface, interface MTU, path MTU per connection, packets per slice, pacing mode, DSCP classes and peer-observed values, including the DSCP the holder observes on each follower's sync requests and each module observes on each peer's hellos, C-152, coalescing state); Clock (discipline mode, lock state and bound as published by the servo core, epic 008a, C-187, timeline epoch, and at the holder the sync requests dropped above 200 per second, counted per follower, C-204); Alignment (offset, lateness percentiles 50th / 99th / 99.9th / maximum (C-160) and reclaimable latency (C-157), both over a rolling 60 s window, the maximum lateness since the connection went live (C-182), A, required E); Platform (OS build, audio device, driver, cost calibration factor C-106, CPU idle-state restriction C-127); Drops (per-socket overflow, interface drop counters, measurement-buffer overflow); Injection (any active injection, including the test-only hooks of C-120 and C-145); Commissioning (declared commissioning window with its expiry, C-107, C-153); Connections: per connection its session number (C-19), the counters received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159) and the observed DSCP (C-47).
   - Fields the epic still lists as "where published" open questions (connection state, the Demoted health state, admitted and link rate per port and direction, sink D / E / certification state, check-family sample counts, election epoch) are `Option`s, `None` when the epic 013 stamp does not carry them.
5. Command-line skeleton: `hadronyche-monitor [--file <recorded stamp file>] [--headless]`; arguments parsed, modes stubbed.
6. Add a `VecSource` (test-only `ObservationSource` over a `Vec<Observation>`) for later tickets' unit tests.

## Acceptance criteria
1. `cargo build -p hadronyche-tools` produces a binary `hadronyche-monitor`; `hadronyche-monitor --help` lists `--file` and `--headless`.
2. `Observation` and `ObservationSource` compile with exactly the signature above; no receiver-report type exists in the crate.
3. A unit test builds one `StampView` from an epic 013 stamp value and reads back a field from each group, one capability in each of the four states (C-129), and one per-connection summary with all eleven C-159 counters and the observed DSCP (C-47).
4. A unit test drains a `VecSource` of 3 observations and then gets `None`.
5. The crate contains no code that opens a file for writing.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Each View type has rustdoc naming the upstream `hadronyche-core` type (epic 013/015) it projects
- Not applicable (C-125): "Regime stamp fields touched are complete and documented" — this ticket adds or changes no stamp field; it only projects the fields epic 013 defines.

## Out of scope
- Network or file input (029-03, 029-04), stamp section reassembly (029-02), the model (029-06), any UI (029-10 onward).
- Changing the stamp wire object itself (epic 013).
