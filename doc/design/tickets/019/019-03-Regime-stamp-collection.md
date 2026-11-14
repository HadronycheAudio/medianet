# 019-03 — Regime stamp collection
**Epic:** 019-L1-two-machine-harness · **Effort:** S · **Depends on:** 019-02, epic 015

## Context
Every L1 result file must carry **both hosts' complete stamps**: profile, kernel, NIC, driver, coalescing state, path MTU, DSCP observed (epic 019 criterion 019-A). Stamps make results comparable across hardware and are the place where facts such as the probed path MTU (AC-19) and the overflow counter (AC-37) become visible.

Settled facts the stamp fields refer to:
- **Stamp transport (C-18).** A module's regime stamp is a dedicated control message split into sections that each fit 1 252 bytes of UDP payload (C-17). It is multicast to the control group on any regime change and refreshed every 10 s; a unicast request fetches it immediately. The hello carries only a stamp version number, so a version change in the hello signals a regime change.
- **Stamp content (C-47, C-159, C-160, C-152).** Producer and consumer each publish, per connection: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159), and the observed DSCP; lateness percentiles 50th, 99th, 99.9th and the maximum (C-160), computed over a rolling 60 s window, with the maximum since the connection went live reported beside them (C-182). A slice on another timeline epoch has its own counter (C-135). The holder reports the DSCP observed on each follower's sync requests, recorded on its clock thread (C-188); every module reports the DSCP observed on each peer's hellos (C-152). The holder also counts, per follower, sync requests dropped for exceeding 200 per second (C-204). Receiver reports are not multicast and there is no query message.
- **Injections (C-120).** Every use of the injection API or of a test-only hook is stamped.
- **Commissioning window (C-153, C-178).** An open commissioning window is stamped by every module while it is open; ordinary production measurements in stamps (latency, loss, alignment figures) are unqualified while it is open.
- **Network (C-100).** IPv4 only; multicast with TTL 1. Path MTU is probed with a probe pair at admission, sent at the intended media packet size (C-116); fallback is 1 280 bytes (C-06), giving 1 252 bytes of UDP payload. A multi-packet slice regime is stamped (C-05).
- DSCP classes (C-13): sync CS6 (48); media and one-way probes EF (46); control CS5 (40); verified by the receiving peer.
- NIC interrupt coalescing is set with `ethtool -C` and read with `ethtool -c`.
- Receive timestamps are kernel software stamps (`SO_TIMESTAMPNS`, `CLOCK_REALTIME` basis `[kernel-doc]`).
- Fuzzing (C-95): each parser 60 s on every change and 1 hour nightly; the corpus is kept.
- **Strict mode and capabilities.** Strict mode requires real-time scheduling on the data and clock threads and locked memory; restricting CPU idle states is best-effort and stamped (C-127). Under strict mode an unverified capability is refused (C-128). Each capability is stamped in one of four states: present, absent, unverified, not requested (C-129).
- **Receive-timestamp source (C-131).** One of kernel, userspace, mac-hardware, interrupt, each with a C-129 state; L1 hosts are expected to report `kernel`.

This ticket collects the host-side part of the stamp (profile, kernel, NIC, driver, coalescing) with the harness, fetches the runtime's own regime stamp (path MTU, slice regime, DSCP observed, per-connection counters, socket overflow counter, injections) by the C-18 unicast request, and attaches both hosts' stamps to every result file.

The exact section layout of the stamp message and the field names are defined by the wire specification (epic 004) and the hosted runtime (epic 015); this ticket's parser follows them. Where observed DSCP for sync and control is published is settled by C-152 (holder's stamp for sync, each module's stamp for hellos); only the layout remains with epic 004.

## Work
- Module `hadronyche_tools::stamp`:
  - Struct `HostStamp { host, profile, kernel_release, nic_model, nic_driver, driver_version, firmware_version, coalescing: CoalescingState, path_mtu: Option<u32>, slice_regime, dscp_observed: DscpObserved, strict_mode: bool, commissioning_window: Option<WindowState>, capabilities: Vec<(Capability, CapabilityState)>, rx_timestamp_source: (TimestampSource, CapabilityState), connections: Vec<ConnectionCounters>, rx_overflow: Option<u64>, injections: Vec<Injection>, stamp_version }`.
  - Struct `ConnectionCounters { connection, received, late, missing, marked_gap, other_epoch, too_early, duplicates, incomplete_reassemblies, demoted_rounds, interrupted_events, dead, dscp_observed, lateness_p50, lateness_p99, lateness_p999, lateness_max, lateness_max_since_live }` (C-47, C-159, C-160); the four lateness figures are over the stamp's rolling 60 s window and `lateness_max_since_live` is the maximum since the connection went live (C-182).
  - Struct `DscpObserved` holds the media DSCP per connection, the sync DSCP per follower from the holder's stamp, and the hello DSCP per peer from each module's stamp (C-152).
  - Struct `HolderFollower { follower, sync_dscp, requests_dropped_over_rate }` per follower from the holder's stamp (C-152, C-204).
  - Struct `RunStamp { a: HostStamp, b: HostStamp, build_hash, scenario, started_at, ended_at }`.
- Collectors run over the `Transport` from 019-02: `uname -r`, `ethtool -i <if>`, `ethtool -c <if>` (read-only; no privilege needed), the real-time profile identifier from task 016.
- Runtime stamp fetch: send the C-18 unicast stamp request to each module the plan started, reassemble the sections, and parse them. Also record the stamp version from the hellos for the whole run; every version change triggers a fresh fetch, kept in time order.
- Parsers for `ethtool -i` and `ethtool -c` output and for the runtime stamp sections; each parser has a fuzz target (`ethtool_i`, `ethtool_c`, `runtime_stamp`).
- Missing fields are explicit (`Unknown(reason)`), never silently absent; a stamp with any `Unknown` is flagged incomplete in the report.
- Stamp collected at run start and run end and on every stamp version change; if coalescing state, path MTU or slice regime differs, all values are kept and flagged.
- Document every stamp field (name, source command or runtime field, unit) in the harness's module docs and in the design document section describing L1 stamps.
- In dry-run mode, the loopback interface `lo` is stamped; NIC/driver fields carry `Unknown("loopback")`.

## Acceptance criteria
1. A run of `noop` in dry-run mode writes `<out>/stamp.json` containing stamps for both `a` and `b`, each with all 7 required categories present (profile, kernel, NIC, driver, coalescing state, path MTU, DSCP observed), either as a value or as an explicit `Unknown(reason)`.
2. Parsers handle 3 recorded `ethtool -i`/`-c` samples from at least 2 different drivers (fixtures in the repo) and reject truncated input with an error, not a panic.
3. The runtime stamp parser accepts exactly the four capability states of C-129 and the four timestamp sources of C-131, rejecting any other value with a named error; it reassembles a multi-section stamp, rejects a stamp with a missing or duplicated section with a named error, and exposes the eleven C-159 counters, the four C-160 lateness figures (rolling 60 s window) and the maximum since live (C-182) per connection, the C-152 sync and hello DSCP values, the per-follower over-rate drop counter (C-204), and the commissioning window state (C-153).
4. Each of the 3 fuzz targets runs 60 s on every change and 1 hour nightly without a crash, with its corpus kept (C-95).
5. A run in which coalescing state changes between start and end shows both values and a "changed during run" flag in `stamp.json`.
6. A dry-run in which a module's regime changes mid-run (stamp version bump in the hello) yields two time-ordered runtime stamps for that module.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] Every new factual claim carries an evidence tag
- [ ] The frozen POC is not modified (task 003 is the only exception)

## Out of scope
- Changing coalescing settings (019-11 does that).
- Rendering stamps into the human-readable report (019-04).
- Recording the hardware pair (hosts, NICs, switch, cables) in `l1-results.md` (019-06).
- Defining the stamp message layout (epic 004) or emitting it (epic 015).
