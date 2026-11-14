# 019 — L1 two-machine harness

**Phase:** P4 · **Scope:** L (27 tickets, about 34.5 days) · **Depends on:** 008a, 015, 016, 017 · **Blocks:** 028, 029

## Goal

Build and run the first rung of the ladder that leaves one machine: **two Linux hosts on one wire** (through one switch). L1 is where the design's network and capacity assumptions meet hardware for the first time. It produces measured values for several constants that are currently guesses, checks the falsifiers that the constants table assigns to L1, and it decides whether the "detect and conceal, no redundancy" position for v1 survives.

## Background

**Product frame.** Hadronyche Medianets are wired, isolated LANs whose only traffic is this runtime's. Modules exchange *slices* — all content of one stream for one stretch of media time — addressed by *position* on a domain-wide timeline. Linux hosts are the backbone; macOS and Windows machines are edges; embedded devices are followers.

**The ladder `[decision]`.** L0 = loopback in one process (task 017). **L1 = two machines, one wire.** L2 = a reference sink per platform (task 020). L3 = a hosted and an embedded sink together (task 027).

**Why L0 is not enough.** Loopback has an MTU of 65 536 and no NIC, no switch, no interrupt path and no second crystal. Everything about packets on a wire is untested until L1.

**Design facts L1 must confront.** All numbers are from the constants table (`doc/design/02-constants.md`), which wins over this file.

- **Network.** IPv4 only in v1; multicast with TTL 1 (C-100). Every stream on the wire is at the domain rate of 96 000 Hz; there is no other wire rate (C-01).
- **Packets.** A slice is one packet by default (C-05): 48 frames = 0.5 ms at 96 kHz (C-01, C-02), 4 bytes per sample (C-03), 6 channels per stream (C-04): 192 bytes per channel per slice, 1 152 bytes of audio, no per-channel id byte (the channel map is in the manifest). About 1 320 bytes on the wire including all Ethernet overhead (C-20), 2 000 packets/s and about 21.1 Mbit/s per stream (C-21). A slice larger than the path MTU is carried in several packets; this multi-packet regime is explicit and stamped (C-05). IP fragmentation is forbidden; every packet has don't-fragment set.
- **Bundles.** A wide source emits a bundle of narrow aligned streams. 96 channels = 16 streams = 338 Mbit/s = 32 000 packets/s (C-22). A bundle has at most 32 members, a controller member included (C-08, C-209); a bundle source's controllers always ride in their own member stream (C-181, C-205). The L1 bundle sources therefore carry no controllers, so each bundle is exactly 16 audio members and the C-92 load stays 32 streams.
- **Admission ceiling.** Each endpoint link is admitted to 70 % of its rate (C-23), per direction (C-117). One GbE port therefore carries about 198 channel-copies (C-24): two 96-channel bundles, not three.
- **Two-bundle capacity test (C-92).** One Linux consumer on one GbE port aligns 2 × 96 channels (32 streams, 64 000 packets/s, 676 Mbit/s) for 1 hour with zero late slices at the commissioned A; a third bundle is refused. This is acceptance criterion AC-13 below.
- **Receive cost** is a guess: 2 to 4 µs per packet, 13 to 26 % of the slice period for two bundles (C-93) `[memory]`. Its falsifier is "measured cost makes C-92 fail"; that is also the criterion for tightening the 10 GbE rule (any port above about 198 channel-copies, C-25).
- **Bounded drain.** A consumer drains at most 2 × the admitted packets per slice from one input per cycle (C-16); the cycle is one slice period (C-156). Its falsifier is "L1 shows legitimate bursts exceeding it".
- **Egress spread.** A source spreads its packets over at most 25 % of the slice period (125 µs at defaults), plus a per-source phase offset derived from the module id (C-14); the phase offset is in [0, 75 %) of the period, so offset plus spread stay inside the slice period (C-155). Its falsifier is "L1 shows queue drops with the spread on, or consumer margins broken by it". The comparison run turns the spread off with the pacing override of the operator policy in the commissioning configuration (C-165).
- **Loss floor** is from memory (`[memory]` is replaced by `[standard]`, with the clause, once the bit-error-rate citation is checked, C-132): at a bit error rate of 1e-12, about 1.1e-8 per default packet (about 1 320 bytes on the wire), one loss per stream per link about every 13 hours at 2 000 packets/s (C-90) `[memory]`. The loss soak runs 24 hours at the two-bundle load (64 000 packets/s, about 5.5e9 packets); the statistic is aggregate loss over all streams, where the floor predicts about 58 losses (C-118). The loss alarm threshold is set from the clean-network loss measured at L1, alarm when exceeded (C-91); its value is 10 × the measured clean-wire loss (C-167).
- **Redundancy.** v1 has none (no retransmission, no forward error correction); it detects, classifies, conceals and accounts. That stance assumes clean-medianet loss sits near the floor. Clean-medianet loss above 10 × the published floor (C-90), about 580 losses in the 24-hour soak (C-118), reopens the redundancy decision; between 1 × and 10 × it is recorded and investigated as a cabling or host problem and does not (C-179).
- **Jitter profile.** Simulation currently uses a provisional synthetic wired profile: base one-way delay 100 µs, exponential jitter with 20 µs mean, 1 % of packets in bursts delayed by 1 ms (C-94). L1 replaces it with the measured profile.
- **Clock.** The follower servo must hold a lock bound of at most 25 µs (C-62) with slew at most 10 ppm (C-60) and rate change at most 1 ppm/s (C-61). Followers send 50 sync requests/s by default (C-36); a holder accepts 10 to 200 per second (C-112). One-way latency figures are accurate to half the path asymmetry, which no in-band method can observe; the worst-case asymmetry bound is published with every one-way figure. The asymmetry check therefore tests each published figure against its own published bound; it needs no separate fixed value. The clock lock state and bound that accompany every one-way figure are published by the servo core (epic 008a) through the published time view (C-187). The holder answers sync requests on its clock thread, never on housekeeping, and records there the DSCP observed on each follower's requests (C-188); requests beyond 200 per second from one follower are dropped before any work and counted per follower in the holder's stamp (C-204).
- **Receive timestamps** are kernel software stamps (`SO_TIMESTAMPNS`, `CLOCK_REALTIME` basis `[kernel-doc]`): receive-timestamp source `kernel` in the regime stamp, reported with a capability state (C-131). NIC interrupt coalescing delays them, and delays only one direction of a sync exchange, which biases the offset.
- **Strict mode and capabilities.** Strict mode requires real-time scheduling on the data and clock threads and locked memory; restricting CPU idle states stays best-effort and is stamped (C-127). Under strict mode an unverified capability is refused, as if absent (C-128). The regime stamp reports each capability in one of four states: present, absent, unverified, not requested (C-129). Every L1 result records the mode and these states for both hosts. Any run whose numbers enter the constants table runs in strict mode (C-147). If a thread layout's seqlock safety depends on a CPU pin and that pin fails or is unverified, the module refuses to start in every mode, strict or not (C-177, C-201); default priorities (clock above data) never depend on pinning.
- **Alignment.** A fan-in consumer measures origin-to-arrival lateness for 4 s (C-50), sets its offset to the maximum observed plus headroom (C-51) — the largest of 25 % of that maximum, one slice period, twice the clock bound (C-52) — and then fires at the fixed time `T(p) + offset`. An input that is persistently late (at least one late slice in each of 10 consecutive 1 s windows, C-101) steps its offset up by one slice period (C-53) up to the ceiling A. **A** is the audition offset of the presentation group: no default, one per presentation group, proposed by commissioning (C-55); it is also the skew budget cap, and alignment rings are sized to it (C-96). "The commissioned A" in the criteria below is the A the harness computes for the run with the commissioning rule — the largest over sinks of maximum lateness + D + headroom — and records with the results (C-148). An input persistently late with its offset already at A enters the health state Demoted (C-109).
- **Late versus missing.** At the deadline an absent slice is counted *missing*; if it arrives later while its position is still within the ring span it is reclassified as *late* (missing −1, late +1) and discarded (C-108). For an aligned multi-packet input the reassembly deadline is the consumer's own deadline (C-110); for an arrival-driven one it is 2 slice periods (C-15).
- **Health.** Only missing or late slices count toward the interrupted threshold of 8 consecutive slices (C-38, C-102); dead is 2 s without a slice with content or a marked gap (C-39, C-111). A source assigns one session number per stream, bumped on restart and on timeline epoch change (C-19), so loss accounting is per stream and session.
- **Holder serving.** At most 64 followers per holder (C-35, owner L1, task 019); the 65th is refused with a named event. Holder serving lateness is reply send time minus request receive time at the 99.9th percentile; "no growth" means within 20 % from 1 to 64 followers (C-167). With at most 64 modules per medianet (C-34) a holder has at most 63 real followers, so the 64-follower load and the refusal of a 65th are tested with synthetic followers (C-114).
- **DSCP (C-13).** sync CS6 (48); media and one-way probes EF (46); control CS5 (40); verified by the receiving peer.
- **Path MTU.** A probe pair at admission, sent at the intended media packet size and exempt from the control message bound (C-116); fallback 1 280 bytes (C-06), which gives 1 252 bytes of UDP payload (C-100); re-probe when large packets vanish while liveness holds. A default slice fits the fallback: 1 152 bytes of audio plus at most 100 bytes for all headers and the checksum (C-137). Admission refuses with PathMtuTooSmall only when the probe at the 1 280-byte fallback also fails (C-173).
- **Regime stamps.** A module's stamp travels in a dedicated control message split into sections that each fit 1 252 bytes (C-17); it is multicast to the control group on any regime change and refreshed every 10 s, and a unicast request fetches it immediately; the hello carries only a stamp version number (C-18). Producer and consumer each publish per-connection summary counters in their own stamp: received, late, missing, marked gap, other-epoch, too-early, duplicates, incomplete reassemblies, demoted rounds, interrupted events, dead (C-159), and the observed DSCP; lateness percentiles 50th, 99th, 99.9th and the maximum (C-160), over a rolling 60 s window, with the maximum since the connection went live reported beside them (C-182). The holder reports the DSCP observed on each follower's sync requests, and every module the DSCP observed on each peer's hellos (C-152). Receiver reports are not multicast and there is no query message (C-47); the harness reads receiver reports from the producer host's records. While a commissioning window is open (announced with an expiry and stamped by every module, C-153), ordinary production measurements in stamps are unqualified (C-178); no L1 measured run has a window open.
- **Test hooks.** Beyond the public injection API, test-only hooks can force a module id and nonce, force a stream onto a given timeline epoch, and send a raw packet on a media or control socket; each use is stamped like any injection (C-120). L1 uses them for synthetic followers and for the flood.
- **Bad input.** Processing one bad input is proposed to be linear in packet length and at most 10 µs per packet, pending L1 (C-164); L1 measures it.
- **Fuzzing.** Each parser is fuzzed 60 s on every change and 1 hour nightly; the corpus is kept (C-95).
- **Privilege.** CI runs the harness unprivileged on loopback. Steps that need root on real hosts (`ethtool -C`, MTU changes, `rmem` sysctls) run only on the hardware pair; in dry-run they are recorded as skipped with a reason.

**What the POC measured on two machines `[code]`.** The POC's experiments (`experiments/run1..3`, `tools/metric_experiment.py`, `tools/stress.sh`) ran sender and echo on two hosts under CPU, I/O, memory and network stress. Their one design-relevant finding — machines under I/O pressure should not hold the clock — rests on metrics files whose integrity task 002 must confirm. `tools/stress.sh` is a usable load generator and may be reused, unmodified, as an external tool; the `stress` scenario takes its list of loads from the script's own usage text.

## Deliverables

Crate `hadronyche-tools` in the design set's repository (C-133), binary `hadronyche-l1`:

- Orchestrates two hosts over SSH: deploys the same build, starts reference modules (from `hadronyche-reference`), runs a named scenario, collects both hosts' record files and stamps, produces one report.
- A dry-run mode that runs every scenario on loopback, unprivileged, with shortened durations and privileged steps recorded as skipped.
- Scenarios:
  - `baseline` — one 6-channel stream each way, 1 hour; includes the DSCP check (AC-34).
  - `two-bundles` — the C-92 test (AC-13), with parameters for one or two bundles and for egress spread on or off (C-14).
  - `asymmetry` — compares in-band one-way figures with an external reference (see open sub-questions).
  - `coalescing` — repeats `baseline` with NIC interrupt coalescing on and off (`ethtool -C`), records servo offset and one-way figures.
  - `multi-packet` — a slice forced larger than the path MTU by a stream wider than the default 6 channels; records first-packet and slice-complete arrival.
  - `blackhole` — a switch port or interface MTU lowered below the probe size.
  - `serving` — 1, 16, 32 and 63 real follower processes against one holder, then 64 and 65 followers using synthetic followers (C-114).
  - `stress` — `baseline` under each load of `tools/stress.sh` on either host.
  - `rcvbuf-shrink` — loss induced by shrinking a receive buffer (AC-37).
  - `flood` — one input flooded at 10 × its admitted rate from the other host (AC-11).
  - `loss-soak` — 24 hours at the two-bundle load, counts every lost packet per stream per direction, aggregate statistic over all streams (C-118).
  - `bad-input` — malformed packets of a range of lengths sent to one input through the raw-packet hook (C-120); measures the per-packet processing cost against C-164.
  - Short scenarios (`coalescing`, `asymmetry`, `multi-packet`, `blackhole`, `serving` per step, `stress`, `rcvbuf-shrink`, `flood`, `bad-input`) run 10 minutes per run (C-167).
- Before each measured run the harness computes A with the commissioning rule (largest over sinks of maximum lateness + D + headroom) and records it with the results (C-148); the maximum lateness is the maximum since the connection went live, not the rolling 60 s figure (C-182).
- A hardware record: two Linux hosts with GbE NICs, at least one NIC whose interrupt coalescing can be changed, a managed switch with settable port MTU and readable drop counters (for `blackhole` and queue drops), a small unmanaged switch (for the burst question), cables, and whatever the chosen asymmetry reference needs.
- A measurements file `doc/design/reports/l1-results.md` (L1 results live in `doc/design/reports/`, C-130; the constants table is `doc/design/02-constants.md`) with, for each constant below, the measured value, the hardware, and the stamp.

Constants this task must measure or check, and write back to the constants table:

| Constant | Currently | Measured how |
|---|---|---|
| C-93 receive cost per packet | 2 to 4 µs, guessed | Data-thread time in batch receive divided by packets, at 2 000, 32 000 and 64 000 packets/s |
| C-92 two-bundle capacity | pending-measurement | `two-bundles` (AC-13) |
| C-90 loss floor | from memory | `loss-soak` on a clean wire (C-118) |
| C-91 loss alarm threshold | rule set: 10 × the measured clean-wire loss (C-167); value unset | Applied to the clean-network loss measured in `loss-soak` |
| C-94 wired jitter profile | provisional, synthetic | One-way delay distribution from `baseline` and `two-bundles` records |
| C-35 followers per holder | 64, decided (owner L1, task 019) | `serving`: serving lateness against follower count (C-167) |
| C-14 egress spread | 25 % of the slice period, decided | Falsifier check: queue drops and consumer lateness margin with spread on and off |
| C-16 bounded drain | 2 × admitted packets per slice, decided | Falsifier check: largest legitimate burst per input per cycle |
| Coalescing bias | unknown | `coalescing` |
| C-164 bound on processing one bad input | linear in packet length, at most 10 µs per packet, proposal pending L1 | Data-thread time per malformed packet injected through the raw-packet hook (C-120), over a range of packet lengths |

The falsifier of C-50 to C-52 (late slices after alignment on a clean network) is checked by AC-14 and AC-13. Every measurement in this table comes from a strict-mode run (C-147), at the A computed by the harness (C-148).

## Dependencies

- **Needs:** the servo core (008a), publisher of the clock lock state and bound that accompany every one-way figure (C-187); the hosted runtime (015), including the pacing override of the commissioning configuration (C-165); real-time setup (016), including strict mode (C-127, C-147); reference modules, injection and the test-only hooks of C-120 (017).
- **Provides:** measured constants, the measured jitter profile for the simulation, and switch findings to the commissioning tool (028); the go/no-go on v1 redundancy; receive-cost data that decides whether the 10 GbE rule (any port above about 198 channel-copies, C-25) needs to be tightened, by the C-93 falsifier.

## Acceptance criteria

1. **AC-13 (two bundles, C-92).** One Linux consumer on one GbE port aligns 2 × 96 channels — 32 streams, 64 000 packets/s, 676 Mbit/s — for 1 hour with **zero late slices** at the commissioned A; a third bundle is **refused at admission** with a named reason.
2. **AC-14.** Zero late slices for 1 hour on a clean medianet at the commissioned A in `baseline`.
3. **AC-24.** The asymmetry bound holds against the external check: every in-band one-way figure lies within its own published bound of the external value; aggregate loss at or below the floor in `loss-soak` (C-90, C-118); in the multi-packet regime, first-packet and slice-complete latency differ by the serialisation time (about 12 µs per 1 500-byte packet at 1 Gbit/s) within ±5 µs (C-167); coalescing bias measured and recorded.
4. **AC-25.** 64 followers at 50 requests/s each with no growth in holder serving lateness (99.9th percentile within 20 % from 1 to 64 followers, C-167); the 65th refused with a named event. Followers beyond 63 are synthetic (C-114).
5. **AC-19 (L1 part).** On a real switch with a lowered MTU, the probe pair detects at admission, the connection goes live at 1 280 bytes, and the path MTU appears in the stamp. Re-probe after a live MTU drop is recorded but is not part of this criterion.
6. **AC-34.** Receiver reports and the receiving peer's regime stamp (C-47) show sync 48, media and probes 46, control 40 as received.
7. **AC-37.** Loss induced by shrinking a receive buffer appears in the socket overflow counter, the stamp and the receiver report.
8. **AC-11 (L1 part).** One input flooded at 10 × its admitted rate from the other host: no missed deadline on the consumer, excess visible in the overflow counter.
9. **019-A (stamps).** Every result file carries both hosts' complete stamps (profile, kernel, NIC, driver, coalescing state, path MTU, DSCP observed), including strict mode, the capability states (C-129) and the receive-timestamp source (C-131).
10. **019-B (redundancy verdict).** The report states, for the redundancy question, measured clean-wire aggregate loss against C-90 and a recommendation by the C-179 rule (above 10 × C-90, about 580 losses in the soak, reopens the decision; between 1 × and 10 × is investigated as a cabling or host problem).
11. **019-C (jitter profile, C-94).** A measured wired jitter profile is recorded in the same terms as C-94 (base delay, jitter distribution, burst fraction and delay) together with the empirical distribution, and handed to the simulation owner.
12. **019-D (bounded drain, C-16).** For every legitimate-traffic scenario the report gives the largest number of packets drained from one input in one cycle and the number of cycles that reached the bound.

## Definition of done

- [ ] Tests green in CI, unprivileged
- [ ] Every parser touched is fuzzed
- [ ] Regime stamp fields touched are complete and documented
- [ ] Design documents updated where behaviour differs, constants table first
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] The named hardware pair (hosts, NICs, switch, cables) is recorded
- [ ] Each pending-measurement constant in the table above has a measured value tagged `[measured]`, or a stated reason why not
- [ ] The harness itself (orchestration, report generation) has a dry-run mode that passes in CI on loopback

Not applicable in this epic (C-125): none. Every line has a subject here: the harness has parsers (hosts file, `ethtool` output, runtime stamps and records, the asymmetry reference), reads and documents regime stamp fields, reuses the frozen POC's `tools/stress.sh`, and runs on named hardware.

## Open sub-questions

| Question | Owner | Falsifier |
|---|---|---|
| External reference for the asymmetry check (a hardware timestamping NIC pair, a shared pulse, or a swapped-cable method) | This task | No available method resolves below the bound the runtime publishes in `baseline` |
| Is clean-wire loss near the floor? | This task (`loss-soak`, C-118) | Aggregate loss above 10 × C-90, about 580 losses over the soak (C-179; about 58 predicted, C-118): the redundancy decision reopens. Between 1 × and 10 × it is recorded and investigated as a cabling or host problem (C-179) |
| Does the consumer keep its deadlines at 64 000 packets/s? | This task (`two-bundles`) | Any late slice: tighten the port rule (C-25) or the per-stream channel count (C-93 falsifier) |
| Does a small unmanaged switch absorb coherent bursts (32 packets, about 42 KB, per slice period)? | This task, then task 028 as a commissioning test | Drops with the egress spread on (C-14) |
| Runtime instrumentation: data-thread time in batch receive (C-93), drain-bound hits per input per slice period (C-16, C-156), and data-thread time per bad packet (C-164) | Epic 015 | The counters are absent or perturb the cost they measure |
| Layout of the stamp message sections, the receiver report and the per-connection counters (content settled by C-47, C-152, C-159, C-160); record file format | Epic 004 (stamp, receiver report), epic 015 (record file) | The harness cannot read one of the 7 stamp categories, a C-159 counter or a DSCP class from them |
| Whether the runtime accepts a stream wider than the default 6 channels (for `multi-packet` and `blackhole`) | Epic 015 | A wider stream is refused, so the multi-packet regime cannot be forced without lowering the MTU |
