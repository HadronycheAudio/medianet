# 027-07 — Report rendering with sink stamps
**Epic:** 027-L3-cross-sink-rig · **Effort:** S · **Depends on:** 027-05, 027-06, epic 018, epic 026

## Context
- `report.rs` of `hadronyche-l3` must output, per scenario: skew percentiles and maximum, both sinks' stamps, declared E values, and pass/fail (`p999_abs_ns <= limit_ns`, C-79).
- **027-AC-10:** both sinks' complete stamps (profile, timestamp source with its capability state, D and method, E, certification state) are in the report.
- **027-AC-9 (report part):** the capture device's own inter-channel skew is in the report (it is measured and subtracted in 027-10).
- Sink facts the stamps describe: D = declared plumbing delay (driver buffer + resampler + converter), each sink hands sample p to hardware at `submit(p) = emit(p) − D`; E = 99.9th percentile of |actual emission − (T(p)+A)| over 1 h, max beside it (C-73); Linux target E = 50 µs (C-70); device E declared per device; a sink with no certified E declares it unknown (C-103).
- Timestamp source values (C-131): `kernel` (hosted OS kernel stamp), `userspace` (clock read after receive), `mac-hardware` (Ethernet MAC timestamp register, embedded), `interrupt` (stamped in the receive interrupt, embedded). Each is reported with one of four capability states (C-129): present, absent, unverified, not requested.
- Each sink's stamp also carries per-connection counters — received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159, completing C-47) — the observed DSCP (C-47), and lateness percentiles 50th, 99th, 99.9th and maximum (C-160), computed over a rolling 60 s window, with the maximum since the connection went live reported beside them (C-182).
- Only the certification state commissioned gives a numeric E; uncertified and self-monitored sinks declare E as unknown (C-140, C-103). Strict mode (C-147) is shown by the capability states of real-time scheduling and memory lock (C-127, C-129); the device stamps both present by construction.
- The limit uses target E values, never the E measured in the same run (C-149): C-70 for the Linux sink, the device's declared E for the device.
- At run time the stamps are fetched from the sinks by unicast request (C-18, ticket 027-22), which writes them as one JSON stamp file per sink. This ticket defines that JSON form and reads it; the same files serve as CI fixtures.

## Work
1. `struct SinkStamp { profile, timestamp_source: TimestampSource, timestamp_source_state: CapabilityState, d_ns, d_method, e_ns: Option<i64>, e_max_ns, certification_state, rt_scheduling: CapabilityState, memory_lock: CapabilityState, lateness: LatenessPercentiles, counters: ConnectionCounters }` with serde (de)serialisation. `TimestampSource` has exactly the four C-131 values; `CapabilityState` the four C-129 values; `e_ns = None` means E unknown (C-103), and a certification state other than commissioned forces `None` (C-140). `ConnectionCounters` holds the eleven C-159 counters and the observed DSCP (C-47); `LatenessPercentiles` the four C-160 values over the rolling 60 s window and the maximum since the connection went live (C-182). All fields mandatory; a missing field ⇒ error naming the field.
2. `struct ScenarioReport { scenario, started_at, duration, skew: SkewReport, drift/step/recovery results where applicable with the thresholds used, linux: SinkStamp, device: SinkStamp, stamp_updates (regime changes logged during the run), capture_interchannel_skew_ns, selfcheck_skew_ns, unpaired_markers, a_ns and its inputs (C-148, ticket 027-23), eligibility (027-24), notes }`. `limit_ns` is computed from the target E values (C-149).
3. Output as JSON (machine) and Markdown (human) — the Markdown table is what goes into `l3-results.md` (027-20).
4. The stamp file parser is fuzzed (`fuzz_sink_stamp`): 60 s on every change and 1 hour nightly, corpus kept (C-95).
5. Document every stamp field (meaning, unit, source C-id) in the module doc and in the report legend.

## Acceptance criteria
1. A report generated from a test series and two test stamp files contains, for each sink, profile, timestamp source, its capability state, D, D method, E, E max and certification state, and `limit_ns` equals the C-70 target plus the device's declared E (C-149), not any measured E.
2. A stamp file missing `d_method` is rejected with an error naming `d_method`.
3. A timestamp source outside the four C-131 values, or a capability state outside the four C-129 values, is rejected with a named error.
4. A stamp with E unknown, or with a certification state other than commissioned, produces a report with `pass = false` and the C-103 / C-140 reason.
8. All eleven C-159 counters, the four C-160 percentiles with the since-live maximum (C-182) and the two strict-mode capability states are parsed and rendered; a stamp missing any of them is rejected with an error naming the field.
5. The report contains `capture_interchannel_skew_ns` and `selfcheck_skew_ns`; when not supplied the report marks them "not measured" and `pass` is forced false.
6. JSON output round-trips (serialise, parse, equal).
7. The stamp fuzz target runs 60 s on every change and 1 hour nightly without crash (C-95).

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] The frozen POC is not modified (task 003 is the only exception)
- Not applicable (C-125): "Design documents updated" — no design behaviour; "No new open item" — opens none; "Every new factual claim carries an evidence tag" — no factual claim; "The skew analysis runs in CI on recorded two-channel files" — 027-08; "The rig is photographed or diagrammed" — no rig work; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Measuring the capture inter-channel skew (027-10).
- Fetching stamps from live sinks over the network (027-22).
- Producing stamps inside the sinks (epics 018, 026).
