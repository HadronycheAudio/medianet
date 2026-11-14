# 029 — Monitor (optional)

**Phase:** P7 · **Scope:** L (25 tickets, about 27 days) · **Depends on:** 008a, 013, 014, 015, 017b, 019 · **Blocks:** none

## Goal

Build a terminal monitor (`hadronyche-monitor`) that shows the live state of a medianet to an operator or developer. It is **optional** for v1 and strictly a **read-only consumer** of what the runtime already publishes: regime stamps (which carry the per-connection summary counters, C-47, C-159) and hellos. It adds no measurement of its own and can never affect a stream. It does not see receiver reports (C-47).

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules exchange *slices* — all content of one stream for one stretch of media time — addressed by *position* on a domain-wide timeline. Every stream on the wire is at the domain rate of 96 kHz (C-01). The network is IPv4 only, with multicast at TTL 1 (C-100), so the monitor must sit on the medianet itself. Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers.

**Why read-only `[decision]`.** The proof of concept put its terminal UI, its statistics and its per-packet file output in the same process as the instrument. A real run then showed what that costs `[measured]`: a metrics file corrupted by four writers on one path, a "lost" counter showing about 21.7 million on a run with zero loss, and two whole check families reporting "0 of 0" while the run looked green. The runtime's rule since then: **numbers are produced by the runtime with a stamp; tools only display them.** Specifically:

- A file has one writer, owned by the process. The monitor writes no measurement files.
- A check family with zero samples is a **failed run**, and the monitor must show it as failed, not blank.
- One-way latency figures are blank until the clock is locked, and carry the clock's error bound afterwards.
- A change of regime starts a new record set; the monitor never averages across it.
- A measurement involving two endpoints carries both endpoints' profiles.

**What the monitor may consume.**

1. **Regime stamps** — a wire object defined in `hadronyche-core` (C-133) that every module publishes (devices cannot write files, so stamps travel on the network and a hosted collector records them). Transport (C-18): a dedicated control message split into sections that each fit the control message bound of 1 252 bytes of UDP payload (C-17); **multicast to the control group on any regime change and refreshed every 10 s**. A unicast request gives an immediate fetch, but the monitor never sends one: a freshly started monitor has every module's stamp within one refresh period (10 s) without sending anything. The hello carries only a stamp version number, which tells the monitor whether the stamp it holds is current. Groups of fields:

   | Group | Fields |
   |---|---|
   | Identity | protocol version, build identity, configuration, domain, profile (linux-std, macos-std, windows-std, embedded) |
   | Capabilities, each in one of four states: present, absent, unverified, not requested (C-129) | receive timestamp source (value kernel, userspace, mac-hardware or interrupt, C-131), real-time scheduling granted, memory locked, drop accounting, DSCP marking, fault injection, holder-eligible, control isolation (housekeeping cannot delay the data path, C-161) |
   | Network | interface, interface MTU, path MTU per connection, packets per slice, pacing mode, DSCP classes and peer-observed values (the holder reports the DSCP observed on each follower's sync requests; every module reports the DSCP observed on each peer's hellos, C-152), coalescing state |
   | Clock | discipline mode, lock state and bound (published by the servo core, epic 008a, C-187), timeline epoch; at the holder, the sync requests dropped above 200 per second, counted per follower (C-204) |
   | Alignment | offset, lateness percentiles (50th, 99th, 99.9th and the maximum, C-160), reclaimable latency (offset − (current maximum lateness + headroom) when positive, C-157), A, required E. The percentiles and reclaimable latency are over a rolling 60 s window, with the maximum since the connection went live reported beside them (C-182) |
   | Platform | OS build, audio device, driver, cost calibration factor (time per abstract operation, C-106, measured by epic 025, C-176), CPU idle-state restriction (best-effort, C-127) |
   | Drops | per-socket overflow, interface drop counters, measurement-buffer overflow |
   | Injection | any active injection, always stamped, including the test-only hooks (forced module id and nonce, forced timeline epoch, raw packet send, C-120; injected thread panic, C-145) |
   | Commissioning | a declared commissioning window, during which the commissioning tool may drive test traffic (C-107). The tool announces it with an expiry time; modules stamp it while open; it closes at expiry or on explicit close (C-153). While it is open, ordinary production measurements (latency, loss, alignment figures in stamps) are unqualified; the commissioning tool's own measurements (D, restart spread, group certification, soak) are valid commissioning results, each recorded with the window it was taken in (C-178) |
   | Connections (C-47, C-159) | per connection, published by the producer and by the consumer in their own stamps: received, late, missing, marked gap, other-epoch (C-135), too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159), and the observed DSCP (C-47) |

2. **Per-connection summaries in the stamp** (C-47, C-159) — the Connections group above. Receiver reports themselves (one per bundle every 100 ms, C-40) are unicast from consumer to producer and are **not** visible to the monitor; there is no multicast copy and no query message. The summaries are as fresh as the last stamp: at most one refresh period (10 s, C-18) old unless a regime change brought a newer one. The *missing* counter can decrease: a slice counted missing at the deadline that arrives while its position is still within the ring span is reclassified as late (missing −1, late +1, C-108).

3. **Hellos** — the discovery messages (period 1 s, C-30): who is present (module id and instance nonce, C-11), who holds the clock, alarm field, control-degraded mark, stamp version number (C-18).

**Vocabulary the display must use correctly.**

- **Connection states:** Requested, Refused (with reason, for example RequiredEExceeded, C-103, or BadMemberList, C-173; 18 reasons in all, including ControlDegraded, the target's housekeeping having stopped restarting, and NoSkewBudget, C-185). VersionMismatch, NameTaken and ModuleCap are raised at discovery, where they stop a module joining, not at admission (C-185), Pending (target not present; no timeout: Pending persists until the target appears or the operator cancels, and is shown as unsatisfied, C-45), Admitted, Locking, Aligning, Live, Interrupted, Dead, Ended. Only Live counts as satisfied; Pending, Locking, Aligning, Interrupted and Demoted are shown by name as unsatisfied (C-154).
- **Connection alarms** (C-173): raised on refusal, interrupted, dead, demoted, cannot-meet-A (maximum lateness + D + headroom exceeds A in each of 10 consecutive 1 s windows, C-174; its lateness is over the rolling 60 s window, C-182) and control-degraded. The monitor shows each by name as published.
- **Health state Demoted** (C-109): an input persistently late (C-101) with its offset already at A; treated as gapped, counted separately, cleared only by re-admission. Each connection's stamp summary counts demoted rounds (C-159).
- **Four gap classes** at a consumer: *marked gap* (upstream alive, loss further up), *late* (arrived after the deadline; the remedy is budget), *missing* (unknown), *dead* (classified). Interrupted after 8 consecutive missed slices (C-38), where only slices absent at the deadline count and a marked gap or a slice on another timeline epoch resets the count (C-102). Dead after 2 s (C-39) counted from the last slice received with content or a marked gap (C-111). A slice on another timeline epoch restarts the dead clock, is not a miss and has its own counter, other-epoch (C-135). Late versus missing: C-108 above. Interrupted and dead describe streams a peer receives; a sink-only device's liveness is judged by the producer timeout (C-41, C-168).
- **Degraded:** a slice computed from a gapped input. The flag follows the content, so on an output with structural latency L (always a whole number of samples, C-105) it appears L after the gapped input position (C-122).
- **Session number:** one per stream, assigned by the source, bumped on restart and on timeline epoch change; every consumer sees the same value (C-19). A new session starts a new series for that connection.
- **Clock states:** unlocked, locking, locked (with bound, target at most 25 µs, C-62), holdover (bound grows 1 µs/s, C-64; alignment claim lapses after about 25 s, C-65). The lapse is when the bound reaches the reference E of 50 µs (C-70): C-65 is (C-70 − C-62) / C-64. The monitor therefore derives the holdover countdown from the published bound, (C-70 − bound) / C-64, counted down from the time of the stamp that published it; it needs no holdover start time. For a module whose E is unknown the reference is twice the lock bound, 2 × C-62, also 50 µs (C-169). When the claim lapses the clock state becomes Unlocked with time unusable: one-way figures and alignment claims stop (C-162).
- **Timeline epoch:** changes only on a timebase discontinuity, including an election won with no locked candidate (C-113) and a holder stall (C-136); distinct from the election epoch.
- **Presentation group:** sinks sharing an audition offset A (C-55) and a required E. Each sink shows its E (99.9th percentile of emission error over an hour, with the maximum reported beside it, C-73), its D (typical values C-81, restart spread C-74), and its certification state (*uncertified*, *self-monitored*, *commissioned*). D is the sink's total plumbing delay from hand-off to emission: driver buffer, resampler and converter (C-170). Only a commissioned sink has a numeric E; uncertified and self-monitored sinks declare E as **unknown** (C-103, C-140). A commissioned sink whose clock is unlocked stays commissioned; its alignment claim is suspended and does not count towards the group's required E until the clock is locked again, then resumes without re-commissioning (C-180).
- **Bundle:** a group of narrow aligned streams from one wide source (6 channels per member at defaults, C-04; up to 32 members, the controller member included, C-08, C-209), with per-member counters. A bundle source's controllers always ride in their own member stream (C-181, C-205), so a source with controllers has at most 31 audio members.
- **Capacity:** per port and per direction (C-117), admitted rate against the 70 % ceiling (C-23); a default stream is about 21.1 Mbit/s (C-21).
- **Control-degraded:** the module's housekeeping thread was restarted; live streams are unaffected. Restarts back off from 100 ms to 10 s; after 5 panics within 60 s restarts stop, the module stays control-degraded with an alarm, its live streams continue, and it accepts no new connection until an operator restarts it (C-139).
- **Id collision:** a module id seen with two instance nonces is quarantined by third parties until only one nonce has been seen for one hello-expiry period of 3 s (C-46, C-31).
- **Departure:** a module that leaves (including a laptop edge going to sleep, C-124) and returns through normal admission (Locking, Aligning).
- **Loss versus the floor:** expected about one loss per stream per link about every 13 hours (C-90) `[memory]`; loss above the alarm threshold (C-91: 10 × the clean-wire loss measured at L1, C-167, proposal pending first runs) is a cable-health alarm. The monitor reads the alarm as published and never hard-codes the threshold.

**POC reference `[code]`.** `src/tui.rs` (668 lines, ratatui) and `src/stats.rs` (549 lines) are the POC's display and aggregation. Their layout ideas (lock colour, playable-latency panel, packet counters) may be reused. Their aggregation must not be: `stats.rs:324` takes the latest record's loss value across all return streams with no per-stream separation. The POC tree is frozen and is not modified (C-130).

## Deliverables

Crate `hadronyche-tools` (the home of harness and tool binaries, C-133), binary `hadronyche-monitor`:

- `collect.rs` — joins the discovery and control multicast groups (IPv4, C-100) read-only and receives the stamps multicast on any regime change and every 10 s (C-18), reassembling their sections of at most 1 252 bytes (C-17); compares the hello's stamp version with the stamp held; or reads a recorded stamp file. Never sends anything except the IGMP membership traffic that joining the groups requires; in particular it never sends the unicast stamp request.
- `model.rs` — an in-memory model keyed by module id (with instance nonce), connection and bundle member; a regime change or a new session number (C-19) starts a new series.
- `views/` — panels:
  - **Medianet:** modules, profiles, holder, election and timeline epochs, alarms by kind (C-173), control-degraded marks (including the stopped-restart state of C-139), id collisions (C-46), departures, active commissioning window (C-107).
  - **Clock:** per module lock state and bound; holdover countdown to lapse, after which the module is shown Unlocked (C-162).
  - **Connections:** state (only Live satisfied; Pending, Locking, Aligning, Interrupted and Demoted shown by name as unsatisfied, C-154), alarms (C-173), refusal reasons (C-185), offset, lateness percentiles (C-160) and reclaimable latency over the rolling 60 s window with the maximum since live beside them (C-182), the C-159 counters, path MTU, DSCP as observed.
  - **Bundles:** per-member counters.
  - **Ports:** admitted rate against ceiling, per direction.
  - **Sinks:** A, D (C-170), E (numeric only when commissioned, otherwise unknown, C-140) with its maximum, required E, certification state (a commissioned sink with an unlocked clock shown with its claim suspended, C-180).
  - **Regime:** the full stamp of a selected module, with absent and unverified capabilities highlighted and not-requested ones shown as deliberate.
- `--headless` mode printing one summary line per interval, for logs.
- Dependencies allowed: `ratatui` (as in the POC) for the terminal UI.

```rust
/// Everything the monitor knows comes through this; it has no other input.
pub enum Observation {
    Hello(HelloView),
    Stamp(StampView),
}
pub trait ObservationSource { fn next(&mut self) -> Option<Observation>; }
```

Receiver reports are not an observation: the monitor cannot see them (C-47).

## Dependencies

- **Needs:** the clock lock state and bound that the stamp carries and the one-way figures depend on, published by the servo core (008a, C-187); the stamp wire object, including the per-connection summaries (013); the receiver report format (014), only for the meaning of the gap-class counters that the stamp summaries publish; the hosted discovery socket and stamp collector, and the sending of the stamp message (015, C-144); the L0 baseline scenario (017b, C-192) and the L1 baseline scenarios (019, C-94), both used by criterion 029-AC-1.
- **Provides:** nothing other tasks depend on. If the phase is short, this task is dropped without consequence; the commissioning tool's `survey` and `watch` cover the essential operator needs.

## Acceptance criteria

1. **029-AC-1 Read-only.** A capture on the medianet interface shows the monitor sends no control, sync, media or measurement packets and no stamp request; its only traffic is IGMP membership. Starting and stopping it during the L0 and L1 baseline scenarios changes no counter in any module.
2. **029-AC-2 Honest blanks.** One-way figures are blank before lock and show the bound after; a check family with zero samples is displayed as *failed*; an injected run is visibly labelled as injected, and so is a run inside a commissioning window (C-107). (Contributes to the global criteria AC-9 and AC-31.)
3. **029-AC-3 Regime change.** Changing a module's regime mid-run (for example a path-MTU fallback to 1 280 bytes, C-06) starts a new series; no average spans the change. A new session number (C-19) does the same for its connection.
4. **029-AC-4 Per-stream separation.** With two return streams reporting different loss, each is shown separately; no last-writer-wins aggregate exists.
5. **029-AC-5 Gap classes.** Injected delay beyond the deadline shows as *late* (after the C-108 reclassification); injected drop as *missing*; an upstream gap as *marked*; after 2 s without content or a marked gap (C-39, C-111), *dead*. Each appears no later than the next stamp refresh (10 s, C-18).
6. **029-AC-6 Replay.** The monitor renders identically from a recorded stamp file and from the live network.
7. **029-AC-7 Scale.** 64 modules (C-34), each sending a hello every 1 s (C-30) and its stamp every 10 s (C-18), with summaries for 32 streams, plus a burst in which all 64 modules change regime within one hello period, display without dropped observations on the development machine. The machine's specification is recorded with the result in `doc/design/reports/` (C-130).
8. **029-AC-8 Fuzzing.** The observation decoders are fuzzed for 60 s each on every change and 1 hour nightly, with the corpus kept (C-95): zero panics.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The headless mode is used by at least one CI job as a log formatter
- [ ] A screenshot or text capture of each panel is in the tool's documentation

**Not applicable in this epic (C-125).** "Regime stamp fields touched are complete and documented": the monitor adds, removes or changes no regime stamp field; it only reads the fields that epic 013 defines and documents. How it displays them is covered by the panel captures in the tool's documentation.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| How does a third party observe stream health, given that receiver reports are unicast to the producer? | Decided (C-47, C-159): producer and consumer each publish per-connection summary counters (received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead) and the observed DSCP in their own regime stamp. The monitor reads stamps only; reports are not multicast and there is no query message | The stamp summaries are too coarse to diagnose a fault |
| How does a freshly started monitor get stamps without sending a request? | Decided (C-18): stamps are multicast on any regime change and refreshed every 10 s | — |
| Which counters does a connection summary carry? | Decided (C-159 completes C-47; C-108 for late versus missing; C-135 for other-epoch) | — |
| How does the monitor count down to the holdover lapse? | Decided (C-64, C-65, C-70): from the published bound, (C-70 − bound) / C-64 | — |
| What load does 029-AC-7 mean, given that reports are invisible? | Decided: hellos at 1 s (C-30), stamps every 10 s (C-18) and a regime-change burst, as stated in 029-AC-7 | — |
| Build it in v1 at all? | Product, at the start of P7 | Operators manage with `survey` and `watch` alone |
| What is the recorded stamp file format, and does it record hellos? Working assumption: it records hellos and stamp sections with arrival times, which 029-AC-6 needs | Epic 015 (stamp collector) | The collector records stamps only, so hello-derived fields (holder, alarms, control-degraded) cannot be replayed |
| Is each bundle member its own connection with its own C-47 summary? Working assumption: yes | Epic 005 (control state machines), reflected in the stamp layout by epic 004 | Summaries are published per bundle only, so per-member counters do not exist |
| Where are the connection state, the Demoted health state (C-109; its rounds are counted per connection, C-159), admitted rate and link rate per port and direction, sink D, E and certification state, check-family sample counts, the election epoch, the commissioning window's expiry (C-153), the stopped-housekeeping state (C-139, as distinct from control-degraded) and the discovery-time refusals that stop a module joining (VersionMismatch, NameTaken, ModuleCap, C-185) published? Working assumption: in the stamp; the monitor shows "not published" otherwise. Lateness percentiles (C-160) and reclaimable latency (C-157) are settled in the alignment group, over a rolling 60 s window with the maximum since live beside them (C-182) | Epic 013 (stamp object), with epic 004 for the hello | A field the panels need is published nowhere |
| Does the stamp version number change only on a regime change, or also when a refresh carries new summary counters? Working assumption: only on a regime change; the model opens a series on a regime change, not on every version | Epic 013, with epic 004 | A refresh with unchanged regime carries a new version number |
| What is D? | Decided (C-170): a sink's total plumbing delay from hand-off to emission (driver buffer, resampler and converter) | — |
| What does a headless line contain, and what is the default interval? | Decided (C-206): output line formats and panel layouts are decided by this epic (ticket 029-18) and recorded in the tool's documentation (029-23); only what another epic or an operator's script depends on goes into the specifications | A CI job's failure cannot be diagnosed from the headless log |
| Which epic defines the L0 baseline scenario? (L1 is task 019, C-94) | Decided (C-192): epic 017b defines it | — |
| Value of the loss alarm threshold (C-91). Rule proposed: 10 × the measured clean-wire loss (C-167); the number awaits the L1 measurement. The monitor reads the alarm as published and does not need the value | Measurement (L1) | A module raises a cable-health alarm that is not visible in the alarm field the monitor reads |
