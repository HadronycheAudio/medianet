# 013-22 — Stamp specification document
**Epic:** 013-Regime-stamp-wire-object · **Effort:** S · **Depends on:** 013-05, 013-11, 013-12, 013-13, 013-17, 013-23, 013-24, 013-25, 013-26, 013-27, epic 001, epic 004

## Context
- Deliverable: `doc/design/spec/stamp.md` with **the field tables, the encoding, how a stamp is published and requested, and the capability-bit table from task 001**.
- Section tickets 013-06 to 013-10, 013-23, 013-25, 013-26 and 013-27 add their field tables as they land; this ticket completes and checks the document.
- Stamp groups `[decision]`: Identity (protocol version, build identity, full configuration, domain, profile); Capabilities (present/absent/unverified/not requested, C-129: receive timestamp source as a C-131 value plus state, real-time scheduling granted, memory locked, CPU idle states restricted, drop accounting, DSCP marking, fault injection, holder-eligible, control isolation per C-161); Network (interface IPv4 only, interface MTU, path MTU per connection, packets per slice, pacing mode, DSCP classes and peer-observed values, interrupt-coalescing state, per-peer observed sync and hello DSCP per C-152, and in a holder's stamp the per-follower dropped sync request count per C-204); Clock (discipline mode, lock state and bound at each one-way figure, timeline epoch; unlocked after holdover lapse, C-162); Alignment (offset, lateness percentiles p50, p99, p99.9 and maximum per C-160 and reclaimable latency per C-157, both over a rolling 60 s window, with the maximum since the connection went live beside them per C-182; audition offset A, own or inherited per C-198, a configured budget per C-142, or none per C-185; required E or none; sink certification state per C-103 and C-140, unchanged while the clock is unlocked per C-180); Platform (OS build, audio device, driver, cost calibration factor per C-106 and C-176); Drops (per-socket overflow, interface drop counters, measurement-buffer overflow); Injection (any active injection including the C-120 and C-145 hooks, always stamped; commissioning window and expiry per C-153, disqualifying production figures only per C-178). Plus per-connection entries, per-connection summaries (C-47, C-159) and the self-test.
- Regime fields versus counters: the `StampId` covers regime fields only; Drops, per-connection summaries, per-peer observed DSCP, the per-follower dropped sync request counts and the alignment measurements (percentiles, since-live maximum, reclaimable latency) are counters (epic 013, settled from C-18, C-47, C-152, C-157, C-160, C-182, C-204).
- Rules 1–13 of the epic `[decision]`, and constants C-05, C-06, C-10, C-11, C-13, C-17, C-18, C-19, C-34, C-47, C-62, C-70/71/72, C-73, C-95, C-100, C-102, C-103, C-106, C-107, C-108, C-109, C-111, C-120, C-127 to C-133, C-135, C-140, C-142, C-144 to C-147, C-152, C-153, C-157, C-159 to C-162, C-165, C-169, C-171, C-176, C-178, C-180, C-182, C-185, C-187, C-188, C-191 to C-193, C-198, C-204, C-206, as listed in the epic. The constants table is `doc/design/02-constants.md` and wins over the spec. Specifications live in `doc/design/spec/` and probe reports in `doc/design/reports/` (C-130).
- Publish and request flow is decided (C-18): dedicated control message in sections of at most 1 252 bytes; multicast (IPv4, TTL 1) on any regime change and refreshed every 10 s; unicast request for an immediate fetch; the hello carries only a stamp version number.
- Platform facts: Linux `SO_TIMESTAMPNS` and `SO_RXQ_OVFL` `[kernel-doc]`; macOS/Windows kernel stamps unverified and no per-socket drop counter `[memory]`; coalescing state recorded on hosted platforms; embedded MAC hardware or interrupt-time stamps; control isolation present on hosted by construction and absent or unverified on a device unless bounded (C-161); real-time scheduling and memory lock present by construction on a device (C-147).

## Work
1. Complete `doc/design/spec/stamp.md`: envelope and section encoding with versioning and the unknown-section rule (skip and count); `StampId` derivation and the regime/counter split; canonical configuration encoding; field tables for all groups plus per-connection entries, summaries and self-test; per-field source on each of the 4 profiles or "absent"; size table and splitting rule (013-13); publish and request flow (C-18; schedule 013-24, sending by epic 015 per C-144, collector 013-17); record-set and collector formats and the collector's location (013-15, 013-17, C-171), limited to what another epic or an operator's script depends on (C-206); the capability-bit table from task 001; rules 1–13; `qualified()` and strict-mode semantics.
2. Update the design constants table first where values or behaviour differ from what was designed.
3. Record every open sub-question of the epic with its owner and falsifier, and list the questions closed since the ticket split with their C-ids.

## Acceptance criteria
1. Every field written by `StampBuilder` appears in a table with type, unit, kind (regime or counter) and a source for each of the 4 profiles (or "absent") — checked by a review listing 0 missing fields.
2. The encoding section is sufficient to reproduce every epic 007 vector by hand (reviewer checks at least one, including one split section).
3. The capability-bit table from task 001 is included with its evidence tags.
4. All 7 open sub-questions of the epic appear with owner and falsifier, and the 4 closed by C-153, C-159, C-160 and C-161 are listed as closed.

## Definition of done
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] Every new factual claim carries an evidence tag
- [ ] Every stamp field has a stated source on each of the four profiles, or is marked absent there
- [ ] The stamp format is versioned independently of the media format, with the rule for unknown sections (skip and count)
- [ ] The frozen POC is not modified (task 003 is the only exception)
- A figure cited from a published standard uses `[standard]` with its clause (C-132).
- Not applicable (C-125): "Tests green in CI, unprivileged" and "Every parser touched is fuzzed" (documentation only, no code).

## Out of scope
- Code changes (other tickets). Measuring capabilities (task 001). Message layouts (epic 004).
- Open sub-questions owned elsewhere are recorded, not answered.
