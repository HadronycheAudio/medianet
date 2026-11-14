# 026a-04 — Embedded profile declaration
**Epic:** 026a-Embedded-HAL-and-host-integration · **Effort:** S · **Depends on:** 026a-02, epic 013

## Context

The regime stamp is a **wire object** (epic 013). A device cannot write files, so it publishes its stamp over the network (026a-11) and a hosted collector records it (026a-12). `profile.rs` holds the embedded capability declaration that goes into that stamp.

Settled rules this ticket applies:

- The stamp travels in a dedicated control message split into **sections that each fit C-17** (1 252 bytes of UDP payload); it need not fit one message as a whole (C-18).
- Capability fields take one of **four states**: present, absent, unverified, not requested; "not requested" marks a deliberate configuration choice, as distinct from a denial (C-129).
- Receive-timestamp source values are those of C-131 (on a device: mac-hardware, interrupt, userspace), each with one of the four states.
- E is the 99.9th percentile of absolute emission error over a one-hour certification run, the maximum reported beside it (C-73). Only the commissioned certification state gives a numeric E; uncertified and self-monitored sinks declare it unknown (C-140, C-103). The earlier "bound or percentile" choice is closed by C-73.
- A test-only fault hook is stamped like any injection (C-120, C-145).
- Strict mode is hosted-only: a device stamps real-time scheduling and memory lock as present by construction (C-147).
- The control-isolation capability means housekeeping cannot delay the data path; on a device it is absent or unverified unless the implementer shows a bound (C-161).

Definition-of-done requirement: the table "what weakens on no_std" is reflected **field by field** in the device's stamp. The table `[decision]`:

| Guarantee on hosted | On the device |
|---|---|
| Housekeeping isolated from the data plane; no shared lock | Not enforced by an OS. The implementer must show **bounded control handling**; otherwise stamped *report-only*. Control isolation absent or unverified unless that bound is shown (C-161) |
| Supervised, restartable housekeeping | **Unsatisfiable.** Any fault is a whole-device reset (watchdog) |
| Privilege report (real-time scheduling, memory lock) | Replaced by a profile declaration: *embedded, by construction*; both stamped present (C-147) |
| Fault injection, interface drop counters, DSCP marking | Each may be **absent**; stamped with the C-129 states |
| Record and replay of exact-class modules | Done on a host: the same generated `no_std` code is rerun against captured input |
| Multi-packet slices, skew buffers | Bounded by declared memory; admission refuses beyond it |
| Files | None; the stamp is published on the network |

Other declared capabilities:
- Holder eligibility: **never** set by a v1 device (follower only) `[decision]`.
- D (plumbing delay, with method) and E as above.

Settled (C-147): strict mode (C-127, C-128) is hosted-only, and a device stamps real-time scheduling and memory lock as present by construction. A strict-mode group therefore sees both present and treats the device like any other module on those two fields.

## Work

In `profile.rs` define a `const`-constructible `EmbeddedProfile` struct (no alloc) with at least:

- `profile: Embedded` ("embedded, by construction" — replaces privilege report);
- `rt_scheduling: Capability::Present` and `memory_lock: Capability::Present`, constant (C-147);
- `control_handling: Bounded { worst_case_poll_ns: u32 } | ReportOnly`, and `control_isolation: Capability` set to present only with `Bounded`, else absent or unverified (C-161);
- `housekeeping: ResetOnFault` (supervised restart unsatisfiable; watchdog);
- capability fields of type `Capability { Present, Absent, Unverified, NotRequested }` (C-129): `fault_injection`, `iface_drop_counters`, `dscp_marking`, `dont_fragment`, `rx_dscp_observation` (needed for the observed-DSCP counter, C-47);
- `replay: HostSide`;
- `buffer_memory_limit_bytes: u32` and `max_streams: u8` (skew buffers bounded by declared memory; the unit of subscription is a 6-channel stream, C-04);
- `files: None`;
- `holder_eligible: false` (constant, not configurable);
- `rx_stamp_source: StampSource` with its `Capability` state (C-131), `declared_jitter_ns: u32`;
- `d_ns` with `d_method` (D as defined by C-170), and `e: Certified { p999_ns, max_ns } | Unknown` (C-73, C-103; `Certified` only from a commissioned run, C-140).

Map each field to the stamp wire fields of epic 013; where a field does not exist in 013's object, add it there per 013's extension rule or record it as an open item with owner epic 013. Document every field (one line each) in the stamp field documentation.

## Acceptance criteria

1. Every one of the 7 rows of the "what weakens" table maps to at least one named field; a test or doc table lists the mapping row by row.
2. `holder_eligible` cannot be set to `true` through any public API (compile-time constant or private field with no setter); `rt_scheduling` and `memory_lock` always encode as present (C-147).
3. `control_isolation` encodes as present only when `control_handling` is `Bounded`; with `ReportOnly` it is absent or unverified (C-161).
4. Every capability field accepts exactly the four C-129 states, and each state round-trips through encode and decode.
5. Encoding the profile into the epic-013 stamp object and decoding it back yields an equal value (round-trip test on host), for E both `Certified` and `Unknown`.
6. Every encoded stamp section containing profile fields is at most 1 252 bytes of UDP payload (C-17, C-18), asserted with all optional fields present.
7. Any new stamp parser path added is covered by a fuzz target run at the C-95 budget.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] The table "what weakens on no_std" is reflected field by field in the device's stamp
- [ ] Every new factual claim carries an evidence tag
- [ ] The crate builds for at least one device target and for the host (for tests) in CI

## Out of scope

- Publishing the stamp (026a-11) and collecting it (026a-12).
- Per-connection counters (026b-11), calibration factor (026b-19), commissioning window (026a-13), lateness percentiles, reclaimable latency and peer DSCP observations (026b-12).
- Measuring the values (worst-case poll 026b-06, jitter 026b-07, D and E 026b-14).
