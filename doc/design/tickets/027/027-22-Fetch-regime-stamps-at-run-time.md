# 027-22 — Fetch regime stamps at run time
**Epic:** 027-L3-cross-sink-rig · **Effort:** M · **Depends on:** 027-07, epic 004

## Context
- The L3 report must carry both sinks' complete stamps (027-AC-10), and several scenarios need the sinks' state during the run: health counters for loss and interruption (027-15, 027-16), session number across an epoch step (027-17), alignment claim and clock bound in holdover (027-18), D after each restart (027-12, 027-13).
- Regime stamp transport (C-18): a dedicated control message split into sections that each fit one packet at the fallback MTU, at most 1 252 bytes of UDP payload (C-17). It is multicast to the control group on any regime change and refreshed every 10 s; a unicast request gives an immediate fetch. The hello carries only a stamp version number.
- Stamp contents relevant here: per-connection counters — received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159, completing C-47) — and the observed DSCP (C-47); lateness percentiles 50th, 99th, 99.9th and maximum (C-160) over a rolling 60 s window, with the maximum since the connection went live beside them (C-182), which 027-23 uses to compute A; certification state (only commissioned gives a numeric E, C-140); the capability states of real-time scheduling and memory lock, which show strict mode (C-127, C-147); receive-timestamp source `kernel`, `userspace`, `mac-hardware` or `interrupt` (C-131), each with one of the capability states present, absent, unverified, not requested (C-129).
- IPv4 only; multicast with TTL 1 (C-100). Control traffic uses DSCP CS5 (C-13).
- The byte layout of the stamp message is defined by the wire specification (epic 004). Which fields carry the clock state (the holdover claim lapses when it becomes Unlocked, C-162) and the clock bound is still open (owner epic 004, with epic 005; the values are published by the servo core, epic 008a, through its time view, C-187); this ticket maps them when they exist.
- The same fetch serves the source module's stamp, whose per-connection counters 027-16 reads to confirm no producer timeout (C-41, C-168).
- Fuzz budget (C-95): each parser 60 s on every change and 1 hour nightly; the corpus is kept.

## Work
1. Module `stamp.rs` in `hadronyche-l3`: send the unicast stamp request to a sink or to the source module, receive and reassemble all sections, and decode them with the epic-004 layout (reuse epic 004's decoder if it provides one; otherwise implement it here).
2. Map the decoded stamp to `SinkStamp` (027-07) and write it as the JSON stamp file used by `analyze`.
3. A listener on the control multicast group that logs every stamp update (version, time, changed fields) during a scenario run, for `ScenarioReport.stamp_updates`.
4. Detect and report incomplete stamps: missing sections after a timeout, or a stamp version that changes between sections (re-fetch once, then fail with a named error).
5. Fuzz target `fuzz_stamp_message` over section reassembly and decoding (C-95).
6. Unit tests against recorded or synthetic stamp messages; no network privileges needed in CI (decoder and reassembly tested on byte buffers, the socket part behind a trait).

## Acceptance criteria
1. Against a sink (or a test responder built from the epic-004 layout) the tool fetches a complete stamp by unicast request and writes a `SinkStamp` JSON that 027-07 accepts.
2. A stamp split into several sections, each at most 1 252 bytes of UDP payload (C-17), is reassembled; a missing section yields a named error, not a partial stamp.
3. The multicast listener logs a regime change within one refresh period (10 s, C-18) of its occurrence, and immediately when the sink multicasts on the change.
4. The eleven C-159 counters, the four C-160 lateness percentiles and the since-live maximum (C-182), the certification state, the real-time scheduling and memory-lock capability states, and the C-131 timestamp source with its C-129 state are present in the output.
5. The fuzz target runs 60 s on every change and 1 hour nightly with no crash; the corpus is kept.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] No new open item without an owner and a falsifier (the clock-state and bound fields are an owned epic sub-question)
- Not applicable (C-125): "Design documents updated" — implements C-18 as written; "Every new factual claim carries an evidence tag" — no factual claim; "The skew analysis runs in CI on recorded two-channel files" — no skew analysis; "The rig is photographed or diagrammed" — no rig change; "`l3-results.md` is committed" — 027-20.

## Out of scope
- Defining the stamp layout (epic 004).
- Producing stamps inside the sinks (epics 018, 026).
- Rendering the stamps in the report (027-07).
